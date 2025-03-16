#![deny(warnings)]
extern crate proc_macro;
use proc_macro2::TokenStream;
use syn::{bracketed, ImplGenerics, LitInt, TypeGenerics};
use syn::{
   braced, parenthesized, parse2, punctuated::Punctuated, spanned::Spanned, Attribute, Error, Expr,
   Generics, Ident, Pat, Result, Token, Type, Visibility,
   WhereClause,
};
use syn::parse::{Parse, ParseStream};
use core::panic;

use quote::ToTokens;
use itertools::Itertools;
use derive_syn_parse::Parse;

use crate::utils::{expr_to_ident,  pat_to_ident};
use crate::syn_utils::pattern_get_vars;


// resources:
// https://blog.rust-lang.org/2018/12/21/Procedural-Macros-in-Rust-2018.html
// https://github.com/dtolnay/syn/blob/master/examples/lazy-static/lazy-static/src/lib.rs
// https://crates.io/crates/quote
// example: https://gitlab.gnome.org/federico/gnome-class/-/blob/master/src/parser/mod.rs

mod kw {
   syn::custom_keyword!(relation);
   syn::custom_keyword!(lattice);
   syn::custom_keyword!(function);
   syn::custom_keyword!(provenance);
   syn::custom_keyword!(index);
   syn::custom_keyword!(database);
   syn::custom_keyword!(arguement);
   syn::custom_keyword!(delta);
   syn::custom_keyword!(stratum);
   syn::custom_keyword!(ID);
   syn::custom_punctuation!(LongLeftArrow, <--);
   syn::custom_keyword!(agg);
   syn::custom_keyword!(ident);
   syn::custom_keyword!(expr);
   syn::custom_keyword!(va_list);
   syn::custom_punctuation!(ExistsBang, >?);
}

#[derive(Clone, Debug)]
pub(crate) struct Signatures {
   pub(crate) declaration: TypeSignature,
   pub(crate) implementation: Option<ImplSignature>,
}

impl Signatures {
   pub fn split_ty_generics_for_impl(&self) -> (ImplGenerics<'_>, TypeGenerics<'_>, Option<&'_ WhereClause>) {
      self.declaration.generics.split_for_impl()
   }

   pub fn split_impl_generics_for_impl(&self) -> (ImplGenerics<'_>, TypeGenerics<'_>, Option<&'_ WhereClause>) {
      let Some(signature) = &self.implementation else {
         return self.split_ty_generics_for_impl();
      };

      let (impl_generics, _, _) = signature.impl_generics.split_for_impl();
      let (_, ty_generics, where_clause) = signature.generics.split_for_impl();

      (impl_generics, ty_generics, where_clause)
   }
}

impl Parse for Signatures {
   fn parse(input: ParseStream) -> Result<Self> {
      let declaration = TypeSignature::parse(input)?;
      let implementation = if input.peek(Token![impl]) {
         Some(ImplSignature::parse(input)?)
      } else {
         None
      };
      Ok(Signatures { declaration, implementation })
   }
}

#[derive(Clone, Parse, Debug)]
pub struct TypeSignature {
   // We don't actually use the Parse impl to parse attrs.
   #[call(Attribute::parse_outer)]
   pub attrs: Vec<Attribute>,
   pub visibility: Visibility,
   pub _struct_kw: Token![struct],
   pub ident: Ident,
   #[call(parse_generics_with_where_clause)]
   pub generics: Generics,
   pub _semi: Token![;]
}

#[derive(Clone, Parse, Debug)]
pub struct ImplSignature {
   pub _impl_kw: Token![impl],
   pub impl_generics: Generics,
   pub ident: Ident,
   #[call(parse_generics_with_where_clause)]
   pub generics: Generics,
   pub _semi: Token![;]
}

/// Parse impl on Generics does not parse WhereClauses, hence this function
fn parse_generics_with_where_clause(input: ParseStream) -> Result<Generics> {
   let mut res = Generics::parse(input)?;
   if input.peek(Token![where]) {
      res.where_clause = Some(input.parse()?);
   }
   Ok(res)
}

