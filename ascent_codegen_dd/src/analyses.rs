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
