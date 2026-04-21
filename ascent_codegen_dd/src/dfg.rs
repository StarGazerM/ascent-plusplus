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
   /// Set-dedup via `.threshold(|_, w| if *w > 0 { 1 } else { 0 })` on
   /// `isize` diffs. FlowLog-parity: one operator (`KeySpine`-backed) that
   /// maintains set semantics. Used for session-path set relations at
   /// bind-leave and sink-attach.
   ///
   /// Why `.threshold` over `.map → .reduce → .map`: DD 0.20's `.threshold`
   /// is a specialized `reduce_abelian` with a *key-only* trace
   /// (`KeySpine`) — no value payload, smaller per-record, cheaper
   /// lookups. Our prior 3-op chain built a `(K, ())` key-value trace
   /// (`ValSpine`) and paid for wrap/unwrap map operators around it.
   ///
   /// Why not `.distinct()`: `.distinct()` ignores multiplicity sign and
   /// always emits 1 when a key is present — wrong for antijoin
   /// correctness, where per-timestamp diffs can transiently go negative.
   /// The `> 0` net check here handles that window.
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

/// `let (var, read): (…) = Variable::new(inner, Product::new(Default::default(), 1));`
/// — the recursive-variable declaration at the top of an iterative scope.
/// Two flavors: incremental (isize diffs, `Variable`) vs batch (BatchDiff,
/// `SemigroupVariable`).
///
/// DD 0.20 API: returns `(Variable, Collection)` — no more `Deref`. Caller
/// reads the companion `read` collection for body shadow bindings.
pub(crate) fn lower_variable_decl(var: &Ident, read: &Ident, tup_ty: &TokenStream, is_batch: bool) -> TokenStream {
   if is_batch {
      quote! {
         let (#var, #read): (
            ::ascent::dd::SemigroupVariable<_, #tup_ty, ::ascent::dd::BatchDiff>,
            ::ascent::dd::differential_dataflow::VecCollection<_, #tup_ty, ::ascent::dd::BatchDiff>,
         ) = ::ascent::dd::SemigroupVariable::new(inner, Product::new(Default::default(), 1));
      }
   } else {
      // FlowLog parity: `i32` diff (was `isize`). Halves per-trace-entry size
      // across all arrangements downstream. i32 is Abelian + Semigroup so
      // Variable::new is fine.
      quote! {
         let (#var, #read): (
            Variable<_, #tup_ty, i32>,
            ::ascent::dd::differential_dataflow::VecCollection<_, #tup_ty, i32>,
         ) = Variable::new(inner, Product::new(Default::default(), 1));
      }
   }
}

/// Bind-leave block for a recursive variable:
/// `{ let next = <materialized>; var.set(next.clone()); next.leave() }`.
/// One per dynamic relation in a looping SCC. `materialized` is the output
/// of `materialize_rel` (concat-chain wrapped in threshold/reduce).
pub(crate) fn lower_bind_leave(var: &Ident, next: &Ident, materialized: TokenStream) -> TokenStream {
   quote! {
      {
         let #next = #materialized;
         #var.set(#next.clone());
         #next.leave()
      }
   }
}

/// `agg pat = <fn>(bound_args...) in rel(rel_args...)` body item.
/// Emits the 4-stage aggregator pipeline:
///   1. `__agg_input`: flat_map-filter reshape rel tuples as
///      `(GroupKey, FullRelTuple)` so set-semantic consolidation won't
///      collapse distinct rel rows sharing bound-arg values.
///   2. `__agg_result`: `reduce` per key — positive-diff input rows
///      feed the user's aggregator function; each yielded row is lifted
///      through `AggResult::into_dd` (e.g. `f64` → `OrderedFloat`).
///   3. `__l`: key the accum by the same (shared cols, computed keys)
///      layout as `__agg_input`.
///   4. `__l.join(__agg_result)`: combine, re-bind `pat` to the
///      aggregator output, emit `out_tuple`.
///
/// Aggregators are stratified (MIR invariant), so `rel_coll` is a
/// sealed collection from a prior SCC — `reduce` runs against stable
/// input without loop-iteration concerns.
#[allow(clippy::too_many_arguments)]
pub(crate) fn lower_agg_clause(
   accum: TokenStream, destr_accum: TokenStream, accum_key_tuple: TokenStream, accum_vals_cloned: TokenStream,
   rel_coll: &Ident, agg_destruct: TokenStream, filter_pred: TokenStream, clause_key_tuple: TokenStream,
   rel_full_value: TokenStream, ref_tuple_for_agg: TokenStream, aggregator: TokenStream,
   join_key_pattern: TokenStream, accum_vals_tuple: TokenStream, pat: TokenStream, out_tuple: TokenStream,
) -> TokenStream {
   quote! {
      {
         // DD 0.20: `reduce` is inherent on Collection — no trait import.
         // Arrange rel_coll as Collection<(GroupKey, FullRelTuple)>.
         let __agg_input = #rel_coll.clone().flat_map(move |#agg_destruct| {
            if #filter_pred {
               ::std::option::Option::Some((#clause_key_tuple, #rel_full_value))
            } else {
               ::std::option::Option::None
            }
         });
         // reduce: per-key, feed the consolidated values to the user's
         // aggregator function and emit each yielded result. `AggResult`
         // lifts non-`Ord` outputs (e.g. `f64` → `OrderedFloat<f64>`) so
         // they satisfy DD's `Data` bound; we unwrap on the other side.
         let __agg_result = __agg_input.reduce(|_k, __input, __output| {
            let __items = __input.iter()
               .filter(|(_, __d)| *__d > 0)
               .map(|(__v, _)| #ref_tuple_for_agg);
            for __res in (#aggregator)(__items) {
               __output.push((::ascent::dd::AggResult::into_dd(__res), 1));
            }
         });
         let __l = (#accum).map(move |__b| {
            #destr_accum
            (#accum_key_tuple, #accum_vals_cloned)
         });
         __l.join(__agg_result).map(|(#join_key_pattern, (#accum_vals_tuple, __agg_out))| {
            // `__agg_out` is the DD-storable form. For `Ord` agg outputs
            // (integers, strings, tuples of these) this is the original type.
            // For `f64` / `f32` it's `OrderedFloat<T>` — user code sees the
            // wrapped form and should `.into_inner()` / `*v` to unwrap.
            let #pat = __agg_out;
            #out_tuple
         })
      }
   }
}