#[derive(Clone, Parse)]
pub struct ExternDatabase {
   pub _extern_kw: Token![extern],
   pub _database: kw::database,
   pub mutable: Option<Token![mut]>,
   pub db_type: Ident,
   pub db_name: Ident,
   #[paren]
   pub _arg_pos_paren: syn::token::Paren,
   #[inside(_arg_pos_paren)]
   #[call(Punctuated::parse_terminated)]
   pub args: Punctuated<Expr, Token![,]>,
   pub _semi_colon: Token![;],
}

#[derive(Clone, Parse)]
pub struct ExternArgument {
   pub _extern_kw: Token![extern],
   pub _arguement: kw::arguement,
   pub _mutable: Option<Token![mut]>,
   pub arg_type: Type,
   pub arg_name: Ident,
   pub _semi_colon: Token![;],
}


#[derive(Clone, Parse)]
pub struct ExtraIndex {
   pub _index_kw: kw::index,
   pub rel_name: Ident,
   #[paren]
   pub _arg_pos_paren: syn::token::Paren,
   #[inside(_arg_pos_paren)]
   #[call(Punctuated::parse_terminated)]
   pub arg_pos: Punctuated<LitInt, Token![,]>,
   pub _semi_colon: Token![;],
}

#[derive(Clone, Parse)]
pub struct InStream {
   pub _input_kw: Token![await],
   pub rel_name: Ident,
   pub _semi_colon: Token![;],
}

#[derive(Clone, Parse)]
pub struct OutStream {
   pub _output_kw: Token![yield],
   pub rel_name: Ident,
   pub _semi_colon: Token![;],
}


#[derive(Clone, Parse)]
pub struct StratumPath {
   pub _stratum_kw: kw::stratum,
   pub hole_name: Ident,
   pub _kw_arrow: kw::LongLeftArrow,
   pub rel_name: Ident,
   pub _semi_colon: Token![;],
}

pub struct FunctionNode {
   pub attrs: Vec<Attribute>,
   pub name: Ident,
   pub field_types: Punctuated<Type, Token![,]>,
   pub _arrow: Token![->],
   pub return_type: Type,
   pub _semi_colon: Token![;],
}

impl Parse for FunctionNode {
   fn parse(input: ParseStream) -> Result<Self> {
      input.parse::<kw::function>()?;
      let attrs = Attribute::parse_outer(input)?;
      let name = input.parse()?;
      let content;
      parenthesized!(content in input);
      let field_types = content.parse_terminated(Type::parse, Token![,])?;
      let arrow = input.parse::<Token![->]>()?;
      let return_type = input.parse()?;
      let _semi_colon = input.parse::<Token![;]>()?;
      Ok(FunctionNode {
         attrs, name, field_types, _arrow: arrow, return_type, _semi_colon
      })
   }
}

// #[derive(Clone)]
pub struct RelationNode{
   pub attrs : Vec<Attribute>,
   pub name: Ident,
   pub field_types : Punctuated<Type, Token![,]>,
   pub initialization: Option<Expr>,
   pub source_db: Option<Ident>,
   pub _semi_colon: Token![;],
   pub is_lattice: bool,
   pub need_id: bool,
   pub is_hole: bool,
   pub is_input: bool,
   // pub is_function: bool,
}
impl Parse for RelationNode {
   fn parse(input: ParseStream) -> Result<Self> {
      let is_input = if input.peek(Token![extern]) {
         input.parse::<Token![extern]>()?;
         true
      } else {
         false
      };
      let is_lattice = input.peek(kw::lattice);
      let is_function = input.peek(kw::function);
      let need_delete = is_function;
      if is_lattice {
         input.parse::<kw::lattice>()?;
      } else if is_function {
         input.parse::<kw::function>()?;
      } else {
         input.parse::<kw::relation>()?;
      };
      let need_id = if input.peek(kw::ID) {
         input.parse::<kw::ID>()?;
         true
      } else {false};
      let name : Ident = input.parse()?;
      // check if name contains "_stratum"
      let is_hole = name.to_string().contains("_stratum");
      let content;
      parenthesized!(content in input);
      let field_types = content.parse_terminated(Type::parse, Token![,])?;
      let initialization = if input.peek(Token![=]) {
         input.parse::<Token![=]>()?;
         Some(input.parse::<Expr>()?)
      } else {None};

      let source_db : Option<Ident>;
      if input.peek(Token![in]) {
         input.parse::<Token![in]>()?;
         source_db = Some(input.parse()?);
      } else {
         source_db = None;
      };

      let _semi_colon = input.parse::<Token![;]>()?;
      if is_lattice && field_types.empty_or_trailing() {
         return Err(input.error("empty lattice is not allowed"));
      }
      Ok(RelationNode{
         attrs: vec![], name, field_types, source_db, _semi_colon, is_lattice,
         initialization, need_id, is_hole, is_input
         // is_function
      })
   }
}

