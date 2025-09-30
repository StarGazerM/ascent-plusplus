#![deny(warnings)]
use std::collections::{HashMap, HashSet};
use std::fmt::Write;

use itertools::Itertools;
use petgraph::algo::condensation;
use petgraph::graphmap::DiGraphMap;
use proc_macro2::{Ident, Span};
use syn::{Expr, Type};

use crate::ascent_hir::{
   extend_grounded_vars, get_indices_given_grounded_variables, AscentConfig, AscentIr, IndexValType, IrAggClause, IrBodyClause, IrBodyItem, IrHeadClause, IrRelation, IrRule, RelationMetadata
};
use crate::ascent_mir::MirRelationVersion::*;
use crate::ascent_syntax::{CondClause, GeneratorNode, JoinStrategy, RelationIdentity, Signatures};
use crate::syn_utils::{expr_get_vars, pattern_get_vars};
use crate::utils::{expr_to_ident, intersects, pat_to_ident, tuple_type};

pub(crate) struct AscentMir {
   pub sccs: Vec<MirScc>,
   #[allow(unused)]
   pub deps: HashMap<usize, HashSet<usize>>,
   pub relations_ir_relations: HashMap<RelationIdentity, HashSet<IrRelation>>,
   pub relations_full_indices: HashMap<RelationIdentity, IrRelation>,
   pub relations_metadata: HashMap<RelationIdentity, RelationMetadata>,
   pub lattices_full_indices: HashMap<RelationIdentity, IrRelation>,
   pub signatures: Signatures,
   pub config: AscentConfig,
   pub is_parallel: bool,
}

pub(crate) struct MirScc {
   pub rules: Vec<MirRule>,
   pub dynamic_relations: HashMap<RelationIdentity, HashSet<IrRelation>>,
   pub body_only_relations: HashMap<RelationIdentity, HashSet<IrRelation>>,
   pub is_looping: bool,
}

pub(crate) fn mir_summary(mir: &AscentMir) -> String {
   let mut res = String::new();
   for (i, scc) in mir.sccs.iter().enumerate() {
      writeln!(&mut res, "scc {}, is_looping: {}:", i, scc.is_looping).unwrap();
      for r in scc.rules.iter() {
         writeln!(&mut res, "  {}", mir_rule_summary(r)).unwrap();
      }
      let sorted_dynamic_relation_keys = scc.dynamic_relations.keys().sorted_by_key(|rel| &rel.name);
      write!(&mut res, "  dynamic relations: ").unwrap();
      writeln!(&mut res, "{}", sorted_dynamic_relation_keys.map(|r| r.name.to_string()).join(", ")).unwrap();
   }
   res
}

#[derive(Clone)]
pub(crate) struct MirRule {
   // TODO rename to head_clauses
   pub head_clause: Vec<IrHeadClause>,
   pub body_items: Vec<MirBodyItem>,
   pub simple_join_start_index: Option<usize>,
   pub reorderable: bool,
   pub join_strategy: Option<JoinStrategy>,
}

pub(crate) fn mir_rule_summary(rule: &MirRule) -> String {
   fn bitem_to_str(bitem: &MirBodyItem) -> String {
      match bitem {
         MirBodyItem::Clause(bcl) => format!("{}_{}", bcl.rel.ir_name, bcl.rel.version.to_string()),
         MirBodyItem::Generator(gen) =>
            format!("for_{}", pat_to_ident(&gen.pattern).map(|x| x.to_string()).unwrap_or_default()),
         MirBodyItem::Cond(CondClause::If(..)) => format!("if ⋯"),
         MirBodyItem::Cond(CondClause::IfLet(..)) => format!("if let ⋯"),
         MirBodyItem::Cond(CondClause::Let(..)) => format!("let ⋯"),
         MirBodyItem::Agg(agg) => format!("agg {}", agg.rel.ir_name()),
      }
   }
   format!(
      "{} <-- {}{simple_join}{reorderable}",
      rule.head_clause.iter().map(|hcl| hcl.rel.name.to_string()).join(", "),
      rule.body_items.iter().map(bitem_to_str).join(", "),
      simple_join = if rule.simple_join_start_index.is_some() { " [SIMPLE JOIN]" } else { "" },
      reorderable = if rule.simple_join_start_index.is_some() && !rule.reorderable { " [NOT REORDERABLE]" } else { "" }
   )
}

