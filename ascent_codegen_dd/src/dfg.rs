//! Dataflow-graph IR (MLIR-style lowering layer).
//!
//! A thin, additive IR that sits between MIR and raw `quote!` TokenStreams.
//! Each `DataflowOp` variant corresponds to exactly one DD operator. Codegen
//! builds small `DataflowOp` values and calls `lower` to emit tokens — same
//! text as the old inline `quote!`, but now the planning decision (which op
//! to use) is separate from the emission detail (what tokens that op prints).
//!
//! Progression: expansion is additive. Today's variants cover the per-row
//! projection path (`MapProject` / `FilterMapProject`), the bind-leave
//! concat chain (`ConcatChain`), and the seed-entry pattern
//! (`ConsolidateEnter`). Future slices add Join / Reduce / Leave paths and
//! eventually per-SCC fn extraction.

use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::Type;

/// A single dataflow operator applied to an upstream Collection expression.
///
/// Invariant: `lower(op, upstream)` produces an expression whose type is
/// the same `Collection<G, _, _>` shape as `upstream` — modulo row-type
/// transforms declared by the variant.
pub(crate) enum DataflowOp {
   /// Per-row total projection. Emits `.map(|destructure| -> RowTy { rebinds; head })`.
   /// Total = no filtering; every input row produces exactly one output row.
   MapProject {
      destructure: TokenStream,
      row_ty: Type,
      var_rebinds: Vec<TokenStream>,
      head_expr: TokenStream,
   },
   /// Per-row filter-then-project. Emits
   /// `.flat_map(|destructure| -> Option<RowTy> { rebinds; if pred { Some(head) } else { None } })`.
   /// Used when any filter clause can reject a row.
   FilterMapProject {
      destructure: TokenStream,
      row_ty: Type,
      var_rebinds: Vec<TokenStream>,
      filters: Vec<TokenStream>,
      head_expr: TokenStream,
   },
   /// `.consolidate().enter(scope)` — the FlowLog-style seed entry pattern.
   /// Used at the top of a `scope.iterative(|inner| …)` body to lift an
   /// outer collection into the inner scope once, pre-consolidated so
   /// downstream operators see a canonical trace.
   ConsolidateEnter { inner_scope: Ident },
   /// `.enter(scope)` — lift a collection or arrangement trace handle
   /// into a nested timely scope without re-consolidating. Used for
   /// arrangement traces built outside the iterative loop and for the
   /// shared `__unit_scope` singleton.
   Enter { scope: Ident },
   /// `.flat_map(|destructure| Some((key_tuple, val_tuple))).arrange_by_key()`
   /// — the shared-arrangement pattern. Reshapes each row into a
   /// `(key_cols_tuple, val_cols_tuple)` pair (no filtering) and builds
   /// an `Arranged` trace. One emission per unique `(rel, key_indices)`
   /// in an SCC.
   ArrangeByKeyReshape {
      destructure: TokenStream,
      key_tuple: TokenStream,
      val_tuple: TokenStream,
   },
   /// Set-dedup via reduce on `isize` diffs. Emits
   /// `.map(|x| (x, ())).reduce(net-positive -> ((), 1)).map(|(k, ())| k)`.
   /// Used for session-path set relations at bind-leave and sink-attach.
   ///
   /// Why not `.distinct()`: the `> 0` net check (not `!= 0`) is required
   /// for antijoin correctness — antijoin's per-timestamp diffs can
   /// transiently cancel without being dropped; `.distinct()` would
   /// mis-emit in that window. See materialize_rel note in lib.rs.
   SetDedupReduce,
   /// Batch-path set-dedup via `threshold_semigroup`. Emits each key
   /// the first time it's seen — insert-only Present-diff semantics
   /// (graspan1.rs canonical DD datalog pattern).
   ///
   /// DD 0.20: `ThresholdTotal::threshold_semigroup` is impl'd directly
   /// on `VecCollection`, so no `.arrange_by_self()` prelude is needed.
   ThresholdSemigroupPresent,
   /// Lattice-aware reduce: group by `key_tuple`, fold contributions
   /// via `<Lat as Lattice>::join_mut`, emit one `(key, joined_lat)` per
   /// key. Caller supplies:
   ///   - `destructure`: pattern matching the input tuple `(k0, k1, …, l)`
   ///   - `key_tuple`: rebuilt key tuple expression from destructured
   ///     idents (used both as `.map` output and pattern after reduce)
   ///   - `lat_ident`: name of the lattice value after destructure
   ///   - `lat_ty`: concrete lattice type (for trait-qualified `join_mut`
   ///     + the Option accumulator)
   ///   - `flat_out_tuple`: output tuple shape flattening `(key, lat)`
   ///     back to `(k0, k1, …, l)`
   ///
   /// Skips contributions with `diff <= 0` (lattice join is idempotent,
   /// so only positive contributions participate).
   LatticeReduce {
      destructure: TokenStream,
      key_tuple: TokenStream,
      lat_ident: Ident,
      lat_ty: Type,
      flat_out_tuple: TokenStream,
   },
}