#[derive(Parse, Clone)]
pub enum BodyItemNode {
   #[peek(Token![for], name = "generative clause")]
   Generator(GeneratorNode),
   #[peek(kw::agg, name = "aggregate clause")]
   Agg(AggClauseNode),
   #[peek(Token![do], name = "subquery")]
   SubQuery(SubQueryNode),
   #[peek_with(peek_macro_invocation, name = "macro invocation")]
   MacroInvocation(syn::ExprMacro),
   #[peek(Ident, name = "body clause")]
   Clause(BodyClauseNode),
   #[peek(Token![!], name = "negation clause")]
   Negation(NegationClauseNode),
   #[peek(syn::token::Paren, name = "disjunction node")]
   Disjunction(DisjunctionNode),
   #[peek_with(peek_if_or_let, name = "if condition or let binding")]
   Cond(CondClause),
   #[peek(Token![%], name = "function call")]
   FunctionCall(FunctionCallNode),
}

fn peek_macro_invocation(parse_stream: ParseStream) -> bool {
   parse_stream.peek(Ident) && parse_stream.peek2(Token![!])
}

fn peek_clause_head(parse_stream: ParseStream) -> bool {
   parse_stream.peek(Ident) || parse_stream.peek(Token![!]) ||
   parse_stream.peek(Token![let]) || parse_stream.peek(Token![~]) ||
   parse_stream.peek(kw::ExistsBang)
}
 
fn peek_if_or_let(parse_stream: ParseStream) -> bool {
   parse_stream.peek(Token![if]) || parse_stream.peek(Token![let])
}


#[derive(Clone, Parse)]
pub struct SubQueryInitArg {
   pub rel: Ident,
   pub _colon: Token![:],
   pub arg: Expr,
}

// syntax sugar subquery
#[derive(Clone)]
pub struct SubQueryNode {
   pub name: Ident,
   pub query_type : Ident,
   pub query_init : Punctuated<SubQueryInitArg, Token![,]>,
   pub query_extern_db: Punctuated<Expr, Token![,]>,
}

impl Parse for SubQueryNode {
   fn parse(input: ParseStream) -> Result<Self> {
      let _ = input.parse::<Token![do]>()?;
      let name = input.parse()?;
      let _ : Token![:] = input.parse()?;
      let query_type = input.parse()?;
      let query_init;
      // use curly braces to parse the subquery init
      braced!(query_init in input);
      let query_init = query_init.parse_terminated(SubQueryInitArg::parse, Token![,])?;
      let query_extern_db;
      parenthesized!(query_extern_db in input);
      let query_extern_db = query_extern_db.parse_terminated(Expr::parse, Token![,])?;
      Ok(SubQueryNode{name, query_type, query_init, query_extern_db})
   }
}

#[derive(Clone)]
pub struct DisjunctionNode {
   pub paren: syn::token::Paren,
   pub disjuncts: Punctuated<Punctuated<BodyItemNode, Token![,]>, Token![||]>,
}

impl Parse for DisjunctionNode {
   fn parse(input: ParseStream) -> Result<Self> {
      let content;
      let paren = parenthesized!(content in input);
      let res: Punctuated<Punctuated<BodyItemNode, Token![,]>, Token![||]> =
         Punctuated::<Punctuated<BodyItemNode, Token![,]>, Token![||]>::parse_terminated_with(&content, Punctuated::<BodyItemNode, Token![,]>::parse_separated_nonempty)?;
      Ok(DisjunctionNode{paren, disjuncts: res})
   }
}

#[derive(Clone)]
pub struct FunctionCallNode {
   pub _percent: Token![%],
   pub name: Ident,
   pub args: Punctuated<BodyClauseArg, Token![,]>,
   pub id_var: Option<Expr>,
   pub _get_kw: Token![->],
   pub return_var: Option<Ident>,
}

