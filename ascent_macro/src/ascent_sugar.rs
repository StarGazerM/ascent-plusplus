#![deny(warnings)]
extern crate proc_macro;
use proc_macro2::{Span, TokenStream};
use syn::token::Comma;
use syn::{
parse2, punctuated::Punctuated, spanned::Spanned, Error, Expr, ExprMacro,
    Ident, Result, Token, Type, 
};
use syn::parse::{Parse, ParseStream, Parser};
use core::panic;
use std::sync::Mutex;
use std::usize;
use std::collections::{HashMap, HashSet};

use quote::ToTokens;
use itertools::Itertools;

use ascent_base::util::update;
use crate::ascent_syntax::{AggClauseNode, AggregatorNode, AscentProgram, BodyClauseArg, BodyClauseNode, BodyItemNode, CondClause, DisjunctionNode, FunctionNode, HeadClauseNode, HeadItemNode, MacroDefNode, MacroParamKind, RelationNode, RuleNode, StratumPath, SubQueryNode};
use crate::utils::{
   expr_to_ident, expr_to_ident_mut, flatten_punctuated, is_wild_card,
   punctuated_map, punctuated_singleton, punctuated_try_map, punctuated_try_unwrap, spans_eq,
   token_stream_replace_macro_idents, Piper,
};
use crate::syn_utils::{
   expr_get_vars, expr_visit_free_vars_mut, expr_visit_idents_in_macros_mut,
   pattern_get_vars, pattern_visit_vars_mut, token_stream_idents, token_stream_replace_ident,
};