/// Whether the body of a `join_core` closure emits the fused head tuple
/// (terminal clause of a simple rule) or the pass-through bound-vars
/// tuple that downstream clauses will consume.
pub(crate) enum JoinEmit {
   /// Non-fused: clone key/lv/rv (refs from join_core can't escape), emit
   /// the bound-vars tuple `out_tuple`. Downstream `compile_*` step wraps
   /// it in another projection.
   PassThrough { out_tuple: TokenStream },
   /// Fused: terminal clause — emit the head projection directly inside
   /// the closure. Uses refs (no clones) since values don't outlive the
   /// closure frame.
   Fused {
      head_row_ty: TokenStream,
      head_expr_tuple: TokenStream,
   },
}

/// `join_core` invocation binding the RHS against an LHS arrangement.
/// Caller supplies the arrangement expressions (shared-arr ident, or a
/// `.map(...).arrange_by_key()` that they emit separately) and the
/// closure destructure layout. `lower_join_core` returns the full
/// `lhs.clone().join_core(rhs, |__k, __lv, __rv| { … })` token tree.
pub(crate) struct JoinCore {
   pub lhs_arr_expr: TokenStream,
   pub rhs_arr_expr: TokenStream,
   pub accum_vals_tuple: TokenStream,
   pub new_vars_tuple: TokenStream,
   pub join_key_pattern: TokenStream,
   pub emit: JoinEmit,
}

pub(crate) fn lower_join_core(op: &JoinCore) -> TokenStream {
   let JoinCore { lhs_arr_expr, rhs_arr_expr, accum_vals_tuple, new_vars_tuple, join_key_pattern, emit } = op;
   let closure_body = match emit {
      JoinEmit::PassThrough { out_tuple } => quote! {
         let (#accum_vals_tuple, #new_vars_tuple) = (__lv.clone(), __rv.clone());
         let #join_key_pattern = __k.clone();
         ::std::option::Option::Some::<_>(#out_tuple)
      },
      JoinEmit::Fused { head_row_ty, head_expr_tuple } => quote! {
         let (#accum_vals_tuple, #new_vars_tuple) = (__lv, __rv);
         let #join_key_pattern = __k;
         ::std::option::Option::Some::<#head_row_ty>(#head_expr_tuple)
      },
   };
   quote! {
      #lhs_arr_expr.join_core(#rhs_arr_expr, |__k, __lv, __rv| {
         #closure_body
      })
   }
}

/// `<accum>.map(move |__b| { destr_accum; (key_tuple, vals_cloned) }).arrange_by_key()`
/// — LHS arrangement built per-rule (carries prior-join state, can't be
/// shared). Emitted when the rule isn't LHS-shared-eligible.
pub(crate) fn lower_map_arrange_lhs(
   accum: TokenStream, destr_accum: TokenStream, key_tuple: TokenStream, vals_cloned: TokenStream,
) -> TokenStream {
   quote! {
      (#accum).map(move |__b| {
         #destr_accum
         (#key_tuple, #vals_cloned)
      }).arrange_by_key()
   }
}

/// `<rel_coll>.clone().flat_map(move |destruct| if pred { Some((key, val)) } else { None }).arrange_by_key()`
/// — per-clause RHS arrangement. Used when the clause has filters that
/// the shared arrangement can't bake in, or no indices (no-key path).
pub(crate) fn lower_flatmap_filter_arrange(
   rel_coll: &Ident, destruct: TokenStream, filter_pred: TokenStream, key_tuple: TokenStream, val_tuple: TokenStream,
) -> TokenStream {
   quote! {
      #rel_coll.clone().flat_map(move |#destruct| {
         if #filter_pred {
            ::std::option::Option::Some((#key_tuple, #val_tuple))
         } else {
            ::std::option::Option::None
         }
      }).arrange_by_key()
   }
}