impl Parse for FunctionCallNode {
   fn parse(input: ParseStream) -> Result<Self> {
      let _percent = input.parse::<Token![%]>()?;
      let name = input.parse()?;
      let content;
      parenthesized!(content in input);
      let args = content.parse_terminated(BodyClauseArg::parse, Token![,])?;
      let id_var = if input.peek(Token![.]) {
         input.parse::<Token![.]>()?;
         Some(input.parse()?)
      } else {None};
      let _get = input.parse::<Token![->]>()?;
      let mut return_var = None;
      if input.peek(Token![?]) {
         input.parse::<Token![?]>()?;
      } else {
         return_var = input.parse()?;
      }  
      Ok(FunctionCallNode{
         _percent: _percent, name, args, id_var, _get_kw: _get, return_var
      })
   }
}

#[derive(Parse, Clone)]
pub struct GeneratorNode {
   pub for_keyword: Token![for],
   #[call(Pat::parse_multi)]
   pub pattern: Pat,
   pub _in_keyword: Token![in],
   pub expr: Expr
}

#[derive(Clone)]
pub struct BodyClauseNode {
   pub rel : Ident,
   pub extern_db_name: Option<Ident>,
   pub args : Punctuated<BodyClauseArg, Token![,]>,
   pub cond_clauses: Vec<CondClause>,
   pub id_var : Option<Expr>,
   pub delta_flag: bool,
}

#[derive(Parse, Clone, PartialEq, Eq, Debug)]
pub enum BodyClauseArg {
   #[peek(Token![?], name = "Pattern arg")]
   Pat(ClauseArgPattern),
   #[peek_with({ |_| true }, name = "Expression arg")]
   Expr(Expr),
}

impl BodyClauseArg {
   pub fn unwrap_expr(self) -> Expr {
      match self {
         Self::Expr(exp) => exp,
         Self::Pat(_) => panic!("unwrap_expr(): BodyClauseArg is not an expr")
      }
   }

   pub fn unwrap_expr_ref(&self) -> &Expr {
      match self {
         Self::Expr(exp) => exp,
         Self::Pat(_) => panic!("unwrap_expr(): BodyClauseArg is not an expr")
      }
   }

   pub fn get_vars(&self) -> Vec<Ident> {
      match self {
         BodyClauseArg::Pat(p) => pattern_get_vars(&p.pattern),
         BodyClauseArg::Expr(e) => expr_to_ident(e).into_iter().collect(),
      }
   }
}
impl ToTokens for BodyClauseArg {
   fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
      match self{
         BodyClauseArg::Pat(pat) => {
            pat.huh_token.to_tokens(tokens);
            pat.pattern.to_tokens(tokens);
         },
         BodyClauseArg::Expr(exp) => exp.to_tokens(tokens),
      }
   }
}

#[derive(Parse, Clone, PartialEq, Eq, Debug)]
pub struct ClauseArgPattern {
   pub huh_token: Token![?],
   #[call(Pat::parse_multi)]
   pub pattern : Pat,
}

#[derive(Parse, Clone, PartialEq, Eq, Hash, Debug)]
pub struct IfLetClause {
   pub if_keyword: Token![if],
   pub let_keyword: Token![let],
   #[call(Pat::parse_multi)]
   pub pattern: Pat,
   pub eq_symbol : Token![=],
   pub exp: syn::Expr,
}

#[derive(Parse, Clone, PartialEq, Eq, Hash, Debug)]
pub struct IfClause {
   pub if_keyword: Token![if],
   pub cond: Expr 
}

#[derive(Parse, Clone, PartialEq, Eq, Hash, Debug)]
pub struct LetClause {
   pub let_keyword: Token![let],
   #[call(Pat::parse_multi)]
   pub pattern: Pat,
   pub eq_symbol : Token![=],
   pub exp: syn::Expr,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum CondClause {
   IfLet(IfLetClause),
   If(IfClause),
   Let(LetClause),
}

impl CondClause {
   pub fn bound_vars(&self) -> Vec<Ident> {
      match self {
        CondClause::IfLet(cl) => pattern_get_vars(&cl.pattern),
        CondClause::If(_) => vec![],
        CondClause::Let(cl) => pattern_get_vars(&cl.pattern),
      }
   }

