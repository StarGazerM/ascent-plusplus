#![deny(warnings)]
use std::collections::{HashMap, HashSet};

use itertools::Itertools;
use petgraph::algo::condensation;
use petgraph::graphmap::DiGraphMap;

use crate::ascent_hir::AscentIr;
use crate::utils::intersects;
// Data types live in `ascent_mir` now. This file only hosts HIR→MIR lowering
// logic and related helpers.
pub(crate) use ascent_mir::{
   AscentMir, IrBodyItem, IrRelation, MirBodyClause, MirBodyItem, MirRelation, MirRelationVersion, MirRule, MirScc,
};
use ascent_mir::{IrRule, MirRelationVersion::*, RelationIdentity};

// mir_summary / mir_rule_summary / types — all moved to `ascent_mir` crate.
// Re-exported above for existing `crate::ascent_mir::…` call sites.

fn get_hir_dep_graph(hir: &AscentIr) -> Vec<(usize, usize)> {
   let mut relations_to_rules_in_head: HashMap<&RelationIdentity, HashSet<usize>> =
      HashMap::with_capacity(hir.rules.len());
   for (i, rule) in hir.rules.iter().enumerate() {
      for head_rel in rule.head_clauses.iter().map(|hcl| &hcl.rel) {
         relations_to_rules_in_head.entry(head_rel).or_default().insert(i);
      }
   }

   let mut edges = vec![];
   for (i, rule) in hir.rules.iter().enumerate() {
      for bitem in rule.body_items.iter() {
         if let Some(body_rel) = bitem.rel() {
            let body_rel_identity = &body_rel.relation;
            if let Some(set) = relations_to_rules_in_head.get(body_rel_identity) {
               for &rule_with_rel_in_head in set.iter().sorted() {
                  edges.push((rule_with_rel_in_head, i));
               }
            }
         }
      }
   }
   edges
}

