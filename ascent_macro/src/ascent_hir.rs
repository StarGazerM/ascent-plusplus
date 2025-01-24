#![deny(warnings)]
use std::{collections::{HashMap, HashSet}, rc::Rc};

use itertools::Itertools;
use proc_macro2::{Ident, Span, TokenStream};
use quote::ToTokens;
use syn::{parse2, parse_quote, punctuated::Punctuated, spanned::Spanned, token::Comma, Attribute, Error, Expr, Pat, Path, Type};

use crate::{AscentProgram, ascent_syntax::{RelationNode, DsAttributeContents, Signatures}, utils::{expr_to_ident, is_wild_card, tuple_type}, syn_utils::{expr_get_vars, pattern_get_vars}};
use crate::ascent_syntax::{BodyClauseArg, BodyItemNode, CondClause, GeneratorNode, RelationIdentity, RuleNode};

#[derive(Clone)]
pub(crate) struct AscentConfig {
   #[allow(dead_code)]
   pub attrs: Vec<Attribute>,
   pub include_rule_times: bool,
   pub generate_run_partial: bool,
   pub inter_rule_parallelism: bool,
   // pub stream_processing: bool,
   pub default_ds: DsAttributeContents,
}

impl AscentConfig {
   const MEASURE_RULE_TIMES_ATTR: &'static str = "measure_rule_times";
   const GENERATE_RUN_TIMEOUT_ATTR: &'static str = "generate_run_timeout";
   const INTER_RULE_PARALLELISM_ATTR: &'static str = "inter_rule_parallelism";
   // const STREAM_PROCESSING_ATTR: &'static str = "stream_processing";

