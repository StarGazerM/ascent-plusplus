//! SCC compilation — lowers one `MirScc` into the dataflow tokens that go
//! inside the `execute_batch` / `build_session_worker` closure.
//!
//! `compile_scc` dispatches by `is_looping`:
//!  - non-looping (stratum-once): rules append straight into `__<rel>_coll`.
//!  - looping (recursive): rules are wrapped in `scope.iterative`, with DD
//!    `Variable`s for each dynamic relation and FlowLog-style bind-leave.

use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

use ascent_mir::utils::tuple_type;
use ascent_mir::{MirRule, MirScc, RelationIdentity, mir_rule_summary};
use itertools::Itertools;

use crate::dfg;
use crate::rule_body::{compile_rule_head_exprs, compile_rule_with_head_target};
use crate::utils::{
   emit_shared_arrangement_binding, materialize_rel, relation_coll_var, shared_arr_ident,
   tuple_of_idents,
};

pub(crate) fn compile_scc(scc: &MirScc, scc_idx: usize, hoist_counter: &mut usize, is_batch: bool) -> (TokenStream, TokenStream) {
   if scc.is_looping {
      compile_looping_scc(scc, scc_idx, hoist_counter, is_batch)
   } else {
      compile_nonlooping_scc(scc, hoist_counter, is_batch)
   }
}

