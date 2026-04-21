//! SCC-level analyses for the DD codegen backend.
//!
//! Each analysis is a pure function over `MirScc` returning a concrete data
//! structure that subsequent emission code consumes. Keeps codegen's hot
//! emission paths free of scanning logic — emission becomes a straight walk
//! over pre-computed structure.
//!
//! Progression target: these analyses are the first step toward a pass-based
//! architecture. Today they're consumed inline by `compile_looping_scc` /
//! `compile_nonlooping_scc`. Once DFG lands, the same analyses feed the
//! DFG-construction pass.

use std::collections::HashSet;

use proc_macro2::{Ident, Span};

use ascent_mir::{MirBodyItem, MirRule, MirScc};

/// Rules partitioned by whether their body clauses reference any relation
/// that's dynamic (recursive) within this SCC.
///
/// FlowLog compiles non-recursive rules OUTSIDE `scope.iterative` (they fire
/// once), recursive rules INSIDE (they feed the Variable/threshold loop).
/// This analysis is what makes the split possible.
pub(crate) struct RuleClassification<'a> {
   pub non_recursive: Vec<&'a MirRule>,
   pub recursive: Vec<&'a MirRule>,
}

impl<'a> RuleClassification<'a> {
   pub fn of(scc: &'a MirScc) -> Self {
      let dyn_names: HashSet<String> =
         scc.dynamic_relations.keys().map(|r| r.name.to_string()).collect();

      let is_recursive = |rule: &&MirRule| -> bool {
         rule.body_items.iter().any(|item| match item {
            MirBodyItem::Clause(cl) => dyn_names.contains(&cl.rel.relation.name.to_string()),
            MirBodyItem::Agg(agg) => dyn_names.contains(&agg.rel.relation.name.to_string()),
            MirBodyItem::Cond(_) | MirBodyItem::Generator(_) => false,
         })
      };

      let (recursive, non_recursive): (Vec<_>, Vec<_>) =
         scc.rules.iter().partition(is_recursive);
      Self { non_recursive, recursive }
   }
}

/// The set of shared arrangement identifiers that some rule body in this SCC
/// actually references — used to filter out MIR's "full-index" IrRelations
/// that are never read by any join (they'd become dead DD operators that
/// still cost per-iteration maintenance).
///
/// Name format matches `shared_arr_ident` in the codegen: `__arr_<rel>_indices_<cols>`.
pub(crate) struct UsedArrangements {
   pub names: HashSet<Ident>,
}

impl UsedArrangements {
   pub fn of(scc: &MirScc) -> Self {
      let mut names = HashSet::new();
      for rule in &scc.rules {
         let rule_has_join =
            rule.body_items.iter().filter(|it| matches!(it, MirBodyItem::Clause(_))).count() >= 2;
         for (i, item) in rule.body_items.iter().enumerate() {
            if let MirBodyItem::Clause(cl) = item {
               // Empty indices = no-key arrangement — never used via shared
               // arrangement path (can_use_shared requires non-empty indices).
               if cl.rel.indices.is_empty() {
                  continue;
               }
               // First-clause only needs its arrangement as LHS-shared target
               // if the rule has ≥2 clauses (prior_rel optimization). For
               // single-clause rules, `compile_first_clause` emits a direct
               // flat_map with no arrangement involved.
               if i == 0 && !rule_has_join {
                  continue;
               }
               let name = format!(
                  "__arr_{}_indices_{}",
                  cl.rel.relation.name,
                  cl.rel.indices.iter().map(|i| i.to_string()).collect::<Vec<_>>().join("_")
               );
               names.insert(Ident::new(&name, cl.rel.relation.name.span()));
            }
         }
      }
      Self { names }
   }

   pub fn contains(&self, name: &Ident) -> bool { self.names.contains(name) }
}

/// Set of relation names that are `body_only` in this SCC (read but not
/// produced here). Used to decide where to emit a shared arrangement:
/// body_only → outside `scope.iterative`, dynamic (produced here) → inside.
pub(crate) struct RelationPlacement {
   body_only_names: HashSet<String>,
}

impl RelationPlacement {
   pub fn of(scc: &MirScc) -> Self {
      Self {
         body_only_names: scc.body_only_relations.keys().map(|r| r.name.to_string()).collect(),
      }
   }

   pub fn is_body_only(&self, rel_name: &Ident) -> bool {
      self.body_only_names.contains(&rel_name.to_string())
   }
}

/// Hide the exact `Span` knob required when constructing idents; the rest of
/// the codegen can keep using `proc_macro2::Ident` without importing `Span`.
#[allow(dead_code)]
pub(crate) fn ident_here(name: &str) -> Ident { Ident::new(name, Span::call_site()) }

/// Relation names whose `__<rel>_coll` (Collection form, not arrangement)
/// is actually referenced by this SCC's rule-body code generation. Used to
/// skip emitting dead `let in_<rel> = …consolidate().enter(inner);` bindings
/// for body-only relations whose reads all go through shared arrangements.
///
/// A rel's Collection gets referenced by `compile_*` in these cases:
///   1. Sole clause of a rule (single-clause body).
///   2. First clause of a 2+ clause rule whose first join CAN'T take the
///      LHS-shared path (so `compile_first_clause`'s `flat_map(rel_coll)`
///      result survives into the final token stream).
///   3. Aggregated relation in an `agg` body item.
///
/// LHS-shared kicks in when `simple_join_start_index == Some(0)`, the first
/// clause has non-empty `indices` + no `cond_clauses` (so `prior_rel` is
/// preserved), and the second clause has non-empty `indices` + no cond
/// clauses. Conservative — may keep a few unnecessary `in_<rel>` bindings
/// in edge cases; never drops a needed one.
pub(crate) struct UsedCollections {
   names: HashSet<Ident>,
}

impl UsedCollections {
   pub fn of(scc: &MirScc) -> Self {
      let mut names = HashSet::new();
      for rule in &scc.rules {
         // (3) Agg clauses always read rel_coll for their aggregated relation.
         for it in &rule.body_items {
            if let MirBodyItem::Agg(agg) = it {
               names.insert(agg.rel.relation.name.clone());
            }
         }

         // Collect just the clause items (skip generators / conds / aggs).
         let clauses: Vec<_> = rule
            .body_items
            .iter()
            .filter_map(|it| if let MirBodyItem::Clause(cl) = it { Some(cl) } else { None })
            .collect();

         if clauses.is_empty() {
            continue;
         }

         // (1) Single-clause rule: the sole clause's Collection drives the flat_map.
         if clauses.len() == 1 {
            names.insert(clauses[0].rel.relation.name.clone());
            continue;
         }

         // (2) Multi-clause: check if the first join can take LHS-shared.
         //     Matches the conditions in `compile_join_clause`:
         //       - simple_join_start_index == Some(0)
         //       - first clause: non-empty indices + no cond_clauses (so
         //         `prior_rel` remains Some through the ccl-handling loop)
         //       - second clause: non-empty indices + no cond_clauses
         //         (so `can_use_shared` and no post-join filter overrides it)
         let lhs_shared_ok = rule.simple_join_start_index == Some(0)
            && !clauses[0].rel.indices.is_empty()
            && clauses[0].cond_clauses.is_empty()
            && !clauses[1].rel.indices.is_empty()
            && clauses[1].cond_clauses.is_empty();

         if !lhs_shared_ok {
            // First clause's `compile_first_clause` flat_map survives into the
            // output — Collection IS referenced.
            names.insert(clauses[0].rel.relation.name.clone());
         }
      }
      Self { names }
   }

   pub fn contains(&self, name: &Ident) -> bool { self.names.contains(name) }
}