#[derive(Clone)]
pub(crate) enum MirBodyItem {
   Clause(MirBodyClause),
   Generator(GeneratorNode),
   Cond(CondClause),
   Agg(IrAggClause),
}

impl MirBodyItem {
   pub fn unwrap_clause(&self) -> &MirBodyClause {
      match self {
         MirBodyItem::Clause(cl) => cl,
         _ => panic!("MirBodyItem: unwrap_clause called on non_clause"),
      }
   }

   pub fn clause(&self) -> Option<&MirBodyClause> {
      match self {
         MirBodyItem::Clause(mir_body_clause) => Some(mir_body_clause),
         _ => None,
      }
   }

   pub fn bound_vars(&self) -> Vec<Ident> {
      match self {
         MirBodyItem::Clause(cl) => {
            let cl_vars = cl.args.iter().filter_map(expr_to_ident);
            let cond_cl_vars = cl.cond_clauses.iter().flat_map(|cc| cc.bound_vars());
            cl_vars.chain(cond_cl_vars).collect()
         },
         MirBodyItem::Generator(gen) => pattern_get_vars(&gen.pattern),
         MirBodyItem::Cond(cond) => cond.bound_vars(),
         MirBodyItem::Agg(agg) => pattern_get_vars(&agg.pat),
      }
   }

   pub fn used_vars(&self) -> Vec<Ident> {
      match self {
         MirBodyItem::Clause(cl) => {
            let mut used_vars = vec![];
            for arg in cl.args.iter() {
               used_vars.extend(expr_get_vars(arg));
            }
            for cond_cl in cl.cond_clauses.iter() {
               used_vars.extend(cond_cl.used_vars());
            }
            used_vars
         },
         MirBodyItem::Generator(gen) => expr_get_vars(&gen.expr),
         MirBodyItem::Cond(cond) => cond.used_vars(),
         MirBodyItem::Agg(agg) => {
            let mut used_vars = vec![];
            for arg in agg.rel_args.iter() {
               used_vars.extend(expr_get_vars(arg));
            }
            used_vars
         },
      }
   }
}

#[derive(Clone)]
pub(crate) struct MirBodyClause {
   pub rel: MirRelation,
   pub args: Vec<Expr>,
   pub rel_args_span: Span,
   pub args_span: Span,
   pub cond_clauses: Vec<CondClause>,
}
impl MirBodyClause {
   pub fn selected_args(&self) -> Vec<Expr> { self.rel.indices.iter().map(|&i| self.args[i].clone()).collect() }

   /// returns a vec of (var_ind, var) of all the variables in the clause
   pub fn vars(&self) -> Vec<(usize, Ident)> {
      self.args.iter().enumerate().filter_map(|(i, v)| expr_to_ident(v).map(|v| (i, v))).collect::<Vec<_>>()
   }

   #[allow(dead_code)]
   pub fn from(ir_body_clause: IrBodyClause, rel: MirRelation) -> MirBodyClause {
      MirBodyClause {
         rel,
         args: ir_body_clause.args,
         rel_args_span: ir_body_clause.rel_args_span,
         args_span: ir_body_clause.args_span,
         cond_clauses: ir_body_clause.cond_clauses,
      }
   }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub(crate) struct MirRelation {
   pub relation: RelationIdentity,
   pub indices: Vec<usize>,
   pub ir_name: Ident,
   pub version: MirRelationVersion,
   pub is_full_index: bool,
   pub is_no_index: bool,
   pub val_type: IndexValType,
}

pub(crate) fn ir_relation_version_var_name(ir_name: &Ident, version: MirRelationVersion) -> Ident {
   let name = format!("{}_{}", ir_name, version.to_string());
   Ident::new(&name, ir_name.span())
}

impl MirRelation {
   pub fn var_name(&self) -> Ident { ir_relation_version_var_name(&self.ir_name, self.version) }

