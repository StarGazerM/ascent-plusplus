// syntax.rs
// define the syntax of slog
// slog mostly use sexprs

use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{Ident, Token, braced, bracketed, parenthesized};

// keywords
mod kw_slog {
   syn::custom_punctuation!(LongLeftArrow, ==>);
   syn::custom_punctuation!(LongRightArrow, <==);
   syn::custom_keyword!(or);
   syn::custom_keyword!(sexpr);
   syn::custom_keyword!(define);
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BangParen {
   pub bang: Token![!],
   pub paren: syn::token::Paren,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct QuestionParen {
   pub question: Token![?],
   pub paren: syn::token::Paren,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ParenType {
   Regular,
   Curly,
   BangParen,
   QuestionParen,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SlogSExprClause {
   pub paren: ParenType,
   pub rel_name: Ident,
   pub args: Vec<SlogClauseArg>,
   pub id_var: Option<Ident>,
}

impl Parse for SlogSExprClause {
   fn parse(input: ParseStream) -> syn::Result<Self> {
      let (content, paren, id_var) = if input.peek(Token![!]) {
         let content;
         let _bang = input.parse::<Token![!]>()?;
         let _paren = parenthesized!(content in input);
         (content, ParenType::BangParen, None)
      } else if input.peek(Token![?]) {
         let content;
         let _question = input.parse::<Token![?]>()?;
         let _paren = parenthesized!(content in input);
         (content, ParenType::QuestionParen, None)
      } else if input.peek(syn::token::Paren) {
         let content;
         let _paren = parenthesized!(content in input);
         if content.peek(Token![=]) {
            let _eq = content.parse::<Token![=]>()?;
            let id_var = content.parse::<Ident>()?;
            let content_inner;
            let _paren2 = parenthesized!(content_inner in content);
            (content_inner, ParenType::Regular, Some(id_var))
         } else {
            (content, ParenType::Regular, None)
         }
      } else if input.peek(syn::token::Brace) {
         let content;
         let _brace = braced!(content in input);
         (content, ParenType::Curly, None)
      } else {
         return Err(input.error("expected regular parentheses or curly braces"));
      };

      let rel_name = content.parse::<Ident>()?;

      let mut args = Vec::new();
      while !content.is_empty() {
         args.push(content.parse::<SlogClauseArg>()?);
      }

      Ok(SlogSExprClause { paren, rel_name, args, id_var })
   }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SlogClauseArg {
   LogicVar(Ident),
   Constant(syn::Lit),
   Wildcard,
   RustExpr(syn::Expr),
   SlogClause(Box<SlogSExprClause>),
}

impl Parse for SlogClauseArg {
   fn parse(input: ParseStream) -> syn::Result<Self> {
      // a escaped rust expr use quasi quote `,(...)`
      if input.peek(syn::token::Comma) {
         let _ = input.parse::<syn::token::Comma>()?;
         let content;
         parenthesized!(content in input);
         let expr = content.parse::<syn::Expr>()?;
         Ok(SlogClauseArg::RustExpr(expr))
      } else if input.peek(syn::token::Paren) || input.peek(syn::token::Brace) {
         let clause = input.parse::<SlogSExprClause>()?;
         Ok(SlogClauseArg::SlogClause(Box::new(clause)))
      } else if input.peek(syn::Lit) {
         let lit = input.parse::<syn::Lit>()?;
         Ok(SlogClauseArg::Constant(lit))
      } else if input.peek(Token![_]) {
         let _ = input.parse::<Token![_]>()?;
         Ok(SlogClauseArg::Wildcard)
      } else {
         let logic_var = input.parse::<Ident>()?;
         Ok(SlogClauseArg::LogicVar(logic_var))
      }
   }
}

// #[derive(Debug, Clone, PartialEq, Eq, Hash)]
// pub struct Disjunction {
//     pub _or: syn::token::Or,
//     pub clauses: Vec<Vec<SlogSExprClause>>,
// }

// impl Parse for Disjunction {
//     fn parse(input: ParseStream) -> syn::Result<Self> {

//     }
// }

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExplicitIDClause {
   pub id_var: Ident,
   pub clause: SlogSExprClause,
}
impl Parse for ExplicitIDClause {
   fn parse(content: ParseStream) -> syn::Result<Self> {
      let clause = content.parse::<SlogSExprClause>()?;
      let _eq = content.parse::<Token![.]>()?;
      let id_var = content.parse::<Ident>()?;
      Ok(ExplicitIDClause {id_var, clause })
   }
}

fn peek_explicit_id_clause(input: ParseStream) -> bool { input.peek(syn::token::Paren) && input.peek2(syn::token::Eq) }

#[derive(Debug, Clone)]
pub enum SlogRuleBodyItem {
   SlogSExprClause(SlogSExprClause),
   NegatedSlogSExprClause(SlogSExprClause),
   ExplicitIDClause(ExplicitIDClause),
   AscentClause(TokenStream),
}

impl SlogRuleBodyItem {
   pub fn get_sexpr_clause(&self) -> Option<&SlogSExprClause> {
      match self {
         SlogRuleBodyItem::SlogSExprClause(clause) => Some(clause),
         SlogRuleBodyItem::NegatedSlogSExprClause(clause) => Some(clause),
         _ => None,
      }
   }
}

impl Parse for SlogRuleBodyItem {
   fn parse(input: ParseStream) -> syn::Result<Self> {
      if input.peek(syn::token::Paren) && input.peek2(Token![.]) {  
         let id_clause = input.parse::<ExplicitIDClause>()?;
         Ok(SlogRuleBodyItem::ExplicitIDClause(id_clause))
      } else if input.peek(Token![~]) && input.peek(syn::token::Paren) {
         let _ = input.parse::<Token![~]>()?;
         let clause = input.parse::<SlogSExprClause>()?;
         Ok(SlogRuleBodyItem::NegatedSlogSExprClause(clause))
      } else if input.peek(syn::token::Paren) {
         // todo!("wwww {} {:?}", input.to_string(), input.peek3(syn::token::Paren));
         let clause = input.parse::<SlogSExprClause>()?;
         Ok(SlogRuleBodyItem::SlogSExprClause(clause))
      } else if input.peek(syn::token::Brace) {
         let content;
         let _ = braced!(content in input);
         Ok(SlogRuleBodyItem::AscentClause(content.parse()?))
      } else {
         Err(input.error(format!("expected slog s-expr clause or explicit id clause:\n{}", input.to_string())))
      }
   }
}

#[derive(Debug, Clone)]
pub enum SlogRuleHeadItem {
   ExplicitIDClause(ExplicitIDClause),
   SlogSExprClause(SlogSExprClause),
}

impl Parse for SlogRuleHeadItem {
   fn parse(input: ParseStream) -> syn::Result<Self> {
      if input.peek(syn::token::Paren) && input.peek2(syn::token::Eq) {
         let id_clause = input.parse::<ExplicitIDClause>()?;
         Ok(SlogRuleHeadItem::ExplicitIDClause(id_clause))
      } else if input.peek(syn::token::Paren) {
         let clause = input.parse::<SlogSExprClause>()?;
         Ok(SlogRuleHeadItem::SlogSExprClause(clause))
      } else {
         Err(input.error(format!("head : expected slog s-expr clause or explicit id clause:\n{}", input.to_string())))
      }
   }
}

impl SlogRuleHeadItem {
   fn get_sexpr_clause(&self) -> Option<&SlogSExprClause> {
      match self {
         SlogRuleHeadItem::SlogSExprClause(clause) => Some(clause),
         _ => None,
      }
   }
}

#[derive(Debug, Clone)]
pub struct SlogRule {
   pub heads: Vec<SlogRuleHeadItem>,
   pub body: Vec<SlogRuleBodyItem>,
}

impl Parse for SlogRule {
   fn parse(input: ParseStream) -> syn::Result<Self> {
      let content;
      let _ = bracketed!(content in input);
      if has_left_arrow(&content) {
         let mut heads = Vec::new();
         while !content.is_empty() && !content.peek(Token![<-]) && !content.peek(Token![->]) {
            heads.push(content.parse::<SlogRuleHeadItem>()?);
         }
         let _arrow = content.parse::<Token![<-]>()?;
         let mut body = Vec::new();
         while !content.is_empty() && !content.peek(Token![<-]) && !content.peek(Token![->]) {
            body.push(content.parse::<SlogRuleBodyItem>()?);
         }
         Ok(SlogRule { heads, body })
      } else {
         // right arrow, reverse the order of the body and heads
         let mut body = Vec::new();
         while !content.is_empty() && !content.peek(Token![<-]) && !content.peek(Token![->]) {
            body.push(content.parse::<SlogRuleBodyItem>()?);
         }
         let _arrow = content.parse::<Token![->]>()?;
         let mut heads = Vec::new();
         while !content.is_empty() && !content.peek(Token![<-]) && !content.peek(Token![->]) {
            heads.push(content.parse::<SlogRuleHeadItem>()?);
         }
         Ok(SlogRule { heads, body })
      }
   }
}

fn has_left_arrow(input: ParseStream) -> bool {
   let input_str = input.to_string();
   // check if the input contains "<--"
   input_str.contains("<-")
}

#[derive(Debug, Clone)]
pub struct SlogRelationDecl {
   pub _relation: kw_slog::define,
   pub rel_name: Ident,
   pub arg_types: Vec<syn::Type>,
}

impl Parse for SlogRelationDecl {
   fn parse(input: ParseStream) -> syn::Result<Self> {
      let content;
      let _ = parenthesized!(content in input);
      // let id = content.parse::<Ident>()?;
      let _relation = content.parse::<kw_slog::define>()?;
      let rel_name = content.parse::<Ident>()?;
      let mut arg_types = Vec::new();
      while !content.is_empty() {
         if content.peek(kw_slog::sexpr) {
            content.parse::<kw_slog::sexpr>()?;
            arg_types.push(syn::parse2(quote! { usize })?);
         } else {
            arg_types.push(content.parse::<syn::Type>()?);
         }
      }
      Ok(SlogRelationDecl { _relation, rel_name, arg_types })
   }
}

#[derive(Debug, Clone)]
pub enum SlogProgramLine {
   RelationDecl(SlogRelationDecl),
   Rule(SlogRule),
   Fact(SlogSExprClause),
}

impl Parse for SlogProgramLine {
   fn parse(input: ParseStream) -> syn::Result<Self> {
      if input.peek(syn::token::Paren) {
         // get the second token
         // let content;
         // let _ = parenthesized!(content in input);
         // let _relation = content.parse::<kw_slog::relation>()?;
         // todo!("relation not found");
         // check if the first token is a relation
         // let relation_decl = content.parse::<SlogRelationDecl>()?;
         let relation_decl = input.parse::<SlogRelationDecl>()?;
         Ok(SlogProgramLine::RelationDecl(relation_decl))
      } else if input.peek(syn::token::Bracket) {
         let rule = input.parse::<SlogRule>()?;
         Ok(SlogProgramLine::Rule(rule))
      } else {
         unimplemented!("fact parsing");
         // let fact = input.parse::<SlogSExprClause>()?;
         // Ok(SlogProgramLine::Fact(fact))
      }
   }
}

#[derive(Debug, Clone)]
pub struct SlogMeta {
   pub _paren: syn::token::Paren,
   // pub vis: syn::Visibility,
   pub _struct: Token![struct],
   pub struct_name: Ident,
}

impl Parse for SlogMeta {
   fn parse(input: ParseStream) -> syn::Result<Self> {
      let content;
      let _paren = parenthesized!(content in input);
      // let vis = content.parse::<syn::Visibility>()?;
      let _struct = content.parse::<Token![struct]>()?;
      let struct_name = content.parse::<Ident>()?;
      Ok(SlogMeta { _paren, _struct, struct_name })
   }
}

#[derive(Debug, Clone)]
pub struct SlogProgram {
   pub meta: SlogMeta,
   pub lines: Vec<SlogProgramLine>,
}

impl Parse for SlogProgram {
   fn parse(input: ParseStream) -> syn::Result<Self> {
      // search to check if <-- is present
      let meta = input.parse::<SlogMeta>()?;
      let mut lines = Vec::new();
      while !input.is_empty() {
         lines.push(input.parse::<SlogProgramLine>()?);
      }
      Ok(SlogProgram { meta, lines })
   }
}

