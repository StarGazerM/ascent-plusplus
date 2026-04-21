//! Small leaf-level codegen helpers shared across the lowering modules.
//!
//! Everything here is a pure function over MIR/syn inputs — no closure-body
//! or SCC-wiring state. Kept together so `rule_body`, `scc`, `run_body`,
//! `session`, and `lib` can each import a single `crate::utils::*` instead of
//! reaching cross-module for a one-liner.

use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::Expr;

use ascent_mir::RelationIdentity;
use ascent_mir::utils::{expr_to_ident, tuple};

use crate::dfg;

/// Singleton-unit seed collection. Used when a rule body starts with something
/// that needs an upstream Collection to `.flat_map`/`.filter` over but the
/// first item is a `Cond`, `Generator`, or a fact with a head-only projection.
///
/// We require a hoisted binding `__unit_scope` to exist in the outer-closure
/// (created via `scope.new_collection_from(once(()))`). Inside iterative
/// scopes a matching `__unit_inner` binding — created by `.enter(inner)` —
/// is expected. `scope_ident` picks which one. Both are cheap `.clone()`-able.
pub(crate) fn unit_seed(scope_ident: &Ident) -> TokenStream {
   let binding = Ident::new(&format!("__unit_{}", scope_ident), scope_ident.span());
   quote! { #binding.clone() }
}

/// Emit the "materialize current state" step for a relation's accumulator.
/// - Set relations: `.distinct()` — set semantics.
/// - Lattice relations: `.reduce(…)` — group by non-lattice columns, fold
///   contributions via `Lattice::join_mut`, emit one `(K…, L)` per key with
///   the current lattice value.
///
/// `coll_expr` evaluates to the `Collection<(K_cols..., L), isize>` holding
/// raw rule contributions. Output is a `Collection` of the same shape with
/// consolidated-per-key rows.
pub(crate) fn materialize_rel(rel: &RelationIdentity, coll_expr: TokenStream, is_batch: bool) -> TokenStream {
   if !rel.is_lattice {
      if is_batch {
         // Batch path: Present-diff set relation. `threshold_semigroup`
         // emits each key once, the first time it is seen — the canonical
         // DD datalog pattern (graspan1.rs). Removing the `.arrange_by_self`
         // prelude saves one DD arrangement operator per relation.
         return dfg::lower(&dfg::DataflowOp::ThresholdSemigroupPresent, coll_expr);
      }
      // Session path: isize-diff set relation. Emit diff 1 iff net
      // multiplicity > 0. The `> 0` check (not just `!= 0`) is required
      // for antijoin correctness — antijoin's per-timestamp diffs can
      // transiently cancel without being dropped; `reduce` with explicit
      // net check stays correct where `.distinct()` would mis-emit.
      let body = dfg::lower(&dfg::DataflowOp::SetDedupReduce, coll_expr);
      return quote! { { #body } };
   }
   // Lattice relation: non-lattice cols = key, last col = value.
   let arity = rel.field_types.len();
   let key_arity = arity - 1;
   let key_idents: Vec<Ident> =
      (0..key_arity).map(|i| Ident::new(&format!("__k{}", i), Span::call_site())).collect();
   let lat_ident = Ident::new("__l", Span::call_site());

   // Destructure a tuple of arity `arity`:
   let destruct = {
      let mut parts: Vec<TokenStream> = key_idents.iter().map(|i| quote! { #i }).collect();
      parts.push(quote! { #lat_ident });
      tuple_tokens(&parts)
   };

   // Re-key as `(K_tuple, L)`:
   let key_tuple = tuple_of_idents(&key_idents);

   // Reconstruct the flat output tuple after reduce (which gives us
   // `(K_tuple, L_joined)` pairs — we want `(K_cols..., L)`):
   let flat_out_parts: Vec<TokenStream> = key_idents
      .iter()
      .map(|i| quote! { #i })
      .chain(std::iter::once(quote! { __l }))
      .collect();
   let flat_out_tuple = tuple_tokens(&flat_out_parts);

   let last_ty = &rel.field_types[arity - 1];

   // Option-fold the contributions: no `Default` requirement on the lattice
   // type (many lattices — e.g. `Dual<u32>` — don't derive `Default`; we
   // already know there's at least one contribution because `reduce` only
   // fires for keys with non-empty input).
   dfg::lower(
      &dfg::DataflowOp::LatticeReduce {
         destructure: destruct,
         key_tuple,
         lat_ident,
         lat_ty: last_ty.clone(),
         flat_out_tuple,
         is_batch,
      },
      coll_expr,
   )
}

/// Name of the shared arrangement binding for a given `IrRelation`.
/// Matches `compile_join_clause`'s lookup — both sides must agree.
pub(crate) fn shared_arr_ident(ir_rel: &ascent_mir::IrRelation) -> Ident {
   let base = ir_rel.ir_name();
   Ident::new(&format!("__arr_{}", base), base.span())
}

/// Emits `let __arr_<ir_name> = <coll>.flat_map(|tuple| Some(((key_cols,), (val_cols,)))).arrange_by_key();`
/// for one shared arrangement. Key columns come from `ir_rel.indices` (already
/// sorted ascending → canonical). Value columns are every other position in
/// natural order. Both sides of any later `join_core` must mirror this layout.
pub(crate) fn emit_shared_arrangement_binding(ir_rel: &ascent_mir::IrRelation) -> TokenStream {
   let arr_ident = shared_arr_ident(ir_rel);
   let rel_coll = relation_coll_var(&ir_rel.relation.name);
   let arity = ir_rel.relation.field_types.len();
   let col_idents: Vec<Ident> =
      (0..arity).map(|i| Ident::new(&format!("__c{}", i), Span::call_site())).collect();
   let destruct = tuple_of_idents(&col_idents);
   let key_parts: Vec<TokenStream> =
      ir_rel.indices.iter().map(|&i| {
         let c = &col_idents[i];
         quote! { #c }
      }).collect();
   let val_parts: Vec<TokenStream> =
      (0..arity).filter(|i| !ir_rel.indices.contains(i)).map(|i| {
         let c = &col_idents[i];
         quote! { #c }
      }).collect();
   let rhs = dfg::lower(
      &dfg::DataflowOp::ArrangeByKeyReshape {
         destructure: destruct,
         key_tuple: tuple_tokens(&key_parts),
         val_tuple: tuple_tokens(&val_parts),
      },
      quote! { #rel_coll.clone() },
   );
   quote! { let #arr_ident = #rhs; }
}

pub(crate) fn relation_input_var(name: &Ident) -> Ident { Ident::new(&format!("__{}_in", name), name.span()) }
pub(crate) fn relation_coll_var(name: &Ident) -> Ident { Ident::new(&format!("__{}_coll", name), name.span()) }
pub(crate) fn relation_sink_outer(name: &Ident) -> Ident { Ident::new(&format!("__{}_sink", name), name.span()) }
pub(crate) fn relation_sink_inner(name: &Ident) -> Ident { Ident::new(&format!("__{}_sink_inner", name), name.span()) }

pub(crate) fn tuple_of_idents(idents: &[Ident]) -> TokenStream {
   let exprs: Vec<Expr> = idents.iter().map(|i| syn::parse_quote!(#i)).collect();
   let t = tuple(&exprs);
   quote! { #t }
}

pub(crate) fn tuple_tokens(entries: &[TokenStream]) -> TokenStream {
   if entries.len() == 1 {
      let e = &entries[0];
      quote! { ( #e , ) }
   } else {
      quote! { ( #(#entries),* ) }
   }
}

pub(crate) fn tuple_tokens_as_pattern(entries: &[TokenStream]) -> TokenStream { tuple_tokens(entries) }

/// `(x.clone(), y.clone(), …)` — use when every ident might be a ref that
/// needs to escape a closure that borrows from its frame.
///
/// IMPORTANT: uses `ToOwned::to_owned` rather than `.clone()`. When `v` is a
/// reference (which it typically is in our user-visible contexts), Rust's
/// method resolution picks `&T: Clone` which returns another `&T` — the
/// reference doesn't escape its closure frame. `ToOwned::to_owned` on `&T`
/// is unambiguously `<T as Clone>::clone(&*v)` which returns the owned `T`.
pub(crate) fn clone_all_tuple(idents: &[Ident]) -> TokenStream {
   let exprs: Vec<TokenStream> =
      idents.iter().map(|i| quote! { ::std::borrow::ToOwned::to_owned(#i) }).collect();
   if exprs.len() == 1 {
      let e = &exprs[0];
      quote! { (#e,) }
   } else {
      quote! { ( #(#exprs),* ) }
   }
}

/// Build a head-tuple expression from the rule's head args. Simple-ident
/// args get wrapped with `Convert::convert` so refs auto-clone into owned
/// values — the same trick the batch backend uses. Non-ident exprs
/// (`x + 1`, `f(y)`, literals) pass through unchanged because user code can
/// deref explicitly or return owned values directly.
pub(crate) fn build_expr_tuple(args: &[Expr]) -> TokenStream {
   // Plain idents auto-clone via `Convert::convert` — `r(x, y)` at the head
   // works even when `x`, `y` are refs. Complex expressions pass through
   // because wrapping them triggers type-inference deadlocks with
   // context-dependent calls like `xs[..i].into()` (the target type comes
   // from the tuple field, but `.into()` needs it to pick a return type).
   let converted: Vec<Expr> = args
      .iter()
      .map(|a| {
         if let Some(ident) = expr_to_ident(a) {
            syn::parse_quote! { ::ascent::internal::Convert::convert(#ident) }
         } else {
            a.clone()
         }
      })
      .collect();
   let t = tuple(&converted);
   quote! { #t }
}

/// Detect Ascent's negation aggregator so we can lower `!R(args)` (or an
/// explicit `agg () = not() in R(args)`) to `.antijoin` rather than going
/// through the full `Reduce`-based agg machinery (Phase 5).
///
/// Accepts any path whose last segment is `not`. The fully-qualified HIR
/// desugaring emits `::ascent::aggregators::not`; user code under
/// `use ascent::aggregators::*` can write just `not`.
pub(crate) fn is_not_aggregator(expr: &syn::Expr) -> bool {
   if let syn::Expr::Path(p) = expr {
      return p.path.segments.last().map_or(false, |s| s.ident == "not");
   }
   false
}