   pub fn new(attrs: Vec<Attribute>, is_parallel: bool) -> syn::Result<AscentConfig> {
      let include_rule_times = attrs.iter().find(|attr| attr.meta.path().is_ident(Self::MEASURE_RULE_TIMES_ATTR))
         .map(|attr| attr.meta.require_path_only()).transpose()?.is_some();
      let generate_run_partial = attrs.iter().find(|attr| attr.meta.path().is_ident(Self::GENERATE_RUN_TIMEOUT_ATTR))
         .map(|attr| attr.meta.require_path_only()).transpose()?.is_some();
      let inter_rule_parallelism = attrs.iter().find(|attr| attr.meta.path().is_ident(Self::INTER_RULE_PARALLELISM_ATTR))
         .map(|attr| attr.meta.require_path_only()).transpose()?;
      // let stream_processing = attrs.iter().find(|attr| attr.meta.path().is_ident(Self::STREAM_PROCESSING_ATTR))
      //    .map(|attr| attr.meta.require_path_only()).transpose()?.is_some();

      let recognized_attrs = 
         [Self::MEASURE_RULE_TIMES_ATTR, Self::GENERATE_RUN_TIMEOUT_ATTR, Self::INTER_RULE_PARALLELISM_ATTR, REL_DS_ATTR];
      for attr in attrs.iter() {
         if !recognized_attrs.iter().any(|recognized_attr| attr.meta.path().is_ident(recognized_attr)) {
            return Err(Error::new_spanned(attr, 
                       format!("unrecognized attribute. recognized attributes are: {}",
                               recognized_attrs.join(", "))));
         }
      }
      if inter_rule_parallelism.is_some() && !is_parallel {
         return Err(Error::new_spanned(inter_rule_parallelism, "attribute only allowed in parallel Ascent"));
      }
      let default_ds = get_ds_attr(&attrs)?.unwrap_or_else(|| 
         DsAttributeContents { path: parse_quote! {::ascent::rel}, args: TokenStream::default() }
      );
      Ok(AscentConfig {
         inter_rule_parallelism: inter_rule_parallelism.is_some(),
         attrs,
         include_rule_times,
         generate_run_partial,
         // stream_processing,
         default_ds
      })
   }
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub(crate) struct IrExternDB {
   pub db_type: Ident,
   pub db_name: Ident,
}


pub(crate) struct AscentIr {
   pub relations_ir_relations: HashMap<RelationIdentity, HashSet<IrRelation>>,
   pub relations_full_indices: HashMap<RelationIdentity, IrRelation>,
   pub lattices_full_indices: HashMap<RelationIdentity, IrRelation>,
   // pub relations_no_indices: HashMap<RelationIdentity, IrRelation>,
   pub relations_metadata: HashMap<RelationIdentity, RelationMetadata>,
   pub rules: Vec<IrRule>,
   pub extern_dbs: Vec<IrExternDB>, 
   pub signatures: Signatures,
   pub config: AscentConfig,
   pub io: AscentIO,
   pub is_parallel: bool,
}

#[derive(Clone)]
pub(crate) struct RelationMetadata{
   pub initialization: Option<Rc<Expr>>,
   pub attributes: Rc<Vec<Attribute>>,
   pub ds_macro_path: Path,
   pub ds_macro_args: TokenStream
}

pub(crate) struct IrRule {
   pub head_clauses: Vec<IrHeadClause>,
   pub body_items: Vec<IrBodyItem>,
   pub simple_join_start_index: Option<usize>
}

#[allow(unused)]
pub(crate) fn ir_rule_summary(rule: &IrRule) -> String {
   fn bitem_to_str(bi: &IrBodyItem) -> String {
      match bi {
         IrBodyItem::Clause(cl) => cl.rel.ir_name().to_string(),
         IrBodyItem::Generator(_) => "for ⋯".into(),
         IrBodyItem::Cond(CondClause::If(..)) => format!("if ⋯"),
         IrBodyItem::Cond(CondClause::IfLet(..)) => format!("if let ⋯"),
         IrBodyItem::Cond(CondClause::Let(..)) => format!("let ⋯"),
         IrBodyItem::Agg(agg) => format!("agg {}", agg.rel.ir_name()),
      }
   }
   format!("{} <-- {}",
            rule.head_clauses.iter().map(|hcl| hcl.rel.name.to_string()).join(", "),
            rule.body_items.iter().map(bitem_to_str).join(", "))
}

#[derive(Clone)]
pub(crate) struct IrHeadClause{
   pub rel : RelationIdentity,
   pub args : Vec<Expr>,
   pub span: Span,
   pub args_span: Span,
   pub required_flag: bool,
   pub id_name: Option<Ident>,
   pub delete_flag: bool,
}

pub(crate) enum IrBodyItem {
   Clause(IrBodyClause),
   Generator(GeneratorNode),
   Cond(CondClause),
   Agg(IrAggClause)
}

impl IrBodyItem {
   pub(crate) fn rel(&self) -> Option<&IrRelation> {
      match self {
         IrBodyItem::Clause(bcl) => Some(&bcl.rel),
         IrBodyItem::Agg(agg) => Some(&agg.rel),
         IrBodyItem::Generator(_) |
         IrBodyItem::Cond(_) => None,
      }
   }
}

#[derive(Clone)]
pub(crate) struct IrBodyClause {
   pub rel : IrRelation,
   pub extern_db_name: Option<Ident>,
   pub args : Vec<Expr>,
   pub rel_args_span: Span,
   pub args_span: Span,
   pub cond_clauses : Vec<CondClause>,
   pub froce_delta: bool
}

impl IrBodyClause {
   #[allow(dead_code)]
   pub fn selected_args(&self) -> Vec<Expr> {
      self.rel.indices.iter().map(|&i| self.args[i].clone()).collect()
   }
}

#[derive(Clone)]
pub(crate) struct IrAggClause {
   pub span: Span,
   pub pat: Pat,
   pub aggregator: Expr,
   pub bound_args: Vec<Ident>,
   pub rel: IrRelation,
   pub extern_db_name: Option<Ident>,
   pub rel_args: Vec<Expr>
}

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub(crate) struct IrRelation {
   pub relation: RelationIdentity,
   pub indices: Vec<usize>,
   pub val_type: IndexValType,
   pub need_id: bool
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum IndexValType {
   Reference,
   Direct(Vec<usize>)
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct AscentIO {
   pub ins : Vec<RelationIdentity>,
   pub outs: Vec<RelationIdentity>,
}

impl IrRelation {
   pub fn new(relation: RelationIdentity, indices: Vec<usize>, need_id: bool) -> Self {
      // TODO this is not the right place for this
      let val_type = if relation.is_lattice //|| indices.len() == relation.field_types.len() 
      {
         IndexValType::Reference
      } else {
         IndexValType::Direct((0..relation.field_types.len()).filter(|i| !indices.contains(i)).collect_vec())
      };
      IrRelation { relation, indices, val_type, need_id }
   }

   pub fn key_type(&self) -> Type {
      let index_types : Vec<_> = self.indices.iter().map(|&i| self.relation.field_types[i].clone()).collect();
      tuple_type(&index_types)
   }
   pub fn ir_name(&self) -> Ident {
      ir_name_for_rel_indices(&self.relation.name, &self.indices)
   }
   pub fn is_full_index(&self) -> bool {
      self.relation.field_types.len() == self.indices.len()
   }
   pub fn is_no_index(&self) -> bool {
      self.indices.is_empty()
   }

   pub fn value_type(&self) -> Type {
      match &self.val_type {
         IndexValType::Reference => parse_quote!{usize},
         IndexValType::Direct(cols) => {
            let index_types : Vec<_> = cols.iter().map(|&i| self.relation.field_types[i].clone()).collect();
            tuple_type(&index_types)
         },
      }
   }
}

const REL_DS_ATTR: &str = "ds";
const RECOGNIIZED_REL_ATTRS: [&str; 1] = [REL_DS_ATTR];

// fn token_steam_contains(tokens: &TokenStream, ident: &Ident) -> bool {
//    tokens.to_string().contains(ident.to_string().as_str())
// }

// fn compute_body_item_dep(body_items: Vec<BodyItemNode>) -> Vec<(usize, usize)> {
//    let mut dep = vec![];
//    for (i, bitem) in body_items.iter().enumerate() {
//       if let BodyItemNode::Clause(bcl) = bitem {
//          let bcl_args =
//             bcl.args.iter().cloned()
//             .filter_map(|arg| {
//                match &arg  {
//                 BodyClauseArg::Pat(_) => {None},
//                 BodyClauseArg::Expr(expr) => {expr_to_ident(expr)},
//                }
//             })
//             .collect::<Vec<_>>();
//          // check all other clauses, if it's arguements are used in any other clause
//          // if ture meanwhile
//          // - other clause is a clause, then its bidirectional dependency
//          // - other clause is a generator, then it depends on the clause
//          // - other clause is a agg/neg, then that clause depends on this clause
//          // - other clause is a cond, then that clause depends on this clause
//          for (j, other_bitem) in body_items.iter().enumerate() {
//             if i == j {
//                continue;
//             }
//             match other_bitem {
//                BodyItemNode::Clause(other_bcl) => {
//                   if other_bcl.args.iter().any(|arg| {
//                      token_steam_contains(&arg.to_token_stream(), &bcl.rel)
//                   }) {
//                      dep.push((i, j));
//                      dep.push((j, i));
//                   }
//                },
//                BodyItemNode::Generator(gen) => {
//                   if token_steam_contains(&gen.pattern.to_token_stream(), &bcl.rel) {
//                      dep.push((j, i));
//                   }
//                },
//                BodyItemNode::Agg(agg) => {
//                   if agg.bound_args.iter().any(|arg| bcl_args.contains(arg)) {
//                      dep.push((j, i));
//                   } else if agg.rel_args.iter().any(|arg| {
//                      token_steam_contains(&arg.to_token_stream(), &bcl.rel)
//                   }) {
//                      dep.push((i, j));
//                   }
//                },
//                BodyItemNode::Cond(cond) => {
//                   if cond.bound_vars().iter().any(|v| bcl_args.contains(v)) {
//                      dep.push((j, i));
//                   } else if token_steam_contains(&cond.expr().to_token_stream(), &bcl.rel) {
//                      dep.push((i, j));
//                   }
//                },
//                // BodyItemNode::Negation(negation_clause_node) => todo!(),
//                _ => { 
//                   // these should already been desugared
//                }
//             }
//          }

//       }
//    }
//    dep
// }


pub(crate) fn compile_ascent_program_to_hir(prog: &AscentProgram, is_parallel: bool) -> syn::Result<AscentIr>{
   // all relations in the body are dynamic
   let mut dynamic_relation_idents = HashSet::new();
   for rule in prog.rules.iter(){
      for bitem in rule.head_clauses.iter() {
         match &bitem {
            crate::ascent_syntax::HeadItemNode::MacroInvocation(_) => {},
            crate::ascent_syntax::HeadItemNode::HeadFuctionReturn(_) => {},
            crate::ascent_syntax::HeadItemNode::HeadClause(head_clause_node) => {
               dynamic_relation_idents.insert(head_clause_node.rel.clone());
            }
         }
      }
   }

   // enumerate all different order version of rules
   // let mut all_order_rules = vec![];
   // for rule in prog.rules.iter(){
   //    let body_items = rule.body_items.clone();
   //    let dep = compute_body_item_dep(body_items.clone());
   //    // create digraph from dep
   //    let mut rule_graph =
      
   // }

   let ir_rules : Vec<(IrRule, Vec<IrRelation>)> = prog.rules.iter().map(|r| compile_rule_to_ir_rule(r, prog)).try_collect()?;
   let config = AscentConfig::new(prog.attributes.clone(), is_parallel)?;
   let num_relations = prog.relations.len();
   let mut relations_ir_relations: HashMap<RelationIdentity, HashSet<IrRelation>> = HashMap::with_capacity(num_relations);
   let mut relations_full_indices = HashMap::with_capacity(num_relations);
   let mut relations_initializations = HashMap::new();
   let mut relations_metadata = HashMap::with_capacity(num_relations);
   // let mut relations_no_indices = HashMap::new();
   let mut lattices_full_indices = HashMap::new();
   for rel in prog.relations.iter(){
      let rel_identity = RelationIdentity::from(rel);

      if rel.is_lattice {
         let indices = (0 .. rel_identity.field_types.len() - 1).collect_vec();
         let lat_full_index = IrRelation::new(rel_identity.clone(), indices, rel.need_id);
         relations_ir_relations.entry(rel_identity.clone()).or_default().insert(lat_full_index.clone());
         lattices_full_indices.insert(rel_identity.clone(), lat_full_index);
      }

      let full_indices = (0 .. rel_identity.field_types.len()).collect_vec();
      let rel_full_index = IrRelation::new(rel_identity.clone(),full_indices, rel.need_id);

      relations_ir_relations.entry(rel_identity.clone()).or_default().insert(rel_full_index.clone());
      // relations_ir_relations.entry(rel_identity.clone()).or_default().insert(rel_no_index.clone());
      relations_full_indices.insert(rel_identity.clone(), rel_full_index);
      if let Some(init_expr) = &rel.initialization {
         relations_initializations.insert(rel_identity.clone(), Rc::new(init_expr.clone()));
      }
      let ds_attribute = get_ds_attr(&rel.attrs)?.unwrap_or_else(|| config.default_ds.clone());
      
      relations_metadata.insert(
         rel_identity.clone(),
         RelationMetadata {
            initialization: rel.initialization.clone().map(Rc::new),
            attributes: Rc::new(rel.attrs.iter().filter(|attr| attr.meta.path().get_ident().map_or(true, |ident| !RECOGNIIZED_REL_ATTRS.iter().any(|ra| ident == ra))).cloned().collect_vec()),
            ds_macro_path: ds_attribute.path,
            ds_macro_args: ds_attribute.args
         }
      );
      // relations_no_indices.insert(rel_identity, rel_no_index);
   }
   for (ir_rule, extra_relations) in ir_rules.iter(){
      for bitem in ir_rule.body_items.iter(){
         let rel = match bitem {
            IrBodyItem::Clause(bcl) => Some(&bcl.rel),
            IrBodyItem::Agg(agg) => Some(&agg.rel),
            _ => None
         };
         if let Some(rel) = rel {
            let relation = &rel.relation;
            relations_ir_relations.entry(relation.clone()).or_default().insert(rel.clone());
         }
      }
      for extra_rel in extra_relations.iter(){
         relations_ir_relations.entry(extra_rel.relation.clone()).or_default().insert(extra_rel.clone());
      }
   }
   for extra_index in &prog.extra_indices {
      let rel = prog.relations.iter().find(|r| &extra_index.rel_name == &r.name).unwrap();
      let rel_identity = RelationIdentity::from(rel);
      let indices : Vec<usize> =
         extra_index.arg_pos.iter()
         .map(|i| i.base10_parse().unwrap())
         .collect_vec();
      let ir_rel = IrRelation::new(rel_identity.clone(), indices, rel.need_id);
      relations_ir_relations.entry(rel_identity.clone()).or_default().insert(ir_rel.clone());
   }

   // pick out io relations
   let mut ins_rel = vec![];
   for i in prog.in_streams.iter() {
      let rel = prog.relations.iter().find(|r| &i.rel_name == &r.name);
      let rel = match rel {
         Some(rel) => rel,
         None => return Err(Error::new(i.rel_name.span(), format!("Input Relation {} not defined", i.rel_name))),
      };
      ins_rel.push(RelationIdentity::from(rel));
   }
   let mut outs_rel = vec![];
   for o in prog.out_streams.iter() {
      let rel = prog.relations.iter().find(|r| &o.rel_name == &r.name);
      let rel = match rel {
         Some(rel) => rel,
         None => return Err(Error::new(o.rel_name.span(), format!("Output Relation {} not defined", o.rel_name))),
      };
      outs_rel.push(RelationIdentity::from(rel));
   }

   let io = AscentIO {
      ins: ins_rel,
      outs: outs_rel,
   };

   let signatures = prog.signatures.clone().unwrap_or_else(|| parse2(quote! {pub struct AscentProgram;}).unwrap());
   let extern_dbs = prog.extern_dbs.iter().map(|db| IrExternDB {db_type: db.db_type.clone(), db_name: db.db_name.clone()}).collect();
   Ok(AscentIr {
      rules: ir_rules.into_iter().map(|(rule, _extra_rels)| rule).collect_vec(),
      relations_ir_relations,
      relations_full_indices,
      lattices_full_indices,
      relations_metadata,
      // relations_no_indices,
      signatures,
      extern_dbs,
      io,
      config,
      is_parallel
   })
}

fn get_ds_attr(attrs: &[Attribute]) -> syn::Result<Option<DsAttributeContents>> {
   let ds_attrs = attrs.iter()
      .filter(|attr| attr.meta.path().get_ident().map_or(false, |ident| ident == REL_DS_ATTR))
      .collect_vec();
   match &ds_attrs[..] {
      [] => Ok(None),
      [attr] => {
         let res = syn::parse2::<DsAttributeContents>(attr.meta.require_list()?.tokens.clone())?;
         Ok(Some(res))
      },
      [_attr1, attr2, ..] => Err(Error::new(attr2.bracket_token.span.join(), "multiple `ds` attributes specified")),
   }
}

fn compile_rule_to_ir_rule(rule: &RuleNode, prog: &AscentProgram) -> syn::Result<(IrRule, Vec<IrRelation>)> {
   let mut body_items = vec![];
   let mut grounded_vars = vec![];
   fn extend_grounded_vars(grounded_vars: &mut Vec<Ident>, new_vars: impl IntoIterator<Item = Ident>) -> syn::Result<()> {
      for v in new_vars.into_iter() {
         if grounded_vars.contains(&v) {
            // TODO may someday this will work
            let other_var = grounded_vars.iter().find(|&x| x == &v).unwrap();
            let other_err = Error::new(other_var.span(), "variable being shadowed");
            let mut err = Error::new(v.span(), format!("'{}' shadows another variable with the same name", v));
            err.combine(other_err);
            return Err(err);
         }
         grounded_vars.push(v);
      }
      Ok(())
   }

   let first_clause_ind = rule.body_items.iter().enumerate().find(|(_, bi)| matches!(bi, BodyItemNode::Clause(..))).map(|(i, _)| i);
   let mut first_two_clauses_simple = first_clause_ind.is_some() &&
      matches!(rule.body_items.get(first_clause_ind.unwrap() + 1), Some(BodyItemNode::Clause(..)));
   for (bitem_ind, bitem) in rule.body_items.iter().enumerate() {
      match bitem {
         BodyItemNode::Clause(ref bcl) => {
            if first_clause_ind == Some(bitem_ind) && bcl.cond_clauses.iter().any(|c| matches!(c, &CondClause::IfLet(_)))
            {
               first_two_clauses_simple = false;
            }

            if first_clause_ind.map(|x| x + 1) == Some(bitem_ind) && first_two_clauses_simple{
               let mut self_vars = HashSet::new();
               for var in bcl.args.iter().filter_map(|arg| expr_to_ident(arg.unwrap_expr_ref())) {
                  if !self_vars.insert(var) {
                     first_two_clauses_simple = false;
                  }
               }
               for cond_cl in bcl.cond_clauses.iter(){
                  let cond_expr = cond_cl.expr();
                  let expr_idents = expr_get_vars(cond_expr);
                  if !expr_idents.iter().all(|v| self_vars.contains(v)){
                     first_two_clauses_simple = false;
                     break;
                  }
                  self_vars.extend(cond_cl.bound_vars());
               }
            }
            let mut indices = vec![];
            for (i,arg) in bcl.args.iter().enumerate() {
               if let Some(var) = expr_to_ident(arg.unwrap_expr_ref()) {
                  if grounded_vars.contains(&var){
                     indices.push(i);
                     if first_clause_ind == Some(bitem_ind) {
                        first_two_clauses_simple = false;
                     }
                  } else {
                     grounded_vars.push(var);
                  }
               } else {
                  indices.push(i);
                  if bitem_ind < 2 + first_clause_ind.unwrap_or(0) {
                     first_two_clauses_simple = false;
                  }
               }
            }
            let relation = prog_get_relation(prog, &bcl.rel, &bcl.args)?;

            for cond_clause in bcl.cond_clauses.iter() {
               extend_grounded_vars(&mut grounded_vars, cond_clause.bound_vars())?;
            }
            
            let ir_rel = IrRelation::new(relation.into(), indices, relation.need_id);
            let ir_bcl = IrBodyClause {
               rel: ir_rel,
               extern_db_name: bcl.extern_db_name.clone(),
               args: bcl.args.iter().cloned().map(BodyClauseArg::unwrap_expr).collect(),
               rel_args_span: bcl.rel.span().join(bcl.args.span()).unwrap_or_else(|| bcl.rel.span()),
               args_span: bcl.args.span(),
               cond_clauses: bcl.cond_clauses.clone(),
               froce_delta: bcl.delta_flag
            };
            body_items.push(IrBodyItem::Clause(ir_bcl));
         },
         BodyItemNode::Generator(ref gen) => {
            extend_grounded_vars(&mut grounded_vars, pattern_get_vars(&gen.pattern))?;
            body_items.push(IrBodyItem::Generator(gen.clone()));
         },
         BodyItemNode::Cond(ref cl) => {
            body_items.push(IrBodyItem::Cond(cl.clone()));
            extend_grounded_vars(&mut grounded_vars, cl.bound_vars())?;
         },
         BodyItemNode::Agg(ref agg) => {
            extend_grounded_vars(&mut grounded_vars, pattern_get_vars(&agg.pat))?;
            let indices = agg.rel_args.iter().enumerate().filter(|(_i, expr)| {
               if is_wild_card(expr) {
                  return false;
               } else if let Some(ident) = expr_to_ident(expr) {
                  if agg.bound_args.iter().contains(&ident) {
                     return false;
                  }
               }
               true
            }).map(|(i, _expr)| i).collect_vec();
            let relation = prog_get_relation(prog, &agg.rel, &agg.rel_args)?;
            
            let ir_rel = IrRelation::new(relation.into(), indices, relation.need_id);
            let ir_agg_clause = IrAggClause {
               span: agg.agg_kw.span,
               pat: agg.pat.clone(),
               aggregator: agg.aggregator.get_expr(),
               bound_args: agg.bound_args.iter().cloned().collect_vec(),
               rel: ir_rel,
               extern_db_name: agg.extern_db_name.clone(),
               rel_args: agg.rel_args.iter().cloned().collect_vec(),
            };
            body_items.push(IrBodyItem::Agg(ir_agg_clause));
         },
         _ => panic!("unrecognized body item")
      }
      
   }
   let mut head_clauses = vec![];
   for hcl_node in rule.head_clauses.iter(){
      let hcl_node = hcl_node.clause();
      let rel = prog.relations.iter().find(|r| hcl_node.rel == r.name);
      let rel = match rel {
         Some(rel) => rel,
         None => return Err(Error::new(hcl_node.rel.span(), format!("relation {} not defined", hcl_node.rel))),
      };
      let rel_identity = RelationIdentity::from(rel);
      
      let head_clause = IrHeadClause {
         rel: rel_identity,
         args : hcl_node.args.iter().cloned().collect(),
         span: hcl_node.span(),
         args_span: hcl_node.args.span(),
         required_flag: hcl_node.required_flag,
         id_name: hcl_node.id_name.clone(),
         delete_flag: hcl_node.delete_flag,
      };
      head_clauses.push(head_clause);
   }
   
   let is_simple_join = first_two_clauses_simple && body_items.len() >= 2;
   let simple_join_start_index = if is_simple_join {first_clause_ind} else {None};

   let simple_join_ir_relations = if let Some(start_ind) = simple_join_start_index {
      let (bcl1, bcl2) = match &body_items[start_ind..start_ind + 2] {
         [IrBodyItem::Clause(bcl1), IrBodyItem::Clause(bcl2)] => (bcl1, bcl2),
          _ => panic!("incorrect simple join handling in ascent_hir")
      };  
      let bcl2_vars = bcl2.args.iter().filter_map(expr_to_ident).collect_vec();
      let indices = get_indices_given_grounded_variables(&bcl1.args, &bcl2_vars);
      let new_cl1_ir_relation = IrRelation::new(bcl1.rel.relation.clone(), indices, bcl1.rel.need_id);
      vec![new_cl1_ir_relation]
   } else {vec![]};

   if let Some(start_ind) = simple_join_start_index {
      if let IrBodyItem::Clause(cl1) = &mut body_items[start_ind] {
         cl1.rel = simple_join_ir_relations[0].clone();
      }
   }

   Ok((IrRule {
      simple_join_start_index,
      head_clauses, 
      body_items, 
   }, vec![]))
}

pub fn ir_name_for_rel_indices(rel: &Ident, indices: &[usize]) -> Ident {
   let indices_str = if indices.is_empty() {format!("none")} else {indices.iter().join("_")};
   let name = format!("{}_indices_{}", rel, indices_str);
   Ident::new(&name, rel.span())
}

/// for a clause with args, returns the indices assuming vars are grounded.
pub fn get_indices_given_grounded_variables(args: &[Expr], vars: &[Ident]) -> Vec<usize>{
   let mut res = vec![];
   for (i, arg) in args.iter().enumerate(){
      if let Some(arg_var) = expr_to_ident(arg){
         if vars.contains(&arg_var) {
            res.push(i);
         }
      } else {
         res.push(i);
      }  
   }
   res
}

pub(crate) fn prog_get_relation<'a, T: ToTokens>(prog: &'a AscentProgram, name: &Ident, args: &Punctuated<T, Comma>) -> syn::Result<&'a RelationNode> {
   let relation = prog.relations.iter().find(|r| name == &r.name);
   let arity = args.len();
   match relation {
      Some(rel) => {
         if rel.field_types.len() != arity {
            Err(Error::new(
               name.span(), 
               format!("Wrong arity for relation {}. Actual arity: {}, but get {} : {:?}",
                              name, rel.field_types.len(), arity,
                              args.iter().map(|arg| quote!{#arg}.to_string()).collect::<Vec<_>>().join(", "))))
         } else {
            Ok(rel)
         }
      },
      None => Err(Error::new(name.span(), format!("Relation {} not defined", name))),
   }
}