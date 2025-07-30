// compile slog program to ascent program

use crate::{
   syntax::{
      ExplicitIDClause, ParenType, SlogClauseArg, SlogMeta, SlogProgram, SlogProgramLine, SlogRelationDecl, SlogRule,
      SlogRuleBodyItem, SlogRuleHeadItem, SlogSExprClause,
   },
   util::new_ident,
};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, Result};

fn compile_slog_meta(meta: &SlogMeta) -> Result<TokenStream> {
   // let vis = meta.vis.clone();
   let vis = quote! { pub };
   let struct_name = meta.struct_name.clone();
   Ok(quote! {
      #vis struct #struct_name;
   })
}

fn compile_slog_relation_decl(decl: &SlogRelationDecl) -> Result<TokenStream> {
   let rel_name = decl.rel_name.clone();
   let arg_types = decl.arg_types.clone();
   Ok(quote! {
      relation ID #rel_name(#(#arg_types),*);
   })
}

fn compile_slog_clause_unstructured(clause: &SlogSExprClause) -> Result<TokenStream> {
   let rel_name = clause.rel_name.clone();
   let args = clause.args.clone();
   let args_tokens = args
      .iter()
      .map(|arg| match arg {
         // TODO: add inflation code
         SlogClauseArg::LogicVar(id, _) => Some(quote! { #id }),
         SlogClauseArg::SlogClause(_) => None,
         SlogClauseArg::Constant(constant) => Some(quote! { #constant }),
         SlogClauseArg::Wildcard => Some(quote! { _ }),
         SlogClauseArg::RustExpr(expr) => Some(quote! { #expr }),
      })
      .collect::<Vec<_>>();
   // if any is none, return error
   if args_tokens.iter().any(|arg| arg.is_none()) {
      return Err(syn::Error::new_spanned(
         clause.rel_name.clone(),
         "unstructured clause must be all logic vars or all constants",
      ));
   }
   let id_tag_code = if clause.id_var.is_some() {
      let id_var = clause.id_var.clone().unwrap();
      quote! {
         .#id_var
      }
   } else {
      quote! {}
   };
   let bang_tag_code = if clause.paren == ParenType::BangParen {
      quote! {
         !
      }
   } else {
      quote! {}
   };
   Ok(quote! {
       #rel_name(#(#args_tokens),*)#id_tag_code #bang_tag_code
   })
}

fn compile_slog_line(line: &SlogProgramLine) -> Result<TokenStream> {
   match line {
      SlogProgramLine::Rule(rule) => compile_slog_rule_unstructured(rule),
      SlogProgramLine::RelationDecl(decl) => compile_slog_relation_decl(decl),
      SlogProgramLine::Fact(clause) => {
         // desugar to a rule with empty body rule
         let rule = SlogRule {
            heads: vec![SlogRuleHeadItem::SlogSExprClause(clause.clone())],
            body: vec![],
         };
         compile_slog_rule_unstructured(&rule)
      },
   }
}

pub fn compile_slog_program(program: &SlogProgram, is_parallel: bool) -> Result<TokenStream> {
   let meta = compile_slog_meta(&program.meta)?;
   let lines = program.lines.iter().map(|line| compile_slog_line(line)).collect::<Result<Vec<_>>>()?;
   let lines = quote! {
      #(#lines)*
   };
   let slog_mode = if !is_parallel {
      quote! {
         ascent!
      }
   } else {
      quote! {
         ascent_par!
      }
   };
   Ok(quote! {
      #slog_mode {
         // #![egglog_mode]
         #meta
         #lines
      }
   })
}

pub(crate) fn compile(tokens: TokenStream, is_parallel: bool) -> Result<TokenStream> {
   let program: SlogProgram = syn::parse2(tokens)?;
   let destructed_program = destruct_slog_program(&program);

   let compiled_program = compile_slog_program(&destructed_program, is_parallel)?;
   Ok(compiled_program)
}

fn compile_slog_rule_unstructured(rule: &SlogRule) -> Result<TokenStream> {
   let heads = rule
      .heads
      .iter()
      .map(|head| match head {
         SlogRuleHeadItem::SlogSExprClause(sexpr) => compile_slog_clause_unstructured(sexpr),
         SlogRuleHeadItem::ExplicitIDClause(clause) => {
            let id_var = clause.id_var.clone();
            let compiled_clause = compile_slog_clause_unstructured(&clause.clause);
            compiled_clause.map(|clause| {
               quote! {
                  let #id_var = #clause
               }
            })
         }
      })
      .collect::<Result<Vec<_>>>()?;
   let mut bodys = rule
      .body
      .iter()
      .map(|body| match body {
         SlogRuleBodyItem::SlogSExprClause(sexpr) => compile_slog_clause_unstructured(sexpr),
         SlogRuleBodyItem::NegatedSlogSExprClause(sexpr) => {
            let compiled_clause = compile_slog_clause_unstructured(sexpr);
            compiled_clause.map(|clause| {
               quote! {
                  !#clause
               }
            })
         }
         SlogRuleBodyItem::ExplicitIDClause(clause) => {
            let id_var = clause.id_var.clone();
            let compiled_clause = compile_slog_clause_unstructured(&clause.clause);
            compiled_clause.map(|clause| {
               quote! {
                  #clause.#id_var
               }
            })
         }
         SlogRuleBodyItem::AscentClause(content) => {
            let content = content.clone();
            Ok(quote! {
               #content
            })
         }
      })
      .collect::<Result<Vec<_>>>()?;
   // if body is empty, add a dummy body ()
   if bodys.is_empty() {
      bodys.push(quote! { () });
   }

   Ok(quote! {
       #(#heads),* <-- #(#bodys),*;
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
         }
         SlogRuleHeadItem::ExplicitIDClause(clause) => {
            let (new_bodys, new_head) = desugar_question_nested_sexpr(&clause.clause);
            new_body.extend(new_bodys);
            let new_clause = ExplicitIDClause { id_var: clause.id_var.clone(), clause: new_head };
            new_heads.push(SlogRuleHeadItem::ExplicitIDClause(new_clause));
         }
      }
   }
   new_body.extend(rule.body.clone());
   SlogRule { heads: new_heads, body: new_body }
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
      paren: sexpr.paren.clone(),
      rel_name: sexpr.rel_name.clone(),
      args: new_args,
      id_var: sexpr.id_var.clone(),
   })
}

fn desugar_question_head_sexpr(sexpr: &SlogSExprClause) -> Option<(SlogRuleBodyItem, Ident)> {
   if let ParenType::QuestionParen = &sexpr.paren {
      let new_rel_name_id = new_ident(&sexpr.rel_name.to_string());
      Some((
         SlogRuleBodyItem::ExplicitIDClause(ExplicitIDClause {
            id_var: new_rel_name_id.clone(),
            clause: sexpr.clone(),
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
      let mut new_heads = vec![];
      for head in &desugared_rule.heads {
         let new_items = destruct_slog_rule_head_item(head);
         new_heads.extend(new_items);
      }
      let mut new_body = vec![];
      for body in &desugared_rule.body {
         let new_items = destruct_slog_body_item(body);
         new_body.extend(new_items);
      }
      let desugared_rule = SlogRule { heads: new_heads, body: new_body };
      SlogProgramLine::Rule(desugared_rule)
   } else if let SlogProgramLine::Fact(clause) = line {
      // convert fact to rule
      let rule = SlogRule {
         heads: vec![SlogRuleHeadItem::SlogSExprClause(clause.clone())],
         body: vec![],
      };
      let fact_line = SlogProgramLine::Rule(rule);
      destruct_slog_line(&fact_line)
   } else {
      line.clone()
   }
}

fn destruct_slog_rule_head_item(item: &SlogRuleHeadItem) -> Vec<SlogRuleHeadItem> {
   let slog_expr = match item {
      SlogRuleHeadItem::SlogSExprClause(sexpr) => sexpr,
      SlogRuleHeadItem::ExplicitIDClause(clause) => &clause.clause,
   };
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
   let new_sexpr = SlogSExprClause {
      paren: slog_expr.paren.clone(),
      rel_name: slog_expr.rel_name.clone(),
      args: new_args,
      id_var: slog_expr.id_var.clone(),
   };
   let id_var = match item {
      SlogRuleHeadItem::SlogSExprClause(_) => new_ident(&slog_expr.rel_name.to_string()),
      SlogRuleHeadItem::ExplicitIDClause(clause) => clause.id_var.clone(),
   };
   let new_item = SlogRuleHeadItem::ExplicitIDClause(ExplicitIDClause { id_var, clause: new_sexpr });
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
      } else {
         new_args.push(arg.clone());
      }
   }
   let new_item = SlogRuleHeadItem::ExplicitIDClause(ExplicitIDClause {
      id_var: id_var.clone(),
      clause: SlogSExprClause {
         paren: sexpr.paren.clone(),
         rel_name: sexpr.rel_name.clone(),
         args: new_args,
         id_var: sexpr.id_var.clone(),
      },
   });
   new_items.push(new_item);
   new_items
}

fn destruct_slog_body_item(item: &SlogRuleBodyItem) -> Vec<SlogRuleBodyItem> {
   let mut before = vec![];
   let mut after = vec![];
   if let Some(sexpr) = item.get_sexpr_clause() {
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
      let deconstructed_sexpr = SlogSExprClause {
         paren: sexpr.paren.clone(),
         rel_name: sexpr.rel_name.clone(),
         args: new_args,
         id_var: sexpr.id_var.clone(),
      };
      let new_items =
         before.into_iter().chain(vec![SlogRuleBodyItem::SlogSExprClause(deconstructed_sexpr)]).chain(after);
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
   let new_item = SlogRuleBodyItem::ExplicitIDClause(ExplicitIDClause {
      id_var,
      clause: SlogSExprClause {
         paren: sexpr.paren.clone(),
         rel_name: sexpr.rel_name.clone(),
         args: new_args,
         id_var: sexpr.id_var.clone(),
      },
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
   SlogProgram { meta: program.meta.clone(), lines: new_lines }
}