/// Non-looping SCC: rules append productions directly to the outer-scope
/// `__<rel>_coll` of each head relation. Body clauses read `__<rel>_coll`.
fn compile_nonlooping_scc(scc: &MirScc, hoist_counter: &mut usize, is_batch: bool) -> (TokenStream, TokenStream) {
   let mut out = TokenStream::new();
   let mut hoists = TokenStream::new();
   let head_target = |name: &Ident| relation_coll_var(name);
   let outer_scope = Ident::new("scope", Span::call_site());
   // Shared arrangements at outer scope — matches the looping-SCC pattern but
   // without the `scope.iterative` wrapper. Non-looping SCCs still have
   // body clauses that need arrangements for `join_core`.
   // Only `body_only_relations` need arrangements here — those are read by
   // this SCC's rule bodies. `dynamic_relations` (the SCC's OUTPUT) are
   // populated by the rules below via `concat`, so arranging them at this
   // point would capture pre-concat (empty) state; downstream SCCs'
   // arrangement step will arrange them correctly over the populated
   // collection. Emit body_only arrangements BEFORE the rules so join_core
   // inside rule bodies can reference them.
   // Collect actually-used arrangement names from this SCC's rule bodies
   // (skip first-clauses without a subsequent join, and empty-indices which
   // are never referenced via the shared-arr path).
   use ascent_mir::MirBodyItem;
   let mut used_arr_names: std::collections::HashSet<Ident> =
      std::collections::HashSet::new();
   for rule in &scc.rules {
      let rule_has_join = rule.body_items.iter().filter(|it| matches!(it, MirBodyItem::Clause(_))).count() >= 2;
      for (i, item) in rule.body_items.iter().enumerate() {
         if let MirBodyItem::Clause(cl) = item {
            if cl.rel.indices.is_empty() { continue; }
            if i == 0 && !rule_has_join { continue; }
            let name = format!(
               "__arr_{}_indices_{}",
               cl.rel.relation.name,
               cl.rel.indices.iter().map(|i| i.to_string()).collect::<Vec<_>>().join("_")
            );
            used_arr_names.insert(Ident::new(&name, cl.rel.relation.name.span()));
         }
      }
   }
   let mut seen_arrs: std::collections::HashSet<Ident> = std::collections::HashSet::new();
   let rel_iter = scc
      .body_only_relations
      .iter()
      .sorted_by_cached_key(|(rel, _)| rel.name.to_string());
   for (_rel_ident, ir_rels) in rel_iter {
      for ir_rel in ir_rels.iter().sorted_by_cached_key(|r| r.ir_name()) {
         let arr_name = shared_arr_ident(ir_rel);
         if !used_arr_names.contains(&arr_name) {
            continue;
         }
         if !seen_arrs.insert(arr_name.clone()) {
            continue;
         }
         out.extend(emit_shared_arrangement_binding(ir_rel));
      }
   }
   for rule in &scc.rules {
      let summary = format!("rule {}", mir_rule_summary(rule));
      out.extend(quote! { ::ascent::internal::comment(#summary); });
      let (body, pre) = compile_rule_with_head_target(rule, &head_target, &outer_scope, hoist_counter, is_batch);
      out.extend(body);
      hoists.extend(pre);
   }
   // FlowLog parity (incremental): threshold each head `__<rel>_coll`
   // WHEN multiple rules in this SCC contribute to the same head —
   // that's when duplicates are likely (e.g. CSPA's
   //   value_flow(x,x) <-- assign(_,x);
   //   value_flow(x,x) <-- assign(x,_);
   // ). Without this, duplicates propagate into downstream looping SCCs
   // as fat diffs, bloating inner-scope arrangements until bind-leave
   // threshold normalizes at iteration boundaries.
   //
   // Single-rule-per-head heads are skipped: the body can still produce
   // duplicates via cross-products, but that's rarer and threshold
   // isn't free (it's an arrangement).
   //
   // Batch mode consolidates in the seed-enter path instead (see
   // compile_looping_scc); threshold here would interact awkwardly with
   // Present-diff `threshold_semigroup` first-seen accounting.
   if !is_batch {
      use std::collections::{HashMap, HashSet};
      let mut head_rule_count: HashMap<String, usize> = HashMap::new();
      for rule in &scc.rules {
         for hcl in &rule.head_clause {
            *head_rule_count.entry(hcl.rel.name.to_string()).or_insert(0) += 1;
         }
      }
      let mut seen_heads: HashSet<String> = HashSet::new();
      for rule in &scc.rules {
         for hcl in &rule.head_clause {
            let name_str = hcl.rel.name.to_string();
            if head_rule_count.get(&name_str).copied().unwrap_or(0) < 2 {
               continue;
            }
            if !seen_heads.insert(name_str) {
               continue;
            }
            let coll = relation_coll_var(&hcl.rel.name);
            out.extend(quote! {
               #coll = #coll.threshold(|_, __w: &i32| if *__w > 0 { 1i32 } else { 0 });
            });
         }
      }
   }
   (out, hoists)
}

/// Looping SCC: wrap rules in `scope.iterative(...)`, with one DD `Variable`
/// per dynamic relation and `.enter()`'d body-only relations. Each rule
/// accumulates into the dynamic relation's `__<rel>_acc`, which the final
/// `var.set(&acc.distinct()).leave()` returns to outer scope.
fn compile_looping_scc(scc: &MirScc, scc_idx: usize, hoist_counter: &mut usize, is_batch: bool) -> (TokenStream, TokenStream) {
   // Stable iteration order so generated code is deterministic.
   let dyn_rels: Vec<&RelationIdentity> = scc.dynamic_relations.keys().sorted_by_key(|r| &r.name).collect();
   let body_only_rels: Vec<&RelationIdentity> =
      scc.body_only_relations.keys().sorted_by_key(|r| &r.name).collect();

   // FlowLog-parity (incremental): threshold all outer-scope input collections
   // BEFORE entering the iterative scope. Ascent's MIR splits non-recursive
   // rules into separate SCCs — each with 1 rule per head — so the per-SCC
   // `head_rule_count >= 2` threshold in `compile_nonlooping_scc` misses the
   // case where multiple non-looping SCCs contribute to the same head (e.g.
   // CSPA's 3 rules for `value_flow`, each in its own SCC). Without this,
   // duplicates propagate into the loop as fat diffs, bloat inner-scope
   // arrangement traces, and the bind-leave threshold only normalizes at
   // iteration boundaries.
   //
   // Threshold once per rel here IS the FlowLog-equivalent single-threshold-
   // per-non-recursive-stratum; ascent's SCC split makes it the right level.
   // Batch mode is excluded — consolidate-before-enter already does the
   // necessary Present-diff compaction there.
   let mut pre_loop_threshold = TokenStream::new();
   if !is_batch {
      for rel in body_only_rels.iter().chain(dyn_rels.iter()) {
         let coll = relation_coll_var(&rel.name);
         pre_loop_threshold.extend(quote! {
            #coll = #coll.threshold(|_, __w: &i32| if *__w > 0 { 1i32 } else { 0 });
         });
      }
   }

   // 1. FlowLog naming: `in_<rel>` for entered non-recursive Collections,
   //    `in_<arr>` for entered arrangements. Consolidate before entering:
   //    Ascent's MIR splits non-recursive rules into separate SCCs, so
   //    `__<rel>_coll` is a chain of N concats. FlowLog consolidates after
   //    their equivalent chain (`valueflow = t1.concat(t2).concat(t3).consolidate()`).
   //    Without consolidate, duplicates enter the iterative scope and cause
   //    threshold_semigroup's "first-seen" accounting to loop more iterations.
   // Pre-compute which body-only rels have their Collection actually read
   // (vs only reached through shared arrangements). Dead `in_<rel>` saves
   // both generated code size AND a runtime `consolidate()` operator per
   // dead rel.
   let used_colls = crate::analyses::UsedCollections::of(scc);
   let inner_scope_ident = Ident::new("inner", Span::call_site());
   // FlowLog-parity seed entry:
   //   - Batch (Present diffs): MUST consolidate before entering the iterative
   //     scope. Batch-mode `threshold_semigroup` is "first-seen" accounting;
   //     unconsolidated duplicate diffs would fake repeated new tuples.
   //   - Incremental (isize diffs): plain `.enter(inner)`. Duplicates become
   //     higher multiplicities, which the bind-leave `.threshold(w>0?1:0)`
   //     cleanly normalizes. Matches FlowLog's incremental codegen
   //     (`source.enter(inner)` — no consolidate).
   let enter_seed = if is_batch {
      dfg::DataflowOp::ConsolidateEnter { inner_scope: inner_scope_ident.clone() }
   } else {
      dfg::DataflowOp::Enter { scope: inner_scope_ident.clone() }
   };
   let mut inner_body_only_bindings = TokenStream::new();
   for rel in &body_only_rels {
      let name = &rel.name;
      if !used_colls.contains(name) {
         continue;
      }
      let coll = relation_coll_var(name);
      let seed = Ident::new(&format!("in_{}", name), name.span());
      let rhs = dfg::lower(&enter_seed, quote! { #coll.clone() });
      inner_body_only_bindings.extend(quote! {
         let #seed = #rhs;
      });
   }
   let mut inner_dyn_seed_bindings = TokenStream::new();
   for rel in &dyn_rels {
      let name = &rel.name;
      let coll = relation_coll_var(name);
      let seed = Ident::new(&format!("in_{}", name), name.span());
      let rhs = dfg::lower(&enter_seed, quote! { #coll.clone() });
      inner_dyn_seed_bindings.extend(quote! {
         let #seed = #rhs;
      });
   }

   // 2. Variables for dynamic rels + shadowed body-reading bindings.
   //    Annotate the element type explicitly: in mutually-recursive SCCs,
   //    Rust can't infer the Variable's element type through the cycle.
   let mut var_decls = TokenStream::new();
   // Enter the outer unit collection into this iterative scope so fact /
   // generator-first rules inside the SCC can seed from it.
   let unit_enter = dfg::lower(
      &dfg::DataflowOp::Enter { scope: inner_scope_ident.clone() },
      quote! { __unit_scope.clone() },
   );
   var_decls.extend(quote! {
      let __unit_inner = #unit_enter;
   });
   // DD 0.20 API change: `Variable::new` returns `(Variable, Collection)` —
   // no more `Deref`. Unpack here so downstream shadow bindings can read
   // the companion Collection directly (without `*var`) and the var itself
   // is consumed by `.set(...)` at the bind-leave point.
   for rel in &dyn_rels {
      let name = &rel.name;
      let var = Ident::new(&format!("recursive_{}_var", name), name.span());
      let read = Ident::new(&format!("recursive_{}", name), name.span());
      let tup_ty = tuple_type(&rel.field_types);
      var_decls.extend(dfg::lower_variable_decl(&var, &read, &quote! { #tup_ty }, is_batch));
   }

   // 3. Shadow `__<rel>_coll` inside the scope so rule bodies keep their
   //    unchanged textual form (`__<rel>_coll.map(...)`). For body-only
   //    rels, the shadow points at the entered seed. For dynamic rels,
   //    it's a clone of the variable's deref'd Collection.
   let mut shadow_bindings = TokenStream::new();
   for rel in &body_only_rels {
      let name = &rel.name;
      if !used_colls.contains(name) {
         continue;
      }
      let coll = relation_coll_var(name);
      let seed = Ident::new(&format!("in_{}", name), name.span());
      shadow_bindings.extend(quote! {
         let #coll = #seed;
      });
   }
   for rel in &dyn_rels {
      let name = &rel.name;
      let coll = relation_coll_var(name);
      let read = Ident::new(&format!("recursive_{}", name), name.span());
      // DD 0.20: read from the companion Collection returned by `Variable::new`,
      // not from `*var` (Deref removed).
      shadow_bindings.extend(quote! {
         let #coll = #read.clone();
      });
   }

   // 4. No `__<rel>_acc` bindings — FlowLog-style chain-concat at bind_leave
   //    replaces mutable accumulators.

   // 4b. Shared arrangements — one per unique (relation, indices) used as a
   //     clause RHS in this SCC. Emitted once inside the iterative scope so
   //     every rule body references the same arranged trace instead of
   //     rebuilding its own. Covers both `body_only_relations` (read-only in
   //     this SCC) and `dynamic_relations` (recursive; Variable-backed).
   //     `scc.{body_only,dynamic}_relations` is the MIR's pre-deduped set of
   //     `(rel, keying)` specs — the exact planner output FlowLog computes.
   // FlowLog split: arrange body_only (EDB/non-recursive IDB) OUTSIDE the
   // iterative scope, `.enter(inner)` the arrangement handle into the loop.
   // Arrange dynamic (recursive) relations INSIDE the loop on the Variable's
   // read handle (shadow __<rel>_coll above).
   //
   // Analyses (dd_analyses.rs) — pre-compute the data this emission needs.
   let used_arrs = crate::analyses::UsedArrangements::of(scc);
   let placement = crate::analyses::RelationPlacement::of(scc);

   let mut outer_arrs = TokenStream::new();
   let mut outer_arr_enters = TokenStream::new();
   let mut inner_arrs = TokenStream::new();
   let mut seen_arrs: std::collections::HashSet<Ident> = std::collections::HashSet::new();
   let rel_iter = scc
      .body_only_relations
      .iter()
      .chain(scc.dynamic_relations.iter())
      .sorted_by_cached_key(|(rel, _)| rel.name.to_string());
   for (rel_ident, ir_rels) in rel_iter {
      let is_body_only = placement.is_body_only(&rel_ident.name);
      for ir_rel in ir_rels.iter().sorted_by_cached_key(|r| r.ir_name()) {
         let arr_name = shared_arr_ident(ir_rel);
         if !used_arrs.contains(&arr_name) {
            continue;
         }
         if !seen_arrs.insert(arr_name.clone()) {
            continue;
         }
         if is_body_only {
            outer_arrs.extend(emit_shared_arrangement_binding(ir_rel));
            let arr_enter = dfg::lower(
               &dfg::DataflowOp::Enter { scope: inner_scope_ident.clone() },
               quote! { #arr_name },
            );
            outer_arr_enters.extend(quote! {
               let #arr_name = #arr_enter;
            });
         } else {
            inner_arrs.extend(emit_shared_arrangement_binding(ir_rel));
         }
      }
   }

   // Rule classification (dd_analyses.rs): partition rules into non-recursive
   // (body touches no dyn_rel of this SCC) and recursive.
   // FlowLog pattern: non-recursive rules lowered OUTSIDE `scope.iterative`
   // (fire once, feed the seed); recursive rules INSIDE (feed the
   // Variable/threshold loop). Without this split, non-recursive rules
   // re-fire every iteration producing identical tuples.
   let rule_class = crate::analyses::RuleClassification::of(scc);
   let non_rec_rules: &[&MirRule] = &rule_class.non_recursive;
   let rec_rules: &[&MirRule] = &rule_class.recursive;

   let mut non_rec_body = TokenStream::new();
   let mut non_rec_hoists = TokenStream::new();
   let outer_scope = Ident::new("scope", Span::call_site());
   let non_rec_head_target = |name: &Ident| relation_coll_var(name);
   for rule in non_rec_rules {
      let summary = format!("rule [non-rec] {}", mir_rule_summary(rule));
      non_rec_body.extend(quote! { ::ascent::internal::comment(#summary); });
      let (body, pre) =
         compile_rule_with_head_target(rule, &non_rec_head_target, &outer_scope, hoist_counter, is_batch);
      non_rec_body.extend(body);
      non_rec_hoists.extend(pre);
   }

   // Recursive rules: FlowLog-style emission.
   // Step A: emit each rule's body expression as a distinct `let __rule_N = ...;`
   //         binding. Collect per-head-relation Vec<rule_ident>.
   // Step B: at bind_leave, emit `let __X_next = t_0.concat(t_1)...concat(__X_seed).threshold(...);`
   //         then `__X_var.set(__X_next.clone()); __X_next.leave()`.
   // No `__X_acc` mutable accumulator needed — dataflow same result, structure
   // matches FlowLog exactly (rule outputs as named bindings, chain concat at fixpoint build).
   let inner_scope = Ident::new("inner", Span::call_site());
   let mut rules_ts = TokenStream::new();
   let mut rules_hoists = TokenStream::new();
   let mut per_head_rule_idents: std::collections::HashMap<String, Vec<Ident>> =
      std::collections::HashMap::new();
   for (i, rule) in rec_rules.iter().enumerate() {
      let summary = format!("rule [rec] {}", mir_rule_summary(rule));
      rules_ts.extend(quote! { ::ascent::internal::comment(#summary); });
      let (head_exprs, pre) = compile_rule_head_exprs(rule, &inner_scope, hoist_counter, is_batch);
      rules_hoists.extend(pre);
      for (j, (head_rel, expr)) in head_exprs.into_iter().enumerate() {
         let rule_ident = Ident::new(&format!("t_{}_{}", i, head_rel), head_rel.span());
         let _ = j;
         rules_ts.extend(quote! {
            let #rule_ident = #expr;
         });
         per_head_rule_idents.entry(head_rel.to_string()).or_default().push(rule_ident);
      }
   }

   // 6. Bind+leave tuple expression.
   let leaved_idents: Vec<Ident> =
      dyn_rels.iter().map(|r| Ident::new(&format!("__{}_leaved", r.name), r.name.span())).collect();
   let leaved_dest_tuple = tuple_of_idents(&leaved_idents);

   // FlowLog's bind-leave pattern:
   //   let next_X = rule1.concat(rule2)...concat(in_X).threshold_semigroup(...);
   //   var_X.set(next_X.clone());
   //   next_X.leave()
   let mut bind_leave_exprs = vec![];
   for rel in &dyn_rels {
      let name = &rel.name;
      let var = Ident::new(&format!("recursive_{}_var", name), name.span());
      let seed = Ident::new(&format!("in_{}", name), name.span());
      let next = Ident::new(&format!("next_{}", name), name.span());
      let rule_idents =
         per_head_rule_idents.get(&name.to_string()).cloned().unwrap_or_default();
      // Build chain: rule_0.concat(rule_1)...concat(seed). If no rules for this
      // head (shouldn't happen for dyn_rel in looping SCC), use seed alone.
      // FlowLog-exact: `t_0.clone().concat(t_1.clone())...concat(in_X.clone())`.
      let mut chain_sources = rule_idents.clone();
      chain_sources.push(seed.clone());
      let concat_chain = dfg::lower_concat_chain(&chain_sources);
      let materialized = materialize_rel(rel, concat_chain, is_batch);
      bind_leave_exprs.push(dfg::lower_bind_leave(&var, &next, materialized));
   }
   let bind_leave_tuple = if bind_leave_exprs.len() == 1 {
      let e = &bind_leave_exprs[0];
      quote! { ( #e , ) }
   } else {
      quote! { ( #(#bind_leave_exprs),* ) }
   };

   // 7. Reassign outer `__<rel>_coll` from the leaved values.
   let mut outer_reassigns = TokenStream::new();
   for (rel, leaved) in dyn_rels.iter().zip(leaved_idents.iter()) {
      let coll = relation_coll_var(&rel.name);
      outer_reassigns.extend(quote! {
         #coll = #leaved;
      });
   }

   let _ = scc_idx; // label generation skipped for now

   // Batch mode: `()` outer + `u16` inner. `Product<(), Iter>` is TotalOrder
   // so `threshold_semigroup` compiles.
   // Incremental: `u32` outer + `u16` inner (was u32). FlowLog-parity —
   // `u16` is a Timestamp and halves inner-scope trace-entry size. 65535
   // iterations is plenty for practical fixpoint convergence; DD panics
   // cleanly if exceeded (unlike silent wraparound).
   let inner_iter_ty = quote! { u16 };
   // Outer-scope non-recursive rules emitted BEFORE scope.iterative so their
   // results become part of the seed entered into the iterative loop.
   // Body assembly:
   //   1. Non-recursive rules emitted BEFORE scope.iterative — their output
   //      becomes part of the seed entered into the loop.
   //   2. EDB arrangements (outer_arrs) ALSO before scope.iterative — built
   //      once, amortized across all iterations via `.enter(inner)`.
   //   3. Inside scope.iterative: enter seeds, create Variables, shadow
   //      __<rel>_coll, enter outer arrangement handles (outer_arr_enters),
   //      arrange recursive IDBs (inner_arrs), init accumulators, run rules,
   //      bind-and-leave.
   let body = quote! {
      #non_rec_body
      #pre_loop_threshold
      #outer_arrs
      let #leaved_dest_tuple = scope.iterative::<#inner_iter_ty, _, _>(|inner| {
         #inner_body_only_bindings
         #inner_dyn_seed_bindings
         #var_decls
         #shadow_bindings
         #outer_arr_enters
         #inner_arrs
         #rules_ts
         #bind_leave_tuple
      });
      #outer_reassigns
   };
   rules_hoists.extend(non_rec_hoists);
   (body, rules_hoists)
}