   #[allow(dead_code)]
   pub fn key_type(&self) -> Type {
      let index_types: Vec<_> = self.indices.iter().map(|&i| self.relation.field_types[i].clone()).collect();
      tuple_type(&index_types)
   }

   pub fn from(ir_relation: IrRelation, version: MirRelationVersion) -> MirRelation {
      MirRelation {
         ir_name: ir_relation.ir_name(),
         is_full_index: ir_relation.is_full_index(),
         is_no_index: ir_relation.is_no_index(),
         relation: ir_relation.relation,
         indices: ir_relation.indices,
         version,
         val_type: ir_relation.val_type,
      }
   }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum MirRelationVersion {
   TotalDelta,
   Total,
   Delta,
   New,
}

impl MirRelationVersion {
   pub fn to_string(self) -> &'static str {
      use MirRelationVersion::*;
      match self {
         TotalDelta => "total+delta",
         Delta => "delta",
         Total => "total",
         New => "new",
      }
   }
}

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

// reorder all rules in the SCC
fn reorder_mir_scc(scc: &MirScc) -> syn::Result<(MirScc, Vec<MirRelation>)> {
   let mut new_rules: Vec<MirRule> = vec![];
   let mut dep_relations: Vec<MirRelation> = vec![];
   let mut addtional_mir_relations: Vec<MirRelation> = vec![];
   for rule in scc.rules.iter() {
      let (new_rule, new_relations) = reorder_mir_rule(rule)?;
      new_rules.push(new_rule);
      dep_relations.extend(new_relations);
   }
   // add dep_relations to scc.dynamic_relations 
   let mut new_dynamic_relations = scc.dynamic_relations.clone();
   let mut new_body_only_relations = scc.body_only_relations.clone();
   for relation in dep_relations.iter() {
      // check if in dyn rel
      let in_dyn_rel = scc.dynamic_relations.contains_key(&relation.relation);
      let in_body_only_rel = scc.body_only_relations.contains_key(&relation.relation);
      if in_dyn_rel {
         let updated = new_dynamic_relations.entry(relation.relation.clone()) 
            .or_default().insert(mir_relation_to_ir_relation(relation));

         if updated {
            addtional_mir_relations.push(relation.clone());
         }
      }
      if in_body_only_rel {
         let updated = new_body_only_relations.entry(relation.relation.clone())
            .or_default().insert(mir_relation_to_ir_relation(relation));
         if updated {
            addtional_mir_relations.push(relation.clone());
         }
      }
   }
   

   Ok((MirScc {
      rules: new_rules,
      dynamic_relations: new_dynamic_relations,
      body_only_relations: new_body_only_relations,
      is_looping: scc.is_looping,
   }, addtional_mir_relations))
}

fn mir_relation_to_ir_relation(relation: &MirRelation) -> IrRelation {
   IrRelation::new(relation.relation.clone(), relation.indices.clone())
}

// reorder a rule:
fn reorder_mir_rule(rule: &MirRule) -> syn::Result<(MirRule, Vec<MirRelation>)> {
   let need_reorder = if let Some(join_strategy) = &rule.join_strategy {
      if join_strategy.strategy == Ident::new("heuristic_reordering", Span::call_site()) {
         // eprintln!("INFO: reorder rule {} ", mir_rule_summary(rule));
         true
      } else {
         false
      }
   } else {
      false
   };
   if !need_reorder {
      return Ok((rule.clone(), vec![]));
   }
   
   // find the delta clause
   let delta_cls_idx = rule.body_items.iter().position(|bitem| matches!(bitem, MirBodyItem::Clause(bcl) if bcl.rel.version == MirRelationVersion::Delta));
   if delta_cls_idx.is_none() {
      return Ok((rule.clone(), vec![]));
   }
   // if delta clause is alread the first clause, return
   if delta_cls_idx == Some(rule.simple_join_start_index.unwrap_or(0)) {
      return Ok((rule.clone(), vec![]));
   }
   let delta_cls_idx = delta_cls_idx.unwrap();
   let delta_clause = rule.body_items.get(delta_cls_idx).unwrap();
   let delta_clause_bound_vars = delta_clause.bound_vars();
   let delta_clause = match delta_clause {
      MirBodyItem::Clause(bcl) => bcl,
      _ => panic!("delta clause is not a clause"),
   };
   // check if the delta clause doesn't contains bounded variables
   
   // let mut clause_before_delta_idx: Vec<usize> = vec![];
   let mut unselected_idx: Vec<usize> = (0..rule.body_items.len()).filter(|&i| i != delta_cls_idx).collect();
   // loop over all args to see if it contains constants or unbound vars
   for arg in delta_clause.args.iter() {
      if let Some(_var) = expr_to_ident(arg) {
         continue;
      }
      let vars = expr_get_vars(arg);
      if vars.iter().any(|var| !delta_clause_bound_vars.contains(var)) {
         // contains unbound vars, can't be reordered
         eprintln!("WARNING: {} delta clause contains unbound vars, cannot be reordered", mir_rule_summary(rule));
         return Ok((rule.clone(), vec![]));
      }
   }

   let mut clause_after_delta_idx = vec![];
   let mut grounded_vars = delta_clause_bound_vars.clone();
   // reorder by search next connective clause in the rules
   // if not found, pick any of those
   while !unselected_idx.is_empty() {
      let mut selected_idx = None;
      let mut prev_joined_cnts = 0;
      let mut cur_ground = vec![];
      for i in unselected_idx.iter() {
         let bitem = rule.body_items.get(*i).unwrap();
         // populate possible new grounded vars
         let mut self_vars = bitem.bound_vars();
         self_vars.dedup();
         let used_vars = bitem.used_vars();
         // if any used var is not not grounded nor in self_vars, skip
         if used_vars.iter().any(|var| !grounded_vars.contains(var) && !self_vars.contains(var)) {
            continue;
         }
         // count joined vars
         let cur_joined_cnts = used_vars.iter().filter(|var| grounded_vars.contains(var)).count();
         if cur_joined_cnts > prev_joined_cnts || selected_idx.is_none() {
            selected_idx = Some(*i);
            prev_joined_cnts = cur_joined_cnts;
            cur_ground = self_vars;
         } else {
            // less joined vars, skip
            continue;
         }
      }
      if selected_idx.is_none() {
         eprintln!("WARNING: no clause can be reordered, may cause full scan");
         return Ok((rule.clone(), vec![]));
      }
      let selected_idx = selected_idx.unwrap();
      clause_after_delta_idx.push(selected_idx);
      // delete selected_idx from unselected_idx
      let pos = unselected_idx.iter().position(|&i| i == selected_idx);
      if pos.is_none() {
         panic!("selected_idx not found in unselected_idx");
      }
      unselected_idx.remove(pos.unwrap());

      for var in cur_ground.iter() {
         if !grounded_vars.contains(var) {
            grounded_vars.push(var.clone());
         }
      }
   }
   let mut reordered_body_items: Vec<MirBodyItem> = vec![];
   // for i in clause_before_delta_idx.iter() {
   //    reordered_body_items.push(rule.body_items.get(*i).unwrap().clone());
   // }
   reordered_body_items.push(MirBodyItem::Clause(delta_clause.clone()));
   for i in clause_after_delta_idx.iter() {
      reordered_body_items.push(rule.body_items.get(*i).unwrap().clone());
   }
   let reordered_rule = MirRule {
      body_items: reordered_body_items,
      head_clause: rule.head_clause.clone(),
      simple_join_start_index: rule.simple_join_start_index,
      reorderable: rule.reorderable,
      join_strategy: rule.join_strategy.clone(),
   };
   
   // reconstruct the rule from the original rule and the reordered rule
   let res = reselect_index(reordered_rule, rule);
   if res.is_err() {
      eprintln!("WARNING: {} may contains var grounded after, cannot be reordered, may cause full scan",
         mir_rule_summary(&rule));
      return Ok((rule.clone(), vec![]));
   }
   res
}

// reselect the index of reordered rule
// It generate new rule and new indices need to be prepared in SCC.
// This function is also used to reject the reordering if loop is detected.
fn reselect_index(rule: MirRule, fallback: &MirRule) -> syn::Result<(MirRule, Vec<MirRelation>)> {
   let mut new_body_items = vec![];
   let mut grounded_vars = vec![];
   let mut new_relations = vec![];
   let mut grounded_vars_after_first_clause = vec![];

   // let first_clause_ind = rule.simple_join_start_index;
   let first_two_clauses_simple = rule.simple_join_start_index.is_some()
      && matches!(rule.body_items.get(rule.simple_join_start_index.unwrap() + 1), Some(MirBodyItem::Clause(..)));

   for (cls_ind, bitem) in rule.body_items.iter().enumerate() {
      match bitem {
         MirBodyItem::Clause(bcl) => {
            // check if cond_cl contains var ungrounded
            let mut self_vars = HashSet::new();
            self_vars.extend(bcl.args.iter().filter_map(expr_to_ident));
            for cond_cl in bcl.cond_clauses.iter() {
               self_vars.extend(cond_cl.bound_vars());
               let expr_idents_cond = expr_get_vars(cond_cl.expr());
               if expr_idents_cond.iter().any(
                  |v| !(grounded_vars.contains(v) || self_vars.contains(v))) {
                  eprintln!("WARNING: cond clause in {} may contains var grounded after, cannot be reordered, may cause full scan",
                     mir_rule_summary(&rule));
                  return Ok((fallback.clone(), vec![]));
               }
            }
            let mut indices = vec![];
            let mut new_grounded_vars = vec![];
            for (i, arg) in bcl.args.iter().enumerate() {
               if let Some(var) = expr_to_ident(arg) {
                  if grounded_vars.contains(&var) {
                     indices.push(i);
                  } else{
                     new_grounded_vars.push(var);
                  }
               } else {
                  indices.push(i);
               }
            }
            grounded_vars.extend(new_grounded_vars.clone());
            let ir_rel = IrRelation::new(bcl.rel.relation.clone(), indices);
            let mir_rel = MirRelation::from(ir_rel, bcl.rel.version);
            new_relations.push(mir_rel.clone());
            let ir_bcl = MirBodyClause {
               rel: mir_rel,
               args: bcl.args.clone(),
               rel_args_span: bcl.rel_args_span.clone(),
               args_span: bcl.args_span.clone(),
               cond_clauses: bcl.cond_clauses.clone(),
            };
            new_body_items.push(MirBodyItem::Clause(ir_bcl));
            let ind = rule.simple_join_start_index.unwrap_or(0); 
            if cls_ind > ind {
               grounded_vars_after_first_clause.extend(new_grounded_vars);
            }   
         },
         MirBodyItem::Generator(gen) => {
            let new_grounded_vars = pattern_get_vars(&gen.pattern);
            extend_grounded_vars(&mut grounded_vars, new_grounded_vars.clone())?;
            new_body_items.push(bitem.clone());
            let ind = rule.simple_join_start_index.unwrap_or(0); 
            if cls_ind > ind {
               grounded_vars_after_first_clause.extend(new_grounded_vars);
            }
         },
         MirBodyItem::Cond(cond) => {
            new_body_items.push(bitem.clone());
            let new_grounded_vars = cond.bound_vars();
            extend_grounded_vars(&mut grounded_vars, new_grounded_vars.clone())?;
            let ind = rule.simple_join_start_index.unwrap_or(0); 
            if cls_ind > ind {
               grounded_vars_after_first_clause.extend(new_grounded_vars);
            }
         },
         MirBodyItem::Agg(agg) => {
            let new_grounded_vars = pattern_get_vars(&agg.pat);
            extend_grounded_vars(&mut grounded_vars, new_grounded_vars.clone())?;
            new_body_items.push(bitem.clone());
            // TODO: will indices change?
            let ind = rule.simple_join_start_index.unwrap_or(0); 
            if cls_ind > ind {
               grounded_vars_after_first_clause.extend(new_grounded_vars);
            }    
         }
      }
   }
   // check if the first clause contains var grounded from later clauses
   let ind = rule.simple_join_start_index.unwrap_or(0); 
   if let MirBodyItem::Clause(bcl) = &new_body_items[ind] {
      for arg in bcl.args.iter() {
         let used_idents = if let Some(ident) = expr_to_ident(arg) { vec![ident] } else { expr_get_vars(arg) };
         // eprintln!("used_idents: {:?}", used_idents);
         if used_idents.iter().any(|var| grounded_vars_after_first_clause.contains(var)) {
            eprintln!("WARNING: {} may contains var grounded after, cannot be reordered, may cause full scan",
               mir_rule_summary(&rule));
            return Ok((fallback.clone(), vec![]));
         }
      }
   }
   

   // handle index selection for the first clause
   if first_two_clauses_simple {
      let simple_join_ir_relations = if let Some(ind) = rule.simple_join_start_index {
         let (bcl1, bcl2) = match &new_body_items[ind..ind + 2] {
            [MirBodyItem::Clause(bcl1), MirBodyItem::Clause(bcl2)] => (bcl1, bcl2),
            _ => panic!("incorrect simple join handling in ascent_mir"),
         };
         let bcl2_vars = bcl2.args.iter().filter_map(expr_to_ident).collect_vec();
         let indices = get_indices_given_grounded_variables(&bcl1.args, &bcl2_vars);
         let new_cl1_ir_relation = IrRelation::new(bcl1.rel.relation.clone(), indices);
         let new_cl1_mir_relation = MirRelation::from(new_cl1_ir_relation, bcl1.rel.version);
         new_relations.push(new_cl1_mir_relation.clone());
         vec![new_cl1_mir_relation]
      } else {
         vec![]
      };
      if let Some(ind) = rule.simple_join_start_index {
         if let MirBodyItem::Clause(cl1) = &mut new_body_items[ind] {
            cl1.rel = simple_join_ir_relations[0].clone();
         }
      }
   };
   
   let reordered_rule = MirRule {
      body_items: new_body_items,
      head_clause: rule.head_clause.clone(),
      simple_join_start_index: rule.simple_join_start_index,
      reorderable: rule.reorderable,
      join_strategy: rule.join_strategy.clone(),
   };
   eprintln!("reordered_rule: {:?}", mir_rule_summary(&reordered_rule));
   Ok((reordered_rule, new_relations))
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
   let mut updated_ir_relations : HashMap<RelationIdentity, HashSet<IrRelation>>=
      hir.relations_ir_relations.clone();
   let mut updated_full_indices : HashMap<RelationIdentity, IrRelation> =
      hir.relations_full_indices.clone();
   let mut updated_lattices_full_indices : HashMap<RelationIdentity, IrRelation> = 
      hir.lattices_full_indices.clone();
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

      let rules: syn::Result<Vec<_>> = scc
         .iter()
         .map(|&ind| compile_hir_rule_to_mir_rules(&hir.rules[ind], &dynamic_relations_set))
         .collect();
      let rules = rules?.into_iter().flatten().collect_vec();

      for rule in rules.iter() {
         for bi in rule.body_items.iter() {
            if let MirBodyItem::Agg(agg) = bi {
               if dynamic_relations.contains_key(&agg.rel.relation) {
                  if hir.config.allow_non_stratified_agg {
                     continue;
                  } else {
                     return Err(syn::Error::new(
                        agg.span,
                        format!("use of aggregated relation `{}` cannot be stratified", &agg.rel.relation.name),
                     ));
                  }
               }
            }
         }
      }
      let mir_scc = {
         let mir_scc = MirScc { rules, dynamic_relations, body_only_relations, is_looping };
         let (reordered_mir_scc, additional_mir_relations) = reorder_mir_scc(&mir_scc)?;

         for relation in additional_mir_relations.iter() {
            if relation.version == MirRelationVersion::Total {
               if relation.relation.is_lattice {
                  updated_lattices_full_indices.insert(
                     relation.relation.clone(), mir_relation_to_ir_relation(relation));
               } else {
                  // add if not exists
                  updated_full_indices.entry(relation.relation.clone())
                     .or_insert(mir_relation_to_ir_relation(relation));
               }
            } 
            updated_ir_relations.entry(relation.relation.clone())
               .or_default().insert(mir_relation_to_ir_relation(relation));
         }
         reordered_mir_scc
         // mir_scc
      };
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
      // relations_ir_relations: hir.relations_ir_relations.clone(),
      // relations_full_indices: hir.relations_full_indices.clone(),
      // lattices_full_indices: hir.lattices_full_indices.clone(),
      relations_ir_relations: updated_ir_relations,
      relations_full_indices: updated_full_indices,
      lattices_full_indices: updated_lattices_full_indices,
      relations_metadata: hir.relations_metadata.clone(),
      signatures: hir.signatures.clone(),
      config: hir.config.clone(),
      is_parallel: hir.is_parallel,
   })
}