/// `cond` body item: filter the accum by a user predicate. Bound vars
/// are destructured per their `VarKind` into scope before `cond`
/// evaluates.
pub(crate) fn lower_filter(accum: TokenStream, destr: TokenStream, cond: TokenStream) -> TokenStream {
   quote! {
      (#accum).filter(move |__b| {
         #destr
         #cond
      })
   }
}

/// `let pat = expr` body item: per input row, evaluate `expr_as_owned`
/// once (with bound-var destructure already in scope), bind `pat` to its
/// value, and emit a tuple extended with the new bindings.
///
/// `__pat_val` is bound BEFORE re-destructuring `__b` as the prior tuple —
/// that lets `expr_as_owned` still hold refs into `__b` without conflict.
pub(crate) fn lower_let(
   accum: TokenStream, destr: TokenStream, expr_as_owned: TokenStream, prior_tuple: TokenStream,
   pat: TokenStream, new_tuple: TokenStream,
) -> TokenStream {
   quote! {
      (#accum).map(move |__b| {
         let __pat_val = { #destr #expr_as_owned };
         let #prior_tuple = __b;
         let #pat = __pat_val;
         #new_tuple
      })
   }
}

/// `if let pat = expr` body item: retain each input row only when the
/// pattern match succeeds, emitting the tuple extended with pattern vars.
pub(crate) fn lower_if_let(
   accum: TokenStream, destr: TokenStream, pat: TokenStream, expr: TokenStream, owned_tuple: TokenStream,
) -> TokenStream {
   quote! {
      (#accum).flat_map(move |__b| {
         #destr
         if let #pat = (#expr) {
            ::std::option::Option::Some(#owned_tuple)
         } else {
            ::std::option::Option::None
         }
      })
   }
}

/// `for pat in expr` generator: per-input-row, iterate `expr`, bind `pat`,
/// push `out_tuple` per item. Expands the accum stream with one output
/// row per (accum × iter-item) pair.
///
/// Uses an inline `Vec<NewTup>` per input — keeps `__b`'s borrow alive
/// across the iteration without closure-capture gymnastics that a nested
/// `.map` would require.
pub(crate) fn lower_generator(
   accum: TokenStream, destr: TokenStream, expr: TokenStream, pat: TokenStream, out_tuple: TokenStream,
) -> TokenStream {
   quote! {
      (#accum).flat_map(move |__b| {
         let mut __out: ::std::vec::Vec<_> = ::std::vec::Vec::new();
         #destr
         for __item in (#expr) {
            let #pat = __item;
            __out.push(#out_tuple);
         }
         __out
      })
   }
}

