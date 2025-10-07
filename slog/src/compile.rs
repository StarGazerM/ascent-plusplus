// compile slog program to ascent program

use std::collections::HashMap;

use itertools::Either;
use proc_macro2::TokenStream;
use quote::{ToTokens, quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{Ident, Result};

use crate::syntax::{
   ParenType, SlogClauseArg, SlogMeta, SlogProgram, SlogProgramLine, SlogRelationDecl, SlogRule, SlogRuleBodyItem, SlogRuleHeadItem, SlogSExprClause, SlogTheory, SlogType, SlogUnionClause
};
use crate::util::{new_ident, respan_to_call_site};

fn compile_slog_meta(meta: &SlogMeta) -> Result<TokenStream> {
   // let vis = meta.vis.clone();
   let vis = quote! { pub };
   let struct_name = meta.struct_name.clone();
   Ok(quote! {
      #vis struct #struct_name;
   })
}

fn canonicalize_theory(th_name: &Ident, theory: &SlogTheory) -> Option<Ident> {
   if let Some(th) = theory.get_theory_decl_by_name(th_name){
      Some(Ident::new(&format!("canonicalize_{}", th_name.to_token_stream().to_string()),th.unify_rel.span()))
   } else {
      None
   }
}

fn compile_slog_relation_decl(decl: &SlogRelationDecl, theory: &SlogTheory) -> Result<TokenStream> {
   let rel_name = decl.rel_name.clone();
   let arg_types = decl.arg_types.clone();
   // TODO: make prov track code more general
   let prov_track_code = {
      let id_name = new_ident(&format!("{}_idv", rel_name));
      let mut heads = vec![];
      let mut args = vec![];
      for (i, arg_ty) in arg_types.iter().enumerate() {
         // track provenance for theory type if needed
         if let SlogType::Theory(th_name) = arg_ty {
            if let Some(th) = theory.get_theory_decl_by_name(th_name) {
               if th.provenance.is_none() {
                  continue;
               }
               let arg_name = new_ident(&format!("arg_{}", i));
               heads.push(quote_spanned! { th_name.span() =>
                  deriv(#id_name, #arg_name)
               });
               args.push(arg_name);
            }
         }
      }
      if heads.is_empty() {
         quote! {}
      } else {
         quote_spanned! { decl.rel_name.span() =>
            #(#heads),* <--  #rel_name(#(#args),*, #id_name);
         }
      }
   };
   let arg_types_tokens = arg_types.iter().map(|arg_type| match arg_type {
      SlogType::Theory(th_name) => {
         let ty = theory.get_theory_type_by_name(th_name).unwrap();
         quote! { #ty }
      },
      SlogType::Rust(ty) => {
         quote! { #ty }
      },
   });
   Ok(quote_spanned! { decl.rel_name.span() =>
      relation #rel_name(#(#arg_types_tokens),*, usize);
      // add why provenance for relations
      #prov_track_code
   })
}

fn compile_slog_clause_unstructured_head(
   clause: &SlogSExprClause, theory: &SlogTheory, clause_num: usize, rel_to_arg_types: &HashMap<String, Vec<SlogType>>,
   new_id_decls: &mut Vec<TokenStream>,
) -> Result<TokenStream> {
   let rel_name = clause.rel_name.clone();
   let args = clause.args.clone();
   let arg_types = rel_to_arg_types.get(&rel_name.to_string());
   let arg_types = if arg_types.is_none() {
      if clause.rel_name.to_string() == "nil" {
         let ty: syn::Type = syn::parse2(quote! { usize }).unwrap();
         vec![SlogType::Rust(ty.clone()), SlogType::Rust(ty.clone())]
      } else if let Some(th) = theory.get_theory_decl_by_rel_name(&rel_name) {
         vec![SlogType::Rust(th.ty.clone()), SlogType::Rust(th.ty.clone())]
      } else {
         return Err(syn::Error::new_spanned(
            clause.rel_name.clone(),
            format!("relation {} not found in rel_to_arg_types {:?}", rel_name, rel_to_arg_types),
         ));
      }
   } else {
      arg_types.unwrap().clone()
   };
   let mut arg_canonicalization = vec![];
   let mut expr_alias = vec![];

   let args_tokens = args
      .iter()
      .enumerate()
      .map(|(i, arg)| {
         let ty = &arg_types[i];
         match arg {
            SlogClauseArg::LogicVar(id, _) =>
               if let SlogType::Theory(th_name) = ty {
                  let cl_arg = Ident::new(&format!("hcl_{}_{}", clause_num, i), id.span());
                  let canonicalize_fn = canonicalize_theory(&th_name, theory);
                  let unify_rel = theory.get_unify_rel_by_name(&th_name);
                  arg_canonicalization.push(quote_spanned! { id.span() =>
                     magg #cl_arg = #canonicalize_fn(#id) in #unify_rel
                     // let #cl_arg = 0
                  });
                  Some(quote_spanned! { id.span() => #cl_arg })
               } else {
                  Some(quote_spanned! { id.span() => #id })
               },
            SlogClauseArg::SlogClause(_) => None,
            SlogClauseArg::Constant(constant) =>
               if let SlogType::Theory(th_name) = ty {
                  let cl_arg = Ident::new(&format!("hcl_{}_{}", clause_num, i), constant.span());
                  let canonicalize_fn = canonicalize_theory(&th_name, theory);
                  let unify_rel = theory.get_unify_rel_by_name(&th_name);
                  arg_canonicalization.push(quote_spanned! { constant.span() =>
                     let #cl_arg = #canonicalize_fn(#constant) in #unify_rel
                  });
                  Some(quote_spanned! { constant.span() => #cl_arg })
               } else {
                  Some(quote_spanned! { constant.span() => #constant })
               },
            SlogClauseArg::Wildcard => None,
            SlogClauseArg::RustExpr(expr) => {
               let cl_arg_e = Ident::new(&format!("hcl_{}_{}_expr", clause_num, i), expr.span());
               let cl_arg = Ident::new(&format!("hcl_{}_{}", clause_num, i), expr.span());
               if clause_num != 0 {
                  expr_alias.push(quote_spanned! { expr.span() =>
                     let #cl_arg_e = #expr
                  });
                  if let SlogType::Theory(th_name) = ty {
                     let canonicalize_fn = canonicalize_theory(&th_name, theory);
                     let unify_rel = theory.get_unify_rel_by_name(&th_name);
                     arg_canonicalization.push(quote_spanned! { expr.span() =>
                        let #cl_arg = #canonicalize_fn(#cl_arg_e) in #unify_rel
                     });
                  }
               } else {
                  if let SlogType::Theory(th_name) = ty {
                     let canonicalize_fn = canonicalize_theory(&th_name, theory);
                     let unify_rel = theory.get_unify_rel_by_name(&th_name);
                     arg_canonicalization.push(quote_spanned! { expr.span() =>
                        let #cl_arg = #canonicalize_fn(#expr) in #unify_rel
                     });
                  }
               }
               if let SlogType::Theory(_) = ty {
                  if clause_num != 0 {
                     Some(quote_spanned! { expr.span() => #cl_arg_e })
                  } else {
                     Some(quote_spanned! { expr.span() => #cl_arg })
                  }
               } else {
                  if clause_num != 0 {
                     Some(quote_spanned! { expr.span() => #cl_arg_e })
                  } else {
                     Some(quote_spanned! { expr.span() => #expr })
                  }
               }
            },
         }
      })
      .collect::<Vec<_>>();
   if args_tokens.iter().any(|arg| arg.is_none()) {
      return Err(syn::Error::new_spanned(
         clause.rel_name.clone(),
         format!("unstructured clause must be all logic vars or all constants {:?}", clause),
      ));
   }
   new_id_decls.extend(expr_alias);
   new_id_decls.extend(arg_canonicalization);

   // let id_rel_name = Ident::new(&format!("{}_id", rel_name), rel_name.span());
   let rel_name_str = rel_name.to_string();
   if let Some(_th) = theory.get_theory_decl_by_rel_name(&rel_name) {
      Ok(quote_spanned! { rel_name.span() =>
         #rel_name(#(#args_tokens),*)
      })
   } else {
      if clause.id_var.is_some() {
         let id_var = clause.id_var.clone().unwrap();
         new_id_decls.push(quote_spanned! { clause.id_var.clone().unwrap().span() =>
            let #id_var = &calc_id(&(#rel_name_str, #(#args_tokens),*))
         });
         Ok(quote_spanned! { rel_name.span() =>
            #rel_name(#(#args_tokens),*, #id_var)
         })
      } else {
         Ok(quote_spanned! { rel_name.span() =>
            #rel_name(#(#args_tokens),*, calc_id(&(#rel_name_str, #(#args_tokens),*)))
         })
      }
   }
}

fn compile_slog_clause_unstructured_body(
   clause: &SlogSExprClause, theory: &SlogTheory, ignore_id: bool, rel_to_arg_types: &HashMap<String, Vec<SlogType>>,
   clause_num: usize, all_clauses: &Vec<SlogRuleBodyItem>,
) -> Result<TokenStream> {
   let rel_name = clause.rel_name.clone();
   let arg_types = rel_to_arg_types.get(&rel_name.to_string());
   let arg_types = if arg_types.is_none() {
      if clause.rel_name.to_string() == "nil" {
         let ty: syn::Type = syn::parse2(quote! { usize }).unwrap();
         vec![SlogType::Rust(ty.clone()), SlogType::Rust(ty.clone())]
      } else if let Some(th) = theory.get_theory_decl_by_rel_name(&rel_name) {
         vec![SlogType::Rust(th.ty.clone()), SlogType::Rust(th.ty.clone())]
      } else {
         return Err(syn::Error::new_spanned(
            clause.rel_name.clone(),
            format!("relation {} not found in rel_to_arg_types {:?}", rel_name, rel_to_arg_types),
         ));
      }
   } else {
      arg_types.unwrap().clone()
   };
   let is_bound_var = |x: &Ident| {
      all_clauses.iter().take(clause_num).enumerate().any(|(i, clause)| {
         if i == clause_num {
            return false;
         }
         match clause {
            SlogRuleBodyItem::SlogSExprClause(sexpr) => {
               sexpr.args.iter().any(|arg| match arg {
                  SlogClauseArg::LogicVar(id, _) => {
                     // check if their str is the same
                     x.to_string() == id.to_string()
                  },
                  _ => false,
               })
            },
            _ => false,
         }
      })
   };
   let mut clause_after_code_vec = vec![];
   let args = clause.args.clone();
   let args_tokens = args
      .iter()
      .enumerate()
      .map(|(i, arg)| {
         // check if arg type is theory type, which need unification
         if let SlogType::Theory(th_name) = &arg_types[i] {
            let unify_relation = theory.get_unify_rel_by_name(&th_name);
            match arg {
               SlogClauseArg::LogicVar(id, _) => {
                  // binding, inflate and join with equiv
                  if !is_bound_var(id) {
                     let inflated_id = Ident::new(&format!("inflated_{}", id), id.span());
                     clause_after_code_vec.push(quote_spanned! {id.span()=>
                        #unify_relation(#inflated_id, #id)
                     });
                     Some(quote_spanned! {id.span()=> #inflated_id})
                  } else {
                     Some(quote_spanned! {id.span()=> #id})
                  }
               },
               SlogClauseArg::SlogClause(_) => None,
               SlogClauseArg::Constant(constant) => Some(quote_spanned! {constant.span()=> {
                  #constant
               }}),
               SlogClauseArg::Wildcard => Some(quote! { _ }),
               SlogClauseArg::RustExpr(expr) => Some(quote_spanned! {expr.span()=> {
                  #expr
               }}),
            }
         } else {
            match arg {
               SlogClauseArg::LogicVar(id, _) => Some(quote_spanned! {id.span()=> #id }),
               SlogClauseArg::SlogClause(_) => None,
               SlogClauseArg::Constant(constant) => Some(quote_spanned! {constant.span()=> #constant }),
               SlogClauseArg::Wildcard => Some(quote! { _ }),
               SlogClauseArg::RustExpr(expr) => Some(quote_spanned! {expr.span()=> #expr }),
            }
         }
      })
      .collect::<Vec<_>>();
   // if any is none, return error
   if args_tokens.iter().any(|arg| arg.is_none()) {
      return Err(syn::Error::new_spanned(
         clause.rel_name.clone(),
         format!("unstructured clause must be all logic vars or all constants {:?}", clause),
      ));
   }
   if let Some(_th) = theory.get_theory_decl_by_rel_name(&rel_name) {
      Ok(quote_spanned! {clause.rel_name.span()=>
         #rel_name(#(#args_tokens),*)
         #(,#clause_after_code_vec)*
      })
   } else {
      let id_tag_code = if clause.id_var.is_some() && !ignore_id {
         let id_var = clause.id_var.clone().unwrap();
         quote_spanned! {id_var.span()=>
            #id_var
         }
      } else {
         quote! {_}
      };
      Ok(quote_spanned! {clause.rel_name.span()=>
         #rel_name(#(#args_tokens),*, #id_tag_code)
         #(,#clause_after_code_vec)*
      })
   }
}

fn compile_slog_line(
   line: &SlogProgramLine, rel_to_arg_types: &HashMap<String, Vec<SlogType>>, theory: &SlogTheory,
) -> Result<TokenStream> {
   match line {
      SlogProgramLine::Rule(rule) => compile_slog_rule_unstructured(rule, theory, rel_to_arg_types),
      SlogProgramLine::RelationDecl(decl) => compile_slog_relation_decl(decl, theory),
      SlogProgramLine::Fact(clause) => {
         // desugar to a rule with empty body rule
         let rule = SlogRule {
            heads: vec![SlogRuleHeadItem::SlogSExprClause(clause.clone())],
            body: vec![],
            delta_first: None,
         };
         compile_slog_rule_unstructured(&rule, theory, rel_to_arg_types)
      },
      SlogProgramLine::Ascent(content) => Ok(content.clone()),
   }
}

pub fn compile_slog_program(program: &SlogProgram, is_parallel: bool) -> Result<TokenStream> {
   let meta = compile_slog_meta(&program.meta)?;
   // computing a map from relation name to relation type arg
   let rel_to_arg_types: HashMap<_, _> = program
      .lines
      .iter()
      .filter_map(|line| match line {
         SlogProgramLine::RelationDecl(decl) => {
            // eprintln!("decl >>>>>>>>>>>>>>>>>>> {:?}", decl.arg_types.clone());
            Some((decl.rel_name.to_string(), decl.arg_types.clone()))
         },
         _ => None,
      })
      .collect::<HashMap<_, _>>();
   let decl_lines = program
      .lines
      .iter()
      .filter_map(|line| match line {
         SlogProgramLine::RelationDecl(decl) => {
            Some(compile_slog_relation_decl(decl, &program.theory))
         },
         _ => None,
      })
      .collect::<Result<Vec<_>>>()?;
   let query_lines = program
      .lines
      .iter()
      .filter_map(|line| match line {
         SlogProgramLine::RelationDecl(_) => None,
         _ => Some(compile_slog_line(line, &rel_to_arg_types, &program.theory)),
      })
      .collect::<Result<Vec<_>>>()?;

   let slog_mode = if !is_parallel {
      quote! {
         ascent
      }
   } else {
      quote! {
         ascent_par
      }
   };
   let num_theories = program.theory.names.len();
   if num_theories == 0 {
      Ok(quote! {
         #slog_mode! {
            #![allow_non_stratified_agg]
            #meta

            relation nil(usize, usize);
            nil(1, calc_id(&("nil", 1)));
            nil(0, calc_id(&("nil", 0))) <-- nil(1, _);
            // provenance relation
            relation deriv(usize, usize);

            #(#decl_lines)*
            #(#query_lines)*
         }
      })
   } else {
      let mut theory_rel_decls = vec![];
      let mut theorized_code = quote! {};
      for (i, t) in program.theory.uses.iter().enumerate() {
         let macro_call = Ident::new(&format!("theory_rules_{}", t.name.to_string()), t.name.span());
         let macro_args_opt = t.opt_args.iter().map(|arg| quote_spanned! {arg.span()=> #arg}).collect::<Vec<_>>();
         let theory_rel_name = t.unify_rel.clone();
         let theory_rel_ds = t.ds.clone();
         let theory_rel_ty = t.ty.clone();
         let theory_rel_default = quote! { #theory_rel_name(Default::default(), Default::default()); };
         theory_rel_decls.push(quote_spanned! {t.name.span()=>
            #[ds(#theory_rel_ds)]
            relation #theory_rel_name(#theory_rel_ty, #theory_rel_ty);
         });
         if i != num_theories - 1 {
            // theorized_code = quote! {
            //    #macro_call! { (#theory_rel_name #(,#macro_args_opt)*), { #theory_rel_default #(#query_lines)* } }
            // };
            todo!("TODO: multiple theories is not supported yet");
         } else {
            let struct_name = program.meta.struct_name.clone();
            theorized_code = quote! {
               #macro_call! {
                  (#theory_rel_name), (#struct_name), 
                  { #slog_mode },
                  {
                     // #![allow_non_stratified_agg]
                     // #meta
                     relation nil(usize, usize);
                     nil(1, calc_id(&("nil", 1)));
                     nil(0, calc_id(&("nil", 0))) <-- nil(1, _);
                     // provenance relation
                     relation deriv(usize, usize);
                     #(#theory_rel_decls)*
                     #(#decl_lines)*
                  },
                  {
                     #theory_rel_default #(#query_lines)*
                  }
               }
            }
         }
      }
      // respan to call site
      let theorized_code = respan_to_call_site(theorized_code);
      // eprintln!("theorized_code {}", theorized_code.to_string());
      Ok(theorized_code)
   }
}

pub(crate) fn compile(tokens: TokenStream, is_parallel: bool) -> Result<TokenStream> {
   let program: SlogProgram = syn::parse2(tokens)?;
   // pass 1: remove nested union clauses
   let id_union_program = remove_nested_union_clause(&program);
   // pass 2: destruct the program to remove nested sexprs
   let destructed_program = destruct_slog_program(&id_union_program);

   let compiled_program = compile_slog_program(&destructed_program, is_parallel)?;
   Ok(compiled_program)
}

fn compile_slog_rule_unstructured(
   rule: &SlogRule, theory: &SlogTheory, rel_to_arg_types: &HashMap<String, Vec<SlogType>>,
) -> Result<TokenStream> {
   let mut new_id_decls = vec![];
   let heads = rule
      .heads
      .iter()
      .enumerate()
      .map(|(i, head)| match head {
         SlogRuleHeadItem::SlogSExprClause(sexpr) =>
            compile_slog_clause_unstructured_head(sexpr, theory, i, rel_to_arg_types, &mut new_id_decls),
         SlogRuleHeadItem::UnionClause(clause) => {
            // when compiling unstructured union clause, we should only have id here
            // otherwise throw compile error
            if let (Either::Left(id_l), Either::Left(id_r)) = (&clause.clause_lhs, &clause.clause_rhs) {
               Ok(quote! {
                  // #id_l <=> #id_r,
                  unify_eclass(#id_l, #id_r)
               })
            } else {
               return Err(syn::Error::new_spanned(
                  clause._union.clone(),
                  format!("union clause must have id on both sides {:?}", clause),
               ));
            }
         },
      })
      .collect::<Result<Vec<_>>>()?;
   let mut bodys = rule
      .body
      .iter()
      .enumerate()
      .map(|(i, body)| match body {
         SlogRuleBodyItem::SlogSExprClause(sexpr) =>
            compile_slog_clause_unstructured_body(sexpr, theory, false, rel_to_arg_types, i, &rule.body),
         SlogRuleBodyItem::NegatedSlogSExprClause(sexpr) => {
            let compiled_clause =
               compile_slog_clause_unstructured_body(sexpr, theory, false, rel_to_arg_types, i, &rule.body);
            compiled_clause.map(|clause| {
               quote! {
                  !#clause
               }
            })
         },
         SlogRuleBodyItem::AscentClause(content) => {
            let content = content.clone();
            Ok(quote! {
               #content
            })
         },
      })
      .collect::<Result<Vec<_>>>()?;
   // if body is empty, add a dummy body
   if bodys.is_empty() {
      //    let defaults = theory
      //       .uses
      //       .iter()
      //       .map(|u| {
      //          let unify_rel_name = Ident::new(&format!("unify_{}", u.name.to_string()), u.name.span());
      //          quote! { #unify_rel_name(Default::default(), Default::default()) }
      //       })
      //       .collect::<Vec<_>>();
      bodys.push(quote! { nil(0, _)  });
   }
   let delta_first_code = if rule.delta_first.is_some() {
      quote! { #[heuristic_reordering] }
   } else {
      quote! {}
   };
   Ok(quote! {
       #(#heads),* <-- #delta_first_code #(#bodys),* #(,#new_id_decls)*;
   })
}

// desugar syntax sugar
fn desugar_question_paren_rule(rule: &SlogRule) -> SlogRule {
   let mut new_body = vec![];
   let mut new_heads = vec![];
   for head in &rule.heads {
      match head {
         SlogRuleHeadItem::SlogSExprClause(sexpr) => {
            let (new_bodys, new_head) = desugar_question_nested_sexpr(sexpr);
            new_body.extend(new_bodys);
            new_heads.push(SlogRuleHeadItem::SlogSExprClause(new_head));
         },
         // SlogRuleHeadItem::ExplicitIDClause(clause) => {
         //    let (new_bodys, new_head) = desugar_question_nested_sexpr(&clause.clause);
         //    new_body.extend(new_bodys);
         //    let new_clause = ExplicitIDClause { id_var: clause.id_var.clone(), clause: new_head };
         //    new_heads.push(SlogRuleHeadItem::ExplicitIDClause(new_clause));
         // }
         SlogRuleHeadItem::UnionClause(_clause) => {
            // union clause is already removed in pass 1
            new_heads.push(head.clone());
         },
      }
   }
   new_body.extend(rule.body.clone());
   SlogRule { heads: new_heads, body: new_body, delta_first: rule.delta_first.clone() }
}

fn desugar_question_nested_sexpr(sexpr: &SlogSExprClause) -> (Vec<SlogRuleBodyItem>, SlogSExprClause) {
   // recurse on args
   let mut new_res_body = vec![];
   let mut new_args = vec![];
   for arg in &sexpr.args {
      if let SlogClauseArg::SlogClause(sexpr) = arg {
         // if question paren, desugar head
         if let ParenType::QuestionParen = &sexpr.paren {
            let (new_body, new_rel_name_id) = desugar_question_head_sexpr(sexpr).unwrap();
            new_res_body.push(new_body);
            // panic!("desugar_question_nested_sexpr {:?}", sexpr);
            // TODO: add inflation code
            new_args.push(SlogClauseArg::LogicVar(new_rel_name_id, false));
         } else {
            // recurse on nested sexpr
            let (new_body, new_expr) = desugar_question_nested_sexpr(sexpr);
            new_res_body.extend(new_body);
            new_args.push(SlogClauseArg::SlogClause(Box::new(new_expr)));
         }
      } else {
         // add constant or logic var to new args
         new_args.push(arg.clone());
      }
   }
   (new_res_body, SlogSExprClause {
      paren: ParenType::Regular,
      rel_name: sexpr.rel_name.clone(),
      args: new_args,
      id_var: sexpr.id_var.clone(),
   })
}

fn desugar_question_head_sexpr(sexpr: &SlogSExprClause) -> Option<(SlogRuleBodyItem, Ident)> {
   if let ParenType::QuestionParen = &sexpr.paren {
      let new_rel_name_id = new_ident(&sexpr.rel_name.to_string());
      Some((
         SlogRuleBodyItem::SlogSExprClause(SlogSExprClause {
            paren: ParenType::Regular,
            rel_name: sexpr.rel_name.clone(),
            args: sexpr.args.clone(),
            id_var: Some(new_rel_name_id.clone()),
         }),
         new_rel_name_id,
      ))
   } else {
      None
   }
}

fn destruct_slog_line(line: &SlogProgramLine) -> SlogProgramLine {
   if let SlogProgramLine::Rule(rule) = line {
      let desugared_rule = desugar_question_paren_rule(rule);
      // panic!("desugared_rule {:?}", desugared_rule);
      let mut new_heads = vec![];
      for head in &desugared_rule.heads {
         let new_items = destruct_slog_rule_head_item(head);
         new_heads.extend(new_items);
      }
      let mut new_body = vec![];
      for body in &desugared_rule.body {
         let new_items = destruct_slog_body_item(body);
         // panic!("new_items {:?}", new_items);
         new_body.extend(new_items);
      }
      let desugared_rule = SlogRule { heads: new_heads, body: new_body, delta_first: rule.delta_first.clone() };
      let new_line = SlogProgramLine::Rule(desugared_rule);
      // panic!("new_line {:?}", new_line);
      new_line
   } else if let SlogProgramLine::Fact(clause) = line {
      // convert fact to rule
      let rule =
         SlogRule { heads: vec![SlogRuleHeadItem::SlogSExprClause(clause.clone())], body: vec![], delta_first: None };
      let fact_line = SlogProgramLine::Rule(rule);
      destruct_slog_line(&fact_line)
   } else {
      line.clone()
   }
}

fn destruct_slog_rule_head_item(item: &SlogRuleHeadItem) -> Vec<SlogRuleHeadItem> {
   let slog_expr = match item {
      SlogRuleHeadItem::SlogSExprClause(sexpr) => sexpr,
      // SlogRuleHeadItem::ExplicitIDClause(clause) => &clause.clause,
      SlogRuleHeadItem::UnionClause(_clause) => {
         // This control flow is weird, but it works
         return vec![item.clone()];
      },
   };
   // let exist_id_var = match item {
   //    SlogRuleHeadItem::SlogSExprClause(_) => None,
   //    SlogRuleHeadItem::ExplicitIDClause(clause) => Some(clause.id_var.clone()),
   //    SlogRuleHeadItem::UnionClause(_) => None,
   // };
   let mut new_items = vec![];
   let mut new_args = vec![];
   for arg in slog_expr.args.iter() {
      if let SlogClauseArg::SlogClause(nested_sexpr) = arg {
         let new_id_var = new_ident(&nested_sexpr.rel_name.to_string());
         let new_items_inner = destruct_slog_head_nested(nested_sexpr, &new_id_var);
         new_items.extend(new_items_inner);
         // TODO: add inflation code
         new_args.push(SlogClauseArg::LogicVar(new_id_var, false));
      } else {
         new_args.push(arg.clone());
      }
   }
   let id_var = match item {
      SlogRuleHeadItem::SlogSExprClause(_) => new_ident(&slog_expr.rel_name.to_string()),
      // SlogRuleHeadItem::ExplicitIDClause(clause) => clause.id_var.clone(),
      SlogRuleHeadItem::UnionClause(_) => todo!("union clause"),
   };
   let new_sexpr = SlogSExprClause {
      paren: slog_expr.paren.clone(),
      rel_name: slog_expr.rel_name.clone(),
      args: new_args,
      id_var: Some(id_var.clone()),
   };
   // let new_item = SlogRuleHeadItem::ExplicitIDClause(ExplicitIDClause { id_var, clause: new_sexpr });
   let new_item = SlogRuleHeadItem::SlogSExprClause(new_sexpr);
   new_items.push(new_item);
   new_items
}

fn destruct_slog_head_nested(sexpr: &SlogSExprClause, id_var: &Ident) -> Vec<SlogRuleHeadItem> {
   let mut new_items = vec![];
   let mut new_args = vec![];
   for arg in sexpr.args.iter() {
      if let SlogClauseArg::SlogClause(nested_sexpr) = arg {
         let new_id_var = new_ident(&nested_sexpr.rel_name.to_string());
         let new_items_inner = destruct_slog_head_nested(nested_sexpr, &new_id_var);
         new_items.extend(new_items_inner);
         new_args.push(SlogClauseArg::LogicVar(new_id_var, false));
      } else {
         new_args.push(arg.clone());
      }
   }

   let new_item = SlogRuleHeadItem::SlogSExprClause(SlogSExprClause {
      paren: sexpr.paren.clone(),
      rel_name: sexpr.rel_name.clone(),
      args: new_args,
      id_var: Some(id_var.clone()),
   });
   new_items.push(new_item);
   new_items
}

fn destruct_slog_body_item(item: &SlogRuleBodyItem) -> Vec<SlogRuleBodyItem> {
   // panic!("destruct_slog_body_item {:?}", item);
   let mut before = vec![];
   let mut after = vec![];
   if let Some(sexpr) = item.get_sexpr_clause() {
      // panic!("sexpr {:?}", sexpr);
      let mut new_args = vec![];
      for arg in &sexpr.args {
         if let SlogClauseArg::SlogClause(sexpr) = arg {
            let new_id_var = new_ident(&sexpr.rel_name.to_string());
            let (new_body_before, new_body_after) = deconstruct_nested_sexpr_body(sexpr, new_id_var.clone());
            before.extend(new_body_before);
            after.extend(new_body_after);
            // TODO: add inflation code
            new_args.push(SlogClauseArg::LogicVar(new_id_var, false));
         } else {
            new_args.push(arg.clone());
         }
      }
      // panic!("new_args {:?}", new_args);
      let deconstructed_sexpr = SlogSExprClause {
         paren: sexpr.paren.clone(),
         rel_name: sexpr.rel_name.clone(),
         args: new_args,
         id_var: sexpr.id_var.clone(),
      };
      let new_item = SlogRuleBodyItem::SlogSExprClause(deconstructed_sexpr);
      let new_items = before.into_iter().chain(vec![new_item]).chain(after);
      new_items.collect()
   } else {
      vec![item.clone()]
   }
}

fn deconstruct_nested_sexpr_body(
   sexpr: &SlogSExprClause, id_var: Ident,
) -> (Vec<SlogRuleBodyItem>, Vec<SlogRuleBodyItem>) {
   let mut new_body_before = vec![];
   let mut new_body_after = vec![];
   let mut new_args = vec![];
   for arg in &sexpr.args {
      if let SlogClauseArg::SlogClause(sexpr) = arg {
         let new_id_var = new_ident(&sexpr.rel_name.to_string());
         let (new_body_before_inner, new_body_after_inner) = deconstruct_nested_sexpr_body(sexpr, new_id_var.clone());
         new_body_before.extend(new_body_before_inner);
         new_body_after.extend(new_body_after_inner);
         // TODO: add inflation code
         new_args.push(SlogClauseArg::LogicVar(new_id_var, false));
      } else {
         new_args.push(arg.clone());
      }
   }
   let new_item = SlogRuleBodyItem::SlogSExprClause(SlogSExprClause {
      paren: sexpr.paren.clone(),
      rel_name: sexpr.rel_name.clone(),
      args: new_args,
      id_var: Some(id_var.clone()),
   });
   if let ParenType::QuestionParen = &sexpr.paren {
      new_body_before.push(new_item);
   } else {
      new_body_after.push(new_item);
   }
   (new_body_before, new_body_after)
}

pub fn destruct_slog_program(program: &SlogProgram) -> SlogProgram {
   let mut new_lines = vec![];
   for line in &program.lines {
      new_lines.push(destruct_slog_line(line));
   }
   SlogProgram { meta: program.meta.clone(), theory: program.theory.clone(), lines: new_lines }
}

fn remove_nested_union_clause(program: &SlogProgram) -> SlogProgram {
   let mut new_lines = vec![];
   for line in &program.lines {
      if let SlogProgramLine::Rule(rule) = line {
         let mut new_heads = vec![];
         let mut new_body = rule.body.clone();
         for head in &rule.heads {
            match head {
               SlogRuleHeadItem::UnionClause(clause) => {
                  let mut transform_union_arg =
                     |arg: &Either<Ident, SlogSExprClause>| -> Either<Ident, SlogSExprClause> {
                        match arg {
                           Either::Left(id) => Either::Left(id.clone()),
                           Either::Right(sexpr) => {
                              // nest the sexpr under a new random id
                              let new_id = new_ident(&format!("__rw_l"));
                              let new_sexpr = SlogSExprClause {
                                 paren: ParenType::Regular,
                                 rel_name: sexpr.rel_name.clone(),
                                 args: sexpr.args.clone(),
                                 id_var: Some(new_id.clone()),
                              };
                              if let ParenType::QuestionParen = &sexpr.paren {
                                 // add end of body
                                 new_body.push(SlogRuleBodyItem::SlogSExprClause(new_sexpr));
                              } else {
                                 // add to start of head
                                 new_heads.push(SlogRuleHeadItem::SlogSExprClause(new_sexpr));
                              }
                              Either::Left(new_id)
                           },
                        }
                     };
                  let new_lhs = transform_union_arg(&clause.clause_lhs);
                  let new_rhs = transform_union_arg(&clause.clause_rhs);
                  new_heads.push(SlogRuleHeadItem::UnionClause(SlogUnionClause {
                     _paren: clause._paren.clone(),
                     _union: clause._union.clone(),
                     clause_lhs: new_lhs,
                     clause_rhs: new_rhs,
                  }));
               },
               _ => new_heads.push(head.clone()),
            }
         }
         let new_rule = SlogRule { heads: new_heads, body: new_body, delta_first: rule.delta_first.clone() };
         new_lines.push(SlogProgramLine::Rule(new_rule));
      } else {
         new_lines.push(line.clone());
      }
   }
   SlogProgram { meta: program.meta.clone(), theory: program.theory.clone(), lines: new_lines }
}
