//! Rule body → DD Collection expression.
//!
//! Compiles one MIR rule's body into a `Collection` of bound-var tuples,
//! handling join planning, filters, negation/antijoin, aggregators, `let`,
//! `if let`, and generators. The two public entry points
//! (`compile_rule_head_exprs`, `compile_rule_with_head_target`) are consumed
//! by `scc.rs`.

use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::Type;
use syn::spanned::Spanned;

use ascent_mir::syn_utils::{expr_get_vars, pattern_get_vars};
use ascent_mir::utils::{expr_to_ident, is_wild_card, tuple_type};
use ascent_mir::{CondClause, IrAggClause, MirRule};

use crate::dfg;
use crate::rule_fn;
use crate::utils::{
   build_expr_tuple, is_not_aggregator, relation_coll_var, shared_arr_ident, tuple_of_idents,
   tuple_tokens, tuple_tokens_as_pattern, unit_seed,
};

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
pub(crate) enum VarKind {
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
   rule: &MirRule, scope_ident: &Ident, hoist_counter: &mut usize, is_batch: bool,
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
   use ascent_mir::MirBodyItem;
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
   let mut prior_rel: Option<ascent_mir::MirRelation> = None;
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
               accum = Some(emit_negation(a, &bound_vars, &var_kinds, &agg.rel.relation.name, &agg.rel_args, is_batch));
            } else {
               let a = accum.take().unwrap_or_else(|| unit_seed(scope_ident));
               let (nb, nk, na) = emit_agg(a, &bound_vars, &var_kinds, agg, is_batch);
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

/// Emit a single-clause body projection — the `Collection → Collection`
/// operator that takes the clause's rel_coll and projects one head tuple
/// per input tuple (when no filters) or optionally per input tuple (when
/// there are filters).
///
/// Without filters, emits `.map(…)` — cheaper than `.flat_map(… Some(…))`
/// at both compile and runtime (no Option allocation / unwrap per row).
/// With filters, keeps the `.flat_map(… if filter { Some(…) } else { None })`
/// shape since the filter can dynamically reject rows.
fn emit_single_clause_projection(
   rel_coll: &Ident, destruct: &TokenStream, row_ty: &Type, var_rebinds: &[TokenStream], filters: &[TokenStream],
   head_expr_tuple: &TokenStream,
) -> TokenStream {
   let op = if filters.is_empty() {
      dfg::DataflowOp::MapProject {
         destructure: destruct.clone(),
         row_ty: row_ty.clone(),
         var_rebinds: var_rebinds.to_vec(),
         head_expr: head_expr_tuple.clone(),
      }
   } else {
      dfg::DataflowOp::FilterMapProject {
         destructure: destruct.clone(),
         row_ty: row_ty.clone(),
         var_rebinds: var_rebinds.to_vec(),
         filters: filters.to_vec(),
         head_expr: head_expr_tuple.clone(),
      }
   };
   dfg::lower(&op, quote! { #rel_coll.clone() })
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

fn compile_first_clause(cl: &ascent_mir::MirBodyClause) -> (Vec<Ident>, TokenStream) {
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
   cl: &ascent_mir::MirBodyClause,
   prior_rel: Option<&ascent_mir::MirRelation>,
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
      use ascent_mir::IrRelation;
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
   // Pick the join's emission style — fused (terminal clause) or pass-through.
   let emit = match fused_head.as_ref() {
      Some((head_row_ty, head_expr_tuple)) => dfg::JoinEmit::Fused {
         head_row_ty: head_row_ty.clone(),
         head_expr_tuple: head_expr_tuple.clone(),
      },
      None => dfg::JoinEmit::PassThrough { out_tuple: out_tuple.clone() },
   };

   // LHS arrangement: either the shared arr from a prior clause (LHS-shared
   // fast path) OR a per-rule `(accum).map(...).arrange_by_key()`.
   let (lhs_arr_expr, lhs_prelude) = match &lhs_shared_arr {
      Some(lhs_arr) if can_use_shared => (quote! { #lhs_arr.clone() }, TokenStream::new()),
      _ => {
         let lhs_build = dfg::lower_map_arrange_lhs(
            accum,
            destr_accum,
            accum_key_tuple,
            accum_vals_cloned,
         );
         (quote! { __l }, quote! { let __l = #lhs_build; })
      },
   };

   // RHS arrangement: shared trace if the clause has no per-use filter and
   // has indices; otherwise a per-clause `flat_map(filter + reshape).arrange`.
   let (rhs_arr_expr, rhs_prelude) = if can_use_shared {
      (quote! { #shared_arr_name.clone() }, TokenStream::new())
   } else {
      let rhs_build = dfg::lower_flatmap_filter_arrange(
         &rel_coll,
         destruct,
         filter_pred,
         clause_key_tuple,
         new_vars_cols_tuple,
      );
      (quote! { __r }, quote! { let __r = #rhs_build; })
   };

   let join = dfg::lower_join_core(&dfg::JoinCore {
      lhs_arr_expr,
      rhs_arr_expr,
      accum_vals_tuple,
      new_vars_tuple,
      join_key_pattern,
      emit,
   });
   let tokens = quote! { { #lhs_prelude #rhs_prelude #join } };

   (out_bound, tokens)
}

/// Emit a filter. Bound vars are bound per their `VarKind` — clause-vars
/// as `&T` (batch convention, so user's `*x` works), generator/let/if-let
/// vars as owned `T`.
fn emit_filter(accum: TokenStream, bound_vars: &[Ident], var_kinds: &[VarKind], cond: &syn::Expr) -> TokenStream {
   let destr = emit_user_destructure(bound_vars, var_kinds, quote! { __b });
   dfg::lower_filter(accum, destr, quote! { #cond })
}
fn emit_negation(
   accum: TokenStream, bound_vars: &[Ident], var_kinds: &[VarKind], rel_name: &Ident, rel_args: &[syn::Expr],
   is_batch: bool,
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

   dfg::lower_antijoin(
      accum, destr_accum, accum_key_tuple, accum_vals_cloned,
      &rel_coll, destruct, filter_pred, clause_key_tuple,
      join_key_pattern, accum_vals_tuple, accum_bound_tuple, is_batch,
   )
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
   accum: TokenStream, bound_vars: &[Ident], var_kinds: &[VarKind], agg: &IrAggClause, is_batch: bool,
) -> (Vec<Ident>, Vec<VarKind>, TokenStream) {
   let rel_name = &agg.rel.relation.name;
   let rel_coll = relation_coll_var(rel_name);
   let (plans, shared_in_clause_order, _new_vars, filters, computed_keys) = plan_clause(&agg.rel_args, bound_vars);

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

   let tokens = dfg::lower_agg_clause(
      accum, destr_accum, accum_key_tuple, accum_vals_cloned,
      &rel_coll, agg_destruct, filter_pred, clause_key_tuple,
      rel_full_value, ref_tuple_for_agg, quote! { #aggregator },
      join_key_pattern, accum_vals_tuple, quote! { #pat }, out_tuple, is_batch,
   );

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

   let tokens = dfg::lower_let(accum, destr, expr_as_owned, prior_tuple, quote! { #pat }, new_tuple);
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

   let tokens = dfg::lower_if_let(accum, destr, quote! { #pat }, quote! { #expr }, owned_tuple);
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

   let tokens = dfg::lower_generator(accum, destr, quote! { #expr }, quote! { #pat }, out_tuple);
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
pub(crate) fn compile_rule_head_exprs(
   rule: &MirRule, scope_ident: &Ident, hoist_counter: &mut usize, is_batch: bool,
) -> (Vec<(Ident, TokenStream)>, TokenStream) {
   if rule.body_items.is_empty() {
      let seed = unit_seed(scope_ident);
      let mut out = Vec::new();
      for hcl in &rule.head_clause {
         let head_tuple = build_expr_tuple(&hcl.args);
         out.push((hcl.rel.name.clone(), dfg::lower_unit_seed_map(seed.clone(), head_tuple)));
      }
      return (out, TokenStream::new());
   }

   use ascent_mir::MirBodyItem;
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
      let var_rebinds: Vec<TokenStream> = plans
         .iter()
         .filter_map(|p| p.var_binding.as_ref().map(|(v, c)| quote! { let #v = &#c; }))
         .collect();
      let expr = emit_single_clause_projection(&rel_coll, &destruct, &row_ty, &var_rebinds, &filters, &head_expr_tuple);
      return (vec![(hcl.rel.name.clone(), expr)], TokenStream::new());
   }

   let (bound_vars, var_kinds, body_expr, hoisted_pre) =
      compile_rule_body_phase1(rule, scope_ident, hoist_counter, is_batch);

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

pub(crate) fn compile_rule_with_head_target(
   rule: &MirRule, head_target: &dyn Fn(&Ident) -> Ident, scope_ident: &Ident, hoist_counter: &mut usize,
   is_batch: bool,
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

   use ascent_mir::MirBodyItem;
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
      // plan_clause's destructure binds col idents (`__c0, __c1`). `build_expr_tuple`
      // emits user var names. Add `let user_var = &col_ident;` rebindings so the
      // head expression can reference user var names (as refs, matching the
      // non-fused path's `#destr` that emit_user_destructure produces).
      let var_rebinds: Vec<TokenStream> = plans
         .iter()
         .filter_map(|p| p.var_binding.as_ref().map(|(v, c)| quote! { let #v = &#c; }))
         .collect();
      // Rule-fn extraction path: emit the `.map(|destruct| { rebinds; head })`
      // body inside a free fn at module level, replace the inline body with a
      // call to that fn. Non-extraction fallback keeps the old inline expansion.
      let prod_expr = if rule_fn::enabled() && filters.is_empty() {
         let in_ty = tuple_type(&cl.rel.relation.field_types);
         let in_ty_ts = quote! { #in_ty };
         let projection_body = quote! {
            __coll.clone().map(move |#destruct| -> #row_ty {
               #(#var_rebinds)*
               #head_expr_tuple
            })
         };
         let fn_name = rule_fn::next_ident();
         let fn_obj = rule_fn::SingleClauseRuleFn {
            name: fn_name.clone(),
            in_ty: in_ty_ts,
            out_ty: row_ty.clone(),
            projection_body,
         };
         rule_fn::push(fn_obj.fn_item());
         fn_obj.call(&rel_coll)
      } else {
         emit_single_clause_projection(&rel_coll, &destruct, &row_ty, &var_rebinds, &filters, &head_expr_tuple)
      };
      return (
         quote! {
            {
               let __prod = #prod_expr;
               #head_target_ident = #head_target_ident.concat(__prod);
            }
         },
         TokenStream::new(),
      );
   }

   let (bound_vars, var_kinds, body_expr, hoisted_pre) =
      compile_rule_body_phase1(rule, scope_ident, hoist_counter, is_batch);

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
      let mapped = dfg::lower(
         &dfg::DataflowOp::MapProject {
            destructure: quote! { __b },
            row_ty,
            var_rebinds: vec![destr],
            head_expr: head_expr_tuple,
         },
         quote! { #rule_body_var.clone() },
      );
      out.extend(quote! {
         {
            let __prod = #mapped;
            #head_target_ident = #head_target_ident.concat(__prod);
         }
      });
   }
   (out, hoisted_pre)
}