/// `(seed).map(move |()| head_tuple)` — body-less rule fires once per
/// `()`-unit seed tuple, projecting to the head tuple. Used for pure-fact
/// rules whose body is empty.
pub(crate) fn lower_unit_seed_map(seed: TokenStream, head_tuple: TokenStream) -> TokenStream {
   quote! { (#seed).map(move |()| #head_tuple) }
}

/// Full antijoin emission: the `!rel(args)` negated-clause pattern.
/// Builds `__r_keys` (keys that must be absent from accum output), keys
/// the accum by the same (shared cols, computed keys) layout, applies
/// `.antijoin`, and projects back to the bound-vars tuple so subsequent
/// clauses see the original binding shape.
///
/// This is a negation-specific shape rather than a generic DataflowOp:
/// the r_keys side produces only keys (no value reshape), and the
/// output projects to `accum_bound_tuple` (user-var layout) rather
/// than a `(key, val)` pair. The pipeline is 3 operators fused into
/// one block.
#[allow(clippy::too_many_arguments)]
pub(crate) fn lower_antijoin(
   accum: TokenStream, destr_accum: TokenStream, accum_key_tuple: TokenStream, accum_vals_cloned: TokenStream,
   rel_coll: &Ident, destruct: TokenStream, filter_pred: TokenStream, clause_key_tuple: TokenStream,
   join_key_pattern: TokenStream, accum_vals_tuple: TokenStream, accum_bound_tuple: TokenStream,
) -> TokenStream {
   quote! {
      {
         let __r_keys = #rel_coll.clone().flat_map(move |#destruct| {
            if #filter_pred {
               ::std::option::Option::Some(#clause_key_tuple)
            } else {
               ::std::option::Option::None
            }
         });
         let __accum_keyed = (#accum).map(move |__b| {
            #destr_accum
            (#accum_key_tuple, #accum_vals_cloned)
         });
         __accum_keyed.antijoin(__r_keys).map(|(#join_key_pattern, #accum_vals_tuple)| #accum_bound_tuple)
      }
   }
}

/// `sink.attach(&(expr), probe)` (incremental) or
/// `sink.attach(sealer.worker_index(), &(expr), probe)` (batch).
/// Wires the output Collection to its Sink and the progress probe.
///
/// Batch passes the per-worker index so each worker writes to its own
/// pre-allocated `Mutex<Vec<T>>` slot (see `BatchSink`) — avoids Mutex
/// contention at high worker counts.
pub(crate) fn lower_sink_attach(
   sink_id: TokenStream, final_expr: TokenStream, probe_expr: TokenStream, worker_index_expr: TokenStream,
) -> TokenStream {
   // Both batch and inc use per-worker slots (perf profiling showed the
   // single Arc<Mutex<Vec>> caused 250× more context-switches than FlowLog
   // on CSPA httpd @ 24 workers — see Sink/BatchSink doc comments).
   // `worker_index_expr` is the expression yielding this worker's index:
   //   - inc `run()` / batch `run()`: `sealer.worker_index()`
   //   - Session mode: literal `0` (single-worker).
   quote! { #sink_id.attach(#worker_index_expr, &(#final_expr), #probe_expr); }
}

/// `let __unit_scope = …;` — singleton `()`-row Collection seeded at the
/// outer scope. Used downstream by rules whose body has no clause (pure
/// generator/cond/fact-first) and need an upstream to `.flat_map` over.
///
/// Two flavors:
///   - incremental (`u32`/`isize`): uses `new_collection_from(once(()))`
///     then advances to 1 so the capability releases past 0 — otherwise
///     probes that bypass `commit()` stall at frontier [0].
///   - batch (`()`/`BatchDiff`): uses `new_collection_from_raw` since
///     `new_collection_from` is hardcoded to `isize`, and Present-diff
///     inputs need the explicit `BATCH_DIFF_ONE` weight. No `advance_to`
///     on a `()` timestamp.
pub(crate) fn lower_unit_scope_decl(is_batch: bool) -> TokenStream {
   if is_batch {
      quote! {
         let __unit_scope: ::ascent::dd::differential_dataflow::VecCollection<_, (), ::ascent::dd::BatchDiff> = {
            use ::ascent::dd::differential_dataflow::input::Input;
            let (mut __s, __c) = scope.new_collection_from_raw::<(), ::ascent::dd::BatchDiff, _>(
               ::std::iter::once(((), (), ::ascent::dd::BATCH_DIFF_ONE))
            );
            __s.flush();
            __c
         };
      }
   } else {
      // FlowLog parity: `i32` diff (was `isize`). `new_collection_from` is
      // hardcoded to `isize`, so we go through `new_collection_from_raw`
      // with an explicit `(data, time, diff)` initial tuple. Incremental
      // outer timestamp is `u32` → middle slot is `0u32` at seed time.
      quote! {
         let __unit_scope: ::ascent::dd::differential_dataflow::VecCollection<_, (), i32> = {
            use ::ascent::dd::differential_dataflow::input::Input;
            let (mut __s, __c) = scope.new_collection_from_raw::<(), i32, _>(
               ::std::iter::once(((), 0u32, 1i32))
            );
            __s.advance_to(1);
            __s.flush();
            __c
         };
      }
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
         (#upstream).threshold(|_, __w: &i32| if *__w > 0 { 1i32 } else { 0 })
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
