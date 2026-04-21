//! MIR + HIR + syntax-level DATA TYPES.
//!
//! These were previously defined across three files in `ascent_macro`. They
//! live here now so any codegen backend (in a separate crate) can consume
//! them without depending on the proc-macro crate.
//!
//! Types only — parsing/desugar/HIR-build/HIR→MIR-lowering/codegen logic
//! stays in `ascent_macro`.

#![allow(clippy::module_inception)]

use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use derive_syn_parse::Parse;
use itertools::Itertools;
use proc_macro2::{Ident, Span, TokenStream};
use syn::parse::{Parse as SynParse, ParseStream};
use syn::{
   Attribute, Error, Expr, Generics, ImplGenerics, Pat, Path, Result, Token, Type, TypeGenerics, Visibility, WhereClause,
   parse2, parse_quote,
};

use crate::utils::{expr_to_ident, pattern_get_vars, tuple_type};

// ============================================================================
// Syntax-layer shared types
// ============================================================================

/// Interned identity of a relation: name + field types + lattice flag.
/// Used everywhere as the stable identifier for a relation.
#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct RelationIdentity {
   pub name: Ident,
   pub field_types: Vec<Type>,
   pub is_lattice: bool,
}

/// `ds(path : args)` attribute contents for selecting a data-structure
/// provider. `args` is opaque token stream given verbatim to the provider.
#[derive(Clone)]
pub struct DsAttributeContents {
   pub path: Path,
   pub args: TokenStream,
}

impl SynParse for DsAttributeContents {
   fn parse(input: ParseStream) -> Result<Self> {
      let path = Path::parse_mod_style(input)?;
      let args = if input.peek(Token![:]) {
         input.parse::<Token![:]>()?;
         TokenStream::parse(input)?
      } else {
         TokenStream::default()
      };
      Ok(Self { path, args })
   }
}

#[derive(Clone, Debug)]
pub struct Signatures {
   pub declaration: TypeSignature,
   pub implementation: Option<ImplSignature>,
}

impl Signatures {
   pub fn split_ty_generics_for_impl(&self) -> (ImplGenerics<'_>, TypeGenerics<'_>, Option<&'_ WhereClause>) {
      self.declaration.generics.split_for_impl()
   }

   pub fn split_impl_generics_for_impl(&self) -> (ImplGenerics<'_>, TypeGenerics<'_>, Option<&'_ WhereClause>) {
      let Some(signature) = &self.implementation else { return self.split_ty_generics_for_impl(); };
      let (impl_generics, _, _) = signature.impl_generics.split_for_impl();
      let (_, ty_generics, where_clause) = signature.generics.split_for_impl();
      (impl_generics, ty_generics, where_clause)
   }
}

impl SynParse for Signatures {
   fn parse(input: ParseStream) -> Result<Self> {
      let declaration = TypeSignature::parse(input)?;
      let implementation = if input.peek(Token![impl]) { Some(ImplSignature::parse(input)?) } else { None };
      Ok(Signatures { declaration, implementation })
   }
}

#[derive(Clone, Parse, Debug)]
pub struct TypeSignature {
   #[call(Attribute::parse_outer)]
   pub attrs: Vec<Attribute>,
   pub visibility: Visibility,
   pub _struct_kw: Token![struct],
   pub ident: Ident,
   #[call(parse_generics_with_where_clause)]
   pub generics: Generics,
   pub _semi: Token![;],
}

#[derive(Clone, Parse, Debug)]
pub struct ImplSignature {
   pub _impl_kw: Token![impl],
   pub impl_generics: Generics,
   pub ident: Ident,
   #[call(parse_generics_with_where_clause)]
   pub generics: Generics,
   pub _semi: Token![;],
}

/// Parses `Generics` but ALSO consumes a trailing `where` clause if present.
/// (Syn's own `Generics::parse` does not consume `where`.)
pub fn parse_generics_with_where_clause(input: ParseStream) -> Result<Generics> {
   let mut res = Generics::parse(input)?;
   if input.peek(Token![where]) {
      res.where_clause = Some(input.parse()?);
   }
   Ok(res)
}