/// Lower a single op to tokens, attached to the given upstream expression.
/// Convention: the caller owns cloning discipline — `lower` treats
/// `upstream` as an already-cloneable expression (e.g. `rel_coll.clone()`).
pub(crate) fn lower(op: &DataflowOp, upstream: TokenStream) -> TokenStream {
   match op {
      DataflowOp::MapProject { destructure, row_ty, var_rebinds, head_expr } => quote! {
         #upstream.map(move |#destructure| -> #row_ty {
            #(#var_rebinds)*
            #head_expr
         })
      },
      DataflowOp::FilterMapProject { destructure, row_ty, var_rebinds, filters, head_expr } => {
         let filter_pred = quote! { #( ( #filters ) )&&* };
         quote! {
            #upstream.flat_map(move |#destructure| -> ::std::option::Option<#row_ty> {
               #(#var_rebinds)*
               if #filter_pred { ::std::option::Option::Some(#head_expr) } else { ::std::option::Option::None }
            })
         }
      },
      DataflowOp::ConsolidateEnter { inner_scope } => quote! {
         #upstream.consolidate().enter(#inner_scope)
      },
      DataflowOp::Enter { scope } => quote! {
         #upstream.enter(#scope)
      },
      DataflowOp::ArrangeByKeyReshape { destructure, key_tuple, val_tuple } => quote! {
         #upstream.flat_map(move |#destructure| {
            ::std::option::Option::Some((#key_tuple, #val_tuple))
         }).arrange_by_key()
      },
      DataflowOp::SetDedupReduce => quote! {
         (#upstream).map(|__x| (__x, ()))
            .reduce(|_k, __input, __output| {
               let __net: isize = __input.iter().map(|(_, d)| *d).sum();
               if __net > 0 { __output.push(((), 1)); }
            })
            .map(|(__k, ())| __k)
      },
      DataflowOp::ThresholdSemigroupPresent => quote! {
         {
            use ::ascent::dd::differential_dataflow::operators::ThresholdTotal;
            (#upstream)
               .threshold_semigroup(|_, _, old: ::std::option::Option<&::ascent::dd::BatchDiff>| {
                  old.is_none().then_some(::ascent::dd::BATCH_DIFF_ONE)
               })
         }
      },
      DataflowOp::LatticeReduce { destructure, key_tuple, lat_ident, lat_ty, flat_out_tuple } => quote! {
         {
            let __keyed = (#upstream).map(|#destructure| (#key_tuple, #lat_ident));
            let __reduced = __keyed.reduce(|_key, __input, __output| {
               let mut __acc: ::std::option::Option<#lat_ty> = ::std::option::Option::None;
               for (__v, __d) in __input.iter() {
                  if *__d > 0 {
                     match &mut __acc {
                        ::std::option::Option::None => { __acc = ::std::option::Option::Some((*__v).clone()); }
                        ::std::option::Option::Some(__a) => {
                           <#lat_ty as ::ascent::Lattice>::join_mut(__a, (*__v).clone());
                        }
                     }
                  }
               }
               if let ::std::option::Option::Some(__l) = __acc {
                  __output.push((__l, 1));
               }
            });
            __reduced.map(|(#key_tuple, __l)| #flat_out_tuple)
         }
      },
   }
}

/// Chain of `.concat(…)` applications over a first source.
///
/// Emits `first.clone().concat(second.clone()).concat(third.clone())…` —
/// the exact shape `compile_looping_scc` builds at bind-leave time when
/// uniting per-rule production collections with the entered seed before
/// threshold.
///
/// This is NOT a `DataflowOp` variant: it takes no upstream (it IS the
/// full expression from `first` onward), and the per-source `.clone()`
/// discipline is load-bearing — callers reuse the rule/seed idents after
/// this expression to feed downstream ops. Keeping it a free fn keeps the
/// cloning explicit in the emission, matching the pre-refactor text.
pub(crate) fn lower_concat_chain(sources: &[Ident]) -> TokenStream {
   assert!(!sources.is_empty(), "concat chain needs ≥1 source");
   let first = &sources[0];
   let rest: Vec<TokenStream> = sources[1..].iter().map(|s| quote! { .concat(#s.clone()) }).collect();
   quote! { #first.clone() #(#rest)* }
}