   /// returns the expression associated with the CondClause. 
   /// Useful for determining clause dependencies
   pub fn expr(&self) -> &Expr {
         match self {
         CondClause::IfLet(cl) => &cl.exp,
         CondClause::If(cl) => &cl.cond,
         CondClause::Let(cl) => &cl.exp,
      }
   }
}
impl Parse for CondClause {
   fn parse(input: ParseStream) -> Result<Self> {
      if input.peek(Token![if]) {
         if input.peek2(Token![let]) {
            let cl: IfLetClause = input.parse()?;
            Ok(Self::IfLet(cl))
         } else {
            let cl: IfClause = input.parse()?;
            Ok(Self::If(cl))
         }
      } else if input.peek(Token![let]) {
         let cl: LetClause = input.parse()?;
         Ok(Self::Let(cl))
      } else {
         Err(input.error("expected either if clause or if let clause"))
      }
   }
}

// impl ToTokens for BodyClauseNode {
//    fn to_tokens(&self, tokens: &mut quote::__private::TokenStream) {
//       self.rel.to_tokens(tokens);
//       self.args.to_tokens(tokens);
//    }
// }

impl Parse for BodyClauseNode{
   fn parse(input: ParseStream) -> Result<Self> {
      let delta_flag = input.peek(kw::delta);
      if delta_flag {
         input.parse::<kw::delta>()?;
      }
      // let rel : Ident = input.parse()?;
      let rel : Ident;
      let extern_db_name;
      if input.peek2(Token![.]) {
         let db_name = input.parse::<Ident>()?;
         input.parse::<Token![.]>()?;
         rel = input.parse::<Ident>()?;
         extern_db_name = Some(db_name);
      } else {
         rel = input.parse::<Ident>()?;
         extern_db_name = None;
      }
      let args_content;
      parenthesized!(args_content in input);
      let args = args_content.parse_terminated(BodyClauseArg::parse, Token![,])?;
      let mut id_var = None;
      if input.peek(Token![.]) {
         input.parse::<Token![.]>()?;
         id_var = input.parse().ok();
      }
      let mut cond_clauses = vec![];
      while let Ok(cl) = input.parse(){
         cond_clauses.push(cl);
      }
      Ok(BodyClauseNode{rel, extern_db_name, args, cond_clauses, id_var, delta_flag})
   }
}

#[derive(Parse, Clone)]
pub struct NegationClauseNode {
   pub neg_token: Token![!],
   pub rel : Ident,
   #[paren]
   pub _rel_arg_paren: syn::token::Paren,
   #[inside(_rel_arg_paren)]
   #[call(Punctuated::parse_terminated)]
   pub args : Punctuated<Expr, Token![,]>,
}


#[derive(Clone, Parse)]
pub enum HeadItemNode {
   #[peek_with(peek_macro_invocation, name = "macro invocation")]
   MacroInvocation(syn::ExprMacro),
   #[peek(Token![%], name = "function return")]
   HeadFuctionReturn(FunctionCallNode),
   #[peek_with(peek_clause_head, name = "head clause")]
   HeadClause(HeadClauseNode),
}

impl HeadItemNode {
   pub fn clause(&self) -> &HeadClauseNode {
      match self {
         HeadItemNode::HeadClause(cl) => cl,
         HeadItemNode::HeadFuctionReturn(_) => panic!("unexpected function return"),
         HeadItemNode::MacroInvocation(_) => panic!("unexpected macro invocation"),
      }
   }
}

#[derive(Clone)]
pub struct HeadClauseNode {
   pub rel : Ident,
   pub extern_db_name: Option<Ident>,
   pub args : Punctuated<Expr, Token![,]>,
   pub required_flag: bool,
   pub id_name: Option<Ident>,
   pub delete_flag: bool,
   pub exists_var: Option<Ident>
}
impl ToTokens for HeadClauseNode {
   fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
      self.rel.to_tokens(tokens);
      self.args.to_tokens(tokens);
   }
}