fn compile_hir_rule_to_mir_rules(rule: &IrRule, dynamic_relations: &HashSet<RelationIdentity>) -> syn::Result<Vec<MirRule>> {
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

   let version_combinations =
      if dynamic_cls.is_empty() { vec![vec![]] } else { versions(&dynamic_cls[..], rule.simple_join_start_index) };

   let mut mir_body_items = Vec::with_capacity(version_combinations.len());

   for version_combination in version_combinations {
      let versions =
         dynamic_cls.iter().zip(version_combination).fold(vec![None; rule.body_items.len()], |mut acc, (i, v)| {
            acc[*i] = Some(v);
            acc
         });
      let mir_bodys =
         rule.body_items.iter().zip(versions).map(|(bi, v)| hir_body_item_to_mir_body_item(bi, v)).collect_vec();
      mir_body_items.push(mir_bodys)
   }

   let mir_rules: Vec<MirRule> = mir_body_items
      .into_iter()
      .map(|bcls| {
         // rule is reorderable if it is a simple join and the second clause does not depend on items
         // before the first clause (e.g., let z = &1, foo(x, y), bar(y, z) is not reorderable)
         let reorderable = rule.simple_join_start_index.is_some_and(|ind| {
            let pre_first_clause_vars = bcls.iter().take(ind).flat_map(MirBodyItem::bound_vars);
            !intersects(pre_first_clause_vars, bcls[ind + 1].bound_vars())
         });
         let mir_rule = MirRule {
            body_items: bcls,
            head_clause: rule.head_clauses.clone(),
            simple_join_start_index: rule.simple_join_start_index,
            reorderable,
            join_strategy: rule.join_strategy.clone(),
         };
         mir_rule
      })
      .collect();
   Ok(mir_rules)
}
