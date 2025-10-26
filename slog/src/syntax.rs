// syntax.rs
// define the syntax of slog
// slog mostly use sexprs

use itertools::Either;
use proc_macro2::TokenStream;
use quote::{ToTokens, quote, quote_spanned};
use syn::parse::{Parse, ParseStream};
use syn::{Expr, Ident, Path, Token, Type, braced, bracketed, parenthesized};

// keywords
pub mod kw_slog {
   syn::custom_punctuation!(LongLeftArrow, ==>);
   syn::custom_punctuation!(LongRightArrow, <==);
   syn::custom_punctuation!(ExistsBang, >?);
   syn::custom_keyword!(or);
   syn::custom_keyword!(sexpr);
   syn::custom_keyword!(eclass);
   syn::custom_keyword!(define);
   syn::custom_keyword!(rewrite);
   syn::custom_keyword!(union);
   syn::custom_keyword!(Δ);
   syn::custom_keyword!(theory);
   syn::custom_keyword!(provenance);

   // for fun
   syn::custom_keyword!(若);
   syn::custom_keyword!(则);
   syn::custom_keyword!(且);
   syn::custom_keyword!(亦);
   syn::custom_keyword!(是矣);
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
   pub canonical_id_huh: usize,
}

impl Parse for SlogSExprClause {
   fn parse(input: ParseStream) -> syn::Result<Self> {
      let (content, paren, id_var, canonical_id_huh) = if input.peek(Token![!]) {
         let content;
         let _bang = input.parse::<Token![!]>()?;
         let _paren = parenthesized!(content in input);
         (content, ParenType::BangParen, None, 0)
      } else if input.peek(syn::token::Paren) || input.peek(Token![?]) {
         let parent_type = if input.peek(Token![?]) {
            let _question = input.parse::<Token![?]>()?;
            ParenType::QuestionParen
         } else {
            ParenType::Regular
         };
         let content;
         let _paren = parenthesized!(content in input);
         if content.peek(Token![=]) {
            let _eq = content.parse::<Token![=]>()?;
            let canonical_id_huh = content.peek(Token![@]);
            if canonical_id_huh {
               let _ = content.parse::<Token![@]>()?;
            }
            let id_var = content.parse::<Ident>()?;
            let content_inner;
            let _paren2 = parenthesized!(content_inner in content);
            (content_inner, parent_type, Some(id_var), if canonical_id_huh { 1 } else { 0 })
         } else {
            (content, parent_type, None, 0)
         }
      } else if input.peek(syn::token::Brace) {
         let content;
         let _brace = braced!(content in input);
         (content, ParenType::Curly, None, 0)
      } else {
         return Err(input.error("expected regular parentheses or curly braces"));
      };

      let rel_name = content.parse::<Ident>()?;

      let mut args = Vec::new();
      while !content.is_empty() {
         args.push(content.parse::<SlogClauseArg>()?);
      }

      Ok(SlogSExprClause { paren, rel_name, args, id_var, canonical_id_huh })
   }
}

fn is_slog_paren(input: &ParseStream) -> bool {
   input.peek(syn::token::Paren) || input.peek(syn::token::Brace) || input.peek(Token![!]) || input.peek(Token![?])
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SlogUnionClause {
   pub _paren: syn::token::Paren,
   pub _union: kw_slog::union,
   pub clause_lhs: Either<Ident, SlogSExprClause>,
   pub clause_rhs: Either<Ident, SlogSExprClause>,
}

impl Parse for SlogUnionClause {
   fn parse(input: ParseStream) -> syn::Result<Self> {
      // remove the paren
      let content;
      let _paren = parenthesized!(content in input);
      let _union = content.parse::<kw_slog::union>()?;
      let clause_lhs = if content.peek(syn::Ident) {
         Either::Left(content.parse::<Ident>()?)
      } else {
         Either::Right(content.parse::<SlogSExprClause>()?)
      };
      // let _arrow = content.parse::<Token![=>]>()?;
      let clause_rhs = if content.peek(syn::Ident) {
         Either::Left(content.parse::<Ident>()?)
      } else {
         Either::Right(content.parse::<SlogSExprClause>()?)
      };
      Ok(SlogUnionClause { _paren, _union, clause_lhs, clause_rhs })
   }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SlogClauseArg {
   LogicVar(Ident, bool),
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
      } else if is_slog_paren(&input) {
         let clause = input.parse::<SlogSExprClause>()?;
         Ok(SlogClauseArg::SlogClause(Box::new(clause)))
      } else if input.peek(syn::Lit) {
         let lit = input.parse::<syn::Lit>()?;
         Ok(SlogClauseArg::Constant(lit))
      } else if input.peek(Token![_]) {
         let _ = input.parse::<Token![_]>()?;
         Ok(SlogClauseArg::Wildcard)
      } else {
         let need_inflation = input.peek(Token![@]);
         if input.peek(Token![@]) {
            let _ = input.parse::<Token![@]>()?;
         }
         let logic_var = input.parse::<Ident>()?;
         Ok(SlogClauseArg::LogicVar(logic_var, need_inflation))
      }
   }
}