impl Parse for HeadClauseNode{
   fn parse(input: ParseStream) -> Result<Self> {
      let mut required_flag  = false; 
      let mut delete_flag = false;
      // check if first token is let
      let mut id_name = None;
      let mut exists_var = None;
      if input.peek(Token![let]) {
         input.parse::<Token![let]>()?;
         if !input.peek(Ident) {
            return Err(input.error("expected identifier after let"));
         }
         id_name = Some(input.parse()?);
         required_flag = true;
         // consume a "=", if not exists throw parsing error
         if !input.peek(Token![=]) {
            return Err(input.error("expected '=' after let"));
         }
         input.parse::<Token![=]>()?;
      }
      if input.peek(kw::ExistsBang) {
         input.parse::<kw::ExistsBang>()?;
         exists_var = Some(input.parse()?);
         input.parse::<Token![.]>()?;
      }
      // check if first token is !
      if input.peek(Token![!]) {
         required_flag = true;
         input.parse::<Token![!]>()?;
      }
      if input.peek(Token![~]) {
         delete_flag = true;
         input.parse::<Token![~]>()?;
      }
      let mut rel : Ident = input.parse()?;
      let mut extern_db_name = None;
      if input.peek(Token![.]) {
         extern_db_name = Some(rel);
         input.parse::<Token![.]>()?;
         rel = input.parse::<Ident>()?;
         // return Err(input.error("unexpected '.'"));
      }
      let args_content;
      parenthesized!(args_content in input);
      let args = args_content.parse_terminated(Expr::parse, Token![,])?;
      Ok(HeadClauseNode{rel, extern_db_name, args, required_flag, id_name, delete_flag, exists_var})
   }
}

#[derive(Clone, Parse)]
pub struct AggClauseNode {
   pub agg_kw: kw::agg,
   #[call(Pat::parse_multi)]
   pub pat: Pat,
   pub _eq_token: Token![=],
   pub aggregator: AggregatorNode,
   #[paren]
   pub _agg_arg_paren: syn::token::Paren,
   #[inside(_agg_arg_paren)]
   #[call(Punctuated::parse_terminated)]
   pub bound_args: Punctuated<Ident, Token![,]>,
   pub _in_kw: Token![in],
   pub rel : Ident,
   pub _dot: Option<Token![.]>,
   pub extern_db_name: Option<Ident>,
   #[paren]
   _rel_arg_paren: syn::token::Paren,
   #[inside(_rel_arg_paren)]
   #[call(Punctuated::parse_terminated)]
   pub rel_args : Punctuated<Expr, Token![,]>
}

#[derive(Clone)]
pub enum AggregatorNode {
   Path(syn::Path),
   Expr(Expr)
}