#[derive(Parse, Clone, PartialEq, Eq, Hash, Debug)]
pub struct IfLetClause {
   pub if_keyword: Token![if],
   pub let_keyword: Token![let],
   #[call(Pat::parse_multi)]
   pub pattern: Pat,
   pub eq_symbol: Token![=],
   pub exp: Expr,
}

#[derive(Parse, Clone, PartialEq, Eq, Hash, Debug)]
pub struct IfClause {
   pub if_keyword: Token![if],
   pub cond: Expr,
}

#[derive(Parse, Clone, PartialEq, Eq, Hash, Debug)]
pub struct LetClause {
   pub let_keyword: Token![let],
   #[call(Pat::parse_multi)]
   pub pattern: Pat,
   pub eq_symbol: Token![=],
   pub exp: Expr,
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

   pub fn expr(&self) -> &Expr {
      match self {
         CondClause::IfLet(cl) => &cl.exp,
         CondClause::If(cl) => &cl.cond,
         CondClause::Let(cl) => &cl.exp,
      }
   }
}

impl SynParse for CondClause {
   fn parse(input: ParseStream) -> Result<Self> {
      if input.peek(Token![if]) {
         if input.peek2(Token![let]) { Ok(Self::IfLet(input.parse()?)) } else { Ok(Self::If(input.parse()?)) }
      } else if input.peek(Token![let]) {
         Ok(Self::Let(input.parse()?))
      } else {
         Err(input.error("expected either if clause or if let clause"))
      }
   }
}

#[derive(Parse, Clone)]
pub struct GeneratorNode {
   pub for_keyword: Token![for],
   #[call(Pat::parse_multi)]
   pub pattern: Pat,
   pub _in_keyword: Token![in],
   pub expr: Expr,
}

// ============================================================================
// HIR-layer types
// ============================================================================

/// Which proc-macro will consume the emitted MIR tokens. Carried as a
/// `syn::Path` — the frontend never inspects it, just copies it into the
/// emitted `#path!(mir_v1 {…})` call. Users point at any third-party
/// backend's proc-macro this way, without central registration.
pub type BackendPath = Path;

/// Parses the `backend(PATH , <opaque args>)` argument list:
///   - `PATH`: a Rust path to the backend's proc-macro (e.g. `::ascent::__backend_dd::compile_mir`).
///   - `<opaque args>`: everything after the first `,` — passed through to the
///     backend unchanged so it can interpret its own sub-config (e.g. modes).
pub struct BackendArgs {
   pub path: Path,
   pub extra: TokenStream,
}

impl SynParse for BackendArgs {
   fn parse(input: ParseStream) -> Result<Self> {
      let path = Path::parse(input)?;
      let extra = if input.peek(Token![,]) {
         input.parse::<Token![,]>()?;
         TokenStream::parse(input)?
      } else {
         TokenStream::new()
      };
      Ok(BackendArgs { path, extra })
   }
}

#[derive(Clone)]
pub struct AscentConfig {
   #[allow(dead_code)]
   pub attrs: Vec<Attribute>,
   pub include_rule_times: bool,
   pub generate_run_partial: bool,
   pub inter_rule_parallelism: bool,
   pub default_ds: DsAttributeContents,
   /// Proc-macro path to dispatch codegen to. Always `Some` — defaults to
   /// the built-in batch backend path.
   pub backend_path: BackendPath,
   /// Opaque tokens passed through from `#![backend(..., extra...)]` to the
   /// backend macro. The backend parses them for its own sub-config.
   pub backend_extra: TokenStream,
   /// Hint from the backend for HIR→MIR lowering: when `true`, skip the
   /// explicit semi-naive variant expansion — the backend performs
   /// semi-naive at the dataflow-operator level (e.g. DD's `Variable`).
   /// Set by shorthand resolution in `#![backend(...)]`; defaults to `false`.
   pub backend_operator_semi_naive: bool,
}