#[derive(Debug, Clone)]
pub enum SlogRuleBodyItem {
   SlogSExprClause(SlogSExprClause),
   NegatedSlogSExprClause(SlogSExprClause),
   AscentClause(TokenStream),
}

impl SlogRuleBodyItem {
   pub fn get_sexpr_clause(&self) -> Option<SlogSExprClause> {
      match self {
         SlogRuleBodyItem::SlogSExprClause(clause) => Some(clause.clone()),
         SlogRuleBodyItem::NegatedSlogSExprClause(clause) => Some(clause.clone()),
         SlogRuleBodyItem::AscentClause(_) => None,
      }
   }
}

impl Parse for SlogRuleBodyItem {
   fn parse(input: ParseStream) -> syn::Result<Self> {
      if input.peek(Token![~]) && input.peek(syn::token::Paren) {
         let _ = input.parse::<Token![~]>()?;
         let clause = input.parse::<SlogSExprClause>()?;
         Ok(SlogRuleBodyItem::NegatedSlogSExprClause(clause))
      } else if input.peek(syn::token::Paren) {
         let clause = input.parse::<SlogSExprClause>()?;
         Ok(SlogRuleBodyItem::SlogSExprClause(clause))
      } else if input.peek(Token![,]) {
         let _ = input.parse::<Token![,]>()?;
         let content;
         let _ = parenthesized!(content in input);
         Ok(SlogRuleBodyItem::AscentClause(content.parse()?))
      } else {
         Err(input.error(format!("body:expected slog s-expr clause or explicit id clause:\n{}", input.to_string())))
      }
   }
}

#[derive(Debug, Clone)]
pub enum SlogRuleHeadItem {
   SlogSExprClause(SlogSExprClause),
   UnionClause(SlogUnionClause),
   AscentClause(TokenStream),
}

impl Parse for SlogRuleHeadItem {
   fn parse(input: ParseStream) -> syn::Result<Self> {
      if input.peek(syn::token::Paren) {
         // continue to parse by fork and peek inside paren
         let input_fork = input.fork();
         let content;
         let _ = parenthesized!(content in input_fork);
         if content.peek(kw_slog::union) {
            let union_clause = input.parse::<SlogUnionClause>()?;
            Ok(SlogRuleHeadItem::UnionClause(union_clause))
         } else {
            let clause = input.parse::<SlogSExprClause>()?;
            Ok(SlogRuleHeadItem::SlogSExprClause(clause))
         }
      } else if input.peek(Token![,]) {
         let _ = input.parse::<Token![,]>()?;
         let content;
         let _ = parenthesized!(content in input);
         Ok(SlogRuleHeadItem::AscentClause(content.parse()?))
      } else {
         Err(input.error(format!("head : expected slog s-expr clause or explicit id clause:\n{}", input.to_string())))
      }
   }
}

#[derive(Debug, Clone)]
pub struct SlogRule {
   pub heads: Vec<SlogRuleHeadItem>,
   pub body: Vec<SlogRuleBodyItem>,
   pub delta_first: Option<kw_slog::Δ>,
   pub theory_unification: bool,
}