impl Parse for AggregatorNode {
   fn parse(input: ParseStream) -> Result<Self> {
      if input.peek(syn::token::Paren) {
         let inside_parens;
         parenthesized!(inside_parens in input);
         Ok(AggregatorNode::Expr(inside_parens.parse()?))
      } else {
         Ok(AggregatorNode::Path(input.parse()?))
      }
   }
}
impl AggregatorNode {
   pub fn get_expr(&self) -> Expr {
      match self {
         AggregatorNode::Path(path) => parse2(quote!{#path}).unwrap(),
         AggregatorNode::Expr(expr) => expr.clone(),
      }
   }
}

pub struct RuleNode {
   pub head_clauses: Punctuated<HeadItemNode, Token![,]>,
   pub body_items: Vec<BodyItemNode>// Punctuated<BodyItemNode, Token![,]>,
}

impl Parse for RuleNode {
   fn parse(input: ParseStream) -> Result<Self> {
      let head_clauses = if input.peek(syn::token::Brace) {
         let content;
         braced!(content in input);
         Punctuated::<HeadItemNode, Token![,]>::parse_terminated(&content)?
      } else {
         Punctuated::<HeadItemNode, Token![,]>::parse_separated_nonempty(input)?
      };

      if input.peek(Token![;]){
         // println!("fact rule!!!");
         input.parse::<Token![;]>()?;
         Ok(RuleNode{head_clauses, body_items: vec![] /*Punctuated::default()*/})
      } else {
         input.parse::<Token![<]>()?;
         input.parse::<Token![-]>()?;
         input.parse::<Token![-]>()?;
         // NOTE this does not work with quote!
         // input.parse::<kw::LongLeftArrow>()?;

         let body_items = Punctuated::<BodyItemNode, Token![,]>::parse_separated_nonempty(input)?;
         input.parse::<Token![;]>()?;
         Ok(RuleNode{ head_clauses, body_items: body_items.into_iter().collect()})
      }
   }
}

// TODO maybe remove?
#[allow(dead_code)]
pub(crate) fn rule_node_summary(rule: &RuleNode) -> String {
   fn bitem_to_str(bitem: &BodyItemNode) -> String {
      match bitem {
         BodyItemNode::Generator(gen) => format!("for_{}", pat_to_ident(&gen.pattern).map(|x| x.to_string()).unwrap_or_default()),
         BodyItemNode::Clause(bcl) => format!("{}", bcl.rel),
         BodyItemNode::Disjunction(_) => todo!(),
         BodyItemNode::Cond(_cl) => format!("if_"),
         BodyItemNode::Agg(agg) => format!("agg {}", agg.rel),
         BodyItemNode::Negation(neg) => format!("! {}", neg.rel),
         BodyItemNode::MacroInvocation(m) => format!("{:?}!(..)", m.mac.path),
         BodyItemNode::FunctionCall(f) => format!("%{} -> {:?}", f.name, f.return_var),
         BodyItemNode::SubQuery(sub_query_node) => format!("subquery {} ...", sub_query_node.name),
      }
   }
   fn hitem_to_str(hitem: &HeadItemNode) -> String {
      match hitem {
         HeadItemNode::MacroInvocation(m) => format!("{:?}!(..)", m.mac.path),
         HeadItemNode::HeadFuctionReturn(f) => format!("%{} -> {:?}", f.name, f.return_var),
         HeadItemNode::HeadClause(cl) => cl.rel.to_string(),
      }
   }
   format!("{} <-- {}",
            rule.head_clauses.iter().map(hitem_to_str).join(", "),
            rule.body_items.iter().map(bitem_to_str).join(", "))
}

#[derive(Parse)]
pub struct MacroDefParam {
   _dollar: Token![$],
   pub name: Ident,
   _colon: Token![:],
   pub kind: MacroParamKind
}

#[derive(Parse)]
#[allow(unused)]
pub enum MacroParamKind {
   #[peek(kw::ident, name = "ident")]
   Expr(Ident),
   #[peek(kw::expr, name = "expr")]
   Ident(Ident),
   #[peek(kw::va_list, name = "va_list")]
   VaList(Ident),
}

#[derive(Parse)]
pub struct MacroDefNode {
   pub _mac: Token![macro],
   pub name: Ident,
   #[paren]
   pub _arg_paren: syn::token::Paren,
   #[inside(_arg_paren)]
   #[call(Punctuated::parse_terminated)]
   pub params: Punctuated<MacroDefParam, Token![,]>,
   #[brace]
   pub _body_brace: syn::token::Brace,
   #[inside(_body_brace)]
   pub body: TokenStream,
}

// #[derive(Clone)]
pub(crate) struct AscentProgram {
   pub rules : Vec<RuleNode>,
   pub relations : Vec<RelationNode>,
   pub signatures: Option<Signatures>,
   pub attributes: Vec<syn::Attribute>,
   pub macros: Vec<MacroDefNode>,
   pub macro_invocs: Vec<syn::ExprMacro>,
   pub functions: Vec<FunctionNode>,
   pub extern_dbs: Vec<ExternDatabase>,
   pub extern_args: Vec<ExternArgument>,
   pub extra_indices: Vec<ExtraIndex>,
   pub stratum_paths: Vec<StratumPath>,
   pub in_streams: Vec<InStream>,
   pub out_streams: Vec<OutStream>,
}

impl Parse for AscentProgram {
   fn parse(input: ParseStream) -> Result<Self> {
      let attributes = Attribute::parse_inner(input)?;
      let mut struct_attrs = Attribute::parse_outer(input)?;
      let signatures = if input.peek(Token![pub]) || input.peek(Token![struct]) {
         let mut signatures = Signatures::parse(input)?;
         signatures.declaration.attrs = std::mem::take(&mut struct_attrs);
         Some(signatures)
      } else {
         None
      };
      let mut rules = vec![];
      let mut relations = vec![];
      let mut extern_dbs = vec![];
      let mut extern_args = vec![];
      let mut extra_indices = vec![];
      let mut functions = vec![];
      let mut macros = vec![];
      let mut macro_invocs = vec![];
      let mut stratum_paths = vec![];
      let mut in_streams = vec![];
      let mut out_streams = vec![];
      while !input.is_empty() {
         let attrs = if !struct_attrs.is_empty() {std::mem::take(&mut struct_attrs)} else {Attribute::parse_outer(input)?};
         if input.peek(kw::relation) || input.peek(kw::lattice){
            let mut relation_node = RelationNode::parse(input)?;
            relation_node.attrs = attrs;
            relations.push(relation_node);
         } else if input.peek(Token![extern]) {
            if input.peek2(kw::database) {
               let extern_db = ExternDatabase::parse(input)?;
               extern_dbs.push(extern_db);
            } else if input.peek2(kw::arguement) {
               let extern_arg = ExternArgument::parse(input)?;
               extern_args.push(extern_arg);
            } else {
               let mut relation_node = RelationNode::parse(input)?;
               relation_node.attrs = attrs;
               relation_node.is_input = true;
               relations.push(relation_node);
            }
         } else if input.peek(Token![await]) {
            let in_stream = InStream::parse(input)?;
            in_streams.push(in_stream);
         } else if input.peek(Token![yield]) {
            let out_stream = OutStream::parse(input)?;
            out_streams.push(out_stream);
         } else if input.peek(kw::stratum) {
            let stratum_path = StratumPath::parse(input)?;
            stratum_paths.push(stratum_path);
         } else if input.peek(kw::index) {
            let extra_index = ExtraIndex::parse(input)?;
            extra_indices.push(extra_index);
         } else if input.peek(kw::function) {
            let mut function_node = FunctionNode::parse(input)?;
            function_node.attrs = attrs;
            functions.push(function_node);
         } else if input.peek(Token![macro]) {
            if !attrs.is_empty() {
               return Err(Error::new(attrs[0].span(), "unexpected attribute(s)"));
            }
            macros.push(MacroDefNode::parse(input)?);
         } else if input.peek(Token![@]) {
            input.parse::<Token![@]>()?;
            let expr_macro = input.parse()?;
            macro_invocs.push(expr_macro);
            input.parse::<Token![;]>()?;
         } else {
            if !attrs.is_empty() {
               return Err(Error::new(attrs[0].span(), "unexpected attribute(s)"));
            }
            rules.push(RuleNode::parse(input)?);
         }
      }
      Ok(AscentProgram{
         rules, relations, signatures, attributes, macros, macro_invocs,
         functions, extern_dbs, extern_args, extra_indices, stratum_paths,
         in_streams, out_streams
      })
   }
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub(crate) struct RelationIdentity {
   pub name: Ident,
   pub field_types: Vec<Type>,
   pub extern_db_name: Option<Ident>,
   pub is_lattice: bool,
   pub need_id: bool,
   pub is_hole: bool,
   pub is_input: bool,
}

impl From<&RelationNode> for RelationIdentity{
   fn from(relation_node: &RelationNode) -> Self {
      RelationIdentity {
         name: relation_node.name.clone(),
         field_types: relation_node.field_types.iter().cloned().collect(),
         extern_db_name: relation_node.source_db.clone(),
         is_lattice: relation_node.is_lattice,
         need_id: relation_node.need_id,
         is_hole: relation_node.is_hole,
         is_input: relation_node.is_input,
      }
   }
} 

#[derive(Clone)]
pub(crate) struct DsAttributeContents {
   pub path: syn::Path,
   pub args: TokenStream,
}

impl Parse for DsAttributeContents {
   fn parse(input: ParseStream) -> Result<Self> {
      let content = input;
      // parenthesized!(content in input);

      let path = syn::Path::parse_mod_style(&content)?;
      let args = if content.peek(Token![:]) {
         content.parse::<Token![:]>()?;
         TokenStream::parse(&content)?
      } else {
         TokenStream::default()
      };

      Ok(Self { path, args })
   }
}


#[derive(Clone, Debug)]
pub(crate) struct AscentCall {
   // parened path list
   pub used_paths: Punctuated<syn::ExprPath, Token![,]>,
   pub ascent_macro: Ident,
   pub code: TokenStream,
}

impl Parse for AscentCall {
   fn parse(input: ParseStream) -> Result<Self> {
      let ascent_macro = input.parse()?;
      // parse the bang
      let _ = input.parse::<Token![!]>()?;
      let content;
      braced!(content in input);
      let used_paths;
      if content.peek(Token![use]) {
         content.parse::<Token![use]>()?;
         let pt;
         bracketed!(pt in content);
         used_paths = pt.parse_terminated(syn::ExprPath::parse, Token![,])?;   
         content.parse::<Token![;]>()?; 
      } else {
         used_paths = Punctuated::default();
      }
      
      let code = content.parse()?;
      Ok(AscentCall { used_paths, ascent_macro, code })
   }
}