fn rule_desugar_disjunction_nodes(rule: RuleNode) -> Vec<RuleNode> {
    fn bitem_desugar(bitem: &BodyItemNode) -> Vec<Vec<BodyItemNode>> {
       match bitem {
          BodyItemNode::Generator(_) => vec![vec![bitem.clone()]],
          BodyItemNode::Clause(_) => vec![vec![bitem.clone()]],
          BodyItemNode::Cond(_) => vec![vec![bitem.clone()]],
          BodyItemNode::Agg(_) => vec![vec![bitem.clone()]],
          BodyItemNode::Negation(_) => vec![vec![bitem.clone()]],
          BodyItemNode::Disjunction(d) => {
             let mut res = vec![];
             for disjunt in d.disjuncts.iter() {
                for conjunction in bitems_desugar(&disjunt.iter().cloned().collect_vec()){
                   res.push(conjunction);
                }
             } 
            res
          },
         BodyItemNode::FunctionCall(_) => vec![vec![bitem.clone()]], 
         BodyItemNode::MacroInvocation(m) => panic!("unexpected macro invocation: {:?}", m.mac.path),
         BodyItemNode::SubQuery(_) => vec![vec![bitem.clone()]],
       }
    }
    fn bitems_desugar(bitems: &[BodyItemNode]) -> Vec<Vec<BodyItemNode>> {
       let mut res = vec![];
       if !bitems.is_empty() {
          let sub_res = bitems_desugar(&bitems[0 .. bitems.len() - 1]);
          let last_desugared = bitem_desugar(&bitems[bitems.len() - 1]);
          for sub_res_item in sub_res.into_iter() {
             for last_item in last_desugared.iter() {
                let mut res_item = sub_res_item.clone();
                res_item.extend(last_item.clone());
                res.push(res_item);
             }
          }
       } else {
          res.push(vec![]);
       }
 
       res
    }
 
    let mut res = vec![];
    for conjunction in bitems_desugar(&rule.body_items){
       res.push(RuleNode {
          body_items: conjunction,
          head_clauses: rule.head_clauses.clone()
       })
    }
    res
 }
 
 fn rule_desugar_exists_bang(rule: RuleNode) -> RuleNode {
    fn hitems_desugar(hitems: &Vec<&HeadItemNode>) -> Vec<HeadItemNode> {
       let mut res = vec![];
       for hiterm in hitems {
          let desugered_it = hitem_bang_desugar(hiterm);
          res.extend(desugered_it);
       }
       res
    }
    let desugared_head_items = hitems_desugar(&rule.head_clauses.iter().collect_vec());
    let head_clauses = Punctuated::from_iter(desugared_head_items);
    RuleNode{head_clauses, body_items: rule.body_items.clone()}
 }
 
 fn hitem_bang_desugar(hi: &HeadItemNode) -> Vec<HeadItemNode> {
    match hi {
       HeadItemNode::MacroInvocation(_) => vec![hi.clone()],
       HeadItemNode::HeadFuctionReturn(_) => panic!("unexpected function return, should be desugared before bang desugar"),
       HeadItemNode::HeadClause(cl) => {
          if let Some(exists_var) = cl.clone().exists_var {
             let mut new_cl = cl.clone();
             new_cl.exists_var = None;
             new_cl.required_flag = true;
             new_cl.id_name = Some(exists_var.clone());
             let mut new_cl_id = cl.clone();
             new_cl_id.exists_var = None;
             new_cl_id.rel = Ident::new(&format!("{}_id", cl.rel), cl.rel.span());
             new_cl_id.required_flag = cl.required_flag;
             new_cl_id.args.push(syn::parse2(quote!{#exists_var}).unwrap());
             vec![HeadItemNode::HeadClause(new_cl), HeadItemNode::HeadClause(new_cl_id)]
          } else {
             vec![HeadItemNode::HeadClause(cl.clone())]
          }
       }
    }
 }
 
 fn body_item_get_bound_vars(bi: &BodyItemNode) -> Vec<Ident> {
    match bi {
       BodyItemNode::Generator(gen) => pattern_get_vars(&gen.pattern),
       BodyItemNode::Agg(agg) => pattern_get_vars(&agg.pat),
       BodyItemNode::Clause(cl) => cl.args.iter().flat_map(|arg| arg.get_vars()).collect(),
       BodyItemNode::Negation(_cl) => vec![],
       BodyItemNode::SubQuery(_sq) => vec![],
       BodyItemNode::Disjunction(disj) => disj.disjuncts.iter()
                                           .flat_map(|conj| conj.iter().flat_map(body_item_get_bound_vars))
                                           .collect(),
       BodyItemNode::Cond(cl) => cl.bound_vars(),
       BodyItemNode::MacroInvocation(_) => vec![],
       BodyItemNode::FunctionCall(f) => {
          let mut res: Vec<Ident> = f.args.iter().flat_map(|arg| arg.get_vars()).collect();
          if let Some(ident) = &f.return_var {
             // res.push(ident.clone());
             res.push(ident.clone());
          }
          res
       },
    }
 }
 
 fn body_item_visit_bound_vars_mut(bi: &mut BodyItemNode, visitor: &mut dyn FnMut(&mut Ident)) {
    match bi {
       BodyItemNode::Generator(gen) => pattern_visit_vars_mut(&mut gen.pattern, visitor),
       BodyItemNode::Agg(agg) => pattern_visit_vars_mut(&mut agg.pat, visitor),
       BodyItemNode::Clause(cl) => {
          for arg in cl.args.iter_mut() {
             match arg {
                BodyClauseArg::Pat(p) => pattern_visit_vars_mut(&mut p.pattern, visitor),
                BodyClauseArg::Expr(e) => if let Some(ident) = expr_to_ident_mut(e) {visitor(ident)},
             }
          }
       },
       BodyItemNode::Negation(_cl) =>(),
       BodyItemNode::SubQuery(_sq) => (),
       BodyItemNode::Disjunction(disj) =>{
          for conj in disj.disjuncts.iter_mut() {
             for bi in conj.iter_mut() {
                body_item_visit_bound_vars_mut(bi, visitor)
             }
          }
       },
       BodyItemNode::Cond(cl) => match cl {
          CondClause::IfLet(cl) => pattern_visit_vars_mut(&mut cl.pattern, visitor),
          CondClause::If(_cl) => (),
          CondClause::Let(cl) => pattern_visit_vars_mut(&mut cl.pattern, visitor),
       },
       BodyItemNode::MacroInvocation(_) => (),
       BodyItemNode::FunctionCall(f) => {
          for arg in f.args.iter_mut() {
             match arg {
                BodyClauseArg::Pat(p) => pattern_visit_vars_mut(&mut p.pattern, visitor),
                BodyClauseArg::Expr(e) => if let Some(ident) = expr_to_ident_mut(e) {visitor(ident)},
             }
          }
          if let Some(ident) = &mut f.return_var {visitor(ident)}
       },
    }
 }
 
 fn body_item_visit_exprs_free_vars_mut(bi: &mut BodyItemNode, visitor: &mut dyn FnMut(&mut Ident), visit_macro_idents: bool){
    let mut visit = |expr: &mut Expr| {
       expr_visit_free_vars_mut(expr, visitor);
       if visit_macro_idents {
          expr_visit_idents_in_macros_mut(expr, visitor);
       }
    };
    match bi {
       BodyItemNode::Generator(gen) => visit(&mut gen.expr),
       BodyItemNode::Agg(agg) => {
          for arg in agg.rel_args.iter_mut() {visit(arg)}
          if let AggregatorNode::Expr(e) = &mut agg.aggregator {visit(e)}
       },
       BodyItemNode::Clause(cl) => {
          for arg in cl.args.iter_mut() {
             if let BodyClauseArg::Expr(e) = arg {
                visit(e);
             }
          }
       },
       BodyItemNode::Negation(cl) => {
          for arg in cl.args.iter_mut() {
             visit(arg);
          }
       },
       BodyItemNode::SubQuery(_sq) => {},
       BodyItemNode::Disjunction(disj) => {
          for conj in disj.disjuncts.iter_mut() {
             for bi in conj.iter_mut() {
                body_item_visit_exprs_free_vars_mut(bi, visitor, visit_macro_idents);
             }
          }
       },
       BodyItemNode::Cond(cl) => match cl {
          CondClause::IfLet(cl) => visit(&mut cl.exp),
          CondClause::If(cl) => visit(&mut cl.cond),
          CondClause::Let(cl) => visit(&mut cl.exp),
       },
       BodyItemNode::MacroInvocation(m) => {
          update(&mut m.mac.tokens, |ts| token_stream_replace_ident(ts, visitor));
       },
       BodyItemNode::FunctionCall(f) => {
          for arg in f.args.iter_mut() {
             if let BodyClauseArg::Expr(e) = arg {
                visit(e);
             }
          }
       },
    }
 }

fn rule_desugar_id_unification(rule: RuleNode) -> RuleNode {
    let mut desugared_body_items = vec![];
    for bi in rule.body_items.iter() {
       match bi {
          BodyItemNode::Clause(cl) => {
             let mut new_cl = cl.clone();
             if let Some(id_var) = cl.id_var.clone() {
                let id_ident = Ident::new(&format!("{}_id", cl.rel), cl.rel.span());
                let id_expr: syn::Expr = syn::parse2(quote!{#id_var}).unwrap();
                new_cl.args.push(syn::parse2(quote!{#id_expr}).unwrap());
                new_cl.rel = id_ident;
                new_cl.id_var = None;
             }
             desugared_body_items.push(BodyItemNode::Clause(new_cl));
          },
          _ => desugared_body_items.push(bi.clone())
       }
    }
    RuleNode{head_clauses: rule.head_clauses.clone(), body_items: desugared_body_items}
 }
 
 #[derive(Clone)]
 struct GenSym(HashMap<String, u32>, fn(&str) -> String);
 impl GenSym {
    pub fn next(&mut self, ident: &str) -> String {
       match self.0.get_mut(ident) {
          Some(n) => {*n += 1; format!("{}{}", self.1(ident), *n - 1)},
          None => {self.0.insert(ident.into(), 1); self.1(ident)},
       }
    }
    pub fn next_ident(&mut self, ident: &str, span: Span) -> Ident {
       Ident::new(&self.next(ident), span)
    }
    pub fn new(transformer: fn(&str) -> String) -> Self {
       Self(Default::default(), transformer)
    }
 }
 impl Default for GenSym {
    fn default() -> Self {
       Self(Default::default(), |x| format!("{}_", x))
    }
 }
 
 fn body_items_rename_macro_originated_vars(bis: &mut [&mut BodyItemNode], macro_def: &MacroDefNode, gensym: &mut GenSym) {
    let bi_vars = bis.iter().flat_map(|bi| body_item_get_bound_vars(bi)).collect_vec();
    let mut mac_body_idents = token_stream_idents(macro_def.body.clone());
    mac_body_idents.retain(|ident| bi_vars.contains(ident));
 
    let macro_originated_vars = bi_vars.iter()
       .filter(|v| mac_body_idents.iter().any(|ident| spans_eq(&v.span(), &ident.span())))
       .cloned()
       .collect::<HashSet<_>>();
    
    let var_mappings = macro_originated_vars.iter()
       .map(|v| (v, gensym.next(&v.to_string()))).collect::<HashMap<_,_>>();
    let mut visitor = |ident: &mut Ident| {
       if let Some(replacement) = var_mappings.get(ident) {
          if mac_body_idents.iter().any(|mac_ident| spans_eq(&mac_ident.span(), &ident.span())) {
             *ident = Ident::new(replacement, ident.span())
          }
       }
    };
    for bi in bis.iter_mut() {
       body_item_visit_bound_vars_mut(bi, &mut visitor);
       body_item_visit_exprs_free_vars_mut(bi, &mut visitor, true);
    }
 }
 
 fn clause_desugar_pattern_args(body_clause: BodyClauseNode, gensym: &mut GenSym) -> BodyClauseNode {
   let mut new_args = Punctuated::new();
   let mut new_cond_clauses = vec![];
   for arg in body_clause.args.into_pairs() {
      let (arg, punc) = arg.into_tuple();
      let new_arg = match arg {
         BodyClauseArg::Expr(_) => arg,
         BodyClauseArg::Pat(pat) => {
            let pattern = pat.pattern;
            let ident = gensym.next_ident("__arg_pattern", pattern.span());
            let new_cond_clause = quote!{ if let #pattern = #ident};
            let new_cond_clause = CondClause::IfLet(syn::parse2(new_cond_clause).unwrap());
            new_cond_clauses.push(new_cond_clause);
            BodyClauseArg::Expr(syn::parse2(quote!{#ident}).unwrap())
         }
      };
      new_args.push_value(new_arg);
      if let Some(punc) = punc {new_args.push_punct(punc)}
   }
   new_cond_clauses.extend(body_clause.cond_clauses);
   BodyClauseNode{
      extern_db_name: body_clause.extern_db_name,
      args: new_args,
      cond_clauses: new_cond_clauses,
      rel: body_clause.rel,
      id_var: body_clause.id_var,
      delta_flag: body_clause.delta_flag
   }
 }
 fn rule_desugar_pattern_args(rule: RuleNode) -> RuleNode {
    let mut gensym = GenSym::default();
    use BodyItemNode::*;
    RuleNode {
       body_items: rule.body_items.into_iter().map(|bi| match bi {
          Clause(cl) => Clause(clause_desugar_pattern_args(cl, &mut gensym)),
          _ => bi}).collect(),
       head_clauses: rule.head_clauses
    }
 }
 
 fn rule_desugar_repeated_vars(mut rule: RuleNode) -> RuleNode {
    
    let mut grounded_vars = HashMap::<Ident, usize>::new();
    for i in 0..rule.body_items.len(){
       let bitem = &mut rule.body_items[i];
       match bitem {
          BodyItemNode::Clause(cl) => {
             let mut new_cond_clauses = vec![];
             for arg_ind in 0..cl.args.len() {
                let expr = cl.args[arg_ind].unwrap_expr_ref();
                let expr_has_vars_from_same_clause =
                   expr_get_vars(expr).iter()
                   .any(|var| if let Some(cl_ind) = grounded_vars.get(var) {*cl_ind == i} else {false});
                if expr_has_vars_from_same_clause {
                   let new_ident = fresh_ident(&expr_to_ident(expr).map(|e| e.to_string()).unwrap_or_else(|| "expr_replaced".to_string()), expr.span());
                   new_cond_clauses.push(CondClause::If(
                      parse2(quote_spanned! {expr.span()=> if #new_ident.eq(&(#expr))}).unwrap()
                   ));
                   cl.args[arg_ind] = BodyClauseArg::Expr(parse2(new_ident.to_token_stream()).unwrap());
                } else if let Some(ident) = expr_to_ident(expr) {
                   grounded_vars.entry(ident).or_insert(i);
                }
             }
             for new_cond_cl in new_cond_clauses.into_iter().rev() {
                cl.cond_clauses.insert(0, new_cond_cl);
             }
          },
          BodyItemNode::Generator(gen) => {
             for ident in pattern_get_vars(&gen.pattern) {
                grounded_vars.entry(ident).or_insert(i);
             }
          },
          BodyItemNode::Cond(ref cond_cl @ CondClause::IfLet(_)) |
          BodyItemNode::Cond(ref cond_cl @ CondClause::Let(_)) => {
             for ident in cond_cl.bound_vars() {
                grounded_vars.entry(ident).or_insert(i);
             }
          }
          BodyItemNode::Cond(CondClause::If(_)) => (),
          BodyItemNode::Agg(agg) => {
             for ident in pattern_get_vars(&agg.pat){
                grounded_vars.entry(ident).or_insert(i);
             }
          },
          BodyItemNode::Negation(_) => (),
          BodyItemNode::SubQuery(_) => (),
          BodyItemNode::Disjunction(_) => panic!("unrecognized BodyItemNode variant"),
          BodyItemNode::MacroInvocation(m) => panic!("unexpected macro invocation: {:?}", m.mac.path),
          BodyItemNode::FunctionCall(_) => panic!("function call should already be desugared before repeated vars desugaring"),
       }
    }
    rule
 }
 
 fn rule_desugar_wildcards(mut rule: RuleNode) -> RuleNode {
    let mut gensym = GenSym::default();
    gensym.next("_"); // to move past "_"
    for bi in &mut rule.body_items[..] {
       if let BodyItemNode::Clause(bcl) = bi {
          for arg in bcl.args.iter_mut() {
             match arg {
                BodyClauseArg::Expr(expr) => {
                   if is_wild_card(expr) {
                      let new_ident = gensym.next_ident("_", expr.span());
                      *expr = parse2(quote! {#new_ident}).unwrap();
                   }
                }
                BodyClauseArg::Pat(_) => (),
             }
          }
       }
    }
    rule
 }
 
 fn rule_desugar_negation(mut rule: RuleNode) -> RuleNode {
    for bi in &mut rule.body_items[..] {
       if let BodyItemNode::Negation(neg) = bi {
          let rel = &neg.rel;
          let args = &neg.args;
          let replacement = quote_spanned! {neg.neg_token.span=> 
             agg () = ::ascent::aggregators::not() in #rel(#args)
          };
          let replacement: AggClauseNode = parse2(replacement).unwrap();
          *bi = BodyItemNode::Agg(replacement);
       }      
    }
    rule
 }
 
 fn rule_desugar_function_impl_decl_clause(rule:RuleNode) -> RuleNode {
    let mut desugared_body_items = vec![];
    for bi in rule.body_items {
       if let BodyItemNode::FunctionCall(f) = bi {
          if let None = f.return_var {
             desugared_body_items.push(
                BodyItemNode::Clause(BodyClauseNode {
                   rel: Ident::new(&format!("{}_do", f.name), f.name.span()),
                   extern_db_name: None,
                   args: f.args.clone(),
                   id_var: f.id_var.clone(),
                   cond_clauses: vec![],
                   delta_flag: false
                })
             );
          } else {
             desugared_body_items.push(BodyItemNode::FunctionCall(f));
          }
       } else {
          desugared_body_items.push(bi);
       }
    }
    RuleNode{head_clauses: rule.head_clauses, body_items: desugared_body_items}
 }
 
 fn rule_desugar_functional_call(rule: RuleNode) -> Vec<RuleNode> {
    // let mut generated_do_call_head = vec![];
    let mut desugared_body_items = vec![];
    let mut res_rules = vec![];
    let mut gensym = GenSym::default();
    let mut bi_pos = 0;
    for bi in &rule.body_items {
       if let BodyItemNode::FunctionCall(f) = bi {
          let arg_vec = f.args.clone();
          if f.return_var.is_none() {
             panic!("function call without return var!");
          }
          let ret_v = f.return_var.clone().unwrap();
          // TODO: support pattern arg here?
          // let clause_rel_name = if let Some(id_var) = &f.id_var {
          //    Ident::new(&format!("{}_id", f.name), id_var.span())
          // } else {
          //    f.name.clone()
          // };
          let do_clause_arg : Punctuated<BodyClauseArg, Comma> = arg_vec.clone();
          let var_do_clause_id = gensym.next_ident(&format!("{}_do__", f.name), f.name.span());
         let do_clause_used = BodyClauseNode {
             rel: Ident::new(&format!("{}_do", f.name), f.name.span()),
             extern_db_name: None,
             args: do_clause_arg,
             id_var: Some(syn::parse2(quote!{#var_do_clause_id}).unwrap()),
             cond_clauses: vec![],
             delta_flag: false
         };
          let use_result_clause = BodyClauseNode {
             rel: f.name.clone(),
             extern_db_name: None,
             args: Punctuated::from_iter(vec![
                BodyClauseArg::Expr(parse2(quote!{#var_do_clause_id}).unwrap()),
                BodyClauseArg::Expr(parse2(quote!{#ret_v}).unwrap())
             ]),
             id_var: f.id_var.clone(),
             cond_clauses: vec![],
             delta_flag: false,
          };
          desugared_body_items.push(BodyItemNode::Clause(do_clause_used));
          desugared_body_items.push(BodyItemNode::Clause(use_result_clause));
 
          let mut generate_call_body_items = vec![]; 
          let mut other_bi_pos = 0;
          for other_bi in &rule.body_items {
             if bi_pos == other_bi_pos {
                break;
             }
             // check if return var is used in other body items
             if bi_pos < other_bi_pos {
                if let BodyItemNode::FunctionCall(_) = other_bi {
                   continue;
                } else {
                    generate_call_body_items.push(other_bi.clone());
                }
             } else {
                if let BodyItemNode::FunctionCall(other_f) = other_bi {
                   generate_call_body_items.push(
                      BodyItemNode::Clause(BodyClauseNode {
                         rel: Ident::new(&format!("{}_do", other_f.name), other_f.name.span()),
                         extern_db_name: None,
                         args: other_f.args.clone(),
                         id_var: None,
                         cond_clauses: vec![],
                         delta_flag: false    
                      })
                   );
                } else {
                    generate_call_body_items.push(other_bi.clone());
                }
             }
             other_bi_pos += 1;
          }
          let generated_do_rule = RuleNode {
             head_clauses:  Punctuated::from_iter(vec![HeadItemNode::HeadClause(HeadClauseNode{
                rel: Ident::new(&format!("{}_do", f.name), f.name.span()),
                args: f.args.iter().map(|arg| match arg {
                    BodyClauseArg::Expr(expr) => expr.clone(),
                    _ => panic!("Pattern is not allowed in head!, found {:?}", arg),
                }).collect(),
                required_flag: false,
                id_name: None,
                delete_flag: false,
                exists_var: Some(gensym.next_ident(&format!("{}_exists", f.name), f.name.span()))
             })]),
             body_items: generate_call_body_items
          };
          res_rules.push(generated_do_rule);
       } else {
          desugared_body_items.push(bi.clone());
       }
       bi_pos += 1;
    }
    let use_result_rule = RuleNode {
       head_clauses: rule.head_clauses.clone(),
       body_items: desugared_body_items
    };
    res_rules.push(use_result_rule);
 
    res_rules
 }
 
 // fn is_free_var_in_body_item(ident: &Ident, bi: &BodyItemNode) -> bool {
 //    let mut res = false;
 //    let mut visitor = |i: &mut Ident| {
 //       // check if Ident string is equal to the given ident
 //       if i.to_string() == ident.to_string() {
 //          res = true;
 //       }
 //    };
 //    body_item_visit_exprs_free_vars_mut(&mut bi.clone(), &mut visitor, true);
 //    res
 // }
 
 fn rule_desugar_function_return(rule: RuleNode) -> RuleNode {
    let mut desugared_body_items = rule.body_items.clone();
    let mut desugared_head_items = vec![];
    let mut gensym = GenSym::default();
    for hi in &rule.head_clauses {
       if let HeadItemNode::HeadFuctionReturn(f) = hi {
          let do_call_id = gensym.next_ident(&format!("{}_ret__", f.name), f.name.span());
          let head_exists_var = gensym.next_ident(&format!("{}_exists", f.name), f.name.span());
          if let Some (ret_var) = &f.return_var {
            let new_do_clause = BodyClauseNode{
               rel: Ident::new(&format!("{}_do", f.name), f.name.span()),
               extern_db_name: None,
               args: f.args.clone(),
               id_var: Some(syn::parse2(quote!{#do_call_id}).unwrap()),
               cond_clauses: vec![],
               delta_flag: false
            };
            let mut gensym = GenSym::default();
            let pattern_desugared_do = clause_desugar_pattern_args(new_do_clause, &mut gensym);
            // insert to head of desugared_body_items
            desugared_body_items.insert(0, BodyItemNode::Clause(pattern_desugared_do));
            desugared_head_items.push(HeadItemNode::HeadClause(HeadClauseNode{
                rel: f.name.clone(),
                args: Punctuated::from_iter(vec![
                    parse2::<Expr>(quote!{#do_call_id}).unwrap(),
                    parse2::<Expr>(quote!{#ret_var}).unwrap()
                ]),
                required_flag: false,   // TODO: check if this is correct
                id_name: None,
                delete_flag: false,
                exists_var: Some(head_exists_var)
            }));
          } else {
            desugared_head_items.push(HeadItemNode::HeadClause(HeadClauseNode{
                rel: Ident::new(&format!("{}_do", f.name), f.name.span()),
                args: f.args.iter().map(|arg| match arg {
                    BodyClauseArg::Expr(expr) => expr.clone(),
                    _ => panic!("Pattern is not allowed in head!, found {:?}", arg),
                }).collect(),
                required_flag: false,
                id_name: None,
                delete_flag: false,
                exists_var: Some(head_exists_var)
            }));
          }
       } else {
          desugared_head_items.push(hi.clone());
       }
    }
 
    RuleNode{
       head_clauses: Punctuated::from_iter(desugared_head_items),
       body_items: desugared_body_items
    }
 }
 
 fn invoke_macro(invocation: &ExprMacro, definition: &MacroDefNode) -> Result<TokenStream> {
    let tokens = invocation.mac.tokens.clone();
 
    fn parse_args(definition: &MacroDefNode, args: ParseStream, span: Span) -> Result<HashMap<Ident, TokenStream>> {
       let mut ident_replacement = HashMap::new();
 
       for pair in definition.params.pairs(){
          if args.is_empty() {
             return Err(Error::new(span, "expected more arguments"));
          }
          let (param, comma) = pair.into_tuple();
          let mut va_flag = false;
          let arg = match param.kind {
             MacroParamKind::Expr(_) => args.parse::<Ident>()?.into_token_stream(),
             MacroParamKind::Ident(_) => args.parse::<Expr>()?.into_token_stream(),
             MacroParamKind::VaList(_) => {
                va_flag = true;
                // parse to the end even if its ","
                let mut res_args = TokenStream::new();
                while !args.is_empty() {
                   if args.peek(Token![,]) {
                      res_args.extend(args.parse::<Token![,]>().unwrap().into_token_stream());
                   } else {
                      res_args.extend(args.parse::<Expr>()?.into_token_stream());
                   }
                }
                res_args
             }
          };
          ident_replacement.insert(param.name.clone(), arg);
          if va_flag {
             break;
          }
          if comma.is_some() {
             if args.is_empty() {
                return Err(Error::new(span, "expected more arguments"));
             }
             args.parse::<Token![,]>()?;
          }
       }
       
       Ok(ident_replacement)
    }
 
    let args_parser = |inp: ParseStream| parse_args(definition, inp, invocation.mac.span());
    let args_parsed = Parser::parse2(args_parser, tokens)?;
 
    let (replaced_body, _) = token_stream_replace_macro_idents(definition.body.clone(), &args_parsed);
    Ok(replaced_body)
 }
 
 fn rule_expand_macro_invocations(rule: RuleNode, macros: &HashMap<Ident, &MacroDefNode>) -> Result<RuleNode> {
 
    const RECURSIVE_MACRO_ERROR: &'static str = "recursively defined Ascent macro";
    fn body_item_expand_macros(bi: BodyItemNode, macros: &HashMap<Ident, &MacroDefNode>, gensym: &mut GenSym, depth: i16, span: Option<Span>) 
    -> Result<Punctuated<BodyItemNode, Token![,]>> 
    {
       if depth <= 0 {
          return Err(Error::new(span.unwrap_or_else(Span::call_site), RECURSIVE_MACRO_ERROR))
       }
       match bi {
          BodyItemNode::MacroInvocation(m) => {
             let mac_def = macros.get(m.mac.path.get_ident().unwrap())
                          .ok_or_else(|| Error::new(m.span(), "undefined macro"))?;
             let macro_invoked = invoke_macro(&m, mac_def)?;
             let expanded_bis = Parser::parse2(Punctuated::<BodyItemNode, Token![,]>::parse_terminated, macro_invoked)?;
             let mut recursively_expanded = 
                punctuated_try_map(expanded_bis, |ebi| body_item_expand_macros(ebi, macros, gensym, depth - 1, Some(m.span())))?
                .pipe(flatten_punctuated);
             body_items_rename_macro_originated_vars(&mut recursively_expanded.iter_mut().collect_vec(), mac_def, gensym);
             Ok(recursively_expanded)
          },
          BodyItemNode::Disjunction(disj) => {
             let new_disj: Punctuated<Result<_>, _> = punctuated_map(disj.disjuncts, |bis|{
                let new_bis = punctuated_map(bis,|bi| {
                   body_item_expand_macros(bi, macros, gensym, depth - 1, Some(disj.paren.span.join()))
                });
                Ok(flatten_punctuated(punctuated_try_unwrap(new_bis)?))
             });
             
             Ok(punctuated_singleton(BodyItemNode::Disjunction(DisjunctionNode{disjuncts: punctuated_try_unwrap(new_disj)?, .. disj})))
          },
          _ => Ok(punctuated_singleton(bi))
       }
    }
 
    fn head_item_expand_macros(hi: HeadItemNode, macros: &HashMap<Ident, &MacroDefNode>, depth: i16, span: Option<Span>) 
    -> Result<Punctuated<HeadItemNode, Token![,]>> 
    {
       if depth <= 0 {
          return Err(Error::new(span.unwrap_or_else(Span::call_site), RECURSIVE_MACRO_ERROR))
       }
       match hi {
          HeadItemNode::MacroInvocation(m) => {
             let mac_def = macros.get(m.mac.path.get_ident().unwrap())
                           .ok_or_else(|| Error::new(m.span(), "undefined macro"))?;
             let macro_invoked = invoke_macro(&m, mac_def)?;
             let expanded_his = Parser::parse2(Punctuated::<HeadItemNode, Token![,]>::parse_terminated, macro_invoked)?;
 
             Ok(punctuated_map(expanded_his, |ehi| head_item_expand_macros(ehi, macros, depth - 1, Some(m.span())))
                .pipe(punctuated_try_unwrap)?
                .pipe(flatten_punctuated))
          },
          HeadItemNode::HeadFuctionReturn(_) => Ok(punctuated_singleton(hi)),
          HeadItemNode::HeadClause(_) => Ok(punctuated_singleton(hi)),
       }
    }
 
    let mut gensym = GenSym::new(|s| format!("__{}_", s));
 
    let new_body_items = rule.body_items.into_iter()
                         .map(|bi| body_item_expand_macros(bi, macros, &mut gensym, 100, None))
                         .collect::<Result<Vec<_>>>()?
                         .into_iter().flatten().collect_vec();
 
    let new_head_items = punctuated_map(rule.head_clauses, |hi| head_item_expand_macros(hi, macros, 100, None))
                         .pipe(punctuated_try_unwrap)?
                         .pipe(flatten_punctuated);
                         
    Ok(RuleNode {body_items: new_body_items, head_clauses: new_head_items})
 }
 
 fn desugar_relation_with_id(rel: RelationNode) -> Vec<RelationNode> {
    if rel.need_id {
       let mut new_field_types = rel.field_types.clone();
       new_field_types.push( Type::Verbatim(quote!{usize}));
       let new_rel = RelationNode{
          attrs: rel.attrs.clone(),
          name: Ident::new(&format!("{}_id", rel.name), rel.name.span()),
          field_types: new_field_types,
          initialization: rel.initialization.clone(),
          source_db: None,
          _semi_colon: rel._semi_colon.clone(),
          is_lattice: rel.is_lattice,
          need_id: false,
          is_hole: false,
          is_input: rel.is_input,
       };
       vec![rel, new_rel]
    } else {
       vec![rel]
    }
 }
 
 fn desugar_function(rel: &FunctionNode) -> Vec<RelationNode> {
       // do rule associated with function
       // do rule takes 0..n-1 arguments
    // let mut arg_vec = rel.field_types.iter().cloned().collect_vec();
    // arg_vec.push(Type::Verbatim(quote!{usize}));
    let do_relation = RelationNode{
       attrs: rel.attrs.clone(),
       name: Ident::new(&format!("{}_do", rel.name), rel.name.span()),
       field_types: rel.field_types.clone(),
       initialization: None,
       source_db: None,
       _semi_colon: syn::token::Semi::default(),
       is_lattice: false,
       need_id: true,
       is_hole: false,
       is_input: false,
    };
    let usize_type = Type::Verbatim(quote!{usize});
    let res_relation = RelationNode{
       attrs: rel.attrs.clone(),
       name: rel.name.clone(),
       field_types: Punctuated::from_iter(vec![
          usize_type.clone(), rel.return_type.clone()
       ]),
       initialization: None,
       source_db: None,
       _semi_colon: syn::token::Semi::default(),
       is_lattice: false,
       need_id: true,
       is_hole: false,
       is_input: false,
    };
 
    vec![do_relation, res_relation]
 }

 fn add_stratum(stratum_name: &Ident) -> RelationNode {
   let hole_rel_name = Ident::new(&format!("{}_stratum", stratum_name), stratum_name.span());
   parse2(quote_spanned! {stratum_name.span()=>
      relation #hole_rel_name (&'static str);
   }).unwrap()
}

fn stratum_path_to_rules(hole_path: &StratumPath, relations: &Vec<RelationNode>) -> Vec<RuleNode> {
   let hole_rel_name = Ident::new(
      &format!("{}_stratum", hole_path.hole_name), hole_path.hole_name.span());
   // TODO: add error handling 
   let mut input_rel_arity = 0;
   for rel in relations.iter() {
      if rel.name == hole_path.rel_name {
         input_rel_arity = rel.field_types.len();
         break;
      }
   }
   let default_args = (0..input_rel_arity).into_iter()
      .map(|_| quote! {Default::default()})
      .collect_vec();
   let input_rel_name = hole_path.rel_name.clone();

   let relnode_in : RuleNode = syn::parse2(
      quote_spanned! { hole_path.hole_name.span() =>
         #hole_rel_name ("") <-- #input_rel_name #(#default_args)*, if false;
      }).unwrap();
   let relnode_out : RuleNode = syn::parse2(
      quote_spanned! { hole_path.hole_name.span() =>
         #input_rel_name #(#default_args)* <-- #hole_rel_name (""), , if false;
      }).unwrap();
   vec![relnode_in, relnode_out]
}

fn desugar_subquery(subquery: &SubQueryNode) -> Vec<BodyItemNode> {
   let sub_db_name = &subquery.name;
   let db_type = &subquery.query_type;
   let init_args = subquery.query_init.iter().map(|arg| {
      let arg_name = &arg.rel;
      let arg_v = &arg.arg;
      quote_spanned! {arg_name.span() => #arg_name: #arg_v}
   }).collect::<Vec<_>>();
   let create_query_db : BodyItemNode = if init_args.len() != 0 { syn::parse2(
      quote_spanned! {sub_db_name.span() =>
         let mut #sub_db_name = #db_type {
            #(#init_args),*
            , ..Default::default()
         }
      }).unwrap()
   } else { syn::parse2(
      quote_spanned! {sub_db_name.span() =>
         let mut #sub_db_name = #db_type::default()
      }).unwrap()
   };
   let run_args = &subquery.query_extern_db;
   let run_db : BodyItemNode = syn::parse2(
      quote_spanned! {sub_db_name.span() =>
         let _ = #sub_db_name.run(#run_args)
      }).unwrap();
   vec![create_query_db, run_db]
}

fn desugar_subquery_runs(rules: &Vec<RuleNode>) -> Vec<RuleNode> {
   let mut res = vec![];
   for rule in rules {
      let mut new_body_items = vec![];
      for bi in rule.body_items.iter() {
         match bi {
            BodyItemNode::SubQuery(sq) => {
               new_body_items.extend(desugar_subquery(&sq));
            },
            _ => new_body_items.push(bi.clone())
         }
      }
      res.push(RuleNode{
         head_clauses: rule.head_clauses.clone(),
         body_items: new_body_items});
   }
   res
}
 
 pub(crate) fn desugar_ascent_program(mut prog: AscentProgram) -> Result<AscentProgram> {
    let macros = &prog.macros.iter().map(|m| (m.name.clone(), m)).collect::<HashMap<_,_>>(); 
 
    for invoke in prog.macro_invocs.iter() {
       let mac_def = macros.get(invoke.mac.path.get_ident().unwrap())
                   .ok_or_else(|| Error::new(invoke.span(), "undefined macro"))?;
       let expanded = invoke_macro(invoke, mac_def)?;
       // parse as program
       let expanded_prog = Parser::parse2(AscentProgram::parse, expanded)?;
       prog.rules.extend(expanded_prog.rules);
       prog.relations.extend(expanded_prog.relations);
    }
 
    let mut rules_macro_expanded = 
       prog.rules.into_iter()
       .map(|r| rule_expand_macro_invocations(r, &macros))
       .collect::<Result<Vec<_>>>()?;
     rules_macro_expanded = desugar_subquery_runs(&rules_macro_expanded);
 
    let relation_from_functions = prog.functions.iter()
       .flat_map(desugar_function)
       .collect_vec();
    let relation_from_hole = prog.stratum_paths.iter()
      .map(|w| w.rel_name.clone())
      .dedup()
      .map(|w| add_stratum(&w))
      .collect_vec();
    prog.relations.extend(relation_from_hole);
    let rule_from_hole = prog.stratum_paths.iter()
      .flat_map(|w| stratum_path_to_rules(w, &prog.relations))
      .collect_vec();
    rules_macro_expanded.extend(rule_from_hole); 

    prog.relations.extend(relation_from_functions);
    prog.functions = vec![];
    prog.relations = prog.relations.into_iter()
       .flat_map(desugar_relation_with_id)
       .collect_vec();
    let rule_desugared_disjunction = rules_macro_expanded.into_iter()
       .flat_map(rule_desugar_disjunction_nodes)
       .collect_vec();
    prog.rules = 
       rule_desugared_disjunction.into_iter()
       .map(rule_desugar_function_impl_decl_clause)
       .map(rule_desugar_function_return)
       .flat_map(rule_desugar_functional_call)
       .map(rule_desugar_exists_bang)
       .map(rule_desugar_id_unification)
       .map(rule_desugar_pattern_args)
       .map(rule_desugar_wildcards)
       .map(rule_desugar_negation)
       .map(rule_desugar_repeated_vars)
       .collect_vec();
 
    Ok(prog)
 }
 
 lazy_static::lazy_static! {
    static ref IDENT_COUNTERS: Mutex<HashMap<String, u32>> = Mutex::new(HashMap::default());
 }
 fn fresh_ident(prefix: &str, span: Span) -> Ident {
    let mut ident_counters_lock = IDENT_COUNTERS.lock().unwrap();
    let counter = if let Some(entry) = ident_counters_lock.get_mut(prefix) {
       let counter = *entry;
       *entry = counter + 1;
       format!("{}", counter)
    } else {
       ident_counters_lock.insert(prefix.to_string(), 1);
       "".to_string()
    };
    
    Ident::new(&format!("{}_{}", prefix, counter), span)
 }