impl Parse for SlogRule {
   fn parse(input: ParseStream) -> syn::Result<Self> {
      let content;
      let _ = bracketed!(content in input);
      if has_left_arrow(&content) {
         let mut heads = Vec::new();
         while !content.is_empty() && !content.peek(Token![<-]) && !content.peek(Token![-]) {
            heads.push(content.parse::<SlogRuleHeadItem>()?);
         }
         let _arrow = content.parse::<Token![<-]>()?;
         let _ = content.parse::<Token![-]>()?;
         let delta_first = if content.peek(kw_slog::Δ) { Some(content.parse::<kw_slog::Δ>()?) } else { None };
         let mut body = Vec::new();
         while !content.is_empty() && !content.peek(Token![<-]) && !content.peek(Token![-]) {
            body.push(content.parse::<SlogRuleBodyItem>()?);
         }
         // TODO: come up with better syntax to let user manually control theory unification
         Ok(SlogRule { heads, body, delta_first, theory_unification: true })
      } else {
         // right arrow, reverse the order of the body and heads
         let mut body = Vec::new();
         while !content.is_empty() && !content.peek(Token![<-]) && !content.peek(Token![-]) {
            body.push(content.parse::<SlogRuleBodyItem>()?);
         }
         let delta_first = if content.peek(kw_slog::Δ) { Some(content.parse::<kw_slog::Δ>()?) } else { None };
         let _arrow = content.parse::<Token![-]>()?;
         let _ = content.parse::<Token![->]>()?;
         let mut heads = Vec::new();
         while !content.is_empty() && !content.peek(Token![<-]) && !content.peek(Token![-]) {
            heads.push(content.parse::<SlogRuleHeadItem>()?);
         }
         Ok(SlogRule { heads, body, delta_first, theory_unification: true })
      }
   }
}

fn has_left_arrow(input: ParseStream) -> bool {
   let input_str = input.to_string();
   // check if the input contains "<--"
   input_str.contains("<-")
}

#[derive(Debug, Clone)]
pub enum SlogType {
   Theory(Ident),
   Rust(syn::Type),
}

impl Parse for SlogType {
   fn parse(input: ParseStream) -> syn::Result<Self> {
      if input.peek(Token![@]) {
         let _ = input.parse::<Token![@]>()?;
         let id = input.parse::<Ident>()?;
         Ok(SlogType::Theory(id))
      } else {
         let ty = input.parse::<syn::Type>()?;
         Ok(SlogType::Rust(ty))
      }
   }
}

#[derive(Debug, Clone)]
pub struct SlogRelationDecl {
   pub _relation: kw_slog::define,
   pub rel_name: Ident,
   pub ds: Option<Path>,
   pub arg_types: Vec<SlogType>,
   pub id_type: syn::Type,
}

