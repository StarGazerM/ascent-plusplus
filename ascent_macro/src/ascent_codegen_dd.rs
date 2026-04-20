//! Differential-dataflow codegen backend.
//!
//! Generates code against the runtime exposed at `::ascent::dd` (the
//! `ascent_dd_runtime` crate re-export). The surface shape (`struct` with
//! `pub <rel>: Vec<Tup>` fields, `Default::default()`, `run()`) matches the
//! batch backend so ascent programs are swap-in compatible at the user API.
//!
//! Phase status:
//!   0: struct + noop `run()`                                        (always)
//!   1: non-recursive rules, all-ident clause/head args
//!   2: recursion via DD `Variable` / `scope.iterative`              <-- HERE
//!   3+: conditions / generators / negation / aggregation / lattices
//!
//! When the program is outside the current phase envelope, `run()` falls back
//! to a noop so the struct still compiles — tests for unsupported features
//! stay `#[ignore]`'d rather than block the build.

#![allow(dead_code)]

use itertools::Itertools;
use proc_macro2::{Ident, Span, TokenStream};
use syn::Expr;
use syn::spanned::Spanned;

use crate::ascent_hir::IrAggClause;
use crate::ascent_mir::{AscentMir, MirBodyItem, MirRule, MirScc, mir_rule_summary};
use crate::ascent_syntax::{CondClause, RelationIdentity};
use crate::syn_utils::{expr_get_vars, pattern_get_vars};
use crate::utils::{expr_to_ident, is_wild_card, tuple, tuple_type};

// ---------------------------------------------------------------------------
// Phase envelope check
// ---------------------------------------------------------------------------

/// Returns `None` if the MIR is within the current backend envelope:
/// Phases 1, 2 and 3a are supported (non-recursive + recursive rules, rich
/// clause args — wildcards, literals, repeated vars — and `if`/`let`
/// conditions). Rejects: `if let`, generators, negation, aggregation, and
/// lattice relations.
fn phase1_blocker(mir: &AscentMir) -> Option<String> {
   for scc in &mir.sccs {
      for rule in &scc.rules {
         for bi in &rule.body_items {
            match bi {
               MirBodyItem::Clause(cl) => {
                  // Lattice bodies are supported: rules read the materialized
                  // (key-col-tuple..., lat_val) form as a regular set-diff
                  // Collection. See `phase6_materialize_lattice_rel` for the
                  // reduce step that keeps each key's current join.
                  let _ = cl.rel.relation.is_lattice;
                  let _ = &cl.cond_clauses;
               },
               MirBodyItem::Generator(_) => {
                  // supported in Phase 3c via `.flat_map`
               },
               MirBodyItem::Cond(_) => {
                  // if / let / if let — all supported in Phase 3
               },
               MirBodyItem::Agg(_agg) => {
                  // `not` → `.antijoin` (Phase 3). Other aggregators → `.reduce`
                  // (Phase 5). Both handled in `compile_rule_body_phase1`.
               },
            }
         }
         for hcl in &rule.head_clause {
            // Lattice heads are supported: contributions are concat'd into
            // the head accumulator; the reduce happens at materialization.
            let _ = hcl.rel.is_lattice;
         }
      }
   }
   None
}

// ---------------------------------------------------------------------------
// Rule body → DD Collection expression
// ---------------------------------------------------------------------------

/// Natural type a user expects to see for a bound variable — mirrors batch
/// Ascent semantics:
/// - clauses give the user `&T` (because batch iterates stored tuples by ref),
/// - generators / `let` / `if let` bindings give owned `T` (pattern-bound from
///   the expr's result).
///
/// DD Collections always hold owned tuple elements, so at every user-code
/// destructure site we emit a uniform `let (v0, v1, …) = &__b;` (match-
/// ergonomics → refs) and then shadow `Owned` vars back to owned via
/// `(*v).clone()` (forces owned clone regardless of Copy-ness).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum VarKind {
   Ref,
   Owned,
}

