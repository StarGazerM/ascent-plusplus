//! MIR v1 token-surface: `emit_mir` / `parse_mir`.
//!
//! Stable textual MIR — the boundary between the Ascent frontend and any
//! codegen backend. Grammar spec lives in the design doc; every `emit_*`
//! function has a matching `parse_*` below it.

use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::parse::{Parse, ParseStream, Parser};
use syn::spanned::Spanned;
use syn::{Attribute, Expr, LitInt, Pat, Token, Type, braced, bracketed, parenthesized};

use crate::types::{
   AscentConfig, AscentMir, CondClause, DdConfig, DdMode, DsAttributeContents, GeneratorNode, IfClause, IfLetClause,
   ImplSignature, IndexValType, IrAggClause, IrHeadClause, IrRelation, LetClause, MirBodyClause, MirBodyItem,
   MirRelation, MirRelationVersion, MirRule, MirScc, RelationIdentity, RelationMetadata, Signatures, TypeSignature,
   ir_name_for_rel_indices,
};

// ---------- small helpers ---------------------------------------------------

fn kw(name: &str) -> Ident { Ident::new(name, Span::call_site()) }

fn bool_kw(b: bool) -> Ident { kw(if b { "true" } else { "false" }) }

fn uint_list(xs: &[usize]) -> TokenStream {
   let items = xs.iter().map(|&i| {
      let lit = syn::LitInt::new(&i.to_string(), Span::call_site());
      quote! { #lit }
   });
   quote! { #( #items ),* }
}

fn attr_list(attrs: &[Attribute]) -> TokenStream {
   // Always render as outer `#[...]`. Inner-vs-outer style is a Rust parser
   // position concern, not semantic — backends never use it (they introspect
   // `attr.meta.path()`), and the MIR grammar chose outer as canonical.
   let items = attrs.iter().map(|a| {
      let meta = &a.meta;
      quote! { #[ #meta ] }
   });
   quote! { #( #items )* }
}

fn opt_expr(e: Option<&Expr>) -> TokenStream {
   match e {
      Some(e) => quote! { { #e } },
      None => quote! { - },
   }
}

// ---------- top-level dispatch ---------------------------------------------

/// Emit the full MIR program as a token stream matching the MIR v1 grammar.
pub fn emit_mir(mir: &AscentMir, is_ascent_run: bool) -> TokenStream {
   let program = emit_program(mir, is_ascent_run);
   let relations = emit_relations(mir);
   let sccs = emit_sccs(mir);
   quote! {
      mir_v1 {
         #program
         #relations
         #sccs
      }
   }
}

// ---------- Program block ---------------------------------------------------

fn emit_program(mir: &AscentMir, is_ascent_run: bool) -> TokenStream {
   let cfg = &mir.config;
   let kind = kw(if is_ascent_run { "ascent_run" } else { "ascent" });
   let parallel = bool_kw(mir.is_parallel);
   let inter_rule_parallelism = bool_kw(cfg.inter_rule_parallelism);
   let include_rule_times = bool_kw(cfg.include_rule_times);
   let generate_run_partial = bool_kw(cfg.generate_run_partial);
   // Backend path is emitted as an opaque path wrapped in braces. The
   // frontend never inspected it beyond copying, and the backend macro
   // (called via this path) doesn't read this back — this line exists so
   // the MIR round-trip stays lossless.
   let backend_path = &cfg.backend_path;
   let backend_extra = &cfg.backend_extra;
   let operator_semi_naive = bool_kw(cfg.backend_operator_semi_naive);
   let default_ds = emit_ds_attr(&cfg.default_ds);
   let config_attrs = attr_list(&cfg.attrs);
   let signature = emit_signature(&mir.signatures);
   // DD config is optional. `-` means no `#![dd(...)]` attr was present;
   // any other form is `{ mode = MODE }`. Parsed symmetrically in
   // `parse_program`. Keeps the textual MIR self-describing.
   let dd_config_tok = match &cfg.dd_config {
      None => quote! { - },
      Some(dd) => {
         let mode = match dd.mode {
            DdMode::Incremental => kw("incremental"),
            DdMode::Batch => kw("batch"),
         };
         quote! { { mode = #mode } }
      },
   };

   quote! {
      program {
         kind                         #kind ;
         parallel                     #parallel ;
         inter_rule_parallelism       #inter_rule_parallelism ;
         include_rule_times           #include_rule_times ;
         generate_run_partial         #generate_run_partial ;
         backend                      { #backend_path } ;
         backend_extra                { #backend_extra } ;
         backend_operator_semi_naive  #operator_semi_naive ;
         dd_config                    #dd_config_tok ;
         default_ds                   #default_ds ;
         config_attrs { #config_attrs }
         signature { #signature }
      }
   }
}

fn emit_ds_attr(ds: &DsAttributeContents) -> TokenStream {
   let path = &ds.path;
   let args = &ds.args;
   quote! { { path : { #path } , args : { #args } } }
}

fn emit_signature(sigs: &Signatures) -> TokenStream {
   // `TypeSignature` / `ImplSignature` are already well-formed Rust items;
   // keep them as opaque token blobs so the backend reconstructs them with
   // `syn::parse` and gets spans for free.
   let decl_attrs = &sigs.declaration.attrs;
   let decl_vis = &sigs.declaration.visibility;
   let decl_ident = &sigs.declaration.ident;
   let decl_generics = &sigs.declaration.generics;
   let decl_where = &sigs.declaration.generics.where_clause;
   let decl_tokens = quote! {
      #(#decl_attrs)*
      #decl_vis struct #decl_ident #decl_generics #decl_where ;
   };

   let impl_block = match &sigs.implementation {
      Some(imp) => {
         let impl_generics = &imp.impl_generics;
         let ident = &imp.ident;
         let ty_generics = &imp.generics;
         let where_clause = &imp.generics.where_clause;
         let tokens = quote! {
            impl #impl_generics #ident #ty_generics #where_clause ;
         };
         quote! { impl { #tokens } }
      },
      None => TokenStream::new(),
   };

   quote! {
      decl { #decl_tokens }
      #impl_block
   }
}

// ---------- Relations block -------------------------------------------------

fn emit_relations(mir: &AscentMir) -> TokenStream {
   // Deterministic order: by relation name. Hashmaps in `AscentMir` would
   // otherwise give nondeterministic output and break snapshot tests.
   let mut rels: Vec<(&RelationIdentity, &IrRelation)> = mir.relations_full_indices.iter().collect();
   rels.sort_by(|a, b| a.0.name.to_string().cmp(&b.0.name.to_string()));

   let items = rels.iter().map(|(ident, full_idx)| emit_relation(mir, ident, full_idx));

   quote! {
      relations { #( #items )* }
   }
}

fn emit_relation(mir: &AscentMir, ident: &RelationIdentity, full_idx: &IrRelation) -> TokenStream {
   let kind = kw(if ident.is_lattice { "lattice" } else { "relation" });
   let name = &ident.name;
   let types: Vec<TokenStream> = ident.field_types.iter().map(emit_type).collect();
   let type_list = quote! { #( #types ),* };

   // All IrRelations (arrangements) used for this relation.
   let all_ir_rels: &HashSet<IrRelation> = mir
      .relations_ir_relations
      .get(ident)
      .expect("every RelationIdentity must have an entry in relations_ir_relations");
   let mut all_ir_rels: Vec<&IrRelation> = all_ir_rels.iter().collect();
   all_ir_rels.sort_by(|a, b| a.indices.cmp(&b.indices));
   let indices_entries = all_ir_rels.iter().map(|ir| emit_ir_rel_decl(ir));
   let indices_list = quote! { #( #indices_entries ),* };

   let full_index_ref = emit_ir_rel_ref(&full_idx.indices);

   let lattice_full_index_ref = if ident.is_lattice {
      let lat = mir
         .lattices_full_indices
         .get(ident)
         .expect("lattice relation must have an entry in lattices_full_indices");
      emit_ir_rel_ref(&lat.indices)
   } else {
      quote! { - }
   };

   let metadata: &RelationMetadata = mir
      .relations_metadata
      .get(ident)
      .expect("every RelationIdentity must have an entry in relations_metadata");
   let init_tokens = opt_expr(metadata.initialization.as_deref());
   let attrs_tokens = attr_list(&metadata.attributes);
   let ds_tokens = match &metadata.ds_attr {
      Some(ds) => emit_ds_attr(ds),
      None => quote! { - },
   };
   // Compose-API flags. Serialize as `+`/`-` so the reader can do a single
   // ident-peek to deserialize. These cross the two-stage MIR-text boundary
   // — without this round-trip, the DD backend's compose emission would
   // see all-false and silently emit nothing.
   let compose_in_tok = if metadata.is_compose_input { quote! { + } } else { quote! { - } };
   let compose_out_tok = if metadata.is_compose_output { quote! { + } } else { quote! { - } };

   quote! {
      #kind #name ( #type_list ) {
         indices             [ #indices_list ] ;
         full_index          #full_index_ref ;
         lattice_full_index  #lattice_full_index_ref ;
         initialization      #init_tokens ;
         attrs               { #attrs_tokens } ;
         ds                  #ds_tokens ;
         compose_input       #compose_in_tok ;
         compose_output      #compose_out_tok ;
      }
   }
}

fn emit_type(t: &Type) -> TokenStream {
   quote! { { #t } }
}

fn emit_ir_rel_decl(ir: &IrRelation) -> TokenStream {
   let idx = uint_list(&ir.indices);
   let val = emit_val_type(&ir.val_type);
   quote! { [ #idx ] : #val }
}

fn emit_ir_rel_ref(indices: &[usize]) -> TokenStream {
   let idx = uint_list(indices);
   quote! { [ #idx ] }
}

fn emit_val_type(vt: &IndexValType) -> TokenStream {
   match vt {
      IndexValType::Reference => quote! { reference },
      IndexValType::Direct(cols) => {
         let list = uint_list(cols);
         quote! { direct [ #list ] }
      },
   }
}

// ---------- SCCs block ------------------------------------------------------

fn emit_sccs(mir: &AscentMir) -> TokenStream {
   let items = mir.sccs.iter().enumerate().map(|(i, scc)| emit_scc(i, scc));
   quote! {
      sccs { #( #items )* }
   }
}

fn emit_scc(index: usize, scc: &MirScc) -> TokenStream {
   let idx_lit = syn::LitInt::new(&index.to_string(), Span::call_site());
   let looping = bool_kw(scc.is_looping);
   let dynamic = emit_rel_inst_list(&scc.dynamic_relations);
   let body_only = emit_rel_inst_list(&scc.body_only_relations);
   let rules = scc.rules.iter().map(emit_rule);
   quote! {
      scc #idx_lit {
         looping   #looping ;
         dynamic   { #dynamic }
         body_only { #body_only }
         rules { #( #rules )* }
      }
   }
}

fn emit_rel_inst_list(map: &HashMap<RelationIdentity, HashSet<IrRelation>>) -> TokenStream {
   // Sort by relation name for determinism.
   let mut entries: Vec<(&RelationIdentity, &HashSet<IrRelation>)> = map.iter().collect();
   entries.sort_by(|a, b| a.0.name.to_string().cmp(&b.0.name.to_string()));
   let out = entries.iter().map(|(ident, ir_set)| {
      let name = &ident.name;
      let mut irs: Vec<&IrRelation> = ir_set.iter().collect();
      irs.sort_by(|a, b| a.indices.cmp(&b.indices));
      let refs = irs.iter().map(|ir| emit_ir_rel_ref(&ir.indices));
      quote! {
         #name : [ #( #refs ),* ] ;
      }
   });
   quote! { #( #out )* }
}

// ---------- Rules -----------------------------------------------------------

fn emit_rule(rule: &MirRule) -> TokenStream {
   let meta = emit_rule_meta(rule);
   let heads = rule.head_clause.iter().map(emit_head_clause);
   let body = rule.body_items.iter().map(emit_body_item);
   quote! {
      rule #meta {
         head { #( #heads ),* }
         body { #( #body )* }
      }
   }
}

fn emit_rule_meta(rule: &MirRule) -> TokenStream {
   let mut entries: Vec<TokenStream> = Vec::new();
   if let Some(ind) = rule.simple_join_start_index {
      let lit = syn::LitInt::new(&ind.to_string(), Span::call_site());
      entries.push(quote! { simple_join = #lit });
   }
   if rule.reorderable {
      // Only emit when true — absence implies false, keeps default-case output tidy.
      entries.push(quote! { reorderable = true });
   }
   if entries.is_empty() {
      TokenStream::new()
   } else {
      quote! { [ #( #entries ),* ] }
   }
}

fn emit_head_clause(hcl: &IrHeadClause) -> TokenStream {
   // Re-ident the relation name at the clause-site span so round-trip preserves
   // the span that downstream codegen uses for generated identifiers (notably
   // `__matching` et al). Using `hcl.rel.name` directly would bake in the
   // relation's DECLARATION span — which may have a different hygiene context
   // when the source came through an `ascent_source!` / `include_source!` chain.
   let name = Ident::new(&hcl.rel.name.to_string(), hcl.span);
   let args = hcl.args.iter().map(emit_expr);
   quote! { #name ( #( #args ),* ) }
}

// ---------- Body items ------------------------------------------------------

fn emit_body_item(bi: &MirBodyItem) -> TokenStream {
   match bi {
      MirBodyItem::Clause(cl) => {
         let t = emit_clause_item(cl);
         quote! { #t ; }
      },
      MirBodyItem::Generator(g) => {
         let t = emit_generator(g);
         quote! { #t ; }
      },
      MirBodyItem::Cond(cc) => {
         let t = emit_cond_clause(cc);
         quote! { #t ; }
      },
      MirBodyItem::Agg(agg) => {
         let t = emit_agg(agg);
         quote! { #t ; }
      },
   }
}

fn emit_clause_item(cl: &MirBodyClause) -> TokenStream {
   // Re-ident at call-site (rel_args_span) so span round-trips correctly when
   // the code came through macro_rules expansion (e.g. `include_source!`).
   // See comment on emit_head_clause.
   let name = Ident::new(&cl.rel.relation.name.to_string(), cl.rel_args_span);
   let meta = emit_clause_meta(&cl.rel);
   let args = cl.args.iter().map(emit_expr);
   let cond_tail = emit_cond_tail(&cl.cond_clauses);
   quote! {
      clause #name #meta ( #( #args ),* ) #cond_tail
   }
}

fn emit_clause_meta(rel: &MirRelation) -> TokenStream {
   let indices = emit_ir_rel_ref(&rel.indices);
   let version = emit_version(rel.version);
   let val_type = emit_val_type(&rel.val_type);
   quote! { [ indices = #indices , version = #version , val_type = #val_type ] }
}

fn emit_version(v: MirRelationVersion) -> TokenStream {
   let k = kw(match v {
      MirRelationVersion::Total => "total",
      MirRelationVersion::Delta => "delta",
      MirRelationVersion::TotalDelta => "total_delta",
      MirRelationVersion::New => "new",
   });
   quote! { #k }
}

fn emit_cond_tail(conds: &[CondClause]) -> TokenStream {
   if conds.is_empty() {
      return TokenStream::new();
   }
   let items = conds.iter().map(emit_cond_clause);
   quote! { where { #( #items ),* } }
}

fn emit_cond_clause(cc: &CondClause) -> TokenStream {
   match cc {
      CondClause::If(c) => {
         let e = &c.cond;
         quote! { if { #e } }
      },
      CondClause::IfLet(c) => {
         let p = &c.pattern;
         let e = &c.exp;
         quote! { if_let { #p } = { #e } }
      },
      CondClause::Let(c) => {
         let p = &c.pattern;
         let e = &c.exp;
         quote! { let { #p } = { #e } }
      },
   }
}

fn emit_generator(g: &GeneratorNode) -> TokenStream {
   let p = &g.pattern;
   let e = &g.expr;
   quote! { for { #p } in { #e } }
}

fn emit_agg(agg: &IrAggClause) -> TokenStream {
   let pat = &agg.pat;
   let aggregator = &agg.aggregator;
   let bound_args = &agg.bound_args;
   // Re-ident at agg-site span for the same macro-hygiene reason as clauses.
   let rel_name = Ident::new(&agg.rel.relation.name.to_string(), agg.span);
   // Build a synthetic MirRelation just to reuse emit_clause_meta (agg rel is
   // never versioned — IrAggClause has no version field, so use Total).
   let ir = &agg.rel;
   let indices = emit_ir_rel_ref(&ir.indices);
   let val_type = emit_val_type(&ir.val_type);
   let meta = quote! { [ indices = #indices , version = total , val_type = #val_type ] };
   let args = agg.rel_args.iter().map(emit_expr);
   quote! {
      agg { #pat } = { #aggregator } ( #( #bound_args ),* ) in #rel_name #meta ( #( #args ),* )
   }
}

fn emit_expr(e: &Expr) -> TokenStream {
   quote! { { #e } }
}

// ============================================================================
// Parser: MIR v1 token stream → AscentMir
// ============================================================================

/// Parse a MIR v1 token stream produced by `emit_mir`. Returns the MIR plus
/// the `is_ascent_run` flag carried in the `program.kind` field.
pub fn parse_mir(tokens: TokenStream) -> syn::Result<(AscentMir, bool)> {
   Parser::parse2(parse_mir_inner, tokens)
}

fn parse_mir_inner(input: ParseStream) -> syn::Result<(AscentMir, bool)> {
   expect_kw(input, "mir_v1")?;
   let body;
   braced!(body in input);

   let (config, signatures, is_parallel, is_ascent_run) = parse_program(&body)?;
   let parsed = parse_relations(&body)?;
   let sccs = parse_sccs(&body, &parsed.by_name)?;

   let ParsedRelations { full, all, lattices, metadata, by_name: _ } = parsed;

   Ok((
      AscentMir {
         sccs,
         deps: HashMap::new(),
         relations_ir_relations: all,
         relations_full_indices: full,
         lattices_full_indices: lattices,
         relations_metadata: metadata,
         signatures,
         config,
         is_parallel,
      },
      is_ascent_run,
   ))
}

// ---------- parse helpers ---------------------------------------------------

fn expect_kw(input: ParseStream, kw: &str) -> syn::Result<()> {
   let id: Ident = input.parse()?;
   if id != kw {
      return Err(syn::Error::new(id.span(), format!("expected `{kw}`, got `{id}`")));
   }
   Ok(())
}

fn peek_kw(input: ParseStream, kw: &str) -> bool {
   input.peek(syn::Ident) && input.fork().parse::<Ident>().map(|i| i == kw).unwrap_or(false)
}

fn parse_bool(input: ParseStream) -> syn::Result<bool> {
   // `true` / `false` are Rust keywords, not idents — use syn::LitBool.
   let lit: syn::LitBool = input.parse()?;
   Ok(lit.value)
}

/// Parse `[ u, u, u ]` → Vec<usize>.
fn parse_uint_list_bracketed(input: ParseStream) -> syn::Result<Vec<usize>> {
   let inner;
   bracketed!(inner in input);
   parse_uint_list_inner(&inner)
}

fn parse_uint_list_inner(inner: ParseStream) -> syn::Result<Vec<usize>> {
   let mut out = Vec::new();
   while !inner.is_empty() {
      let lit: LitInt = inner.parse()?;
      out.push(lit.base10_parse::<usize>()?);
      if inner.peek(Token![,]) {
         inner.parse::<Token![,]>()?;
      } else {
         break;
      }
   }
   Ok(out)
}

/// Parse a braced Rust fragment `{ tokens }` and return the raw tokens.
fn parse_braced_tokens(input: ParseStream) -> syn::Result<TokenStream> {
   let inner;
   braced!(inner in input);
   let ts: TokenStream = inner.parse()?;
   Ok(ts)
}

/// Parse a braced Rust fragment and `syn::parse2` its content into `T`.
fn parse_braced_as<T: Parse>(input: ParseStream) -> syn::Result<T> {
   let inner;
   braced!(inner in input);
   inner.parse::<T>()
}

/// Parse optional `{ tokens }` or literal `-` → Option<T>.
fn parse_braced_or_dash_as<T: Parse>(input: ParseStream) -> syn::Result<Option<T>> {
   if input.peek(Token![-]) {
      input.parse::<Token![-]>()?;
      Ok(None)
   } else {
      Ok(Some(parse_braced_as(input)?))
   }
}

fn parse_plus_or_dash(input: ParseStream) -> syn::Result<bool> {
   if input.peek(Token![+]) {
      input.parse::<Token![+]>()?;
      Ok(true)
   } else if input.peek(Token![-]) {
      input.parse::<Token![-]>()?;
      Ok(false)
   } else {
      Err(input.error("expected `+` or `-`"))
   }
}

// ---------- Program parser --------------------------------------------------

fn parse_program(input: ParseStream) -> syn::Result<(AscentConfig, Signatures, bool, bool)> {
   expect_kw(input, "program")?;
   let body;
   braced!(body in input);

   // Expected, in order:
   //   kind KIND ; parallel B ; inter_rule_parallelism B ;
   //   include_rule_times B ; generate_run_partial B ;
   //   backend KIND ; dd_mode KIND ;
   //   default_ds DS ;
   //   config_attrs { ... }
   //   signature { decl {...} impl? }

   expect_kw(&body, "kind")?;
   let kind_id: Ident = body.parse()?;
   let is_ascent_run = match kind_id.to_string().as_str() {
      "ascent" => false,
      "ascent_run" => true,
      other => return Err(syn::Error::new(kind_id.span(), format!("unknown kind `{other}`"))),
   };
   body.parse::<Token![;]>()?;

   expect_kw(&body, "parallel")?;
   let is_parallel = parse_bool(&body)?;
   body.parse::<Token![;]>()?;

   expect_kw(&body, "inter_rule_parallelism")?;
   let inter_rule_parallelism = parse_bool(&body)?;
   body.parse::<Token![;]>()?;

   expect_kw(&body, "include_rule_times")?;
   let include_rule_times = parse_bool(&body)?;
   body.parse::<Token![;]>()?;

   expect_kw(&body, "generate_run_partial")?;
   let generate_run_partial = parse_bool(&body)?;
   body.parse::<Token![;]>()?;

   expect_kw(&body, "backend")?;
   let backend_path_inner;
   braced!(backend_path_inner in body);
   let backend_path: syn::Path = backend_path_inner.parse()?;
   body.parse::<Token![;]>()?;

   expect_kw(&body, "backend_extra")?;
   let backend_extra_inner;
   braced!(backend_extra_inner in body);
   let backend_extra: TokenStream = backend_extra_inner.parse()?;
   body.parse::<Token![;]>()?;

   expect_kw(&body, "backend_operator_semi_naive")?;
   let backend_operator_semi_naive = parse_bool(&body)?;
   body.parse::<Token![;]>()?;

   expect_kw(&body, "dd_config")?;
   let dd_config = if body.peek(Token![-]) {
      body.parse::<Token![-]>()?;
      None
   } else {
      let inner;
      braced!(inner in body);
      // { mode = incremental | batch }
      expect_kw(&inner, "mode")?;
      inner.parse::<Token![=]>()?;
      let mode_ident: Ident = inner.parse()?;
      let mode = if mode_ident == "incremental" {
         DdMode::Incremental
      } else if mode_ident == "batch" {
         DdMode::Batch
      } else {
         return Err(syn::Error::new(mode_ident.span(), format!("unknown DD mode `{mode_ident}` in MIR text")));
      };
      Some(DdConfig { mode })
   };
   body.parse::<Token![;]>()?;

   expect_kw(&body, "default_ds")?;
   let default_ds = parse_ds_attr(&body)?;
   body.parse::<Token![;]>()?;

   expect_kw(&body, "config_attrs")?;
   let config_attrs = parse_attrs_block(&body)?;

   expect_kw(&body, "signature")?;
   let signatures = parse_signature_block(&body)?;

   let config = AscentConfig {
      attrs: config_attrs,
      include_rule_times,
      generate_run_partial,
      inter_rule_parallelism,
      default_ds,
      backend_path,
      backend_extra,
      backend_operator_semi_naive,
      dd_config,
   };
   Ok((config, signatures, is_parallel, is_ascent_run))
}

fn parse_ds_attr(input: ParseStream) -> syn::Result<DsAttributeContents> {
   let inner;
   braced!(inner in input);
   expect_kw(&inner, "path")?;
   inner.parse::<Token![:]>()?;
   let path_inner;
   braced!(path_inner in inner);
   let path: syn::Path = path_inner.parse()?;
   inner.parse::<Token![,]>()?;
   expect_kw(&inner, "args")?;
   inner.parse::<Token![:]>()?;
   let args = parse_braced_tokens(&inner)?;
   Ok(DsAttributeContents { path, args })
}

fn parse_attrs_block(input: ParseStream) -> syn::Result<Vec<Attribute>> {
   let inner;
   braced!(inner in input);
   Attribute::parse_outer(&inner)
}

fn parse_signature_block(input: ParseStream) -> syn::Result<Signatures> {
   let inner;
   braced!(inner in input);
   expect_kw(&inner, "decl")?;
   let decl: TypeSignature = parse_braced_as(&inner)?;
   let implementation = if inner.peek(Token![impl]) {
      inner.parse::<Token![impl]>()?;
      Some(parse_braced_as::<ImplSignature>(&inner)?)
   } else {
      None
   };
   Ok(Signatures { declaration: decl, implementation })
}

// ---------- Relations parser ------------------------------------------------

struct ParsedRelations {
   full: HashMap<RelationIdentity, IrRelation>,
   all: HashMap<RelationIdentity, HashSet<IrRelation>>,
   lattices: HashMap<RelationIdentity, IrRelation>,
   metadata: HashMap<RelationIdentity, RelationMetadata>,
   by_name: HashMap<String, RelationIdentity>,
}

fn parse_relations(input: ParseStream) -> syn::Result<ParsedRelations> {
   expect_kw(input, "relations")?;
   let body;
   braced!(body in input);

   let mut out = ParsedRelations {
      full: HashMap::new(),
      all: HashMap::new(),
      lattices: HashMap::new(),
      metadata: HashMap::new(),
      by_name: HashMap::new(),
   };

   while !body.is_empty() {
      parse_one_relation(&body, &mut out)?;
   }
   Ok(out)
}

fn parse_one_relation(input: ParseStream, out: &mut ParsedRelations) -> syn::Result<()> {
   let kind_id: Ident = input.parse()?;
   let is_lattice = match kind_id.to_string().as_str() {
      "relation" => false,
      "lattice" => true,
      other => return Err(syn::Error::new(kind_id.span(), format!("expected `relation` or `lattice`, got `{other}`"))),
   };

   let name: Ident = input.parse()?;
   let type_inner;
   parenthesized!(type_inner in input);
   let mut field_types: Vec<Type> = Vec::new();
   while !type_inner.is_empty() {
      let t = parse_braced_as::<Type>(&type_inner)?;
      field_types.push(t);
      if type_inner.peek(Token![,]) {
         type_inner.parse::<Token![,]>()?;
      } else {
         break;
      }
   }

   let rel_ident = RelationIdentity { name: name.clone(), field_types: field_types.clone(), is_lattice };

   // Body of the relation
   let rel_body;
   braced!(rel_body in input);

   // indices [...]
   expect_kw(&rel_body, "indices")?;
   let indices_outer;
   bracketed!(indices_outer in rel_body);
   let mut ir_rels: HashSet<IrRelation> = HashSet::new();
   while !indices_outer.is_empty() {
      let idx = parse_uint_list_bracketed(&indices_outer)?;
      indices_outer.parse::<Token![:]>()?;
      let val_type = parse_val_type(&indices_outer)?;
      ir_rels.insert(IrRelation { relation: rel_ident.clone(), indices: idx, val_type });
      if indices_outer.peek(Token![,]) {
         indices_outer.parse::<Token![,]>()?;
      } else {
         break;
      }
   }
   rel_body.parse::<Token![;]>()?;

   // full_index [...]
   expect_kw(&rel_body, "full_index")?;
   let full_idx_indices = parse_uint_list_bracketed(&rel_body)?;
   rel_body.parse::<Token![;]>()?;
   // Resolve full IrRelation from ir_rels set (matching indices).
   let full_ir = ir_rels
      .iter()
      .find(|ir| ir.indices == full_idx_indices)
      .cloned()
      .ok_or_else(|| syn::Error::new(name.span(), "full_index indices not present in `indices` list"))?;

   // lattice_full_index (REF | -)
   expect_kw(&rel_body, "lattice_full_index")?;
   let lat_full = if rel_body.peek(Token![-]) {
      rel_body.parse::<Token![-]>()?;
      None
   } else {
      let lat_idx = parse_uint_list_bracketed(&rel_body)?;
      let lat_ir = ir_rels
         .iter()
         .find(|ir| ir.indices == lat_idx)
         .cloned()
         .ok_or_else(|| syn::Error::new(name.span(), "lattice_full_index indices not present in `indices` list"))?;
      Some(lat_ir)
   };
   rel_body.parse::<Token![;]>()?;

   // initialization ({expr} | -)
   expect_kw(&rel_body, "initialization")?;
   let initialization: Option<Rc<Expr>> = parse_braced_or_dash_as::<Expr>(&rel_body)?.map(Rc::new);
   rel_body.parse::<Token![;]>()?;

   // attrs { ... }
   expect_kw(&rel_body, "attrs")?;
   let attrs = parse_attrs_block(&rel_body)?;
   rel_body.parse::<Token![;]>()?;

   // ds (DS | -)
   expect_kw(&rel_body, "ds")?;
   let ds_attr = if rel_body.peek(Token![-]) {
      rel_body.parse::<Token![-]>()?;
      None
   } else {
      Some(parse_ds_attr(&rel_body)?)
   };
   rel_body.parse::<Token![;]>()?;

   // compose_input / compose_output (+ | -). Mirror what `emit_relation_decl`
   // serializes — single token each, terminated by `;`.
   expect_kw(&rel_body, "compose_input")?;
   let is_compose_input = parse_plus_or_dash(&rel_body)?;
   rel_body.parse::<Token![;]>()?;
   expect_kw(&rel_body, "compose_output")?;
   let is_compose_output = parse_plus_or_dash(&rel_body)?;
   rel_body.parse::<Token![;]>()?;

   // Post-validation: ds must be None iff lattice.
   if is_lattice && ds_attr.is_some() {
      return Err(syn::Error::new(name.span(), "lattice relations must not carry a `ds` attribute"));
   }
   if !is_lattice && ds_attr.is_none() {
      return Err(syn::Error::new(name.span(), "non-lattice relations must carry a `ds` attribute"));
   }
   if is_lattice && lat_full.is_none() {
      return Err(syn::Error::new(name.span(), "lattice relations must have a `lattice_full_index`"));
   }

   // Register.
   out.full.insert(rel_ident.clone(), full_ir);
   out.all.insert(rel_ident.clone(), ir_rels);
   if let Some(lat) = lat_full {
      out.lattices.insert(rel_ident.clone(), lat);
   }
   out.metadata.insert(rel_ident.clone(), RelationMetadata {
      initialization,
      attributes: Rc::new(attrs),
      ds_attr,
      is_compose_input,
      is_compose_output,
   });
   out.by_name.insert(name.to_string(), rel_ident);
   Ok(())
}

fn parse_val_type(input: ParseStream) -> syn::Result<IndexValType> {
   let id: Ident = input.parse()?;
   match id.to_string().as_str() {
      "reference" => Ok(IndexValType::Reference),
      "direct" => {
         let cols = parse_uint_list_bracketed(input)?;
         Ok(IndexValType::Direct(cols))
      },
      other => Err(syn::Error::new(id.span(), format!("expected `reference` or `direct`, got `{other}`"))),
   }
}

// ---------- SCCs parser -----------------------------------------------------

fn parse_sccs(input: ParseStream, by_name: &HashMap<String, RelationIdentity>) -> syn::Result<Vec<MirScc>> {
   expect_kw(input, "sccs")?;
   let body;
   braced!(body in input);
   let mut sccs = Vec::new();
   while !body.is_empty() {
      sccs.push(parse_one_scc(&body, by_name)?);
   }
   Ok(sccs)
}

fn parse_one_scc(input: ParseStream, by_name: &HashMap<String, RelationIdentity>) -> syn::Result<MirScc> {
   expect_kw(input, "scc")?;
   let _idx: LitInt = input.parse()?; // index is cosmetic; SCCs already in schedule order
   let body;
   braced!(body in input);

   expect_kw(&body, "looping")?;
   let is_looping = parse_bool(&body)?;
   body.parse::<Token![;]>()?;

   expect_kw(&body, "dynamic")?;
   let dynamic_relations = parse_rel_inst_list_block(&body, by_name)?;

   expect_kw(&body, "body_only")?;
   let body_only_relations = parse_rel_inst_list_block(&body, by_name)?;

   expect_kw(&body, "rules")?;
   let rules_body;
   braced!(rules_body in body);
   let mut rules = Vec::new();
   while !rules_body.is_empty() {
      rules.push(parse_rule(&rules_body, by_name)?);
   }

   Ok(MirScc { rules, dynamic_relations, body_only_relations, is_looping })
}

fn parse_rel_inst_list_block(
   input: ParseStream, by_name: &HashMap<String, RelationIdentity>,
) -> syn::Result<HashMap<RelationIdentity, HashSet<IrRelation>>> {
   let body;
   braced!(body in input);
   let mut out: HashMap<RelationIdentity, HashSet<IrRelation>> = HashMap::new();
   while !body.is_empty() {
      let rel_name: Ident = body.parse()?;
      body.parse::<Token![:]>()?;
      let list_inner;
      bracketed!(list_inner in body);
      let rel_ident = by_name
         .get(&rel_name.to_string())
         .ok_or_else(|| syn::Error::new(rel_name.span(), format!("unknown relation `{rel_name}`")))?
         .clone();
      let mut ir_set: HashSet<IrRelation> = HashSet::new();
      while !list_inner.is_empty() {
         let indices = parse_uint_list_bracketed(&list_inner)?;
         // val_type: re-derive to avoid asking it twice. This mirrors
         // IrRelation::new (since serialized RelInstList omits val_type).
         let ir = IrRelation::new(rel_ident.clone(), indices);
         ir_set.insert(ir);
         if list_inner.peek(Token![,]) {
            list_inner.parse::<Token![,]>()?;
         } else {
            break;
         }
      }
      out.insert(rel_ident, ir_set);
      body.parse::<Token![;]>()?;
   }
   Ok(out)
}

// ---------- Rule parser -----------------------------------------------------

fn parse_rule(input: ParseStream, by_name: &HashMap<String, RelationIdentity>) -> syn::Result<MirRule> {
   expect_kw(input, "rule")?;
   // optional RuleMeta
   let (simple_join_start_index, reorderable) = if input.peek(syn::token::Bracket) {
      parse_rule_meta(input)?
   } else {
      (None, false)
   };

   let body;
   braced!(body in input);

   expect_kw(&body, "head")?;
   let head_body;
   braced!(head_body in body);
   let mut head_clause = Vec::new();
   while !head_body.is_empty() {
      head_clause.push(parse_head_clause(&head_body, by_name)?);
      if head_body.peek(Token![,]) {
         head_body.parse::<Token![,]>()?;
      } else {
         break;
      }
   }

   expect_kw(&body, "body")?;
   let body_body;
   braced!(body_body in body);
   let mut body_items: Vec<MirBodyItem> = Vec::new();
   while !body_body.is_empty() {
      let bi = parse_body_item(&body_body, by_name)?;
      body_body.parse::<Token![;]>()?;
      body_items.push(bi);
   }

   Ok(MirRule { head_clause, body_items, simple_join_start_index, reorderable })
}

fn parse_rule_meta(input: ParseStream) -> syn::Result<(Option<usize>, bool)> {
   let inner;
   bracketed!(inner in input);
   let mut simple_join = None;
   let mut reorderable = false;
   while !inner.is_empty() {
      let key: Ident = inner.parse()?;
      inner.parse::<Token![=]>()?;
      match key.to_string().as_str() {
         "simple_join" => {
            let lit: LitInt = inner.parse()?;
            simple_join = Some(lit.base10_parse::<usize>()?);
         },
         "reorderable" => {
            reorderable = parse_bool(&inner)?;
         },
         other => return Err(syn::Error::new(key.span(), format!("unknown rule meta `{other}`"))),
      }
      if inner.peek(Token![,]) {
         inner.parse::<Token![,]>()?;
      } else {
         break;
      }
   }
   Ok((simple_join, reorderable))
}

fn parse_head_clause(input: ParseStream, by_name: &HashMap<String, RelationIdentity>) -> syn::Result<IrHeadClause> {
   let name: Ident = input.parse()?;
   let rel = by_name
      .get(&name.to_string())
      .ok_or_else(|| syn::Error::new(name.span(), format!("unknown relation `{name}`")))?
      .clone();
   let args_paren: syn::token::Paren;
   let args_inner;
   args_paren = parenthesized!(args_inner in input);
   let mut args: Vec<Expr> = Vec::new();
   while !args_inner.is_empty() {
      let e = parse_braced_as::<Expr>(&args_inner)?;
      args.push(e);
      if args_inner.peek(Token![,]) {
         args_inner.parse::<Token![,]>()?;
      } else {
         break;
      }
   }
   // Clause-site span — preserves user-code hygiene context for downstream
   // codegen that uses `hcl.span` when quoting generated idents.
   let span = name.span();
   let args_span = args_paren.span.join();
   Ok(IrHeadClause { rel, args, span, args_span })
}

// ---------- Body item parsers ----------------------------------------------

fn parse_body_item(input: ParseStream, by_name: &HashMap<String, RelationIdentity>) -> syn::Result<MirBodyItem> {
   if peek_kw(input, "clause") {
      Ok(MirBodyItem::Clause(parse_clause_item(input, by_name)?))
   } else if input.peek(Token![for]) {
      Ok(MirBodyItem::Generator(parse_generator_item(input)?))
   } else if input.peek(Token![if]) || peek_kw(input, "if_let") || input.peek(Token![let]) {
      Ok(MirBodyItem::Cond(parse_cond_clause(input)?))
   } else if peek_kw(input, "agg") {
      Ok(MirBodyItem::Agg(parse_agg_item(input, by_name)?))
   } else {
      Err(input.error("expected a body item (clause/for/if/if_let/let/agg)"))
   }
}

fn parse_clause_item(
   input: ParseStream, by_name: &HashMap<String, RelationIdentity>,
) -> syn::Result<MirBodyClause> {
   expect_kw(input, "clause")?;
   let name: Ident = input.parse()?;
   let rel_ident = by_name
      .get(&name.to_string())
      .ok_or_else(|| syn::Error::new(name.span(), format!("unknown relation `{name}`")))?
      .clone();

   let (indices, version, val_type) = parse_clause_meta(input)?;
   let args_paren: syn::token::Paren;
   let args_inner;
   args_paren = parenthesized!(args_inner in input);
   let mut args: Vec<Expr> = Vec::new();
   while !args_inner.is_empty() {
      args.push(parse_braced_as::<Expr>(&args_inner)?);
      if args_inner.peek(Token![,]) {
         args_inner.parse::<Token![,]>()?;
      } else {
         break;
      }
   }

   let cond_clauses = if input.peek(Token![where]) {
      input.parse::<Token![where]>()?;
      let tail_inner;
      braced!(tail_inner in input);
      let mut conds = Vec::new();
      while !tail_inner.is_empty() {
         conds.push(parse_cond_clause(&tail_inner)?);
         if tail_inner.peek(Token![,]) {
            tail_inner.parse::<Token![,]>()?;
         } else {
            break;
         }
      }
      conds
   } else {
      Vec::new()
   };

   let ir_name = ir_name_for_rel_indices(&rel_ident.name, &indices);
   let is_full_index = indices.len() == rel_ident.field_types.len();
   let is_no_index = indices.is_empty();
   // `is_no_index` is a field set from `indices.is_empty()`; keep construction
   // aligned with `MirRelation::from`.
   let _ = is_no_index;
   let rel = MirRelation {
      relation: rel_ident,
      indices,
      ir_name,
      version,
      is_full_index,
      is_no_index,
      val_type,
   };
   // `rel_args_span`: span of the emitted relation name (carries both the
   // user call-site position and its original hygiene context — preserved
   // across the emit→parse round-trip because we emitted the name at that
   // very span in `emit_clause_item`).
   let rel_args_span = name.span();
   // `args_span`: span of the `(…)` delimiter pair, matching what HIR gave us
   // originally (`bcl.args.span()`).
   let args_span = args_paren.span.join();
   Ok(MirBodyClause { rel, args, rel_args_span, args_span, cond_clauses })
}

fn parse_clause_meta(input: ParseStream) -> syn::Result<(Vec<usize>, MirRelationVersion, IndexValType)> {
   let inner;
   bracketed!(inner in input);
   let mut indices: Option<Vec<usize>> = None;
   let mut version: Option<MirRelationVersion> = None;
   let mut val_type: Option<IndexValType> = None;
   while !inner.is_empty() {
      let key: Ident = inner.parse()?;
      inner.parse::<Token![=]>()?;
      match key.to_string().as_str() {
         "indices" => indices = Some(parse_uint_list_bracketed(&inner)?),
         "version" => version = Some(parse_version(&inner)?),
         "val_type" => val_type = Some(parse_val_type(&inner)?),
         other => return Err(syn::Error::new(key.span(), format!("unknown clause meta `{other}`"))),
      }
      if inner.peek(Token![,]) {
         inner.parse::<Token![,]>()?;
      } else {
         break;
      }
   }
   Ok((
      indices.ok_or_else(|| input.error("clause meta missing `indices`"))?,
      version.ok_or_else(|| input.error("clause meta missing `version`"))?,
      val_type.ok_or_else(|| input.error("clause meta missing `val_type`"))?,
   ))
}

fn parse_version(input: ParseStream) -> syn::Result<MirRelationVersion> {
   let id: Ident = input.parse()?;
   match id.to_string().as_str() {
      "total" => Ok(MirRelationVersion::Total),
      "delta" => Ok(MirRelationVersion::Delta),
      "total_delta" => Ok(MirRelationVersion::TotalDelta),
      "new" => Ok(MirRelationVersion::New),
      other => Err(syn::Error::new(id.span(), format!("unknown version `{other}`"))),
   }
}

fn parse_generator_item(input: ParseStream) -> syn::Result<GeneratorNode> {
   input.parse::<Token![for]>()?;
   let pat: Pat = parse_braced_as_pat(input)?;
   input.parse::<Token![in]>()?;
   let expr: Expr = parse_braced_as::<Expr>(input)?;
   // GeneratorNode derives Parse from `for PAT in EXPR`; reconstruct via quote + parse2.
   syn::parse2::<GeneratorNode>(quote! { for #pat in #expr })
}

fn parse_cond_clause(input: ParseStream) -> syn::Result<CondClause> {
   if input.peek(Token![if]) {
      input.parse::<Token![if]>()?;
      let expr: Expr = parse_braced_as::<Expr>(input)?;
      let cl: IfClause = syn::parse2(quote! { if #expr })?;
      Ok(CondClause::If(cl))
   } else if peek_kw(input, "if_let") {
      expect_kw(input, "if_let")?;
      let pat: Pat = parse_braced_as_pat(input)?;
      input.parse::<Token![=]>()?;
      let expr: Expr = parse_braced_as::<Expr>(input)?;
      let cl: IfLetClause = syn::parse2(quote! { if let #pat = #expr })?;
      Ok(CondClause::IfLet(cl))
   } else if input.peek(Token![let]) {
      input.parse::<Token![let]>()?;
      let pat: Pat = parse_braced_as_pat(input)?;
      input.parse::<Token![=]>()?;
      let expr: Expr = parse_braced_as::<Expr>(input)?;
      let cl: LetClause = syn::parse2(quote! { let #pat = #expr })?;
      Ok(CondClause::Let(cl))
   } else {
      Err(input.error("expected `if`/`if_let`/`let`"))
   }
}

fn parse_braced_as_pat(input: ParseStream) -> syn::Result<Pat> {
   let inner;
   braced!(inner in input);
   Pat::parse_multi(&inner)
}

fn parse_agg_item(input: ParseStream, by_name: &HashMap<String, RelationIdentity>) -> syn::Result<IrAggClause> {
   expect_kw(input, "agg")?;
   let pat: Pat = parse_braced_as_pat(input)?;
   input.parse::<Token![=]>()?;
   let aggregator: Expr = parse_braced_as::<Expr>(input)?;
   let bound_inner;
   parenthesized!(bound_inner in input);
   let mut bound_args: Vec<Ident> = Vec::new();
   while !bound_inner.is_empty() {
      bound_args.push(bound_inner.parse::<Ident>()?);
      if bound_inner.peek(Token![,]) {
         bound_inner.parse::<Token![,]>()?;
      } else {
         break;
      }
   }
   input.parse::<Token![in]>()?;
   let rel_name: Ident = input.parse()?;
   let rel_ident = by_name
      .get(&rel_name.to_string())
      .ok_or_else(|| syn::Error::new(rel_name.span(), format!("unknown relation `{rel_name}`")))?
      .clone();
   let (indices, _version, val_type) = parse_clause_meta(input)?;
   // Agg has no version semantically — grammar emits `total` as a filler.
   let rel = IrRelation { relation: rel_ident, indices, val_type };
   let args_inner;
   parenthesized!(args_inner in input);
   let mut rel_args: Vec<Expr> = Vec::new();
   while !args_inner.is_empty() {
      rel_args.push(parse_braced_as::<Expr>(&args_inner)?);
      if args_inner.peek(Token![,]) {
         args_inner.parse::<Token![,]>()?;
      } else {
         break;
      }
   }
   let span = pat.span();
   Ok(IrAggClause { span, pat, aggregator, bound_args, rel, rel_args })
}