pub(crate) fn compile_hir_to_mir(hir: &AscentIr) -> syn::Result<AscentMir> {
   let dep_graph = get_hir_dep_graph(hir);
   let mut dep_graph = DiGraphMap::<_, ()>::from_edges(&dep_graph);
   for i in 0..hir.rules.len() {
      dep_graph.add_node(i);
   }
   let dep_graph = dep_graph.into_graph::<usize>();
   // println!("{:?}", Dot::with_config(&dep_graph, &[Config::EdgeNoLabel]));
   let mut sccs = condensation(dep_graph, true);

   let mut mir_sccs = vec![];
   for scc in sccs.node_weights().collect_vec().iter().rev() {
      let mut dynamic_relations: HashMap<RelationIdentity, HashSet<IrRelation>> = HashMap::new();
      let mut body_only_relations: HashMap<RelationIdentity, HashSet<IrRelation>> = HashMap::new();

      let mut dynamic_relations_set = HashSet::new();
      for &rule_ind in scc.iter() {
         let rule = &hir.rules[rule_ind];
         for bitem in rule.body_items.iter() {
            if let Some(rel) = bitem.rel() {
               body_only_relations.entry(rel.relation.clone()).or_default().insert(rel.clone());
            }
         }

         for hcl in hir.rules[rule_ind].head_clauses.iter() {
            dynamic_relations_set.insert(hcl.rel.clone());
            dynamic_relations.entry(hcl.rel.clone()).or_default();
            // TODO why this?
            // ... we can add only indices used in bodies in the scc, that requires the codegen to be updated.
            for rel_ind in &hir.relations_ir_relations[&hcl.rel] {
               dynamic_relations.get_mut(&hcl.rel).unwrap().insert(rel_ind.clone());
            }
         }
      }

      let mut is_looping = false;
      for rel in dynamic_relations.keys().cloned().collect_vec() {
         if let Some(indices) = body_only_relations.remove(&rel) {
            is_looping = true;
            for ind in indices {
               dynamic_relations.entry(rel.clone()).or_default().insert(ind);
            }
         }
      }

      // Backends that do semi-naive at the operator level (e.g. DD's
      // `Variable` / `scope.iterative`) can handle ONE rule variant —
      // explicit MIR-level variant expansion would produce N redundant
      // rule bodies. The frontend gets this hint from
      // `AscentConfig::backend_operator_semi_naive`; batch backends
      // (which need the variants) set it to `false`.
      let is_dd = hir.config.backend_operator_semi_naive;
      let rules = scc
         .iter()
         .map(|&ind| compile_hir_rule_to_mir_rules(&hir.rules[ind], &dynamic_relations_set, is_dd))
         .collect::<syn::Result<Vec<_>>>()?
         .into_iter()
         .flatten()
         .collect_vec();

      for rule in rules.iter() {
         for bi in rule.body_items.iter() {
            if let MirBodyItem::Agg(agg) = bi {
               if dynamic_relations.contains_key(&agg.rel.relation) {
                  return Err(syn::Error::new(
                     agg.span,
                     format!("use of aggregated relation `{}` cannot be stratified", &agg.rel.relation.name),
                  ));
               }
            }
         }
      }
      let mir_scc = MirScc { rules, dynamic_relations, body_only_relations, is_looping };
      mir_sccs.push(mir_scc);
   }

   sccs.reverse();
   let sccs_nodes_count = sccs.node_indices().count();
   let mut sccs_dep_graph = HashMap::with_capacity(sccs_nodes_count);
   for n in sccs.node_indices() {
      //the nodes in the sccs graph is in reverse topological order, so we do this
      sccs_dep_graph.insert(
         sccs_nodes_count - n.index() - 1,
         sccs.neighbors(n).map(|n| sccs_nodes_count - n.index() - 1).collect(),
      );
   }

   Ok(AscentMir {
      sccs: mir_sccs,
      deps: sccs_dep_graph,
      relations_ir_relations: hir.relations_ir_relations.clone(),
      relations_full_indices: hir.relations_full_indices.clone(),
      lattices_full_indices: hir.lattices_full_indices.clone(),
      relations_metadata: hir.relations_metadata.clone(),
      signatures: hir.signatures.clone(),
      config: hir.config.clone(),
      is_parallel: hir.is_parallel,
   })
}

