//! `run()` body + per-mode top-level wiring.
//!
//! Emits the shared dataflow-construction body that lives inside the worker
//! closure (`emit_closure_body`) plus the two per-mode wrappers around it:
//! incremental (`phase1_run_body`) and batch (`phase1_run_body_batch`).
//! The batch-mode top-level entry (`compile_mir_dd_batch`) also lives here
//! since it wires the struct/default/run plumbing for that mode.

use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

use ascent_mir::utils::tuple_type;
use ascent_mir::{AscentMir, RelationIdentity};
use itertools::Itertools;

use crate::dfg;
use crate::scc::compile_scc;
use crate::utils::{
   materialize_rel, relation_coll_var, relation_input_var, relation_sink_inner, relation_sink_outer,
};
use crate::{emit_struct_and_default, phase1_blocker};

pub(crate) fn emit_closure_body(
   mir: &AscentMir,
   sorted_rels: &[&RelationIdentity],
   init_coll: impl Fn(&RelationIdentity) -> TokenStream,
   sink_ident: impl Fn(&RelationIdentity) -> TokenStream,
   probe_expr: TokenStream,
   is_batch: bool,
   // Expression for "this worker's index" in the enclosing scope.
   // `run()` path (batch or inc): `sealer.worker_index()`.
   // Session path (single-worker Session dataflow): literal `0`.
   worker_index_expr: TokenStream,
) -> (TokenStream, TokenStream, usize) {
   let mut body = TokenStream::new();
   // DD 0.20: `join`, `reduce`, `arrange_by_*`, `consolidate` are inherent
   // on Collection. Only `threshold_semigroup` still needs its trait import.
   if is_batch {
      body.extend(quote! {
         use ::ascent::dd::differential_dataflow::operators::ThresholdTotal;
      });
   }
   for rel in sorted_rels {
      body.extend(init_coll(rel));
   }
   // `__unit_scope` seeds generator-only rules. Its diff type must match
   // the rest of the dataflow's diff (i32 incremental, Present batch —
   // FlowLog parity).
   let unit_seed_ty = if is_batch {
      quote! { ::ascent::dd::BatchDiff }
   } else {
      quote! { i32 }
   };
   // `__unit_scope` seeds generator-only rules. `new_collection_from` is
   // hardcoded to `isize` diff; batch mode uses `new_collection_from_raw`
   // to get a `Present`-diff Collection. Batch also skips `advance_to(1)`
   // since the `()` timestamp has nothing to advance to.
   body.extend(quote! {
      // DD 0.20: `Variable<G, C>` is 2-arg; the 3-arg alias is `VecVariable`.
      // Our runtime re-exports both under the name `Variable` (= `VecVariable`)
      // so codegen's `Variable<_, TupTy, Diff>` keeps compiling.
      use ::ascent::dd::Variable;
      use ::ascent::dd::timely::order::Product;
      use ::ascent::dd::timely::dataflow::Scope;
   });
   body.extend(dfg::lower_unit_scope_decl(is_batch));
   let _ = unit_seed_ty;
   let mut hoists = TokenStream::new();
   let mut hoist_counter: usize = 0;
   for (scc_idx, scc) in mir.sccs.iter().enumerate() {
      // SCC landmark — names the SCC index and its dyn/body-only rels so
      // `cargo expand` output stays navigable even for large programs.
      let dyn_names: Vec<String> =
         scc.dynamic_relations.keys().sorted_by_key(|r| &r.name).map(|r| r.name.to_string()).collect();
      let body_only_names: Vec<String> =
         scc.body_only_relations.keys().sorted_by_key(|r| &r.name).map(|r| r.name.to_string()).collect();
      let scc_label = format!(
         "scc {} — dyn: [{}], body-only: [{}]",
         scc_idx,
         dyn_names.join(", "),
         body_only_names.join(", ")
      );
      body.extend(quote! { ::ascent::internal::comment(#scc_label); });
      let (b, h) = compile_scc(scc, scc_idx, &mut hoist_counter, is_batch);
      body.extend(b);
      hoists.extend(h);
   }
   // Rels already deduped upstream never need sink-level materialize:
   //   - Recursive IDBs are thresholded at bind-leave inside scope.iterative.
   //   - Rels that feed a looping SCC pass through pre_loop_threshold.
   // Compute the set of "already clean" rel names so we can skip the
   // sink-attach threshold for them. FlowLog-parity — matches FlowLog's
   // output wiring which only thresholds before printsize (for set rels),
   // not twice.
   let prededuped: std::collections::HashSet<String> = {
      let mut s = std::collections::HashSet::new();
      for scc in mir.sccs.iter() {
         if scc.is_looping {
            for r in scc.dynamic_relations.keys() { s.insert(r.name.to_string()); }
            for r in scc.body_only_relations.keys() { s.insert(r.name.to_string()); }
         }
      }
      s
   };
   for rel in sorted_rels {
      let coll = relation_coll_var(&rel.name);
      let sink_id = sink_ident(rel);
      // FlowLog pattern: recursive IDBs threshold at bind-leave inside
      // `scope.iterative` — re-thresholding via `materialize_rel` is pure
      // wasted work. Lattices still need reduce (handled inside
      // materialize_rel). Batch's `threshold_semigroup` is a distinct
      // operator that must run here (first-seen accounting); for inc,
      // only lattice or non-pre-deduped set rels need a materialize step.
      let already_clean = !rel.is_lattice && prededuped.contains(&rel.name.to_string());
      let final_expr = if is_batch && !rel.is_lattice {
         quote! { (#coll).clone() }
      } else if !is_batch && already_clean {
         quote! { (#coll).clone() }
      } else {
         materialize_rel(rel, quote! { #coll }, is_batch)
      };
      // Both batch and inc Sink types take (worker_index, &coll, probe):
      // per-worker slot writes avoid mutex contention at high worker counts.
      body.extend(dfg::lower_sink_attach(sink_id, final_expr, probe_expr.clone(), worker_index_expr.clone()));
   }
   (body, hoists, hoist_counter)
}

pub(crate) fn phase1_run_body(mir: &AscentMir, target: &TokenStream) -> TokenStream {
   let sorted_rels = mir.relations_ir_relations.keys().sorted_by_key(|r| &r.name).collect_vec();

   // Pre-closure wiring: snapshot self.<rel> into an input Vec; create outer
   // + inner sink clones (outer drains after the closure returns).
   let mut inputs_setup = vec![];
   let mut sinks_decl = vec![];
   let mut sinks_clone = vec![];
   let mut sink_drains = vec![];
   for rel in &sorted_rels {
      let name = &rel.name;
      let tuple_ty = tuple_type(&rel.field_types);
      let in_var = relation_input_var(name);
      let sink_outer = relation_sink_outer(name);
      let sink_inner = relation_sink_inner(name);

      inputs_setup.push(quote! {
         let #in_var: ::std::vec::Vec<#tuple_ty> = #target.#name.clone();
      });
      sinks_decl.push(quote! {
         let #sink_outer: ::ascent::dd::Sink<#tuple_ty> = ::ascent::dd::Sink::new();
      });
      sinks_clone.push(quote! {
         let #sink_inner = #sink_outer.clone();
      });
      sink_drains.push(quote! {
         #target.#name = #sink_outer.into_vec();
      });
   }

   // Shared core: imports, collections, unit scope, SCC bodies, attach.
   let (closure_body, hoists, hoist_count) = emit_closure_body(
      mir,
      &sorted_rels,
      |rel| {
         // Batch init: create the Collection from the input Vec via Sealer.
         let coll = relation_coll_var(&rel.name);
         let in_var = relation_input_var(&rel.name);
         // Pass `&in_var` — Sealer::input takes `&[T]`. All 24 workers share
         // one Vec; each worker hash-partitions and inserts only its 1/peers
         // slice. Avoids 24× Vec<T> clones the `Fn` build closure previously
         // triggered.
         quote! { let mut #coll = sealer.input(scope, &#in_var); }
      },
      |rel| {
         // Batch attach target: inner sink clone moved into the closure.
         let sink_inner = relation_sink_inner(&rel.name);
         quote! { #sink_inner }
      },
      quote! { probe },
      false,
      quote! { sealer.worker_index() },
   );

   // Parallel-DD Fn compliance: for every `__hoist_gen_N` bound OUTSIDE
   // the closure, shadow it with a fresh clone INSIDE so inner DD
   // `move` closures (e.g. `.flat_map`) can move the clone without
   // consuming the outer-captured Vec. Per-call cost is one clone per
   // hoist; outer closure stays `Fn`-callable across workers.
   let hoist_rebinds: TokenStream = (1..=hoist_count)
      .map(|i| {
         let id = Ident::new(&format!("__hoist_gen_{i}"), Span::call_site());
         quote! { let #id = #id.clone(); }
      })
      .collect();

   quote! {
      #(#inputs_setup)*
      #(#sinks_decl)*
      #(#sinks_clone)*
      #hoists
      ::ascent::dd::execute_batch(move |scope, sealer, probe| {
         #hoist_rebinds
         #closure_body
      });
      #(#sink_drains)*
   }
}

/// Batch-mode `run()` body: Present diff, `threshold_semigroup` distinct,
/// `execute_batch_present` multi-worker runtime. FlowLog-equivalent shape for
/// the subset of programs allowed by `compile_mir_dd_batch` (no negation,
/// no lattice, no aggregates).
fn phase1_run_body_batch(mir: &AscentMir, target: &TokenStream) -> TokenStream {
   let sorted_rels = mir.relations_ir_relations.keys().sorted_by_key(|r| &r.name).collect_vec();

   let mut inputs_setup = vec![];
   let mut sinks_decl = vec![];
   let mut sinks_clone = vec![];
   let mut sink_drains = vec![];
   for rel in &sorted_rels {
      let name = &rel.name;
      let tuple_ty = tuple_type(&rel.field_types);
      let in_var = relation_input_var(name);
      let sink_outer = relation_sink_outer(name);
      let sink_inner = relation_sink_inner(name);
      inputs_setup.push(quote! {
         let #in_var: ::std::vec::Vec<#tuple_ty> = #target.#name.clone();
      });
      sinks_decl.push(quote! {
         let #sink_outer: ::ascent::dd::BatchSink<#tuple_ty> = ::ascent::dd::BatchSink::new();
      });
      sinks_clone.push(quote! {
         let #sink_inner = #sink_outer.clone();
      });
      sink_drains.push(quote! {
         #target.#name = #sink_outer.into_vec();
      });
   }

   let (closure_body, hoists, hoist_count) = emit_closure_body(
      mir,
      &sorted_rels,
      |rel| {
         let coll = relation_coll_var(&rel.name);
         let in_var = relation_input_var(&rel.name);
         // `.consolidate()` after input: matches FlowLog's pattern. Sorts
         // tuples into a canonical trace, drops zero-weight/duplicates.
         // Critical because downstream `arrange_by_key` + `threshold_semigroup`
         // assume a consolidated input; without consolidation, duplicate diffs
         // create redundant per-iteration maintenance work inside the
         // iterative scope.
         quote! {
            let mut #coll = sealer.input(scope, #in_var.clone()).consolidate();
         }
      },
      |rel| {
         let sink_inner = relation_sink_inner(&rel.name);
         quote! { #sink_inner }
      },
      quote! { probe },
      true,
      quote! { sealer.worker_index() },
   );

   let hoist_rebinds: TokenStream = (1..=hoist_count)
      .map(|i| {
         let id = Ident::new(&format!("__hoist_gen_{i}"), Span::call_site());
         quote! { let #id = #id.clone(); }
      })
      .collect();

   quote! {
      #(#inputs_setup)*
      #(#sinks_decl)*
      #(#sinks_clone)*
      #hoists
      ::ascent::dd::execute_batch_present(move |scope, sealer, probe| {
         #hoist_rebinds
         #closure_body
      });
      #(#sink_drains)*
   }
}

/// Batch-mode entry point. FlowLog-style DD codegen: Present diff,
/// `threshold_semigroup` distinct. Negation/aggregation/lattice all go
/// through the i32-roundtrip trick where DD's `reduce` / `antijoin` would
/// otherwise require an `Abelian` diff `Present` can't provide.
pub(crate) fn compile_mir_dd_batch(mir: &AscentMir, is_ascent_run: bool) -> syn::Result<TokenStream> {
   let blocker = phase1_blocker(mir);
   let struct_and_default = emit_struct_and_default(mir, !is_ascent_run);

   let self_target: TokenStream = quote!(self);
   let (impl_impl_generics, impl_ty_generics, impl_where_clause) = mir.signatures.split_impl_generics_for_impl();
   let struct_name = &mir.signatures.declaration.ident;

   let summary = format!(
      "DD backend (batch) — {} relations, phase1_blocker: {:?}",
      mir.relations_ir_relations.len(),
      blocker
   );
   let summary_fn = if is_ascent_run {
      quote! { pub fn summary(&self) -> &'static str { #summary } }
   } else {
      quote! { pub fn summary() -> &'static str { #summary } }
   };

   let run_body = match &blocker {
      None => phase1_run_body_batch(mir, &self_target),
      Some(reason) => {
         let comment = format!("dd batch backend: falling back to noop run() — {reason}");
         quote! {
            let _ = #comment;
         }
      },
   };
   let run_func = if is_ascent_run {
      quote! {}
   } else {
      quote! {
         #[doc = "Runs the Ascent program to a fixed point (DD backend, batch mode)."]
         pub fn run(&mut self) {
            #![allow(unused_imports, unused_mut, unused_variables, clippy::all)]
            #run_body
         }
      }
   };

   let methods = quote! {
      impl #impl_impl_generics #struct_name #impl_ty_generics #impl_where_clause {
         #run_func
         #summary_fn
         pub fn relation_sizes_summary(&self) -> ::std::string::String { ::std::string::String::new() }
         pub fn scc_times_summary(&self) -> ::std::string::String { ::std::string::String::new() }
      }
   };

   let ascent_run_wrapper = if is_ascent_run {
      let inputs_assign: TokenStream = mir
         .relations_ir_relations
         .keys()
         .sorted_by_key(|r| &r.name)
         .map(|rel| {
            let name = &rel.name;
            quote! { __run_res.#name = #name; }
         })
         .collect();
      let run_body = phase1_run_body_batch(mir, &quote!(__run_res));
      quote! {
         {
            #![allow(unused_imports, unused_mut, unused_variables, clippy::all)]
            let mut __run_res = <#struct_name #impl_ty_generics as ::std::default::Default>::default();
            #inputs_assign
            #run_body
            __run_res
         }
      }
   } else {
      quote! {}
   };

   Ok(if is_ascent_run {
      quote! {
         {
            #struct_and_default
            #methods
            #ascent_run_wrapper
         }
      }
   } else {
      quote! {
         #struct_and_default
         #methods
      }
   })
}