impl AscentConfig {
   const MEASURE_RULE_TIMES_ATTR: &'static str = "measure_rule_times";
   const GENERATE_RUN_TIMEOUT_ATTR: &'static str = "generate_run_timeout";
   const INTER_RULE_PARALLELISM_ATTR: &'static str = "inter_rule_parallelism";
   const BACKEND_ATTR: &'static str = "backend";

   pub fn new(attrs: Vec<Attribute>, is_parallel: bool) -> Result<AscentConfig> {
      let include_rule_times = attrs
         .iter()
         .find(|attr| attr.meta.path().is_ident(Self::MEASURE_RULE_TIMES_ATTR))
         .map(|attr| attr.meta.require_path_only())
         .transpose()?
         .is_some();
      let generate_run_partial = attrs
         .iter()
         .find(|attr| attr.meta.path().is_ident(Self::GENERATE_RUN_TIMEOUT_ATTR))
         .map(|attr| attr.meta.require_path_only())
         .transpose()?
         .is_some();
      let inter_rule_parallelism = attrs
         .iter()
         .find(|attr| attr.meta.path().is_ident(Self::INTER_RULE_PARALLELISM_ATTR))
         .map(|attr| attr.meta.require_path_only())
         .transpose()?;

      // `#![backend(PATH [, extra...])]` — PATH is any proc-macro path.
      // Shorthand: `batch` → `::ascent::__backend_batch::compile_mir`,
      //            `dd`    → `::ascent::__backend_dd::compile_mir`.
      let backend_attr = attrs.iter().find(|attr| attr.meta.path().is_ident(Self::BACKEND_ATTR));
      let (backend_path, backend_extra, backend_operator_semi_naive) = match backend_attr {
         None => (parse_quote! { ::ascent::__backend_batch::compile_mir }, TokenStream::new(), false),
         Some(attr) => {
            let list = attr.meta.require_list()?;
            let parsed: BackendArgs = parse2(list.tokens.clone())?;
            // Shorthand expansion. Known shorthand also sets the
            // `operator_semi_naive` hint for HIR→MIR lowering.
            let (path, op_semi_naive) = if parsed.path.is_ident("batch") {
               (parse_quote! { ::ascent::__backend_batch::compile_mir }, false)
            } else if parsed.path.is_ident("dd") {
               (parse_quote! { ::ascent::__backend_dd::compile_mir }, true)
            } else {
               // Custom path: assume batch semantics unless backend says otherwise.
               (parsed.path, false)
            };
            (path, parsed.extra, op_semi_naive)
         },
      };

      let _ = is_parallel;
      let _ = backend_attr;

      let recognized_attrs = [
         Self::MEASURE_RULE_TIMES_ATTR,
         Self::GENERATE_RUN_TIMEOUT_ATTR,
         Self::INTER_RULE_PARALLELISM_ATTR,
         Self::BACKEND_ATTR,
         REL_DS_ATTR,
      ];
      for attr in attrs.iter() {
         if !recognized_attrs.iter().any(|recognized_attr| attr.meta.path().is_ident(recognized_attr)) {
            let recognized_attrs = recognized_attrs.iter().map(|attr| format!("`{attr}`")).join(", ");
            return Err(Error::new_spanned(
               attr,
               format!("unrecognized attribute. recognized attributes are: {recognized_attrs}"),
            ));
         }
      }
      if inter_rule_parallelism.is_some() && !is_parallel {
         return Err(Error::new_spanned(inter_rule_parallelism, "attribute only allowed in parallel Ascent"));
      }
      let default_ds = get_ds_attr(&attrs)?
         .unwrap_or_else(|| DsAttributeContents { path: parse_quote! {::ascent::rel}, args: TokenStream::default() });
      Ok(AscentConfig {
         inter_rule_parallelism: inter_rule_parallelism.is_some(),
         attrs,
         include_rule_times,
         generate_run_partial,
         default_ds,
         backend_path,
         backend_extra,
         backend_operator_semi_naive,
      })
   }
}