impl Parse for SlogRelationDecl {
   fn parse(input: ParseStream) -> syn::Result<Self> {
      let content;
      let _ = parenthesized!(content in input);
      // let id = content.parse::<Ident>()?;
      let _relation = content.parse::<kw_slog::define>()?;
      let rel_name = content.parse::<Ident>()?;
      let ds = if content.peek(Token![:]) {
         let _ = content.parse::<Token![:]>()?;
         Some(content.parse::<Path>()?)
      } else {
         None
      };
      let id_type = if content.peek(Token![#]) {
         let _ = content.parse::<Token![#]>()?;
         content.parse::<syn::Type>()?
      } else {
         // default to usize
         syn::parse2(quote! { usize })?
      };
      let mut arg_types = Vec::new();
      while !content.is_empty() {
         arg_types.push(content.parse::<SlogType>()?);
      }
      Ok(SlogRelationDecl { _relation, rel_name, ds, arg_types, id_type })
   }
}

#[derive(Debug, Clone)]
pub enum SlogProgramLine {
   RelationDecl(SlogRelationDecl),
   Rule(SlogRule),
   Ascent(TokenStream),
   Fact(SlogSExprClause),
}

impl Parse for SlogProgramLine {
   fn parse(input: ParseStream) -> syn::Result<Self> {
      if input.peek(syn::token::Paren) {
         // get the second token
         let input_fork = input.fork();
         let content;
         let _ = parenthesized!(content in input_fork);
         if content.peek(kw_slog::define) {
            let relation_decl = input.parse::<SlogRelationDecl>()?;
            Ok(SlogProgramLine::RelationDecl(relation_decl))
         } else if content.peek(kw_slog::rewrite) {
            // rewrite is syntax sugar for union with 2 joined body clauses
            let content_inner;
            let _ = parenthesized!(content_inner in input);
            let _rewrite = content_inner.parse::<kw_slog::rewrite>()?;
            let _ = content_inner.parse::<Token![!]>()?;
            let mut body = vec![];
            let mut lhs_sexpr = content_inner.parse::<SlogSExprClause>()?;
            let mut rhs_sexpr = content_inner.parse::<SlogSExprClause>()?;
            lhs_sexpr.id_var = Some(Ident::new("lhs_id", _rewrite.span));
            rhs_sexpr.id_var = Some(Ident::new("rhs_id", _rewrite.span));
            let head = SlogRuleHeadItem::UnionClause(SlogUnionClause {
               _paren: syn::token::Paren::default(),
               _union: kw_slog::union(_rewrite.span),
               clause_lhs: Either::Left(lhs_sexpr.id_var.clone().unwrap()),
               clause_rhs: Either::Left(rhs_sexpr.id_var.clone().unwrap()),
            });
            body.push(SlogRuleBodyItem::SlogSExprClause(lhs_sexpr));
            body.push(SlogRuleBodyItem::SlogSExprClause(rhs_sexpr));
            Ok(SlogProgramLine::Rule(SlogRule {
               heads: vec![head],
               body,
               delta_first: Some(kw_slog::Δ(_rewrite.span)),
               theory_unification: true,
            }))
         } else {
            // parse the fact
            let fact = input.parse::<SlogSExprClause>()?;
            Ok(SlogProgramLine::Fact(fact))
         }
      } else if input.peek(syn::token::Bracket) {
         let rule = input.parse::<SlogRule>()?;
         Ok(SlogProgramLine::Rule(rule))
      } else if input.peek(Token![,]) {
         let _ = input.parse::<Token![,]>()?;
         let content;
         let _ = parenthesized!(content in input);
         Ok(SlogProgramLine::Ascent(content.parse()?))
      } else {
         // fact
         input.parse::<Token![#]>()?;
         let fact = input.parse::<SlogSExprClause>()?;
         Ok(SlogProgramLine::Fact(fact))
      }
   }
}

#[derive(Debug, Clone)]
pub struct SlogTheoryDecl {
   pub name: Ident,
   pub ty: syn::Type,
   pub unify_rel: Ident,
   pub ds: Path,
   pub provenance: Option<kw_slog::provenance>,
   pub opt_args: Vec<Expr>, // for hygenic macros
}

impl Parse for SlogTheoryDecl {
   fn parse(input: ParseStream) -> syn::Result<Self> {
      let content;
      let _ = parenthesized!(content in input);
      let name = content.parse::<Ident>()?;
      let ty = content.parse::<syn::Type>()?;
      let unify_rel = content.parse::<Ident>()?;
      let ds = content.parse::<Path>()?;
      let provenance =
         if content.peek(kw_slog::provenance) { Some(content.parse::<kw_slog::provenance>()?) } else { None };
      let mut opt_args = Vec::new();
      while !content.is_empty() {
         opt_args.push(content.parse::<Expr>()?);
      }
      Ok(SlogTheoryDecl { name, ty, unify_rel, ds, provenance, opt_args })
   }
}

impl ToTokens for SlogTheoryDecl {
   fn to_tokens(&self, tokens: &mut TokenStream) {
      let name = self.name.clone();
      let ty = self.ty.clone();
      let unify_rel = self.unify_rel.clone();
      let ds = self.ds.clone();
      let provenance = self.provenance.clone();
      let opt_args = self.opt_args.clone();
      let code = quote_spanned! {name.span() =>
         (#name #ty #unify_rel #ds #provenance #(#opt_args)*)
      };
      tokens.extend(code);
   }
}

#[derive(Debug, Clone)]
pub struct SlogTheory {
   pub _theory: kw_slog::theory,
   pub names: Vec<Ident>,
   pub uses: Vec<SlogTheoryDecl>,
}

impl ToTokens for SlogTheory {
   fn to_tokens(&self, tokens: &mut TokenStream) {
      let uses = self.uses.clone();
      let _theory = self._theory.clone();
      let code = quote_spanned! {_theory.span =>
         (#_theory #(#uses)*)
      };
      tokens.extend(code);
   }
}

impl SlogTheory {
   pub fn get_unify_rel_by_name(&self, th_name: &Ident) -> Option<Ident> {
      for u in self.uses.iter() {
         if u.name == *th_name {
            return Some(u.unify_rel.clone());
         }
      }
      None
   }

   pub fn get_theory_decl_by_name(&self, th_name: &Ident) -> Option<SlogTheoryDecl> {
      for u in self.uses.iter() {
         if u.name == *th_name {
            return Some(u.clone());
         }
      }
      None
   }

   pub fn get_theory_decl_by_rel_name(&self, rel_name: &Ident) -> Option<SlogTheoryDecl> {
      for u in self.uses.iter() {
         if u.unify_rel == *rel_name {
            return Some(u.clone());
         }
      }
      None
   }

   pub fn get_theory_type_by_name(&self, th_name: &Ident) -> Option<Type> {
      for u in self.uses.iter() {
         if &u.name == th_name {
            return Some(u.ty.clone());
         }
      }
      None
   }
}

impl Parse for SlogTheory {
   fn parse(input: ParseStream) -> syn::Result<Self> {
      let content;
      let _ = parenthesized!(content in input);
      let _theory = content.parse::<kw_slog::theory>()?;
      let mut uses = Vec::new();
      while !content.is_empty() {
         uses.push(content.parse::<SlogTheoryDecl>()?);
      }
      let names = uses.iter().map(|u| u.name.clone()).collect();
      Ok(SlogTheory { _theory, names, uses })
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
   pub theory: SlogTheory,
   pub lines: Vec<SlogProgramLine>,
}

impl Parse for SlogProgram {
   fn parse(input: ParseStream) -> syn::Result<Self> {
      // search to check if <-- is present
      let meta = input.parse::<SlogMeta>()?;
      // fork the input, and peek see if it is a theory
      let input_fork = input.fork();
      let forked_content;
      let _ = parenthesized!(forked_content in input_fork);
      let theory = if forked_content.peek(kw_slog::theory) {
         input.parse::<SlogTheory>()?
      } else {
         SlogTheory { _theory: kw_slog::theory(meta._struct.span), names: Vec::new(), uses: Vec::new() }
      };

      let mut lines = Vec::new();
      while !input.is_empty() {
         lines.push(input.parse::<SlogProgramLine>()?);
      }
      Ok(SlogProgram { meta, theory, lines })
   }
}

pub struct ShareDbInput {
   pub db_name: Ident,
   pub theory: SlogTheory,
   pub content: Vec<SlogRelationDecl>,
   pub kont_macro: Path,
   pub shared_idb: Option<TokenStream>,
}

impl syn::parse::Parse for ShareDbInput {
   fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
      let db_name = input.parse::<Ident>()?;
      let _ = input.parse::<syn::Token![,]>()?;
      // peek if its paren
      let theory = if input.peek(syn::token::Paren) {
         let theory = input.parse::<SlogTheory>()?;
         let _ = input.parse::<syn::Token![,]>()?;
         theory
      } else {
         SlogTheory { _theory: kw_slog::theory(db_name.span()), names: vec![], uses: vec![] }
      };
      let content;
      let _braced = braced!(content in input);
      // let content = content.parse_terminated(SlogRelationDecl::parse, syn::Token![,])?;
      let mut decls = vec![];
      while !content.is_empty() {
         decls.push(content.parse::<SlogRelationDecl>()?);
      }
      let _ = input.parse::<syn::Token![,]>()?;
      let shared_idb = if input.peek(syn::token::Brace) {
         let content;
         let _ = braced!(content in input);
         let idb = content.parse::<TokenStream>()?;
         let _ = input.parse::<Token![,]>()?;
         Some(idb)
      } else {
         None
      };
      let kont_macro = input.parse::<Path>()?;
      Ok(ShareDbInput { db_name, theory, content: decls, kont_macro, shared_idb })
   }
}

#[derive(Debug, Clone)]
pub struct SlogGenInput {
   pub struct_name: Ident,
   pub prev_code: TokenStream,
   pub new_code: TokenStream,
   pub slog_macro: Ident,
}

impl syn::parse::Parse for SlogGenInput {
   fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
      let struct_name = input.parse::<Ident>()?;
      let _ = input.parse::<syn::Token![,]>()?;
      let prev_content;
      let _ = braced!(prev_content in input);
      let prev_code = prev_content.parse::<TokenStream>()?;
      let _ = input.parse::<syn::Token![,]>()?;
      let new_content;
      let _ = braced!(new_content in input);
      let new_code = new_content.parse::<TokenStream>()?;
      let _ = input.parse::<syn::Token![,]>()?;
      let slog_macro = input.parse::<Ident>()?;
      Ok(SlogGenInput { struct_name, prev_code, new_code, slog_macro })
   }
}

#[derive(Debug, Clone)]
pub struct SlogSourceInput {
   pub struct_name: Ident,
   pub opt_args: Vec<Ident>,
   pub content: TokenStream,
}

impl syn::parse::Parse for SlogSourceInput {
   fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
      let struct_name = input.parse::<Ident>()?;
      let opt_content;
      let _ = parenthesized!(opt_content in input);
      let opt_args = opt_content.parse_terminated(Ident::parse, syn::Token![,])?;
      let _ = input.parse::<syn::Token![:]>()?;
      let content = input.parse::<TokenStream>()?;
      Ok(SlogSourceInput { struct_name, opt_args: opt_args.into_iter().collect(), content })
   }
}