/// Emit `let (v0, v1, ..) = <source>;` (ref-bindings via match-ergonomics)
/// plus a shadow `let v_i = (*v_i).clone();` for each `Owned` var. `<source>`
/// should be an expression that's either `&Tup` (e.g., `&__b`) or a place
/// expression that match-ergonomics can rewrite into ref bindings.
fn emit_user_destructure(
   bound_vars: &[Ident], var_kinds: &[VarKind], source: TokenStream,
) -> TokenStream {
   let bound_tuple = tuple_of_idents(bound_vars);
   let mut out = quote! { let #bound_tuple = #source; };
   for (v, kind) in bound_vars.iter().zip(var_kinds.iter()) {
      if matches!(kind, VarKind::Owned) {
         // `(*v).clone()` forces an owned clone via `<T as Clone>::clone(&*v)`.
         // Works for both `Copy` types (where `*v` copies) and non-Copy Clone
         // types (where `*v` is a place, `.clone()` auto-refs to call Clone).
         out.extend(quote! { let #v = (*#v).clone(); });
      }
   }
   out
}

/// For `if let <pat> = <expr>`: decide the `VarKind` to assign to newly
/// pattern-bound vars. Heuristic: if `<expr>` is just an ident that is
/// already bound as `Ref`, match-ergonomics will walk the reference and
/// pattern-bound vars will come out as `&_` — so they also behave as `Ref`
/// in user code. Otherwise (owned value / computed expr), treat as `Owned`.
fn if_let_new_var_kind(expr: &syn::Expr, bound_vars: &[Ident], var_kinds: &[VarKind]) -> VarKind {
   if let Some(id) = expr_to_ident(expr) {
      for (v, k) in bound_vars.iter().zip(var_kinds.iter()) {
         if v == &id {
            return *k;
         }
      }
   }
   VarKind::Owned
}

/// Produce an expression that yields an OWNED `T` from the ident `v` given
/// its current `VarKind`. Used at internal key/val/output tuple construction
/// sites where we need owned values regardless of how the user accesses `v`.
///
/// - `Ref` (`v: &T`): `ToOwned::to_owned(v)` → `<T as Clone>::clone(v)` → `T`.
/// - `Owned` (`v: T`): `Clone::clone(&v)` — explicit autoref avoids the
///   `<&T as Clone>` ambiguity and forces owned.
fn own_value_expr(v: &Ident, kind: VarKind) -> TokenStream {
   match kind {
      VarKind::Ref => quote! { ::std::borrow::ToOwned::to_owned(#v) },
      VarKind::Owned => quote! { ::std::clone::Clone::clone(&#v) },
   }
}

fn own_values_tuple(idents: &[Ident], kinds: &[VarKind]) -> TokenStream {
   let exprs: Vec<TokenStream> = idents.iter().zip(kinds.iter()).map(|(v, k)| own_value_expr(v, *k)).collect();
   tuple_tokens(&exprs)
}

/// Pick the `VarKind` for each ident in `subset` by looking it up in the
/// parallel `(bound_vars, var_kinds)` arrays. Panics if a `subset` ident
/// isn't found — callers must only pass idents from `bound_vars`.
fn kinds_for(subset: &[Ident], bound_vars: &[Ident], var_kinds: &[VarKind]) -> Vec<VarKind> {
   subset
      .iter()
      .map(|v| {
         let idx = bound_vars.iter().position(|b| b == v).expect("subset var not in bound_vars");
         var_kinds[idx]
      })
      .collect()
}

/// Compile a rule body into a `Collection` of bound-var tuples.
///
/// Returns `(bound_vars, var_kinds, body_expr, hoisted_pre)`.
fn compile_rule_body_phase1(
   rule: &MirRule, scope_ident: &Ident, hoist_counter: &mut usize,
) -> (Vec<Ident>, Vec<VarKind>, TokenStream, TokenStream) {
   let mut bound_vars: Vec<Ident> = Vec::new();
   let mut var_kinds: Vec<VarKind> = Vec::new();
   let mut accum: Option<TokenStream> = None;
   let mut hoisted_pre = TokenStream::new();

   // Track whether current `accum` is a raw first-clause flat_map of a
   // single relation (Some(rel)) or a derived/filtered intermediate (None).
   // If Some, compile_join_clause can reuse the shared arrangement for that
   // relation's current keying instead of building its own LHS arrangement —
   // matches FlowLog's pattern of never duplicating arrangements of the
   // same (rel, keying).
   // For FlowLog-style fusion: detect the "simple N-clause rule" case where
   // all body items are plain Clauses (no cond/let/neg/agg/gen), the rule
   // has exactly ONE head clause, AND the body has ≥2 clauses (otherwise
   // there's no join_core to fuse into — the single-clause path emits a
   // raw flat_map of different shape than the head).
   use crate::ascent_mir::MirBodyItem;
   let clause_count = rule.body_items.iter().filter(|i| matches!(i, MirBodyItem::Clause(_))).count();
   let all_clauses_simple = clause_count >= 2
      && rule.body_items.iter().all(|item| match item {
         MirBodyItem::Clause(cl) => cl.cond_clauses.is_empty(),
         _ => false,
      });
   let last_clause_idx = rule.body_items.iter().rposition(|it| matches!(it, MirBodyItem::Clause(_)));
   let fused_head_expr: Option<(TokenStream, TokenStream)> =
      if all_clauses_simple && rule.head_clause.len() == 1 {
         let hcl = &rule.head_clause[0];
         let row_ty = tuple_type(&hcl.rel.field_types);
         let head_tuple = build_expr_tuple(&hcl.args);
         Some((quote! { #row_ty }, head_tuple))
      } else {
         None
      };
   let mut prior_rel: Option<crate::ascent_mir::MirRelation> = None;
   for (i, item) in rule.body_items.iter().enumerate() {
      let is_last_clause = Some(i) == last_clause_idx;
      let fused_for_this = if is_last_clause { fused_head_expr.clone() } else { None };
      let _ = is_last_clause;
      match item {
         MirBodyItem::Clause(cl) => {
            if accum.is_none() {
               let (new_bound, new_accum) = compile_first_clause(cl);
               var_kinds = vec![VarKind::Ref; new_bound.len()];
               bound_vars = new_bound;
               accum = Some(new_accum);
               prior_rel = if cl.cond_clauses.is_empty() { Some(cl.rel.clone()) } else { None };
            } else {
               let old_len = bound_vars.len();
               let (new_bound, new_accum) = compile_join_clause(
                  &bound_vars, &var_kinds, accum.take().unwrap(), cl,
                  prior_rel.as_ref(), fused_for_this.clone(),
               );
               for _ in old_len..new_bound.len() {
                  var_kinds.push(VarKind::Ref);
               }
               bound_vars = new_bound;
               accum = Some(new_accum);
               // After a join, accum is an intermediate collection, not a raw
               // single-relation flat_map. Reset — the next join must build
               // its own LHS arrangement.
               prior_rel = None;
            }
            for cc in &cl.cond_clauses {
               match cc {
                  CondClause::If(ic) => {
                     accum = Some(emit_filter(accum.take().unwrap(), &bound_vars, &var_kinds, &ic.cond));
                     prior_rel = None;
                  },
                  CondClause::Let(lc) => {
                     let (nb, na) = emit_let(bound_vars.clone(), &var_kinds, accum.take().unwrap(), &lc.pattern, &lc.exp);
                     let let_kind = if_let_new_var_kind(&lc.exp, &bound_vars, &var_kinds);
                     for _ in bound_vars.len()..nb.len() {
                        var_kinds.push(let_kind);
                     }
                     bound_vars = nb;
                     accum = Some(na);
                     prior_rel = None;
                  },
                  CondClause::IfLet(ic) => {
                     let (nb, na) = emit_if_let(bound_vars.clone(), &var_kinds, accum.take().unwrap(), &ic.pattern, &ic.exp);
                     let iflet_kind = if_let_new_var_kind(&ic.exp, &bound_vars, &var_kinds);
                     for _ in bound_vars.len()..nb.len() {
                        var_kinds.push(iflet_kind);
                     }
                     bound_vars = nb;
                     accum = Some(na);
                     prior_rel = None;
                  },
               }
            }
         },
         MirBodyItem::Cond(CondClause::If(ic)) => {
            let a = accum.take().unwrap_or_else(|| unit_seed(scope_ident));
            accum = Some(emit_filter(a, &bound_vars, &var_kinds, &ic.cond));
            prior_rel = None;
         },
         MirBodyItem::Cond(CondClause::Let(lc)) => {
            let a = accum.take().unwrap_or_else(|| unit_seed(scope_ident));
            let (nb, na) = emit_let(bound_vars.clone(), &var_kinds, a, &lc.pattern, &lc.exp);
            let let_kind = if_let_new_var_kind(&lc.exp, &bound_vars, &var_kinds);
            for _ in bound_vars.len()..nb.len() {
               var_kinds.push(let_kind);
            }
            bound_vars = nb;
            accum = Some(na);
            prior_rel = None;
         },
         MirBodyItem::Agg(agg) => {
            if is_not_aggregator(&agg.aggregator) {
               let a = accum.take().expect("negation before first clause not supported");
               accum = Some(emit_negation(a, &bound_vars, &var_kinds, &agg.rel.relation.name, &agg.rel_args));
            } else {
               let a = accum.take().unwrap_or_else(|| unit_seed(scope_ident));
               let (nb, nk, na) = emit_agg(a, &bound_vars, &var_kinds, agg);
               bound_vars = nb;
               var_kinds = nk;
               accum = Some(na);
            }
            prior_rel = None;
         },
         MirBodyItem::Cond(CondClause::IfLet(ic)) => {
            let a = accum.take().unwrap_or_else(|| unit_seed(scope_ident));
            let iflet_kind = if_let_new_var_kind(&ic.exp, &bound_vars, &var_kinds);
            let (nb, na) = emit_if_let(bound_vars.clone(), &var_kinds, a, &ic.pattern, &ic.exp);
            for _ in bound_vars.len()..nb.len() {
               var_kinds.push(iflet_kind);
            }
            bound_vars = nb;
            accum = Some(na);
            prior_rel = None;
         },
         MirBodyItem::Generator(gen) => {
            let prior_len = bound_vars.len();
            let seed = accum.take().unwrap_or_else(|| unit_seed(scope_ident));
            // Hoist the generator's expression into an owned `Vec` *outside*
            // the `execute_batch` closure if it doesn't reference any prior
            // bound var. DD operator closures must be `'static`, which rules
            // out capturing non-'static borrows like `edges: &'a [...]` from
            // an enclosing fn. Pre-collecting to `Vec<_>` sidesteps that.
            let captures_bound = {
               let refd = expr_get_vars(&gen.expr);
               refd.iter().any(|r| bound_vars.iter().any(|b| b == r))
            };
            let gen_expr: syn::Expr = if captures_bound {
               gen.expr.clone()
            } else {
               *hoist_counter += 1;
               let hoist_ident = Ident::new(&format!("__hoist_gen_{}", hoist_counter), Span::call_site());
               let expr = &gen.expr;
               // Force OWNED items in the hoisted Vec regardless of whether
               // the user's iterator yields `&T` (`r.iter()`) or `T`
               // (`r.iter().cloned()`, `0..n`, …). `collect_owned` uses
               // `Borrow<T>` so both `Item = &T` and `Item = T` resolve to
               // an owned `T`, matching batch Ascent's generator convention
               // and avoiding non-'static refs in the DD Collection.
               hoisted_pre.extend(quote! {
                  let #hoist_ident = ::ascent::dd::collect_owned::<_, _>(#expr);
               });
               // `.iter().cloned()` on `Vec<T>` yields owned `T` per call.
               syn::parse_quote! { #hoist_ident.iter().cloned() }
            };
            let (nb, na) = emit_generator(bound_vars.clone(), &var_kinds, seed, &gen.pattern, &gen_expr);
            // Generator pattern bindings are Owned-natural (owned items via
            // `.iter().cloned()` from the hoisted Vec<T>, or direct owned
            // from non-hoisted inline iters like `0..n`).
            for _ in prior_len..nb.len() {
               var_kinds.push(VarKind::Owned);
            }
            bound_vars = nb;
            accum = Some(na);
            prior_rel = None;
         },
      }
   }

   (bound_vars, var_kinds, accum.expect("rule has no body clauses"), hoisted_pre)
}

/// Internal shape describing how to interpret one column of a clause.
struct ColPlan {
   /// Name to bind in the destructure pattern; `None` means wildcard.
   pat_ident: Option<Ident>,
   /// If `Some`, this column contributes a variable (var_name, col_ident) that is either
   /// a new binding or shares with a prior bound var.
   var_binding: Option<(Ident, Ident)>,
   /// If `Some`, a boolean filter token to apply at destructure time.
   filter: Option<TokenStream>,
}

/// Walk a clause's args. Result distinguishes:
/// - `shared_vars`: prior-bound idents appearing as clause args (direct join key).
/// - `new_vars`: fresh idents the clause introduces.
/// - `computed_keys`: expressions that reference one or more prior-bound vars —
///   evaluated on the accum side, matched against the raw column on the clause
///   side. This is how `fac(x - 1, sub1fac)` joins correctly.
/// - `filters`: closed-form equality checks (literal/expr that doesn't touch
///   any bound var) applied inside the clause's `flat_map`.
fn plan_clause(
   cl_args: &[syn::Expr], prior_bound: &[Ident],
) -> (
   Vec<ColPlan>,
   Vec<(Ident, Ident)>,    // shared_vars: (prior_bound var, clause col ident)
   Vec<(Ident, Ident)>,    // new_vars
   Vec<TokenStream>,       // filters (no prior-bound refs)
   Vec<(syn::Expr, Ident)>, // computed_keys: (accum-side expression, clause-side col ident)
) {
   let mut plans: Vec<ColPlan> = Vec::with_capacity(cl_args.len());
   let mut seen: std::collections::HashMap<String, Ident> = Default::default();
   let mut shared_in_clause_order: Vec<(Ident, Ident)> = Vec::new();
   let mut new_in_clause_order: Vec<(Ident, Ident)> = Vec::new();
   let mut filters: Vec<TokenStream> = Vec::new();
   let mut computed_keys: Vec<(syn::Expr, Ident)> = Vec::new();

   for (i, arg) in cl_args.iter().enumerate() {
      let col = Ident::new(&format!("__c{}", i), arg.span());

      if is_wild_card(arg) {
         plans.push(ColPlan { pat_ident: None, var_binding: None, filter: None });
      } else if let Some(v) = expr_to_ident(arg) {
         if let Some(prior_col) = seen.get(&v.to_string()) {
            filters.push(quote! { #col == #prior_col });
            plans.push(ColPlan { pat_ident: Some(col.clone()), var_binding: None, filter: None });
         } else {
            seen.insert(v.to_string(), col.clone());
            if prior_bound.iter().any(|pb| pb == &v) {
               shared_in_clause_order.push((v.clone(), col.clone()));
            } else {
               new_in_clause_order.push((v.clone(), col.clone()));
            }
            plans.push(ColPlan { pat_ident: Some(col.clone()), var_binding: Some((v, col)), filter: None });
         }
      } else {
         // Split arg into {pure filter} vs {computed key referencing prior bound}.
         let refd_idents = expr_get_vars(arg);
         let touches_prior = refd_idents.iter().any(|id| prior_bound.iter().any(|pb| pb == id));
         if touches_prior {
            computed_keys.push((arg.clone(), col.clone()));
         } else {
            filters.push(quote! { #col == (#arg) });
         }
         plans.push(ColPlan { pat_ident: Some(col), var_binding: None, filter: None });
      }
   }

   (plans, shared_in_clause_order, new_in_clause_order, filters, computed_keys)
}

fn destructure_pattern(plans: &[ColPlan]) -> TokenStream {
   let parts: Vec<TokenStream> = plans
      .iter()
      .map(|p| match &p.pat_ident {
         Some(ident) => quote! { #ident },
         None => quote! { _ },
      })
      .collect();
   if parts.len() == 1 {
      let p = &parts[0];
      quote! { (#p,) }
   } else {
      quote! { ( #(#parts),* ) }
   }
}

fn compile_first_clause(cl: &crate::ascent_mir::MirBodyClause) -> (Vec<Ident>, TokenStream) {
   // First clause has no prior bound vars, so no shared vars and no computed
   // keys (expressions must be pure since there's nothing to reference).
   let (plans, _shared, new_vars, filters, _computed) = plan_clause(&cl.args, &[]);
   let bound: Vec<Ident> = new_vars.iter().map(|(v, _)| v.clone()).collect();
   let proj_cols: Vec<Ident> = new_vars.iter().map(|(_, c)| c.clone()).collect();
   let rel_coll = relation_coll_var(&cl.rel.relation.name);
   let destruct = destructure_pattern(&plans);
   let proj_tuple = tuple_of_idents(&proj_cols);

   let filter_pred = if filters.is_empty() {
      quote! { true }
   } else {
      quote! { #( ( #filters ) )&&* }
   };

   // DD 0.20: `flat_map` consumes self — clone the `#rel_coll` handle so
   // the same collection can be used by multiple rules / other operators.
   // `.clone()` on a Collection is a cheap Arc handle bump.
   let tokens = quote! {
      #rel_coll.clone().flat_map(move |#destruct| {
         if #filter_pred { ::std::option::Option::Some(#proj_tuple) } else { ::std::option::Option::None }
      })
   };
   (bound, tokens)
}

fn compile_join_clause(
   prior_bound: &[Ident], prior_var_kinds: &[VarKind], accum: TokenStream,
   cl: &crate::ascent_mir::MirBodyClause,
   prior_rel: Option<&crate::ascent_mir::MirRelation>,
   // FlowLog fusion: if this is the TERMINAL clause of a simple rule body
   // (single head clause, all-Clause body, no cond/let/neg/agg), emit the
   // head tuple directly inside the `join_core` closure instead of the
   // pass-through bound-vars tuple. Caller skips the trailing `.map()` step.
   // `(head_row_ty, head_expr_tuple)` — row_ty for a precise Some::<row_ty>
   // annotation, expr_tuple is the head projection evaluated in closure scope.
   fused_head: Option<(TokenStream, TokenStream)>,
) -> (Vec<Ident>, TokenStream) {
   let (plans, shared_in_clause_order, new_vars, filters, computed_keys) = plan_clause(&cl.args, prior_bound);

   // Key layout uses CLAUSE COLUMN ORDER (canonical per `IrRelation.indices`,
   // sorted ascending). Matches the arrangement's flat_map emission so all
   // users of the same `(rel, indices)` produce identical key tuples —
   // prerequisite for arrangement sharing across rule bodies. Was previously
   // ordered by `prior_bound` (per-rule binding order), which prevented share.
   let shared_vars: Vec<Ident> = shared_in_clause_order.iter().map(|(v, _)| v.clone()).collect();
   let shared_cols: Vec<Ident> = shared_in_clause_order.iter().map(|(_, c)| c.clone()).collect();

   let accum_vals: Vec<Ident> =
      prior_bound.iter().filter(|v| !shared_vars.iter().any(|s| s == *v)).cloned().collect();
   let new_vars_names: Vec<Ident> = new_vars.iter().map(|(v, _)| v.clone()).collect();
   let new_vars_cols: Vec<Ident> = new_vars.iter().map(|(_, c)| c.clone()).collect();

   // Shared key layout = (shared_vars..., computed_keys...). Both sides map
   // into this identical tuple shape.
   // Accum side: each shared var is owned per its `VarKind` (Ref → to_owned,
   // Owned → Clone::clone). Computed keys evaluate user expressions against
   // whatever binding kind the user has.
   let shared_kinds = kinds_for(&shared_vars, prior_bound, prior_var_kinds);
   // For computed keys: `.clone()` forces owned. Method resolution on `.clone()`
   // prefers `<T as Clone>::clone(&T) -> T` over `<&T as Clone>::clone(&&T) -> &T`
   // (direct receiver match over autoref), so `(ef.deref()).clone()` where
   // `ef.deref(): &T` returns owned `T`. Essential — without `.clone()`, a
   // computed key expression like `ef.deref()` leaves a `&T` in the key tuple
   // that fails DD's `ExchangeData: 'static` bound.
   let accum_key_entries: Vec<TokenStream> = shared_vars
      .iter()
      .zip(shared_kinds.iter())
      .map(|(v, k)| own_value_expr(v, *k))
      .chain(computed_keys.iter().map(|(expr, _)| quote! { (#expr).clone() }))
      .collect();
   let accum_key_tuple = tuple_tokens(&accum_key_entries);

   // Clause side key entries: shared cols are owned (from clause destructure),
   // so direct use is fine. Computed-key cols are owned too.
   let clause_key_entries: Vec<TokenStream> = shared_cols
      .iter()
      .map(|c| quote! { #c })
      .chain(computed_keys.iter().map(|(_, col)| quote! { #col }))
      .collect();
   let clause_key_tuple = tuple_tokens(&clause_key_entries);

   // Rebuild the destructure pattern for the JOIN OUTPUT: mirror the key
   // layout but with fresh pattern idents that don't collide.
   // For shared_vars, the destructure name equals the user var name.
   // For computed_keys, we destructure to a throwaway wildcard.
   let mut join_key_pattern_parts: Vec<TokenStream> =
      shared_vars.iter().map(|v| quote! { #v }).collect();
   for _ in &computed_keys {
      join_key_pattern_parts.push(quote! { _ });
   }
   let join_key_pattern = tuple_tokens_as_pattern(&join_key_pattern_parts);

   let rel_coll = relation_coll_var(&cl.rel.relation.name);
   let destruct = destructure_pattern(&plans);
   let new_vars_cols_tuple = tuple_of_idents(&new_vars_cols);
   let new_vars_tuple = tuple_of_idents(&new_vars_names);
   let accum_vals_tuple = tuple_of_idents(&accum_vals);

   let mut out_bound: Vec<Ident> = prior_bound.to_vec();
   out_bound.extend(new_vars_names);
   let out_tuple = tuple_of_idents(&out_bound);

   let filter_pred = if filters.is_empty() {
      quote! { true }
   } else {
      quote! { #( ( #filters ) )&&* }
   };

   // Accum-side vals must be owned in the map output — refs can't escape
   // the closure frame, and Owned vars after destructure need an explicit
   // clone to be reused as a value.
   let accum_val_kinds = kinds_for(&accum_vals, prior_bound, prior_var_kinds);
   let accum_vals_cloned = own_values_tuple(&accum_vals, &accum_val_kinds);

   let destr_accum = emit_user_destructure(prior_bound, prior_var_kinds, quote! { &__b });
   // FlowLog-shape emission. Accumulator side is arranged per-rule (inherently
   // rule-specific — carries prior-join state). RHS references the SHARED
   // arrangement `__arr_<ir_name>` emitted once per SCC (see
   // `emit_shared_arrangement_binding`). All users of the same `(rel, keying)`
   // pair reuse the same arranged trace — avoids N-fold trace maintenance.
   //
   // Correctness guards: the shared arrangement's `(key, val)` layout is
   // derived from `ir_rel.indices` (clause-column order, ascending). The key
   // this rule constructs on the LHS uses `shared_in_clause_order` (same
   // canonicalization). If the clause needs per-use filters or computed keys
   // the shared arrangement can't express (e.g. literal-arg filters or let
   // bindings that pick from clause cols), fall back to per-clause
   // `flat_map().arrange_by_key()`.
   let shared_arr_name = shared_arr_ident(&{
      use crate::ascent_hir::IrRelation;
      // Reconstruct `IrRelation` from MirRelation fields — `ir_name` is the
      // product of `rel.name` + `indices`, so this is deterministic.
      IrRelation {
         relation: cl.rel.relation.clone(),
         indices: cl.rel.indices.clone(),
         val_type: cl.rel.val_type.clone(),
      }
   });
   let can_use_shared = filters.is_empty() && !cl.rel.indices.is_empty();
   // LHS-shared: when `prior_rel` is set and matches the shape of the first
   // clause's raw flat_map (no cond/filter), skip building `__l` — reuse
   // `__arr_<prior_rel.name>_indices_<prior_rel.indices>` directly as the
   // LHS arrangement. Matches FlowLog: one arrangement per (rel, keying)
   // shared across all rules, not one per rule-clause. Condition for reuse:
   //   - `prior_rel` is Some (accum is a raw first-clause flat_map)
   //   - `prior_rel.indices` is non-empty (there's a usable key)
   //   - `filters` are empty (shared arr has no filter baked in)
   let lhs_shared_arr: Option<Ident> = match prior_rel {
      Some(pr) if filters.is_empty() && !pr.indices.is_empty() => {
         let nm = format!(
            "__arr_{}_indices_{}",
            pr.relation.name,
            pr.indices.iter().map(|i| i.to_string()).collect::<Vec<_>>().join("_")
         );
         Some(Ident::new(&nm, pr.relation.name.span()))
      },
      _ => None,
   };
   let tokens = if can_use_shared && lhs_shared_arr.is_some() {
      let lhs_arr = lhs_shared_arr.unwrap();
      if let Some((head_row_ty, head_expr_tuple)) = fused_head.as_ref() {
         // Bind lv/rv/k as refs (via match ergonomics: `let (x,) = &(1,);`
         // gives x: &T). Head expressions like `*x` and `Convert::convert(x)`
         // both work uniformly with Ref-kind vars — matching what the
         // non-fused `.map(|__b| { #destr; head })` path does via
         // `emit_user_destructure(&__b)`.
         quote! {
            {
               #lhs_arr.clone().join_core(#shared_arr_name.clone(), |__k, __lv, __rv| {
                  let (#accum_vals_tuple, #new_vars_tuple) = (__lv, __rv);
                  let #join_key_pattern = __k;
                  ::std::option::Option::Some::<#head_row_ty>(#head_expr_tuple)
               })
            }
         }
      } else {
         quote! {
            {
               #lhs_arr.clone().join_core(#shared_arr_name.clone(), |__k, __lv, __rv| {
                  let (#accum_vals_tuple, #new_vars_tuple) = (__lv.clone(), __rv.clone());
                  let #join_key_pattern = __k.clone();
                  ::std::option::Option::Some::<_>(#out_tuple)
               })
            }
         }
      }
   } else if can_use_shared {
      if let Some((head_row_ty, head_expr_tuple)) = fused_head.as_ref() {
         quote! {
            {
               let __l = (#accum).map(move |__b| {
                  #destr_accum
                  (#accum_key_tuple, #accum_vals_cloned)
               }).arrange_by_key();
               __l.join_core(#shared_arr_name.clone(), |__k, __lv, __rv| {
                  let (#accum_vals_tuple, #new_vars_tuple) = (__lv, __rv);
                  let #join_key_pattern = __k;
                  ::std::option::Option::Some::<#head_row_ty>(#head_expr_tuple)
               })
            }
         }
      } else {
         quote! {
            {
               let __l = (#accum).map(move |__b| {
                  #destr_accum
                  (#accum_key_tuple, #accum_vals_cloned)
               }).arrange_by_key();
               __l.join_core(#shared_arr_name.clone(), |__k, __lv, __rv| {
                  let (#accum_vals_tuple, #new_vars_tuple) = (__lv.clone(), __rv.clone());
                  let #join_key_pattern = __k.clone();
                  ::std::option::Option::Some::<_>(#out_tuple)
               })
            }
         }
      }
   } else if let Some((head_row_ty, head_expr_tuple)) = fused_head.as_ref() {
      quote! {
         {
            let __l = (#accum).map(move |__b| {
               #destr_accum
               (#accum_key_tuple, #accum_vals_cloned)
            }).arrange_by_key();
            let __r = #rel_coll.clone().flat_map(move |#destruct| {
               if #filter_pred {
                  ::std::option::Option::Some((#clause_key_tuple, #new_vars_cols_tuple))
               } else {
                  ::std::option::Option::None
               }
            }).arrange_by_key();
            __l.join_core(__r, |__k, __lv, __rv| {
               let (#accum_vals_tuple, #new_vars_tuple) = (__lv, __rv);
               let #join_key_pattern = __k;
               ::std::option::Option::Some::<#head_row_ty>(#head_expr_tuple)
            })
         }
      }
   } else {
      // Fallback: per-clause arrangement (filters or no-key clause).
      quote! {
         {
            let __l = (#accum).map(move |__b| {
               #destr_accum
               (#accum_key_tuple, #accum_vals_cloned)
            }).arrange_by_key();
            let __r = #rel_coll.clone().flat_map(move |#destruct| {
               if #filter_pred {
                  ::std::option::Option::Some((#clause_key_tuple, #new_vars_cols_tuple))
               } else {
                  ::std::option::Option::None
               }
            }).arrange_by_key();
            __l.join_core(__r, |__k, __lv, __rv| {
               let (#accum_vals_tuple, #new_vars_tuple) = (__lv.clone(), __rv.clone());
               let #join_key_pattern = __k.clone();
               ::std::option::Option::Some::<_>(#out_tuple)
            })
         }
      }
   };

   (out_bound, tokens)
}

fn tuple_tokens(entries: &[TokenStream]) -> TokenStream {
   if entries.len() == 1 {
      let e = &entries[0];
      quote! { ( #e , ) }
   } else {
      quote! { ( #(#entries),* ) }
   }
}

fn tuple_tokens_as_pattern(entries: &[TokenStream]) -> TokenStream { tuple_tokens(entries) }

/// Emit a filter. Bound vars are bound per their `VarKind` — clause-vars
/// as `&T` (batch convention, so user's `*x` works), generator/let/if-let
/// vars as owned `T`.
fn emit_filter(accum: TokenStream, bound_vars: &[Ident], var_kinds: &[VarKind], cond: &syn::Expr) -> TokenStream {
   let destr = emit_user_destructure(bound_vars, var_kinds, quote! { __b });
   quote! {
      (#accum).filter(move |__b| {
         #destr
         #cond
      })
   }
}

/// Detect Ascent's negation aggregator so we can lower `!R(args)` (or an
/// explicit `agg () = not() in R(args)`) to `.antijoin` rather than going
/// through the full `Reduce`-based agg machinery (Phase 5).
///
/// Accepts any path whose last segment is `not`. The fully-qualified HIR
/// desugaring emits `::ascent::aggregators::not`; user code under
/// `use ascent::aggregators::*` can write just `not`.
fn is_not_aggregator(expr: &syn::Expr) -> bool {
   if let syn::Expr::Path(p) = expr {
      return p.path.segments.last().map_or(false, |s| s.ident == "not");
   }
   false
}

fn emit_negation(
   accum: TokenStream, bound_vars: &[Ident], var_kinds: &[VarKind], rel_name: &Ident, rel_args: &[syn::Expr],
) -> TokenStream {
   let (plans, shared_in_clause_order, _new_in_clause_order, filters, computed_keys) =
      plan_clause(rel_args, bound_vars);

   let shared_by_accum_order: Vec<(Ident, Ident)> = bound_vars
      .iter()
      .filter_map(|v| shared_in_clause_order.iter().find(|(cv, _)| cv == v).cloned())
      .collect();

   let shared_vars: Vec<Ident> = shared_by_accum_order.iter().map(|(v, _)| v.clone()).collect();
   let shared_cols: Vec<Ident> = shared_by_accum_order.iter().map(|(_, c)| c.clone()).collect();
   let accum_vals: Vec<Ident> =
      bound_vars.iter().filter(|v| !shared_vars.iter().any(|s| s == *v)).cloned().collect();

   // Key layout: (shared..., computed_keys...) for both sides.
   let shared_kinds = kinds_for(&shared_vars, bound_vars, var_kinds);
   let accum_key_entries: Vec<TokenStream> = shared_vars
      .iter()
      .zip(shared_kinds.iter())
      .map(|(v, k)| own_value_expr(v, *k))
      .chain(computed_keys.iter().map(|(expr, _)| quote! { (#expr).clone() }))
      .collect();
   let accum_key_tuple = tuple_tokens(&accum_key_entries);

   let clause_key_entries: Vec<TokenStream> = shared_cols
      .iter()
      .map(|c| quote! { #c })
      .chain(computed_keys.iter().map(|(_, col)| quote! { #col }))
      .collect();
   let clause_key_tuple = tuple_tokens(&clause_key_entries);

   let mut join_key_pattern_parts: Vec<TokenStream> =
      shared_vars.iter().map(|v| quote! { #v }).collect();
   for _ in &computed_keys {
      join_key_pattern_parts.push(quote! { _ });
   }
   let join_key_pattern = tuple_tokens(&join_key_pattern_parts);

   let rel_coll = relation_coll_var(rel_name);
   let destruct = destructure_pattern(&plans);
   let accum_vals_tuple = tuple_of_idents(&accum_vals);
   let accum_bound_tuple = tuple_of_idents(bound_vars);
   let accum_val_kinds = kinds_for(&accum_vals, bound_vars, var_kinds);
   let accum_vals_cloned = own_values_tuple(&accum_vals, &accum_val_kinds);
   let destr_accum = emit_user_destructure(bound_vars, var_kinds, quote! { &__b });

   let filter_pred = if filters.is_empty() {
      quote! { true }
   } else {
      quote! { #( ( #filters ) )&&* }
   };

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

/// Lower an `agg pat = <fn>(bound_args...) in rel(rel_args...)` body item.
///
/// Strategy (Phase 5):
///   1. Arrange `rel` as `Collection<(GroupKey, BoundArgTuple)>`, where
///      `GroupKey` = shared-with-outer-bound-vars + computed keys.
///   2. `reduce`: per group key, turn the consolidated `&[(&V, diff)]` slice
///      (where `V = BoundArgTuple`) into an `Iterator<Item = (&T0, &T1, ...)>`
///      of refs into the aggregator input arity, then invoke the user's
///      aggregator function. Each result emits one output row.
///   3. Join the aggregator output with the outer accum on `GroupKey`.
///   4. Pattern-bind `agg.pat` to the aggregator's scalar output.
///
/// Aggregators are stratified (MIR guarantee): `rel_coll` is a collection
/// from a prior SCC, already sealed at this point. The aggregator's `reduce`
/// runs against a stable input.
fn emit_agg(
   accum: TokenStream, bound_vars: &[Ident], var_kinds: &[VarKind], agg: &IrAggClause,
) -> (Vec<Ident>, Vec<VarKind>, TokenStream) {
   let rel_name = &agg.rel.relation.name;
   let rel_coll = relation_coll_var(rel_name);
   let (plans, shared_in_clause_order, new_vars, filters, computed_keys) = plan_clause(&agg.rel_args, bound_vars);

   // Shared vars reordered by prior-bound order (for consistent key tuple layout).
   let shared_by_accum_order: Vec<(Ident, Ident)> = bound_vars
      .iter()
      .filter_map(|v| shared_in_clause_order.iter().find(|(cv, _)| cv == v).cloned())
      .collect();
   let shared_vars: Vec<Ident> = shared_by_accum_order.iter().map(|(v, _)| v.clone()).collect();
   let shared_cols: Vec<Ident> = shared_by_accum_order.iter().map(|(_, c)| c.clone()).collect();

   // Preserve original-tuple identity in the reduce value so set-semantic
   // consolidation doesn't collapse distinct rel tuples that happen to share
   // the same bound-arg values. Store the FULL rel tuple as `V`, then project
   // to bound_args at reduce time. Use the same span `plan_clause` used, so
   // idents emitted by `filter_pred` (via plans) line up with ours.
   let rel_arity = agg.rel_args.len();
   let rel_col_idents: Vec<Ident> = agg
      .rel_args
      .iter()
      .enumerate()
      .map(|(i, arg)| Ident::new(&format!("__c{}", i), arg.span()))
      .collect();
   let _ = rel_arity;

   // Find which column index each bound_arg corresponds to. `rel_args` may
   // contain non-ident exprs (wildcards/lits/computed); bound_args must be
   // plain idents that `plan_clause` classified as new vars.
   let bound_arg_indices: Vec<usize> = agg
      .bound_args
      .iter()
      .map(|ba| {
         agg.rel_args
            .iter()
            .position(|a| expr_to_ident(a).as_ref() == Some(ba))
            .expect("aggregator bound_arg must appear as a plain ident in rel_args")
      })
      .collect();

   // Key tuple layout (same on both sides): shared cols + computed keys.
   let clause_key_entries: Vec<TokenStream> = shared_cols
      .iter()
      .map(|c| quote! { #c })
      .chain(computed_keys.iter().map(|(_, col)| quote! { #col }))
      .collect();
   let clause_key_tuple = tuple_tokens(&clause_key_entries);

   // Destructure that binds EVERY rel column (no wildcard `_`), so we can
   // pack the whole tuple into the reduce value and preserve distinctness.
   let agg_destruct: TokenStream = {
      let cols: Vec<TokenStream> = rel_col_idents.iter().map(|c| quote! { #c }).collect();
      if cols.len() == 1 {
         let c = &cols[0];
         quote! { (#c,) }
      } else {
         quote! { ( #(#cols),* ) }
      }
   };
   let _ = plans; // superseded by `agg_destruct`; plans still used for `filters`/`computed_keys` derivation
   let filter_pred = if filters.is_empty() {
      quote! { true }
   } else {
      quote! { #( ( #filters ) )&&* }
   };

   // Build the (&T0, &T1, ...) tuple the aggregator expects. `__v` is `&V`
   // where `V` is the full rel tuple `(c0, c1, ...)`. For each bound_arg,
   // take a ref to its column by tuple index.
   let aggregator = &agg.aggregator;
   let ref_tuple_for_agg: TokenStream = if agg.bound_args.is_empty() {
      quote! { () }
   } else {
      let parts: Vec<TokenStream> = bound_arg_indices
         .iter()
         .map(|i| {
            let idx = syn::Index::from(*i);
            quote! { &__v.#idx }
         })
         .collect();
      if parts.len() == 1 {
         let p = &parts[0];
         quote! { (#p,) }
      } else {
         quote! { ( #(#parts),* ) }
      }
   };

   // Accum side (same shape as compile_join_clause).
   let accum_vals: Vec<Ident> =
      bound_vars.iter().filter(|v| !shared_vars.iter().any(|s| s == *v)).cloned().collect();
   let accum_val_kinds = kinds_for(&accum_vals, bound_vars, var_kinds);
   let accum_vals_cloned = own_values_tuple(&accum_vals, &accum_val_kinds);
   let accum_vals_tuple = tuple_of_idents(&accum_vals);

   let shared_kinds = kinds_for(&shared_vars, bound_vars, var_kinds);
   let accum_key_entries: Vec<TokenStream> = shared_vars
      .iter()
      .zip(shared_kinds.iter())
      .map(|(v, k)| own_value_expr(v, *k))
      .chain(computed_keys.iter().map(|(expr, _)| quote! { (#expr).clone() }))
      .collect();
   let accum_key_tuple = tuple_tokens(&accum_key_entries);
   let destr_accum = emit_user_destructure(bound_vars, var_kinds, quote! { &__b });

   let mut join_key_pattern_parts: Vec<TokenStream> = shared_vars.iter().map(|v| quote! { #v }).collect();
   for _ in &computed_keys {
      join_key_pattern_parts.push(quote! { _ });
   }
   let join_key_pattern = tuple_tokens(&join_key_pattern_parts);

   let pat = &agg.pat;
   let pat_vars = pattern_get_vars(pat);

   // Output bound_vars + pat_vars; pat_vars are Owned (aggregator result is a
   // moved value).
   let mut out_bound: Vec<Ident> = bound_vars.to_vec();
   out_bound.extend(pat_vars.clone());
   let out_tuple = tuple_of_idents(&out_bound);
   let mut out_kinds = var_kinds.to_vec();
   for _ in &pat_vars {
      out_kinds.push(VarKind::Owned);
   }

   // Rel value = full tuple (preserves per-tuple distinctness across DD's
   // consolidation). We rebuild `( __c0, __c1, … )` from the destructured cols.
   let rel_full_value: TokenStream = {
      let cols: Vec<TokenStream> = rel_col_idents.iter().map(|c| quote! { #c }).collect();
      if cols.len() == 1 {
         let c = &cols[0];
         quote! { (#c,) }
      } else {
         quote! { ( #(#cols),* ) }
      }
   };

   let tokens = quote! {
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
   };

   (out_bound, out_kinds, tokens)
}

fn emit_let(
   bound_vars: Vec<Ident>, var_kinds: &[VarKind], accum: TokenStream, pat: &syn::Pat, expr: &syn::Expr,
) -> (Vec<Ident>, TokenStream) {
   let new_vars = pattern_get_vars(pat);
   let mut new_bound = bound_vars.clone();
   new_bound.extend(new_vars.iter().cloned());

   let prior_tuple = tuple_of_idents(&bound_vars);
   let new_tuple = tuple_of_idents(&new_bound);
   let destr = emit_user_destructure(&bound_vars, var_kinds, quote! { &__b });

   // Convert the let-RHS to a guaranteed-owned expression. Needed so
   // `__pat_val` doesn't hold a ref into `__b` when we later move `__b`
   // into the owned destructure (would otherwise trigger E0515).
   let expr_as_owned = let_rhs_to_owned(expr, &bound_vars, var_kinds);

   let tokens = quote! {
      (#accum).map(move |__b| {
         let __pat_val = { #destr #expr_as_owned };
         let #prior_tuple = __b;
         let #pat = __pat_val;
         #new_tuple
      })
   };
   (new_bound, tokens)
}

/// Convert a `let`-RHS expression to one that's guaranteed-owned. If the
/// expression is a bare ident that's already bound, clone according to
/// its `VarKind` (Ref → `ToOwned::to_owned`, Owned → `Clone::clone(&_)`).
/// Otherwise — a computed expression, literal, generator, etc. — we pass
/// through unchanged and trust the user's expression to produce owned.
/// (Computed exprs generally don't embed refs to destructure-local vars.)
fn let_rhs_to_owned(expr: &syn::Expr, bound_vars: &[Ident], var_kinds: &[VarKind]) -> TokenStream {
   if let Some(id) = expr_to_ident(expr) {
      for (v, k) in bound_vars.iter().zip(var_kinds.iter()) {
         if v == &id {
            return own_value_expr(v, *k);
         }
      }
   }
   quote! { (#expr) }
}

fn emit_if_let(
   bound_vars: Vec<Ident>, var_kinds: &[VarKind], accum: TokenStream, pat: &syn::Pat, expr: &syn::Expr,
) -> (Vec<Ident>, TokenStream) {
   let new_vars = pattern_get_vars(pat);
   let mut new_bound = bound_vars.clone();
   new_bound.extend(new_vars.iter().cloned());

   let iflet_kind = if_let_new_var_kind(expr, &bound_vars, var_kinds);
   let mut new_kinds: Vec<VarKind> = var_kinds.to_vec();
   for _ in 0..new_vars.len() {
      new_kinds.push(iflet_kind);
   }
   let owned_tuple = own_values_tuple(&new_bound, &new_kinds);
   let destr = emit_user_destructure(&bound_vars, var_kinds, quote! { &__b });

   let tokens = quote! {
      (#accum).flat_map(move |__b| {
         #destr
         if let #pat = (#expr) {
            ::std::option::Option::Some(#owned_tuple)
         } else {
            ::std::option::Option::None
         }
      })
   };
   (new_bound, tokens)
}

/// `for pat in expr`: extends each bound tuple by every item the iterator
/// emits, binding the pattern on each. `expr` sees bound vars per their
/// `VarKind` (Ref = `&T`, Owned = `T`).
///
/// Strategy: inline for-loop that eagerly builds a `Vec<NewTup>` of owned
/// output tuples per input `__b`. Avoids closure-capture gymnastics where
/// an inner `.map` closure would want to move `__b` while items (possibly
/// containing refs into `__b`) still borrow it.
fn emit_generator(
   bound_vars: Vec<Ident>, var_kinds: &[VarKind], accum: TokenStream, pat: &syn::Pat, expr: &syn::Expr,
) -> (Vec<Ident>, TokenStream) {
   let new_vars = pattern_get_vars(pat);
   let mut new_bound = bound_vars.clone();
   new_bound.extend(new_vars.iter().cloned());

   let destr = emit_user_destructure(&bound_vars, var_kinds, quote! { &__b });

   // Build the output tuple using kind-aware owned-exprs for PRIOR vars and
   // `Clone::clone(&v)` for pattern-bound NEW vars (they're owned `T` after
   // `let #pat = __item;` — Copy types trivially clone, non-Copy Clone
   // via autoref).
   let prior_owned_parts: Vec<TokenStream> =
      bound_vars.iter().zip(var_kinds.iter()).map(|(v, k)| own_value_expr(v, *k)).collect();
   let new_owned_parts: Vec<TokenStream> =
      new_vars.iter().map(|v| quote! { ::std::clone::Clone::clone(&#v) }).collect();
   let all_owned: Vec<TokenStream> = prior_owned_parts.into_iter().chain(new_owned_parts).collect();
   let out_tuple = tuple_tokens(&all_owned);

   let tokens = quote! {
      (#accum).flat_map(move |__b| {
         let mut __out: ::std::vec::Vec<_> = ::std::vec::Vec::new();
         #destr
         for __item in (#expr) {
            let #pat = __item;
            __out.push(#out_tuple);
         }
         __out
      })
   };
   (new_bound, tokens)
}

/// Emit the rule body + per-head projection, concatenating each head
/// projection into the target variable chosen by `head_target`.
///
/// `head_target(rel_name)` returns the identifier of the mutable Collection
/// binding that should accumulate productions for that head relation. In the
/// outer scope that's `__<rel>_coll`; inside a looping SCC's iterative scope,
/// it's the per-dynamic-relation `__<rel>_acc`.
///
/// `scope_ident` names the scope variable in the enclosing Rust code —
/// `scope` in outer codegen, `inner` inside `scope.iterative(|inner| { ... })`.
/// Emits rule body expressions per head — one Collection producing the
/// head-shaped tuples for each head clause of `rule`. Returns
/// `(Vec<(head_rel_name, body_expr)>, hoisted_pre)`.
///
/// FlowLog-style: caller decides how to use these — chain-concat for
/// looping SCCs (matches `next_X = rule1.concat(rule2)...threshold()`),
/// or per-head `head_coll = head_coll.concat(body_expr)` for non-looping
/// SCCs.
fn compile_rule_head_exprs(
   rule: &MirRule, scope_ident: &Ident, hoist_counter: &mut usize,
) -> (Vec<(Ident, TokenStream)>, TokenStream) {
   if rule.body_items.is_empty() {
      let seed = unit_seed(scope_ident);
      let mut out = Vec::new();
      for hcl in &rule.head_clause {
         let head_tuple = build_expr_tuple(&hcl.args);
         out.push((hcl.rel.name.clone(), quote! { (#seed).map(move |()| #head_tuple) }));
      }
      return (out, TokenStream::new());
   }

   use crate::ascent_mir::MirBodyItem;
   let clause_count = rule.body_items.iter().filter(|i| matches!(i, MirBodyItem::Clause(_))).count();
   let single_clause_fusion = rule.body_items.len() == 1
      && rule.head_clause.len() == 1
      && matches!(&rule.body_items[0], MirBodyItem::Clause(cl) if cl.cond_clauses.is_empty());
   if single_clause_fusion {
      let cl = match &rule.body_items[0] {
         MirBodyItem::Clause(c) => c,
         _ => unreachable!(),
      };
      let rel_coll = relation_coll_var(&cl.rel.relation.name);
      let (plans, _shared, _new_vars, filters, _ck) = plan_clause(&cl.args, &[]);
      let hcl = &rule.head_clause[0];
      let head_expr_tuple = build_expr_tuple(&hcl.args);
      let row_ty = tuple_type(&hcl.rel.field_types);
      let destruct = destructure_pattern(&plans);
      let filter_pred = if filters.is_empty() { quote! { true } } else { quote! { #( ( #filters ) )&&* } };
      let var_rebinds: Vec<TokenStream> = plans
         .iter()
         .filter_map(|p| p.var_binding.as_ref().map(|(v, c)| quote! { let #v = &#c; }))
         .collect();
      let expr = quote! {
         #rel_coll.clone().flat_map(move |#destruct| -> ::std::option::Option<#row_ty> {
            #(#var_rebinds)*
            if #filter_pred { ::std::option::Option::Some(#head_expr_tuple) } else { ::std::option::Option::None }
         })
      };
      return (vec![(hcl.rel.name.clone(), expr)], TokenStream::new());
   }

   let (bound_vars, var_kinds, body_expr, hoisted_pre) =
      compile_rule_body_phase1(rule, scope_ident, hoist_counter);

   let all_clauses_simple = clause_count >= 2
      && rule.body_items.iter().all(|item| match item {
         MirBodyItem::Clause(cl) => cl.cond_clauses.is_empty(),
         _ => false,
      });
   let fused = all_clauses_simple && rule.head_clause.len() == 1;

   if fused {
      let hcl = &rule.head_clause[0];
      return (vec![(hcl.rel.name.clone(), body_expr)], hoisted_pre);
   }

   // Fallback: body_expr is bound-var tuple; each head gets its own Collection
   // by `.map()`'ing body_expr. For multi-head rules we rely on `body_expr`
   // being a cheap Rust expression (DD Collection construction) that can be
   // inlined per head — DD's operator graph dedups identical subgraphs anyway.
   let mut out = Vec::new();
   for hcl in rule.head_clause.iter() {
      let head_expr_tuple = build_expr_tuple(&hcl.args);
      let row_ty = tuple_type(&hcl.rel.field_types);
      let destr = emit_user_destructure(&bound_vars, &var_kinds, quote! { &__b });
      let body_ref = quote! {
         (#body_expr).map(move |__b| -> #row_ty { #destr #head_expr_tuple })
      };
      out.push((hcl.rel.name.clone(), body_ref));
   }
   (out, hoisted_pre)
}

fn compile_rule_with_head_target(
   rule: &MirRule, head_target: &dyn Fn(&Ident) -> Ident, scope_ident: &Ident, hoist_counter: &mut usize,
) -> (TokenStream, TokenStream) {
   // Returns (body_tokens_inside_closure, hoisted_pre_tokens_outside_closure).
   if rule.body_items.is_empty() {
      let seed = unit_seed(scope_ident);
      let mut out = TokenStream::new();
      for hcl in &rule.head_clause {
         let head_target_ident = head_target(&hcl.rel.name);
         let head_tuple = build_expr_tuple(&hcl.args);
         out.extend(quote! {
            {
               let __fc = (#seed).map(move |()| #head_tuple);
               #head_target_ident = #head_target_ident.concat(__fc);
            }
         });
      }
      return (out, TokenStream::new());
   }

   use crate::ascent_mir::MirBodyItem;
   let clause_count = rule.body_items.iter().filter(|i| matches!(i, MirBodyItem::Clause(_))).count();

   // SINGLE-CLAUSE FUSION: rule has exactly one body clause (a Clause, no
   // generators/conds/aggs) with no cond_clauses, and one head clause.
   // FlowLog emits this as a single `.flat_map` that scans the rel and
   // builds the head tuple. Without fusion we'd emit `flat_map(identity)`
   // followed by `.map(head)` — two operators for what's one.
   let single_clause_fusion = rule.body_items.len() == 1
      && rule.head_clause.len() == 1
      && matches!(&rule.body_items[0], MirBodyItem::Clause(cl) if cl.cond_clauses.is_empty());
   if single_clause_fusion {
      let cl = match &rule.body_items[0] {
         MirBodyItem::Clause(c) => c,
         _ => unreachable!(),
      };
      let rel_coll = relation_coll_var(&cl.rel.relation.name);
      let (plans, _shared, _new_vars, filters, _ck) = plan_clause(&cl.args, &[]);
      let hcl = &rule.head_clause[0];
      let head_target_ident = head_target(&hcl.rel.name);
      let head_expr_tuple = build_expr_tuple(&hcl.args);
      let row_ty = tuple_type(&hcl.rel.field_types);
      let destruct = destructure_pattern(&plans);
      let filter_pred = if filters.is_empty() { quote! { true } } else { quote! { #( ( #filters ) )&&* } };
      // plan_clause's destructure binds col idents (`__c0, __c1`). `build_expr_tuple`
      // emits user var names. Add `let user_var = &col_ident;` rebindings so the
      // head expression can reference user var names (as refs, matching the
      // non-fused path's `#destr` that emit_user_destructure produces).
      let var_rebinds: Vec<TokenStream> = plans
         .iter()
         .filter_map(|p| p.var_binding.as_ref().map(|(v, c)| quote! { let #v = &#c; }))
         .collect();
      return (
         quote! {
            {
               let __prod = #rel_coll.clone().flat_map(move |#destruct| -> ::std::option::Option<#row_ty> {
                  #(#var_rebinds)*
                  if #filter_pred { ::std::option::Option::Some(#head_expr_tuple) } else { ::std::option::Option::None }
               });
               #head_target_ident = #head_target_ident.concat(__prod);
            }
         },
         TokenStream::new(),
      );
   }

   let (bound_vars, var_kinds, body_expr, hoisted_pre) =
      compile_rule_body_phase1(rule, scope_ident, hoist_counter);

   // N-CLAUSE FUSION (N>=2): when all clauses simple + single head, body_expr
   // is ALREADY the head-projected Collection (fusion happened inside the
   // final join_core closure). Skip the trailing .map(), just concat.
   let all_clauses_simple = clause_count >= 2
      && rule.body_items.iter().all(|item| match item {
         MirBodyItem::Clause(cl) => cl.cond_clauses.is_empty(),
         _ => false,
      });
   let fused = all_clauses_simple && rule.head_clause.len() == 1;

   let mut out = TokenStream::new();
   if fused {
      let hcl = &rule.head_clause[0];
      let head_target_ident = head_target(&hcl.rel.name);
      out.extend(quote! {
         #head_target_ident = #head_target_ident.concat(#body_expr);
      });
      return (out, hoisted_pre);
   }

   let rule_body_var = Ident::new("__rule_body", Span::call_site());
   out.extend(quote! {
      let #rule_body_var = #body_expr;
   });
   for hcl in &rule.head_clause {
      let head_target_ident = head_target(&hcl.rel.name);
      let head_expr_tuple = build_expr_tuple(&hcl.args);
      let row_ty = tuple_type(&hcl.rel.field_types);
      let destr = emit_user_destructure(&bound_vars, &var_kinds, quote! { &__b });
      out.extend(quote! {
         {
            let __prod = #rule_body_var.clone().map(move |__b| -> #row_ty {
               #destr
               #head_expr_tuple
            });
            #head_target_ident = #head_target_ident.concat(__prod);
         }
      });
   }
   (out, hoisted_pre)
}

// ---------------------------------------------------------------------------
// Top-level codegen
// ---------------------------------------------------------------------------

pub(crate) fn compile_mir_dd(mir: &AscentMir, is_ascent_run: bool) -> TokenStream {
   match mir.config.dd_mode {
      crate::ascent_hir::DdMode::Incremental => compile_mir_dd_incremental(mir, is_ascent_run),
      crate::ascent_hir::DdMode::Batch => match compile_mir_dd_batch(mir, is_ascent_run) {
         Ok(ts) => ts,
         Err(e) => e.to_compile_error(),
      },
   }
}

fn compile_mir_dd_incremental(mir: &AscentMir, is_ascent_run: bool) -> TokenStream {
   let blocker = phase1_blocker(mir);

   // Struct + Default are phase-agnostic.
   // For `ascent_run!`, relation init expressions must be evaluated at the
   // macro call site (where they can capture outer locals), NOT inside the
   // generated `Default::default()` which is a fn item.
   let struct_and_default = emit_struct_and_default(mir, !is_ascent_run);

   // run() either dispatches to the real dataflow or the Phase 0 noop with
   // a diagnostic comment. Field access target is `self` for the method,
   // `__run_res` for `ascent_run!`. `run_body` is resolved later once we
   // know whether a Session is available for this program (then `run` is
   // a thin wrapper around `Session::commit`, dedup'd).
   let self_target: TokenStream = quote!(self);

   let (ty_impl_generics, ty_ty_generics, ty_where_clause) = mir.signatures.split_ty_generics_for_impl();
   let (impl_impl_generics, impl_ty_generics, impl_where_clause) = mir.signatures.split_impl_generics_for_impl();
   let _ = (ty_impl_generics, ty_where_clause); // kept for symmetry with batch backend

   let ty_signature = &mir.signatures.declaration;
   let struct_name = &ty_signature.ident;

   let summary = format!("DD backend — {} relations, phase1_blocker: {:?}", mir.relations_ir_relations.len(), blocker);
   let summary_fn = if is_ascent_run {
      quote! { pub fn summary(&self) -> &'static str { #summary } }
   } else {
      quote! { pub fn summary() -> &'static str { #summary } }
   };

   // Emit `<Name>Session` only when:
   //   - program is within backend envelope (no `blocker`),
   //   - item-context (`ascent!`, not `ascent_run!`) — `fn new()` can't
   //     capture locals,
   //   - no generics on the struct — propagating them through the whole
   //     session machinery is possible but non-trivial (would need Send +
   //     'static bounds added automatically and correct generic spelling
   //     in every `impl`). Generic programs use the batch `run()` API.
   let has_generics = !mir.signatures.declaration.generics.params.is_empty();
   let emit_session_for_this_program = blocker.is_none() && !is_ascent_run && !has_generics;
   let session_items =
      if emit_session_for_this_program { emit_session(mir) } else { TokenStream::new() };

   let session_accessor = if emit_session_for_this_program {
      let session_name = session_struct_name(struct_name);
      quote! {
         /// Start a live incremental session: returns a long-lived handle
         /// whose `<rel>_insert/remove` + `commit()` let you push updates
         /// and observe output deltas without re-running from scratch.
         pub fn session() -> #session_name { #session_name::new() }
      }
   } else {
      TokenStream::new()
   };

   // `run()` uses `execute_batch` directly (multi-worker capable). We
   // previously dedup'd by routing `run()` through `Session::commit()`
   // for the session-emitted path, but Session is single-worker — that
   // bypassed `execute_batch`'s parallel timely runtime and made `run()`
   // silently serial regardless of `ASCENT_DD_WORKERS`. Restoring the
   // direct `execute_batch` emission here is what makes `run()` actually
   // scale across cores; code-size regains are a follow-up once Session
   // itself supports multi-worker.
   let run_body = match &blocker {
      None => phase1_run_body(mir, &self_target),
      Some(reason) => {
         let comment = format!("dd backend: falling back to noop run() — {reason}");
         quote! {
            // #comment
            let _ = #comment;
         }
      },
   };
   let _ = emit_session_for_this_program; // still controls Session struct emission, just not `run()` body
   let run_func = if is_ascent_run {
      quote! {}
   } else {
      quote! {
         #[doc = "Runs the Ascent program to a fixed point (DD backend)."]
         pub fn run(&mut self) {
            #![allow(unused_imports, unused_mut, unused_variables, clippy::all)]
            #run_body
         }
      }
   };

   let methods = quote! {
      impl #impl_impl_generics #struct_name #impl_ty_generics #impl_where_clause {
         #run_func
         #session_accessor
         #summary_fn
         pub fn relation_sizes_summary(&self) -> ::std::string::String {
            ::std::string::String::new()
         }
         pub fn scc_times_summary(&self) -> ::std::string::String {
            ::std::string::String::new()
         }
      }
   };

   let assembled = quote! {
      #struct_and_default
      #methods
      #session_items
   };

   if !is_ascent_run {
      assembled
   } else {
      let inline_target: TokenStream = quote!(__run_res);
      let run_inline = match &blocker {
         None => phase1_run_body(mir, &inline_target),
         Some(_) => quote! {},
      };
      // Relation initialisations — evaluated here in the enclosing fn body
      // (so they can reference local parameters), stored into the struct
      // BEFORE `run_inline` copies the inputs into the dataflow.
      let mut rel_inits_inline = vec![];
      for rel in mir.relations_ir_relations.keys().sorted_by_key(|r| &r.name) {
         if let Some(init_expr) = mir.relations_metadata[rel].initialization.as_deref() {
            let name = &rel.name;
            rel_inits_inline.push(quote! {
               __run_res.#name = (#init_expr).into_iter().collect();
            });
         }
      }
      quote! {
         {{
            #![allow(unused_imports, unused_mut, unused_variables, clippy::all)]
            #assembled
            let mut __run_res: #struct_name #ty_ty_generics = #struct_name::default();
            #(#rel_inits_inline)*
            { #run_inline }
            __run_res
         }}
      }
   }
}

fn emit_struct_and_default(mir: &AscentMir, include_rel_init_in_default: bool) -> TokenStream {
   let mut relation_fields = vec![];
   let mut field_defaults = vec![];
   for rel in mir.relations_ir_relations.keys().sorted_by_key(|r| &r.name) {
      let name = &rel.name;
      let tuple_ty = tuple_type(&rel.field_types);
      let rel_attrs = &mir.relations_metadata[rel].attributes;
      relation_fields.push(quote! {
         #(#rel_attrs)*
         pub #name: ::std::vec::Vec<#tuple_ty>,
      });
      let init = mir.relations_metadata[rel].initialization.as_deref().cloned();
      let default = match (init, include_rel_init_in_default) {
         (Some(init_expr), true) => quote! { #name: (#init_expr).into_iter().collect(), },
         _ => quote! { #name: ::std::vec::Vec::new(), },
      };
      field_defaults.push(default);
   }

   let ty_signature = &mir.signatures.declaration;
   let (ty_impl_generics, _ty_ty_generics, ty_where_clause) = mir.signatures.split_ty_generics_for_impl();
   let (impl_impl_generics, impl_ty_generics, impl_where_clause) = mir.signatures.split_impl_generics_for_impl();
   let vis = &ty_signature.visibility;
   let struct_name = &ty_signature.ident;
   let struct_attrs = &ty_signature.attrs;

   quote! {
      #(#struct_attrs)*
      #vis struct #struct_name #ty_impl_generics #ty_where_clause {
         #(#relation_fields)*
      }
      impl #impl_impl_generics ::std::default::Default for #struct_name #impl_ty_generics #impl_where_clause {
         fn default() -> Self {
            #struct_name { #(#field_defaults)* }
         }
      }
   }
}

// ---------------------------------------------------------------------------
// Phase 1 run() body
// ---------------------------------------------------------------------------

/// Emit the shared dataflow-construction body inside a timely `scope`:
/// DD imports, per-relation `__<rel>_coll` init (via `init_coll`), unit
/// scope, rule SCCs, then per-relation `materialize_rel` + sink attach
/// (via `sink_ident` + `probe_expr`).
///
/// Returns `(body, hoists, hoist_count)`:
///   - `body` goes inside the dataflow closure.
///   - `hoists` must be emitted OUTSIDE it so user generator expressions that
///     reference non-'static borrows can land in `'static` Vecs before the
///     closure moves them.
///   - `hoist_count` is the total number of `__hoist_gen_*` bindings produced.
///     Callers running the closure as `Fn` (multi-worker `execute_batch`)
///     use this to emit per-call clone-rebindings so the captured Vecs
///     aren't moved out by inner DD operator closures.
fn emit_closure_body(
   mir: &AscentMir,
   sorted_rels: &[&RelationIdentity],
   init_coll: impl Fn(&RelationIdentity) -> TokenStream,
   sink_ident: impl Fn(&RelationIdentity) -> TokenStream,
   probe_expr: TokenStream,
   is_batch: bool,
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
   // the rest of the dataflow's diff (isize incremental, Present batch).
   let unit_seed_ty = if is_batch {
      quote! { ::ascent::dd::BatchDiff }
   } else {
      quote! { isize }
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
   if is_batch {
      body.extend(quote! {
         let __unit_scope: ::ascent::dd::differential_dataflow::VecCollection<_, (), ::ascent::dd::BatchDiff> = {
            use ::ascent::dd::differential_dataflow::input::Input;
            let (mut __s, __c) = scope.new_collection_from_raw::<(), ::ascent::dd::BatchDiff, _>(
               ::std::iter::once(((), (), ::ascent::dd::BATCH_DIFF_ONE))
            );
            __s.flush();
            __c
         };
      });
   } else {
      body.extend(quote! {
         let __unit_scope: ::ascent::dd::differential_dataflow::VecCollection<_, (), isize> = {
            use ::ascent::dd::differential_dataflow::input::Input;
            let (mut __s, __c) = scope.new_collection_from(::std::iter::once(()));
            __s.advance_to(1);
            __s.flush();
            __c
         };
      });
   }
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
   for rel in sorted_rels {
      let coll = relation_coll_var(&rel.name);
      let sink_id = sink_ident(rel);
      // FlowLog pattern: after scope.iterative returns, a recursive IDB
      // collection is already threshold'd inside the loop; re-thresholding
      // via `materialize_rel` is pure wasted work. A non-recursive collection
      // may have concat-duplicates; `consolidate` handles that cheaply.
      // Lattices still need reduce (handled inside materialize_rel).
      // Attach sink WITHOUT additional consolidate/materialize in batch mode.
      // Recursive IDBs are already threshold'd inside `scope.iterative`;
      // non-recursive collections are consolidated at input. FlowLog-style.
      let final_expr = if is_batch && !rel.is_lattice {
         quote! { (#coll).clone() }
      } else {
         materialize_rel(rel, quote! { #coll }, is_batch)
      };
      body.extend(quote! {
         #sink_id.attach(&(#final_expr), #probe_expr);
      });
   }
   (body, hoists, hoist_counter)
}

fn phase1_run_body(mir: &AscentMir, target: &TokenStream) -> TokenStream {
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
         // `.clone()` because `execute_batch`'s build closure is `Fn` (may
         // be invoked per-worker under multi-worker DD). Each invocation
         // gets its own Vec; DD's arrange operators hash-partition so
         // redundant-load-all-workers is correct.
         quote! { let mut #coll = sealer.input(scope, #in_var.clone()); }
      },
      |rel| {
         // Batch attach target: inner sink clone moved into the closure.
         let sink_inner = relation_sink_inner(&rel.name);
         quote! { #sink_inner }
      },
      quote! { probe },
      false,
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
/// `threshold_semigroup` distinct. Rejects features Present can't express
/// (negation via antijoin requires Abelian; lattice reduce needs isize;
/// aggregators cross strata).
fn compile_mir_dd_batch(mir: &AscentMir, is_ascent_run: bool) -> syn::Result<TokenStream> {
   use crate::ascent_mir::MirBodyItem;
   // Feature gate: batch mode only supports pure-datalog joins + generators.
   // In MIR, negation lands as `MirBodyItem::Agg` with a "not" aggregator —
   // `is_not_aggregator` discriminates it from real aggregators.
   for scc in &mir.sccs {
      for rule in &scc.rules {
         for item in &rule.body_items {
            match item {
               MirBodyItem::Cond(_) | MirBodyItem::Clause(_) | MirBodyItem::Generator(_) => {},
               MirBodyItem::Agg(agg) => {
                  if is_not_aggregator(&agg.aggregator) {
                     return Err(syn::Error::new(
                        mir.signatures.declaration.ident.span(),
                        "batch mode does not support negation (`!rel(...)`). Use `mode = \"incremental\"` instead.",
                     ));
                  } else {
                     return Err(syn::Error::new(
                        mir.signatures.declaration.ident.span(),
                        "batch mode does not support aggregation (`agg ...`). Use `mode = \"incremental\"` instead.",
                     ));
                  }
               },
            }
         }
      }
   }
   for rel in mir.relations_ir_relations.keys() {
      if rel.is_lattice {
         return Err(syn::Error::new(
            mir.signatures.declaration.ident.span(),
            format!(
               "batch mode does not support lattice relations (`{}`). Use `mode = \"incremental\"` instead.",
               rel.name
            ),
         ));
      }
   }

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

// ---------------------------------------------------------------------------
// Session codegen (incremental API)
// ---------------------------------------------------------------------------
//
// For each user-declared program `pub struct Tc;` we emit a sibling
// `pub struct TcSession` that owns a long-lived timely worker and per-relation
// input sessions + output sinks + running net-multiplicity state. Users:
//
//   let mut s = Tc::session();
//   s.edge_insert((1, 2));  s.edge_insert((2, 3));
//   s.commit();
//   let p = s.path_snapshot();   // or s.path_deltas() for this epoch's diffs
//
// `commit()` advances every input to the next epoch and steps the worker
// to that frontier. The dataflow construction inside `Session::new` is
// literally the same shape as `run()`'s — the only differences are that the
// InputSessions and Sinks land on the struct instead of being captured into
// a one-shot closure.

fn session_struct_name(base: &Ident) -> Ident { Ident::new(&format!("{}Session", base), base.span()) }

/// `run()` as a Session wrapper. Emits the SAME observable behavior as
/// `phase1_run_body` — seed the dataflow with `self.<rel>`, run to fixed
/// point, write final relation contents back to `self` — but delegates
/// the dataflow construction to `Session::new()`. This cuts generated
/// code roughly in half for programs that also emit a session (the
/// common case).
fn run_via_session_body(mir: &AscentMir, target: &TokenStream, session_name: &Ident) -> TokenStream {
   let sorted_rels = mir.relations_ir_relations.keys().sorted_by_key(|r| &r.name).collect_vec();

   // Drain each `self.<rel>: Vec<Tup>` into the session's InputSession.
   // `drain(..)` moves ownership out of self — batch `run()` semantics
   // have always moved input into the dataflow and written it back out.
   let seed_inputs: Vec<TokenStream> = sorted_rels
      .iter()
      .map(|rel| {
         let name = &rel.name;
         quote! {
            for __t in #target.#name.drain(..) {
               __s.#name.insert(__t);
            }
         }
      })
      .collect();

   // After commit, drain each sink directly and call `into_vec` — matches
   // the old `phase1_run_body` exactly (preserves delta-insertion order,
   // which several tests rely on for `assert_eq!` stability). `_snapshot`
   // would round-trip through a `HashMap` and lose ordering.
   let drain_outputs: Vec<TokenStream> = sorted_rels
      .iter()
      .map(|rel| {
         let name = &rel.name;
         let sink_field = Ident::new(&format!("{}_sink", name), name.span());
         quote! {
            #target.#name = ::std::mem::replace(
               &mut __s.#sink_field,
               ::ascent::dd::Sink::new(),
            ).into_vec();
         }
      })
      .collect();

   // Drive the dataflow explicitly — NOT via `commit()`. `commit()` calls
   // `refresh_deltas()` which drains each sink into the `_state` book-
   // keeping map, leaving the sink itself empty. We want to read sinks
   // directly (matches old `run()` semantics), so we advance+step here
   // and leave sinks untouched.
   quote! {
      let mut __s = #session_name::new();
      #(#seed_inputs)*
      __s.advance_all_to(1);
      __s.step_until(1);
      #(#drain_outputs)*
   }
}

fn emit_session(mir: &AscentMir) -> TokenStream {
   let ty_signature = &mir.signatures.declaration;
   let base_name = &ty_signature.ident;
   let session_name = session_struct_name(base_name);
   let sorted_rels = mir.relations_ir_relations.keys().sorted_by_key(|r| &r.name).collect_vec();

   // Per-relation field declarations / init / methods.
   let mut input_fields = vec![];
   let mut sink_fields = vec![];
   let mut state_fields = vec![];
   let mut last_delta_fields = vec![];
   let mut per_rel_methods = vec![];
   let mut input_advances = vec![]; // inside commit()
   let mut sink_drains = vec![]; // inside commit()

   for rel in &sorted_rels {
      let name = &rel.name;
      let tup_ty = tuple_type(&rel.field_types);
      // Public input field named after the relation itself. Users drive DD
      // directly — `sess.edge.insert(...)`, `sess.edge.advance_to(5)`,
      // `sess.edge.flush()`. Full `InputSession` API available.
      let input_field = name.clone();
      let sink_field = Ident::new(&format!("{}_sink", name), name.span());
      let state_field = Ident::new(&format!("{}_state", name), name.span());
      let last_delta_field = Ident::new(&format!("{}_last_deltas", name), name.span());
      let insert_m = Ident::new(&format!("{}_insert", name), name.span());
      let remove_m = Ident::new(&format!("{}_remove", name), name.span());
      let snapshot_m = Ident::new(&format!("{}_snapshot", name), name.span());
      let deltas_m = Ident::new(&format!("{}_deltas", name), name.span());

      input_fields.push(quote! {
         pub #input_field: ::ascent::dd::SessionInput<#tup_ty>,
      });
      sink_fields.push(quote! {
         /// Raw DD output sink for this relation. Call
         /// `sess.<rel>_sink.drain_deltas()` for direct access to the
         /// accumulated `Vec<(T, isize)>` — bypasses the `_deltas()` /
         /// `_snapshot()` / `refresh_deltas()` book-keeping.
         pub #sink_field: ::ascent::dd::Sink<#tup_ty>,
      });
      state_fields.push(quote! {
         #state_field: ::std::collections::HashMap<#tup_ty, isize>,
      });
      last_delta_fields.push(quote! {
         #last_delta_field: ::std::vec::Vec<(#tup_ty, isize)>,
      });

      per_rel_methods.push(quote! {
         /// Shorthand for `sess.<rel>.insert(tuple)` — DD's `InputSession`
         /// method. Use the field directly for `advance_to`, `flush`,
         /// `update_at`, `time`, etc.
         pub fn #insert_m(&mut self, tuple: #tup_ty) { self.#input_field.insert(tuple); }
         /// Shorthand for `sess.<rel>.remove(tuple)`.
         pub fn #remove_m(&mut self, tuple: #tup_ty) { self.#input_field.remove(tuple); }
         /// Snapshot of net-positive tuples, built from the running state
         /// map. Only reflects deltas that have been drained from the sink
         /// (via `commit()` or `refresh_deltas()`).
         pub fn #snapshot_m(&self) -> ::std::vec::Vec<#tup_ty> {
            self.#state_field.iter().filter(|(_, c)| **c > 0).map(|(t, _)| t.clone()).collect()
         }
         /// Deltas captured on the last `commit()` / `refresh_deltas()`.
         /// Positive diff = insertion, negative = retraction. Reads are
         /// non-destructive and return a clone.
         pub fn #deltas_m(&self) -> ::std::vec::Vec<(#tup_ty, isize)> {
            self.#last_delta_field.clone()
         }
      });

      input_advances.push(quote! {
         self.#input_field.advance_to(next);
         self.#input_field.flush();
      });
      // Drain sink → update running state + cache for `_deltas()` reads.
      sink_drains.push(quote! {
         {
            let deltas = self.#sink_field.drain_deltas();
            for (t, d) in &deltas {
               *self.#state_field.entry(t.clone()).or_insert(0) += d;
            }
            self.#last_delta_field = deltas;
         }
      });
   }

   // Build-dataflow body used inside `Session::new`. Mirrors `phase1_run_body`
   // but with inputs/sinks declared inline (not via Sealer) so they live on
   // the returned tuple.
   let build_body = emit_session_build_body(mir, &sorted_rels);

   // Tuple type of build-closure return value: ((input_a, input_b, ...), (sink_a, sink_b, ...), probe).
   let input_field_idents: Vec<Ident> = sorted_rels.iter().map(|r| r.name.clone()).collect();
   let sink_field_idents: Vec<Ident> =
      sorted_rels.iter().map(|r| Ident::new(&format!("{}_sink", r.name), r.name.span())).collect();

   // State + last-delta field initializers (all default/empty).
   let state_inits: Vec<TokenStream> = sorted_rels
      .iter()
      .flat_map(|r| {
         let s = Ident::new(&format!("{}_state", r.name), r.name.span());
         let d = Ident::new(&format!("{}_last_deltas", r.name), r.name.span());
         [
            quote! { #s: ::std::collections::HashMap::new(), },
            quote! { #d: ::std::vec::Vec::new(), },
         ]
      })
      .collect();

   let input_init_moves: Vec<TokenStream> =
      input_field_idents.iter().map(|i| quote! { #i: handles.inputs.#i, }).collect();
   let sink_init_moves: Vec<TokenStream> =
      sink_field_idents.iter().map(|i| quote! { #i: handles.sinks.#i, }).collect();

   // Anonymous struct "handles" with named fields — simpler than deep tuples.
   // Use a private sub-module per session to scope the helper struct.

   quote! {
      /// Live incremental session for the program.
      ///
      /// # Convenience path
      /// ```ignore
      /// let mut s = MyProgram::session();
      /// s.my_rel_insert((1, 2));
      /// s.commit();
      /// let out = s.my_out_deltas();
      /// ```
      ///
      /// # Raw DD path (no closures — worker lives outside any `dataflow(|scope| …)`)
      /// ```ignore
      /// let mut s = MyProgram::session();
      ///
      /// // InputSession is exposed directly — full DD API.
      /// for p in 0..n { s.manages.insert((p / 2, p)); }
      /// s.advance_all_to(1);          // bump every input's logical time + flush
      /// s.step_until(1);              // drive worker.step() until probe passes
      ///
      /// // Sinks are public: pull deltas without the snapshot book-keeping.
      /// for (tup, diff) in s.output_sink.drain_deltas() {
      ///     println!("{:?} {:+}", tup, diff);
      /// }
      ///
      /// // Need a DD primitive we didn't wrap? The worker is public too:
      /// s.worker.step_or_park_timeout(Some(std::time::Duration::from_millis(10)));
      /// ```
      ///
      /// Public fields: `<rel>` (InputSession), `<rel>_sink` (Sink), `worker`,
      /// `probe`, `epoch`. Drive from any control flow — sync loop, `mpsc::Receiver`,
      /// a tokio task — DD doesn't care.
      pub struct #session_name {
         /// The underlying timely worker. Public so users can call DD
         /// primitives we don't wrap (e.g. `step_or_park_timeout`).
         pub worker: ::ascent::dd::SessionWorker,
         /// Output frontier probe. Cheap to clone; use `.less_than(&t)` to
         /// check whether an epoch has cleared.
         pub probe: ::ascent::dd::timely::dataflow::ProbeHandle<u32>,
         /// Epoch counter used by the convenience `commit()` — advanced by 1
         /// each call. Read-only for external observers; modify only via
         /// `commit()` or by driving inputs directly.
         pub epoch: u32,
         #(#input_fields)*
         #(#sink_fields)*
         #(#state_fields)*
         #(#last_delta_fields)*
      }

      impl #session_name {
         pub fn new() -> Self {
            struct __Handles {
               inputs: __Inputs,
               sinks: __Sinks,
               probe: ::ascent::dd::timely::dataflow::ProbeHandle<u32>,
            }
            #[allow(non_camel_case_types)]
            struct __Inputs { #(#input_fields)* }
            #[allow(non_camel_case_types)]
            struct __Sinks { #(#sink_fields)* }

            let (worker, handles): (_, __Handles) = ::ascent::dd::build_session_worker(|scope| {
               #build_body
            });

            Self {
               worker,
               probe: handles.probe,
               epoch: 0,
               #( #input_init_moves )*
               #( #sink_init_moves )*
               #( #state_inits )*
            }
         }

         /// Advance all inputs to the next logical epoch, flush, drive the
         /// worker to that frontier, and drain sinks. Convenience for the
         /// simple case — for fine-grained control, use the `InputSession`
         /// handles directly + `step_until()` / `refresh_deltas()`.
         ///
         /// After this returns, `<rel>_snapshot` reflects the new fixed
         /// point and `<rel>_deltas` holds the change set.
         pub fn commit(&mut self) {
            let next = self.epoch + 1;
            #(#input_advances)*
            self.step_until(next);
            self.epoch = next;
            self.refresh_deltas();
         }

         /// Advance EVERY input's logical time to `time` and flush its
         /// buffer. Required before `step_until(time)` will make progress —
         /// DD's output frontier is the join of input frontiers, so any
         /// input still at an earlier time stalls everything downstream.
         /// Use this even for derived relations you never `.insert()` into;
         /// they still have an `InputSession` handle whose time must advance.
         pub fn advance_all_to(&mut self, time: u32) {
            #(
               self.#input_field_idents.advance_to(time);
               self.#input_field_idents.flush();
            )*
         }

         /// Step the underlying timely worker once. Use when driving DD
         /// manually (you've flushed inputs yourself and want to make
         /// progress without advancing the fused epoch).
         pub fn step(&mut self) { self.worker.step(); }

         /// Drive the worker until the output frontier has passed `time`.
         /// Cheap no-op if already ahead.
         pub fn step_until(&mut self, time: u32) {
            while self.probe.less_than(&time) {
               self.worker.step();
            }
         }

         /// Drive the worker until outputs reflect every input's current
         /// epoch. Call after advancing+flushing inputs at mixed times to
         /// let DD catch up without picking a specific frontier.
         pub fn step_until_idle(&mut self) {
            let __max: u32 = [#(*self.#input_field_idents.time()),*]
               .into_iter()
               .max()
               .unwrap_or(0);
            self.step_until(__max);
         }

         /// Read-only access to the output probe — useful for
         /// `probe.less_than(t)` / `probe.with_frontier(|f| …)` checks.
         pub fn probe(&self) -> &::ascent::dd::timely::dataflow::ProbeHandle<u32> {
            &self.probe
         }

         /// Drain every relation's sink into its running-state map + cache
         /// the drained vec for `_deltas()` reads. Call after stepping
         /// manually if you've bypassed `commit()`.
         pub fn refresh_deltas(&mut self) {
            #(#sink_drains)*
         }

         #(#per_rel_methods)*
      }

      impl ::std::default::Default for #session_name {
         fn default() -> Self { Self::new() }
      }
   }
}

/// Emit the body of the closure passed to `build_session_worker`. Creates
/// inputs + sinks + probe as local lets, builds the dataflow like the batch
/// backend, and returns a `__Handles` struct.
fn emit_session_build_body(mir: &AscentMir, sorted_rels: &[&RelationIdentity]) -> TokenStream {
   let mut setup = TokenStream::new();

   // Per-relation: create InputSession + initial (non-mut) coll + Sink.
   // `emit_closure_body` will later re-bind `__<rel>_coll` as `let mut`
   // so the `.concat` accumulation pattern works.
   for rel in sorted_rels {
      let name = &rel.name;
      let tup_ty = tuple_type(&rel.field_types);
      let input = name.clone();
      let sink = Ident::new(&format!("{}_sink", name), name.span());
      let coll = relation_coll_var(name);
      setup.extend(quote! {
         let mut #input: ::ascent::dd::SessionInput<#tup_ty> = ::ascent::dd::differential_dataflow::input::InputSession::new();
         let #coll = #input.to_collection(scope);
         let #sink: ::ascent::dd::Sink<#tup_ty> = ::ascent::dd::Sink::new();
      });
   }

   // Probe must be in scope before `emit_closure_body` emits the sink
   // `attach(&mut probe)` calls.
   setup.extend(quote! {
      let mut probe: ::ascent::dd::timely::dataflow::ProbeHandle<u32> = ::ascent::dd::timely::dataflow::ProbeHandle::new();
   });

   // Shared core: imports, mut coll rebind, unit scope, SCC bodies, attach.
   // Session path is FnOnce (single-worker, persistent). No hoist rebinds
   // needed — the build closure runs exactly once inside `worker.dataflow`,
   // inner DD operators can safely move hoisted Vecs without the Fn trap.
   let (body, hoists, _hoist_count) = emit_closure_body(
      mir,
      sorted_rels,
      |rel| {
         // Session init: rebind the per-rel `__<rel>_coll` (already created
         // above from the InputSession) as `let mut` for rule accumulation.
         let coll = relation_coll_var(&rel.name);
         quote! { let mut #coll = #coll; }
      },
      |rel| {
         let sink = Ident::new(&format!("{}_sink", rel.name), rel.name.span());
         quote! { #sink }
      },
      quote! { &mut probe },
      false,
   );
   setup.extend(hoists);
   setup.extend(body);

   // Assemble `__Handles { inputs, sinks, probe }` for `Session::new()` to
   // unpack into struct fields.
   let input_assigns: Vec<TokenStream> = sorted_rels
      .iter()
      .map(|r| {
         let n = r.name.clone();
         quote! { #n, }
      })
      .collect();
   let sink_assigns: Vec<TokenStream> = sorted_rels
      .iter()
      .map(|r| {
         let n = Ident::new(&format!("{}_sink", r.name), r.name.span());
         quote! { #n, }
      })
      .collect();

   setup.extend(quote! {
      __Handles {
         inputs: __Inputs { #(#input_assigns)* },
         sinks: __Sinks { #(#sink_assigns)* },
         probe,
      }
   });

   setup
}

// ---------------------------------------------------------------------------
// SCC compilation
// ---------------------------------------------------------------------------

fn compile_scc(scc: &MirScc, scc_idx: usize, hoist_counter: &mut usize, is_batch: bool) -> (TokenStream, TokenStream) {
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
   use crate::ascent_mir::MirBodyItem;
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
      let (body, pre) = compile_rule_with_head_target(rule, &head_target, &outer_scope, hoist_counter);
      out.extend(body);
      hoists.extend(pre);
   }
   // FlowLog pattern (batch mode only): consolidate each non-looping SCC's
   // head `__<rel>_coll` after the rules have concat'd into it. Multiple
   // rules emitting the same tuple (e.g. CSPA's
   // `value_flow(x,x) <-- assign(_,x)` + `value_flow(x,x) <-- assign(x,_)`)
   // produce duplicates; without consolidation they propagate into the
   // looping SCC's seed as redundant diffs that every iteration's
   // `threshold_semigroup` has to dedupe — O(iter × duplicates) wasted work.
   // Incremental/session path handles this through Variable/isize semantics.
   // Note: FlowLog consolidates ONCE per relation after ALL non-recursive
   // rules producing it have concat'd. Ascent's MIR splits rules into
   // separate SCCs, so consolidating in every non-looping SCC would emit
   // N consolidates per relation. Downstream (looping-SCC threshold /
   // sink-attach) dedups anyway — skip per-SCC consolidate.
   let _ = is_batch;
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

   // 1. FlowLog naming: `in_<rel>` for entered non-recursive Collections,
   //    `in_<arr>` for entered arrangements. Consolidate before entering:
   //    Ascent's MIR splits non-recursive rules into separate SCCs, so
   //    `__<rel>_coll` is a chain of N concats. FlowLog consolidates after
   //    their equivalent chain (`valueflow = t1.concat(t2).concat(t3).consolidate()`).
   //    Without consolidate, duplicates enter the iterative scope and cause
   //    threshold_semigroup's "first-seen" accounting to loop more iterations.
   let mut inner_body_only_bindings = TokenStream::new();
   for rel in &body_only_rels {
      let name = &rel.name;
      let coll = relation_coll_var(name);
      let seed = Ident::new(&format!("in_{}", name), name.span());
      inner_body_only_bindings.extend(quote! {
         let #seed = #coll.clone().consolidate().enter(inner);
      });
   }
   let mut inner_dyn_seed_bindings = TokenStream::new();
   for rel in &dyn_rels {
      let name = &rel.name;
      let coll = relation_coll_var(name);
      let seed = Ident::new(&format!("in_{}", name), name.span());
      inner_dyn_seed_bindings.extend(quote! {
         let #seed = #coll.clone().consolidate().enter(inner);
      });
   }

   // 2. Variables for dynamic rels + shadowed body-reading bindings.
   //    Annotate the element type explicitly: in mutually-recursive SCCs,
   //    Rust can't infer the Variable's element type through the cycle.
   let mut var_decls = TokenStream::new();
   // Enter the outer unit collection into this iterative scope so fact /
   // generator-first rules inside the SCC can seed from it.
   var_decls.extend(quote! {
      let __unit_inner = __unit_scope.clone().enter(inner);
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
      if is_batch {
         var_decls.extend(quote! {
            let (#var, #read): (
               ::ascent::dd::SemigroupVariable<_, #tup_ty, ::ascent::dd::BatchDiff>,
               ::ascent::dd::differential_dataflow::VecCollection<_, #tup_ty, ::ascent::dd::BatchDiff>,
            ) = ::ascent::dd::SemigroupVariable::new(inner, Product::new(Default::default(), 1));
         });
      } else {
         var_decls.extend(quote! {
            let (#var, #read): (
               Variable<_, #tup_ty, isize>,
               ::ascent::dd::differential_dataflow::VecCollection<_, #tup_ty, isize>,
            ) = Variable::new(inner, Product::new(Default::default(), 1));
         });
      }
   }

   // 3. Shadow `__<rel>_coll` inside the scope so rule bodies keep their
   //    unchanged textual form (`__<rel>_coll.map(...)`). For body-only
   //    rels, the shadow points at the entered seed. For dynamic rels,
   //    it's a clone of the variable's deref'd Collection.
   let mut shadow_bindings = TokenStream::new();
   for rel in &body_only_rels {
      let name = &rel.name;
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
   // Only emit arrangements for `(rel, indices)` pairs ACTUALLY REFERENCED
   // by some rule body's clause in this SCC. MIR's `dynamic_relations` /
   // `body_only_relations` include full-index IrRelations that aren't used
   // by any join — emitting them creates dead DD operators that still cost
   // per-iteration maintenance. Pre-pass over rule bodies to collect the
   // used set. (Upstream TODO in ascent_mir.rs: "we can add only indices
   // used in bodies in the scc, that requires the codegen to be updated.")
   use crate::ascent_mir::MirBodyItem;
   let mut used_arr_names: std::collections::HashSet<Ident> =
      std::collections::HashSet::new();
   for rule in &scc.rules {
      for (i, item) in rule.body_items.iter().enumerate() {
         if let MirBodyItem::Clause(cl) = item {
            // Empty indices = no-key arrangement — never used via shared
            // arrangement path (compile_join_clause's `can_use_shared`
            // requires non-empty indices). Skip emitting.
            if cl.rel.indices.is_empty() {
               continue;
            }
            // First-clause (i==0): only needed as LHS-shared target if the
            // rule has >=2 clauses (prior_rel optimization). Otherwise it's
            // scanned via compile_first_clause which doesn't need the
            // arrangement.
            let is_first = i == 0;
            let rule_has_join = rule.body_items.iter().filter(|it| matches!(it, MirBodyItem::Clause(_))).count() >= 2;
            if is_first && !rule_has_join {
               continue;
            }
            let name = format!(
               "__arr_{}_indices_{}",
               cl.rel.relation.name,
               cl.rel.indices.iter().map(|i| i.to_string()).collect::<Vec<_>>().join("_")
            );
            used_arr_names.insert(Ident::new(&name, cl.rel.relation.name.span()));
         }
      }
   }
   let mut outer_arrs = TokenStream::new();
   let mut outer_arr_enters = TokenStream::new();
   let mut inner_arrs = TokenStream::new();
   let mut seen_arrs: std::collections::HashSet<Ident> = std::collections::HashSet::new();
   let body_only_rel_names: std::collections::HashSet<String> =
      scc.body_only_relations.keys().map(|r| r.name.to_string()).collect();
   let rel_iter = scc
      .body_only_relations
      .iter()
      .chain(scc.dynamic_relations.iter())
      .sorted_by_cached_key(|(rel, _)| rel.name.to_string());
   for (rel_ident, ir_rels) in rel_iter {
      let is_body_only = body_only_rel_names.contains(&rel_ident.name.to_string());
      for ir_rel in ir_rels.iter().sorted_by_cached_key(|r| r.ir_name()) {
         let arr_name = shared_arr_ident(ir_rel);
         if !used_arr_names.contains(&arr_name) {
            continue;
         }
         if !seen_arrs.insert(arr_name.clone()) {
            continue;
         }
         if is_body_only {
            outer_arrs.extend(emit_shared_arrangement_binding(ir_rel));
            outer_arr_enters.extend(quote! {
               let #arr_name = #arr_name.enter(inner);
            });
         } else {
            inner_arrs.extend(emit_shared_arrangement_binding(ir_rel));
         }
      }
   }

   // 5. Rule productions. Split into NON-recursive (body touches no dyn_rel
   //    of this SCC) and RECURSIVE (body touches ≥1 dyn_rel). FlowLog pattern:
   //    non-recursive rules lowered OUTSIDE `scope.iterative` — they fire
   //    ONCE on outer-scope `__<rel>_coll`, producing stable base contributions.
   //    Recursive rules lowered INSIDE — they feed the Variable/threshold loop.
   //    Without this split, non-recursive rules re-fire every iteration
   //    producing identical tuples that `threshold_semigroup` has to dedup —
   //    O(iter × EDB-size) wasted work that cascades with recursive growth.
   let dyn_names_set: std::collections::HashSet<String> = dyn_rels.iter().map(|r| r.name.to_string()).collect();
   let is_rule_recursive = |rule: &MirRule| -> bool {
      use crate::ascent_mir::MirBodyItem;
      rule.body_items.iter().any(|item| match item {
         MirBodyItem::Clause(cl) => dyn_names_set.contains(&cl.rel.relation.name.to_string()),
         MirBodyItem::Agg(agg) => dyn_names_set.contains(&agg.rel.relation.name.to_string()),
         MirBodyItem::Cond(_) | MirBodyItem::Generator(_) => false,
      })
   };

   // Non-recursive rules: emit on outer-scope `__<rel>_coll`. Head goes to
   // the outer coll via `relation_coll_var`. These fire once; their output
   // is what the scope.iterative's `inner_dyn_seed_bindings` pulls in.
   let non_rec_rules: Vec<&MirRule> = scc.rules.iter().filter(|r| !is_rule_recursive(r)).collect();
   let rec_rules: Vec<&MirRule> = scc.rules.iter().filter(|r| is_rule_recursive(r)).collect();

   let mut non_rec_body = TokenStream::new();
   let mut non_rec_hoists = TokenStream::new();
   let outer_scope = Ident::new("scope", Span::call_site());
   let non_rec_head_target = |name: &Ident| relation_coll_var(name);
   for rule in &non_rec_rules {
      let summary = format!("rule [non-rec] {}", mir_rule_summary(rule));
      non_rec_body.extend(quote! { ::ascent::internal::comment(#summary); });
      let (body, pre) =
         compile_rule_with_head_target(rule, &non_rec_head_target, &outer_scope, hoist_counter);
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
      let (head_exprs, pre) = compile_rule_head_exprs(rule, &inner_scope, hoist_counter);
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
      let concat_chain: TokenStream = if rule_idents.is_empty() {
         quote! { #seed.clone() }
      } else {
         let first = &rule_idents[0];
         let rest: Vec<TokenStream> = rule_idents[1..]
            .iter()
            .map(|r| quote! { .concat(#r.clone()) })
            .collect();
         quote! {
            #first.clone() #(#rest)* .concat(#seed.clone())
         }
      };
      let materialized = materialize_rel(rel, concat_chain, is_batch);
      bind_leave_exprs.push(quote! {
         {
            let #next = #materialized;
            #var.set(#next.clone());
            #next.leave()
         }
      });
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

   // Batch mode needs a `TotalOrder` inner timestamp paired with its `()`
   // outer so `Product<(), Iter>` satisfies `threshold_semigroup`'s bound.
   // Incremental mode keeps `u32` outer + `u32` inner for Variable::new's
   // arbitrary-Lattice flexibility.
   let inner_iter_ty = if is_batch { quote! { u16 } } else { quote! { u32 } };
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

// ---------------------------------------------------------------------------
// Small helpers
// ---------------------------------------------------------------------------

/// Singleton-unit seed collection. Used when a rule body starts with something
/// that needs an upstream Collection to `.flat_map`/`.filter` over but the
/// first item is a `Cond`, `Generator`, or a fact with a head-only projection.
///
/// We require a hoisted binding `__unit_scope` to exist in the outer-closure
/// (created via `scope.new_collection_from(once(()))`). Inside iterative
/// scopes a matching `__unit_inner` binding — created by `.enter(inner)` —
/// is expected. `scope_ident` picks which one. Both are cheap `.clone()`-able.
fn unit_seed(scope_ident: &Ident) -> TokenStream {
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
fn materialize_rel(rel: &RelationIdentity, coll_expr: TokenStream, is_batch: bool) -> TokenStream {
   if !rel.is_lattice {
      if is_batch {
         // Batch path: Present-diff set relation. `threshold_semigroup`
         // on an arranged-by-self trace emits each key once, the first
         // time it is seen — exactly the canonical DD datalog pattern
         // (graspan1.rs). Insert-only semantics, no multiplicity math.
         return quote! {
            {
               // DD 0.20: `ThresholdTotal::threshold_semigroup` is impl'd
               // DIRECTLY on `VecCollection` (i.e. `Collection<G, Vec<...>>`)
               // — no need to call `.arrange_by_self()` first. FlowLog emits
               // `coll.concat(...).threshold_semigroup(...)` in exactly this
               // form. Removing the extra `arrange_by_self` saves a whole
               // DD arrangement operator per relation (3 per CSPA SCC).
               use ::ascent::dd::differential_dataflow::operators::ThresholdTotal;
               (#coll_expr)
                  .threshold_semigroup(|_, _, old: ::std::option::Option<&::ascent::dd::BatchDiff>| {
                     old.is_none().then_some(::ascent::dd::BATCH_DIFF_ONE)
                  })
            }
         };
      }
      // Session path: isize-diff set relation. Emit diff 1 iff net
      // multiplicity > 0. The `> 0` check (not just `!= 0`) is required
      // for antijoin correctness — antijoin's per-timestamp diffs can
      // transiently cancel without being dropped; `reduce` with explicit
      // net check stays correct where `.distinct()` would mis-emit.
      return quote! {
         {
            (#coll_expr).map(|__x| (__x, ()))
               .reduce(|_k, __input, __output| {
                  let __net: isize = __input.iter().map(|(_, d)| *d).sum();
                  if __net > 0 { __output.push(((), 1)); }
               })
               .map(|(__k, ())| __k)
         }
      };
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
   quote! {
      {
         let __keyed = (#coll_expr).map(|#destruct| (#key_tuple, #lat_ident));
         let __reduced = __keyed.reduce(|_key, __input, __output| {
            let mut __acc: ::std::option::Option<#last_ty> = ::std::option::Option::None;
            for (__v, __d) in __input.iter() {
               if *__d > 0 {
                  match &mut __acc {
                     ::std::option::Option::None => { __acc = ::std::option::Option::Some((*__v).clone()); }
                     ::std::option::Option::Some(__a) => {
                        <#last_ty as ::ascent::Lattice>::join_mut(__a, (*__v).clone());
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
   }
}

/// Name of the shared arrangement binding for a given `IrRelation`.
/// Matches `compile_join_clause`'s lookup — both sides must agree.
fn shared_arr_ident(ir_rel: &crate::ascent_hir::IrRelation) -> Ident {
   let base = ir_rel.ir_name();
   Ident::new(&format!("__arr_{}", base), base.span())
}

/// Emits `let __arr_<ir_name> = <coll>.flat_map(|tuple| Some(((key_cols,), (val_cols,)))).arrange_by_key();`
/// for one shared arrangement. Key columns come from `ir_rel.indices` (already
/// sorted ascending → canonical). Value columns are every other position in
/// natural order. Both sides of any later `join_core` must mirror this layout.
fn emit_shared_arrangement_binding(ir_rel: &crate::ascent_hir::IrRelation) -> TokenStream {
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
   let key_tuple = tuple_tokens(&key_parts);
   let val_tuple = tuple_tokens(&val_parts);
   quote! {
      let #arr_ident = #rel_coll.clone().flat_map(move |#destruct| {
         ::std::option::Option::Some((#key_tuple, #val_tuple))
      }).arrange_by_key();
   }
}

fn relation_input_var(name: &Ident) -> Ident { Ident::new(&format!("__{}_in", name), name.span()) }
fn relation_coll_var(name: &Ident) -> Ident { Ident::new(&format!("__{}_coll", name), name.span()) }
fn relation_sink_outer(name: &Ident) -> Ident { Ident::new(&format!("__{}_sink", name), name.span()) }
fn relation_sink_inner(name: &Ident) -> Ident { Ident::new(&format!("__{}_sink_inner", name), name.span()) }

fn tuple_of_idents(idents: &[Ident]) -> TokenStream {
   let exprs: Vec<Expr> = idents.iter().map(|i| syn::parse_quote!(#i)).collect();
   let t = tuple(&exprs);
   quote! { #t }
}

/// `(x.clone(), y.clone(), …)` — use when every ident might be a ref that
/// needs to escape a closure that borrows from its frame.
///
/// IMPORTANT: uses `ToOwned::to_owned` rather than `.clone()`. When `v` is a
/// reference (which it typically is in our user-visible contexts), Rust's
/// method resolution picks `&T: Clone` which returns another `&T` — the
/// reference doesn't escape its closure frame. `ToOwned::to_owned` on `&T`
/// is unambiguously `<T as Clone>::clone(&*v)` which returns the owned `T`.
fn clone_all_tuple(idents: &[Ident]) -> TokenStream {
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
fn build_expr_tuple(args: &[Expr]) -> TokenStream {
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