fn compile_hir_rule_to_mir_rules(
   rule: &IrRule, dynamic_relations: &HashSet<RelationIdentity>, dd_no_variants: bool,
) -> syn::Result<Vec<MirRule>> {
   fn versions_base(count: usize) -> Vec<Vec<MirRelationVersion>> {
      if count == 0 {
         vec![]
      } else {
         let mut res = versions_base(count - 1);
         for v in &mut res {
            v.push(MirRelationVersion::TotalDelta);
         }
         let mut new_combination = vec![MirRelationVersion::Total; count];
         new_combination[count - 1] = MirRelationVersion::Delta;
         res.push(new_combination);
         res
      }
   }

   // TODO is it worth it?
   fn versions(dynamic_cls: &[usize], simple_join_start_index: Option<usize>) -> Vec<Vec<MirRelationVersion>> {
      fn remove_total_delta_at_index(ind: usize, res: &mut Vec<Vec<MirRelationVersion>>) {
         let mut i = 0;
         while i < res.len() {
            if res[i].get(ind) == Some(&TotalDelta) {
               res.insert(i + 1, res[i].clone());
               res[i][ind] = Total;
               res[i + 1][ind] = Delta;
            }
            i += 1;
         }
      }

      let count = dynamic_cls.len();
      let mut res = versions_base(count);
      let no_total_delta_at_beginning = false;
      if no_total_delta_at_beginning {
         if let Some(ind) = simple_join_start_index {
            remove_total_delta_at_index(ind, &mut res);
            remove_total_delta_at_index(ind + 1, &mut res);
         } else if dynamic_cls.get(0) == Some(&0) {
            remove_total_delta_at_index(0, &mut res);
         }
      }
      res
   }

   fn hir_body_item_to_mir_body_item(hir_bitem: &IrBodyItem, version: Option<MirRelationVersion>) -> MirBodyItem {
      match hir_bitem {
         IrBodyItem::Clause(_) => {},
         _ => assert!(version.is_none()),
      }
      match hir_bitem {
         IrBodyItem::Clause(hir_bcl) => {
            let ver = version.unwrap_or(MirRelationVersion::Total);
            let mir_relation = MirRelation::from(hir_bcl.rel.clone(), ver);
            let mir_bcl = MirBodyClause {
               rel: mir_relation,
               args: hir_bcl.args.clone(),
               rel_args_span: hir_bcl.rel_args_span,
               args_span: hir_bcl.args_span,
               cond_clauses: hir_bcl.cond_clauses.clone(),
            };
            MirBodyItem::Clause(mir_bcl)
         },
         IrBodyItem::Cond(cl) => MirBodyItem::Cond(cl.clone()),
         IrBodyItem::Generator(gen) => MirBodyItem::Generator(gen.clone()),
         IrBodyItem::Agg(agg) => MirBodyItem::Agg(agg.clone()),
      }
   }

   let dynamic_cls = rule
      .body_items
      .iter()
      .enumerate()
      .filter_map(|(i, cl)| match cl {
         IrBodyItem::Clause(cl) if dynamic_relations.contains(&cl.rel.relation) => Some(i),
         _ => None,
      })
      .collect_vec();

   // Either the user's `#[plan(...)]` variants drive the semi-naive
   // expansion, or we fall back to the auto-generated `versions()` set.
   // Each produced variant is a Vec<(permuted_body_index, Option<version>)>
   // — permuted so codegen's left-to-right walk matches the requested plan.
   //
   // DD backend (`dd_no_variants`): always emit exactly one MIR rule per
   // source rule, all `Option<version>` = None. DD's iterative scope with
   // `Variable` does semi-naive AT THE OPERATOR LEVEL — `join_core` sees
   // deltas on either side and produces output deltas without needing
   // N variant copies of the join. User-supplied `#[plan]` is still
   // honored for `order=[...]` (permute body items so codegen's left-to-
   // right walk emits the requested join order); `delta=N` is a no-op
   // under DD (no Delta/Total versions flow into codegen); extra variants
   // beyond index 0 are ignored (DD has no variant-expansion slot).
   // Validation (delta-is-dynamic) still runs for cross-backend
   // consistency so a plan authored for one backend doesn't silently
   // pass under the other.
   let plan_variants: Vec<Vec<(usize, Option<MirRelationVersion>)>> = match &rule.plan {
      None if dd_no_variants => {
         // One variant, natural order, all versions = None (no delta markers).
         vec![(0..rule.body_items.len()).map(|i| (i, None)).collect()]
      },
      None => {
         let version_combinations = if dynamic_cls.is_empty() {
            vec![vec![]]
         } else {
            versions(&dynamic_cls[..], rule.simple_join_start_index)
         };
         version_combinations
            .into_iter()
            .map(|vc| {
               // Natural order: body items kept in original position.
               let versions_per_item: Vec<Option<MirRelationVersion>> = dynamic_cls
                  .iter()
                  .zip(vc)
                  .fold(vec![None; rule.body_items.len()], |mut acc, (i, v)| {
                     acc[*i] = Some(v);
                     acc
                  });
               (0..rule.body_items.len()).map(|i| (i, versions_per_item[i])).collect()
            })
            .collect()
      },
      Some(user_variants) => {
         // Delta-is-dynamic validation on EVERY variant, regardless of backend.
         // Keeps a plan authored under one backend from silently passing under
         // the other (e.g. swapping dd ↔ batch without re-checking the plan).
         for (vi, pv) in user_variants.iter().enumerate() {
            let is_dynamic = match &rule.body_items[pv.delta] {
               IrBodyItem::Clause(cl) => dynamic_relations.contains(&cl.rel.relation),
               _ => false,
            };
            if !is_dynamic {
               let rel_name: String = match &rule.body_items[pv.delta] {
                  IrBodyItem::Clause(cl) => cl.rel.relation.name.to_string(),
                  _ => "<non-clause>".into(),
               };
               return Err(syn::Error::new(
                  rule.head_clauses[0].span,
                  format!(
                     "#[plan] variant {vi}: `delta = {}` points to clause `{rel_name}`, but that relation is not \
                      recursive in this SCC (no delta stream exists). Pick a delta clause that names an IDB \
                      relation participating in the recursion.",
                     pv.delta
                  ),
               ));
            }
         }

         if dd_no_variants {
            // DD: take the first variant's `order` as the body-item permutation;
            // no version markers (DD derives deltas at runtime via Variable).
            // Extra variants (if any) describe alternate orders that batch would
            // run as separate .concat() branches; DD's single join handles all
            // deltas automatically, so there's nowhere for them to land.
            let order = &user_variants[0].order;
            vec![order.iter().map(|&orig_i| (orig_i, None)).collect()]
         } else {
            // Batch: expand each variant into its own MIR rule with
            // Delta/TotalDelta/Total version markers stamped per clause.
            let mut out = Vec::with_capacity(user_variants.len());
            for pv in user_variants.iter() {
               // Build version assignment for this variant in the permuted
               // order. Mirrors `versions_base`'s semantics:
               //   - in the permuted order, the delta clause position is Delta.
               //   - clauses before the delta position are TotalDelta.
               //   - clauses after are Total.
               // (Only dynamic clauses get a version; others stay None.)
               let delta_pos_in_order =
                  pv.order.iter().position(|&i| i == pv.delta).expect("structural validator guarantees this");
               let mut assignment: Vec<(usize, Option<MirRelationVersion>)> = Vec::with_capacity(pv.order.len());
               for (pos_in_order, &orig_i) in pv.order.iter().enumerate() {
                  let is_dyn = match &rule.body_items[orig_i] {
                     IrBodyItem::Clause(cl) => dynamic_relations.contains(&cl.rel.relation),
                     _ => false,
                  };
                  let version = if !is_dyn {
                     None
                  } else if pos_in_order < delta_pos_in_order {
                     Some(MirRelationVersion::TotalDelta)
                  } else if pos_in_order == delta_pos_in_order {
                     Some(MirRelationVersion::Delta)
                  } else {
                     Some(MirRelationVersion::Total)
                  };
                  assignment.push((orig_i, version));
               }
               out.push(assignment);
            }
            out
         }
      },
   };

   // Lower each variant: walk the permuted indices in order, emit MIR body
   // items with the chosen version per clause.
   let mir_body_items: Vec<Vec<MirBodyItem>> = plan_variants
      .iter()
      .map(|assignment| {
         assignment.iter().map(|&(orig_i, v)| hir_body_item_to_mir_body_item(&rule.body_items[orig_i], v)).collect()
      })
      .collect();

   Ok(mir_body_items
      .into_iter()
      .map(|bcls| {
         // rule is reorderable if it is a simple join and the second clause does not depend on items
         // before the first clause (e.g., let z = &1, foo(x, y), bar(y, z) is not reorderable).
         // User-supplied plans already pin the order, so reorderability is moot;
         // the existing heuristic still applies harmlessly.
         let reorderable = rule.simple_join_start_index.is_some_and(|ind| {
            let pre_first_clause_vars = bcls.iter().take(ind).flat_map(MirBodyItem::bound_vars);
            bcls.get(ind + 1).map_or(false, |bi| !intersects(pre_first_clause_vars, bi.bound_vars()))
         });
         MirRule {
            body_items: bcls,
            head_clause: rule.head_clauses.clone(),
            simple_join_start_index: rule.simple_join_start_index,
            reorderable,
         }
      })
      .collect())
}