pub const REL_DS_ATTR: &str = "ds";

pub fn get_ds_attr(attrs: &[Attribute]) -> Result<Option<DsAttributeContents>> {
   let ds_attrs = attrs
      .iter()
      .filter(|attr| attr.meta.path().get_ident().is_some_and(|ident| ident == REL_DS_ATTR))
      .collect_vec();
   match &ds_attrs[..] {
      [] => Ok(None),
      [attr] => {
         let res = parse2::<DsAttributeContents>(attr.meta.require_list()?.tokens.clone())?;
         Ok(Some(res))
      },
      [_attr1, attr2, ..] => Err(Error::new(attr2.bracket_token.span.join(), "multiple `ds` attributes specified")),
   }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum IndexValType {
   Reference,
   Direct(Vec<usize>),
}

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct IrRelation {
   pub relation: RelationIdentity,
   pub indices: Vec<usize>,
   pub val_type: IndexValType,
}

impl IrRelation {
   pub fn new(relation: RelationIdentity, indices: Vec<usize>) -> Self {
      let val_type = if relation.is_lattice {
         IndexValType::Reference
      } else {
         IndexValType::Direct((0..relation.field_types.len()).filter(|i| !indices.contains(i)).collect_vec())
      };
      IrRelation { relation, indices, val_type }
   }

   pub fn key_type(&self) -> Type {
      let index_types: Vec<_> = self.indices.iter().map(|&i| self.relation.field_types[i].clone()).collect();
      tuple_type(&index_types)
   }

   pub fn ir_name(&self) -> Ident { ir_name_for_rel_indices(&self.relation.name, &self.indices) }
   pub fn is_full_index(&self) -> bool { self.relation.field_types.len() == self.indices.len() }
   pub fn is_no_index(&self) -> bool { self.indices.is_empty() }

   pub fn value_type(&self) -> Type {
      match &self.val_type {
         IndexValType::Reference => parse_quote! {usize},
         IndexValType::Direct(cols) => {
            let index_types: Vec<_> = cols.iter().map(|&i| self.relation.field_types[i].clone()).collect();
            tuple_type(&index_types)
         },
      }
   }
}

pub fn ir_name_for_rel_indices(rel: &Ident, indices: &[usize]) -> Ident {
   let indices_str = if indices.is_empty() { format!("none") } else { indices.iter().join("_") };
   let name = format!("{}_indices_{}", rel, indices_str);
   Ident::new(&name, rel.span())
}

#[derive(Clone)]
pub struct RelationMetadata {
   pub initialization: Option<Rc<Expr>>,
   pub attributes: Rc<Vec<Attribute>>,
   /// Will be `Some()` iff the relation is not a lattice.
   pub ds_attr: Option<DsAttributeContents>,
   /// `#[input]` present on the relation decl — relation appears in the
   /// DD backend's `<Name>ComposeInputs` struct and the caller must supply
   /// a seed Collection for it. Consumed only by the DD backend's compose
   /// emission; ignored by batch / `run()` / `session()`.
   pub is_compose_input: bool,
   /// `#[output]` present on the relation decl — relation appears in the
   /// DD backend's `<Name>ComposeOutputs` struct. Consumed only by the
   /// DD backend's compose emission.
   pub is_compose_output: bool,
}

#[derive(Clone)]
pub struct IrHeadClause {
   pub rel: RelationIdentity,
   pub args: Vec<Expr>,
   pub span: Span,
   pub args_span: Span,
}

pub enum IrBodyItem {
   Clause(IrBodyClause),
   Generator(GeneratorNode),
   Cond(CondClause),
   Agg(IrAggClause),
}

impl IrBodyItem {
   pub fn rel(&self) -> Option<&IrRelation> {
      match self {
         IrBodyItem::Clause(bcl) => Some(&bcl.rel),
         IrBodyItem::Agg(agg) => Some(&agg.rel),
         IrBodyItem::Generator(_) | IrBodyItem::Cond(_) => None,
      }
   }
}

#[derive(Clone)]
pub struct IrBodyClause {
   pub rel: IrRelation,
   pub args: Vec<Expr>,
   pub rel_args_span: Span,
   pub args_span: Span,
   pub cond_clauses: Vec<CondClause>,
}

impl IrBodyClause {
   #[allow(dead_code)]
   pub fn selected_args(&self) -> Vec<Expr> { self.rel.indices.iter().map(|&i| self.args[i].clone()).collect() }
}

#[derive(Clone)]
pub struct IrAggClause {
   pub span: Span,
   pub pat: Pat,
   pub aggregator: Expr,
   pub bound_args: Vec<Ident>,
   pub rel: IrRelation,
   pub rel_args: Vec<Expr>,
}

pub struct IrRule {
   pub head_clauses: Vec<IrHeadClause>,
   pub body_items: Vec<IrBodyItem>,
   pub simple_join_start_index: Option<usize>,
   /// User-supplied semi-naive plan (from `#[plan(variant(...), ...)]`).
   pub plan: Option<Vec<IrPlanVariant>>,
}

/// One semi-naive variant as spelled by the user.
#[derive(Clone, Debug)]
pub struct IrPlanVariant {
   pub delta: usize,
   pub order: Vec<usize>,
}

// ============================================================================
// MIR-layer types
// ============================================================================

pub struct AscentMir {
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

pub struct MirScc {
   pub rules: Vec<MirRule>,
   pub dynamic_relations: HashMap<RelationIdentity, HashSet<IrRelation>>,
   pub body_only_relations: HashMap<RelationIdentity, HashSet<IrRelation>>,
   pub is_looping: bool,
}

#[derive(Clone)]
pub struct MirRule {
   // TODO rename to head_clauses
   pub head_clause: Vec<IrHeadClause>,
   pub body_items: Vec<MirBodyItem>,
   pub simple_join_start_index: Option<usize>,
   pub reorderable: bool,
}

#[derive(Clone)]
pub enum MirBodyItem {
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
}

#[derive(Clone)]
pub struct MirBodyClause {
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
pub struct MirRelation {
   pub relation: RelationIdentity,
   pub indices: Vec<usize>,
   pub ir_name: Ident,
   pub version: MirRelationVersion,
   pub is_full_index: bool,
   pub is_no_index: bool,
   pub val_type: IndexValType,
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

pub fn ir_relation_version_var_name(ir_name: &Ident, version: MirRelationVersion) -> Ident {
   let name = format!("{}_{}", ir_name, version.to_string());
   Ident::new(&name, ir_name.span())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MirRelationVersion {
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

// ============================================================================
// Debug summary helpers (used by both codegen backends and diagnostic logs)
// ============================================================================

pub fn mir_summary(mir: &AscentMir) -> String {
   use std::fmt::Write;
   let mut res = String::new();
   for (i, scc) in mir.sccs.iter().enumerate() {
      writeln!(&mut res, "scc {}, is_looping: {}:", i, scc.is_looping).unwrap();
      for r in scc.rules.iter() {
         writeln!(&mut res, "  {}", mir_rule_summary(r)).unwrap();
      }
      let sorted = scc.dynamic_relations.keys().sorted_by_key(|rel| &rel.name);
      write!(&mut res, "  dynamic relations: ").unwrap();
      writeln!(&mut res, "{}", sorted.map(|r| r.name.to_string()).join(", ")).unwrap();
   }
   res
}

pub fn mir_rule_summary(rule: &MirRule) -> String {
   fn bitem_to_str(bitem: &MirBodyItem) -> String {
      match bitem {
         MirBodyItem::Clause(bcl) => format!("{}_{}", bcl.rel.ir_name, bcl.rel.version.to_string()),
         MirBodyItem::Generator(gen) => {
            let pat_s = crate::utils::pat_to_ident(&gen.pattern).map(|x| x.to_string()).unwrap_or_default();
            format!("for_{}", pat_s)
         },
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
