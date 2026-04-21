//! Differential-dataflow codegen backend.
//!
//! Generates code against the runtime exposed at `::ascent::dd` (the
//! `ascent_dd_runtime` crate re-export). The surface shape (`struct` with
//! `pub <rel>: Vec<Tup>` fields, `Default::default()`, `run()`) matches the
//! batch backend so ascent programs are swap-in compatible at the user API.
//!
//! Phase status:
//!   0: struct + noop `run()`                                        (always)
//!   1: non-recursive rules, all-ident clause/head args
//!   2: recursion via DD `Variable` / `scope.iterative`              <-- HERE
//!   3+: conditions / generators / negation / aggregation / lattices
//!
//! When the program is outside the current phase envelope, `run()` falls back
//! to a noop so the struct still compiles — tests for unsupported features
//! stay `#[ignore]`'d rather than block the build.
//!
//! # Module layout
//!
//! The bulk of lowering sits in sibling modules; this file only owns the
//! proc-macro entry, per-mode dispatch, and the structure/default emission
//! that's shared across modes.
//!
//! - [`dfg`]       — per-operator token emission (`.map`, `.join`, …)
//! - [`utils`]     — rel-ident / tuple / shared-arrangement helpers
//! - [`analyses`]  — SCC-level pre-computed structure
//! - [`rule_body`] — one MIR rule → `Collection` of bound-var tuples
//! - [`scc`]       — one `MirScc` → tokens inside the worker closure
//! - [`run_body`]  — `run()` body wiring + batch-mode top-level entry
//! - [`session`]   — `<Name>Session` struct emission (incremental API)

#![allow(dead_code)]

extern crate proc_macro;

mod analyses;
mod dfg;
mod embed;
mod rule_body;
mod rule_fn;
mod run_body;
mod scc;
mod session;
mod utils;

use itertools::Itertools;
use proc_macro2::{Ident, TokenStream};
use quote::quote;

use ascent_mir::utils::tuple_type;
use ascent_mir::{AscentMir, MirBodyItem};

// ============================================================================
// Proc-macro entry point — frontend invokes us via
// `::ascent_codegen_dd::compile_mir! { mir_v1 { … } }` and we return runtime Rust.
// ============================================================================

/// Consume a MIR v1 token stream and emit the DD-backed runtime code.
///
/// Invoked at the second expansion stage. The frontend (`ascent!` /
/// `ascent_run!` et al.) parses user syntax → builds MIR → emits the call
/// `::ascent_codegen_dd::compile_mir! { mir_v1 { … } }` verbatim. This macro
/// then parses the MIR, delegates to `compile_mir_dd`, and returns the final
/// generated code.
///
/// # Debug dump
///
/// Set `ASCENT_DD_DUMP_RUST=<path>` to append every expansion's output
/// Rust to that file. Useful for auditing codegen quality (closure count,
/// clone count, generic instantiations) without re-running the macro by
/// hand. Value `-` prints to stderr. No effect on compiled output.
#[proc_macro]
pub fn compile_mir(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
   match ::ascent_mir::parse_mir(input.into()) {
      Ok((mir, is_ascent_run)) => {
         let code = compile_mir_dd(&mir, is_ascent_run);
         maybe_dump_generated_rust(&mir, &code);
         code.into()
      },
      Err(err) => err.to_compile_error().into(),
   }
}

/// If `ASCENT_DD_DUMP_RUST` is set, append the generated Rust to the file
/// (or stderr when `-`). Prefixes with the program's struct name and a
/// short MIR summary so multi-program projects are greppable.
fn maybe_dump_generated_rust(mir: &AscentMir, code: &TokenStream) {
   let Ok(target) = std::env::var("ASCENT_DD_DUMP_RUST") else { return };

   let backend_path_str = {
      let p = &mir.config.backend_path;
      quote! { #p }.to_string()
   };
   let header = format!(
      "// === {struct_name} :: {n_rels} rels / {n_sccs} SCCs / \
       {n_rules} rules (backend_path={backend_path_str}) ===\n",
      struct_name = mir.signatures.declaration.ident,
      n_rels = mir.relations_ir_relations.len(),
      n_sccs = mir.sccs.len(),
      n_rules = mir.sccs.iter().map(|s| s.rules.len()).sum::<usize>(),
   );

   // Try to pretty-print via rustfmt subprocess; fall back to raw tokens.
   let pretty = rustfmt_tokenstream(code).unwrap_or_else(|_| code.to_string());
   let out = format!("{header}{pretty}\n\n");

   if target == "-" {
      eprint!("{out}");
      return;
   }
   use std::io::Write;
   if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&target) {
      let _ = f.write_all(out.as_bytes());
   }
}

/// Spawn rustfmt and pipe the tokens through it. Returns `Err` if rustfmt
/// is missing, errors, or the tokens don't form a valid file.
fn rustfmt_tokenstream(code: &TokenStream) -> std::io::Result<String> {
   use std::io::Write;
   use std::process::{Command, Stdio};
   // Wrap in a dummy fn — the code may be either a block (ascent_run!) or
   // a sequence of items (ascent!). Wrapping in a block covers both cases
   // so rustfmt always sees something parseable.
   let wrapped = format!("fn __dd_generated() {{\n{}\n}}\n", code);
   let mut child = Command::new("rustfmt")
      .arg("--edition=2021")
      .arg("--emit=stdout")
      .stdin(Stdio::piped())
      .stdout(Stdio::piped())
      .stderr(Stdio::null())
      .spawn()?;
   child.stdin.as_mut().unwrap().write_all(wrapped.as_bytes())?;
   let out = child.wait_with_output()?;
   if !out.status.success() {
      return Err(std::io::Error::new(std::io::ErrorKind::Other, "rustfmt failed"));
   }
   Ok(String::from_utf8_lossy(&out.stdout).to_string())
}

// ---------------------------------------------------------------------------
// Phase envelope check
// ---------------------------------------------------------------------------

/// Returns `None` if the MIR is within the current backend envelope:
/// Phases 1, 2 and 3a are supported (non-recursive + recursive rules, rich
/// clause args — wildcards, literals, repeated vars — and `if`/`let`
/// conditions). Rejects: `if let`, generators, negation, aggregation, and
/// lattice relations.
pub(crate) fn phase1_blocker(mir: &AscentMir) -> Option<String> {
   for scc in &mir.sccs {
      for rule in &scc.rules {
         for bi in &rule.body_items {
            match bi {
               MirBodyItem::Clause(cl) => {
                  // Lattice bodies are supported: rules read the materialized
                  // (key-col-tuple..., lat_val) form as a regular set-diff
                  // Collection. See `phase6_materialize_lattice_rel` for the
                  // reduce step that keeps each key's current join.
                  let _ = cl.rel.relation.is_lattice;
                  let _ = &cl.cond_clauses;
               },
               MirBodyItem::Generator(_) => {
                  // supported in Phase 3c via `.flat_map`
               },
               MirBodyItem::Cond(_) => {
                  // if / let / if let — all supported in Phase 3
               },
               MirBodyItem::Agg(_agg) => {
                  // `not` → `.antijoin` (Phase 3). Other aggregators → `.reduce`
                  // (Phase 5). Both handled in `compile_rule_body_phase1`.
               },
            }
         }
         for hcl in &rule.head_clause {
            // Lattice heads are supported: contributions are concat'd into
            // the head accumulator; the reduce happens at materialization.
            let _ = hcl.rel.is_lattice;
         }
      }
   }
   None
}

// ---------------------------------------------------------------------------
// Top-level codegen
// ---------------------------------------------------------------------------

fn compile_mir_dd(mir: &AscentMir, is_ascent_run: bool) -> TokenStream {
   // DD's `batch` vs `incremental` mode is a backend-private sub-config,
   // passed through `#![backend(dd, mode = "batch")]` → MIR
   // `backend_extra` tokens. Parse here — the frontend doesn't need to
   // know DD's mode space.
   let mode = match parse_dd_mode(&mir.config.backend_extra) {
      Ok(mode) => mode,
      Err(e) => return e.to_compile_error(),
   };
   // Per-rule fn extraction: rules that match simple patterns get lifted
   // out to free `fn` items at module level so rustc's codegen-unit
   // scheduler can parallelize LLVM-backend work across rules. Opt-in
   // via env var during rollout; default off until we measure on enough
   // programs. Only makes sense in item-context (`ascent!`, not
   // `ascent_run!`) because `ascent_run!` expands inside a `{ ... }`
   // block where free fns aren't legal (they'd be inside a fn body).
   let rule_fn_enabled = !is_ascent_run
      && std::env::var("ASCENT_DD_RULE_FNS").ok().map_or(false, |v| v != "0" && !v.is_empty());
   rule_fn::reset(mir.signatures.declaration.ident.clone(), rule_fn_enabled);
   let inner = match mode {
      DdMode::Incremental => compile_mir_dd_incremental(mir, is_ascent_run),
      DdMode::Batch => match run_body::compile_mir_dd_batch(mir, is_ascent_run) {
         Ok(ts) => ts,
         Err(e) => e.to_compile_error(),
      },
   };
   let rule_fn_items = rule_fn::take();
   quote! {
      #rule_fn_items
      #inner
   }
}

/// DD mode discriminator, private to this backend.
#[derive(Clone, Copy, Default)]
enum DdMode {
   #[default]
   Incremental,
   Batch,
}

/// Parse `backend_extra` tokens for DD's `mode = "batch"|"incremental"`.
/// Empty tokens → default (incremental). Any other unrecognised form is
/// an error.
fn parse_dd_mode(tokens: &TokenStream) -> syn::Result<DdMode> {
   if tokens.is_empty() {
      return Ok(DdMode::default());
   }
   struct ModeArgs(Option<DdMode>);
   impl syn::parse::Parse for ModeArgs {
      fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
         if input.is_empty() {
            return Ok(ModeArgs(None));
         }
         let key: Ident = input.parse()?;
         if key != "mode" {
            return Err(syn::Error::new(key.span(), format!("unknown DD arg `{key}`; expected `mode`")));
         }
         input.parse::<syn::Token![=]>()?;
         let lit: syn::LitStr = input.parse()?;
         let mode = match lit.value().as_str() {
            "batch" => DdMode::Batch,
            "incremental" => DdMode::Incremental,
            other => return Err(syn::Error::new(lit.span(), format!("unknown DD mode `{other}`"))),
         };
         Ok(ModeArgs(Some(mode)))
      }
   }
   let parsed: ModeArgs = syn::parse2(tokens.clone())?;
   Ok(parsed.0.unwrap_or_default())
}

fn compile_mir_dd_incremental(mir: &AscentMir, is_ascent_run: bool) -> TokenStream {
   let blocker = phase1_blocker(mir);

   // Struct + Default are phase-agnostic.
   // For `ascent_run!`, relation init expressions must be evaluated at the
   // macro call site (where they can capture outer locals), NOT inside the
   // generated `Default::default()` which is a fn item.
   let struct_and_default = emit_struct_and_default(mir, !is_ascent_run);

   // run() either dispatches to the real dataflow or the Phase 0 noop with
   // a diagnostic comment. Field access target is `self` for the method,
   // `__run_res` for `ascent_run!`. `run_body` is resolved later once we
   // know whether a Session is available for this program (then `run` is
   // a thin wrapper around `Session::commit`, dedup'd).
   let self_target: TokenStream = quote!(self);

   let (ty_impl_generics, ty_ty_generics, ty_where_clause) = mir.signatures.split_ty_generics_for_impl();
   let (impl_impl_generics, impl_ty_generics, impl_where_clause) = mir.signatures.split_impl_generics_for_impl();
   let _ = (ty_impl_generics, ty_where_clause); // kept for symmetry with batch backend

   let ty_signature = &mir.signatures.declaration;
   let struct_name = &ty_signature.ident;

   let summary = format!("DD backend — {} relations, phase1_blocker: {:?}", mir.relations_ir_relations.len(), blocker);
   let summary_fn = if is_ascent_run {
      quote! { pub fn summary(&self) -> &'static str { #summary } }
   } else {
      quote! { pub fn summary() -> &'static str { #summary } }
   };

   // Emit `<Name>Session` only when:
   //   - program is within backend envelope (no `blocker`),
   //   - item-context (`ascent!`, not `ascent_run!`) — `fn new()` can't
   //     capture locals,
   //   - no generics on the struct — propagating them through the whole
   //     session machinery is possible but non-trivial (would need Send +
   //     'static bounds added automatically and correct generic spelling
   //     in every `impl`). Generic programs use the batch `run()` API.
   let has_generics = !mir.signatures.declaration.generics.params.is_empty();
   let emit_session_for_this_program = blocker.is_none() && !is_ascent_run && !has_generics;
   let session_items =
      if emit_session_for_this_program { session::emit_session(mir) } else { TokenStream::new() };
   // `build_in_scope` shares session-mode's envelope — same blocker rules,
   // same no-generics limitation. Emitted regardless of whether the user
   // calls it; dead code elides.
   let compose_items =
      if emit_session_for_this_program { embed::emit_compose(mir) } else { TokenStream::new() };

   let session_accessor = if emit_session_for_this_program {
      let session_name = session::session_struct_name(struct_name);
      quote! {
         /// Start a live incremental session: returns a long-lived handle
         /// whose `<rel>_insert/remove` + `commit()` let you push updates
         /// and observe output deltas without re-running from scratch.
         pub fn session() -> #session_name { #session_name::new() }
      }
   } else {
      TokenStream::new()
   };

   // `run()` uses `execute_batch` directly (multi-worker capable). We
   // previously dedup'd by routing `run()` through `Session::commit()`
   // for the session-emitted path, but Session is single-worker — that
   // bypassed `execute_batch`'s parallel timely runtime and made `run()`
   // silently serial regardless of `ASCENT_DD_WORKERS`. Restoring the
   // direct `execute_batch` emission here is what makes `run()` actually
   // scale across cores; code-size regains are a follow-up once Session
   // itself supports multi-worker.
   let run_body = match &blocker {
      None => run_body::phase1_run_body(mir, &self_target),
      Some(reason) => {
         let comment = format!("dd backend: falling back to noop run() — {reason}");
         quote! {
            // #comment
            let _ = #comment;
         }
      },
   };
   let _ = emit_session_for_this_program; // still controls Session struct emission, just not `run()` body
   let run_func = if is_ascent_run {
      quote! {}
   } else {
      quote! {
         #[doc = "Runs the Ascent program to a fixed point (DD backend)."]
         pub fn run(&mut self) {
            #![allow(unused_imports, unused_mut, unused_variables, clippy::all)]
            #run_body
         }
      }
   };

   let methods = quote! {
      impl #impl_impl_generics #struct_name #impl_ty_generics #impl_where_clause {
         #run_func
         #session_accessor
         #summary_fn
         pub fn relation_sizes_summary(&self) -> ::std::string::String {
            ::std::string::String::new()
         }
         pub fn scc_times_summary(&self) -> ::std::string::String {
            ::std::string::String::new()
         }
      }
   };

   let assembled = quote! {
      #struct_and_default
      #methods
      #session_items
      #compose_items
   };

   if !is_ascent_run {
      assembled
   } else {
      let inline_target: TokenStream = quote!(__run_res);
      let run_inline = match &blocker {
         None => run_body::phase1_run_body(mir, &inline_target),
         Some(_) => quote! {},
      };
      // Relation initialisations — evaluated here in the enclosing fn body
      // (so they can reference local parameters), stored into the struct
      // BEFORE `run_inline` copies the inputs into the dataflow.
      let mut rel_inits_inline = vec![];
      for rel in mir.relations_ir_relations.keys().sorted_by_key(|r| &r.name) {
         if let Some(init_expr) = mir.relations_metadata[rel].initialization.as_deref() {
            let name = &rel.name;
            rel_inits_inline.push(quote! {
               __run_res.#name = (#init_expr).into_iter().collect();
            });
         }
      }
      quote! {
         {{
            #![allow(unused_imports, unused_mut, unused_variables, clippy::all)]
            #assembled
            let mut __run_res: #struct_name #ty_ty_generics = #struct_name::default();
            #(#rel_inits_inline)*
            { #run_inline }
            __run_res
         }}
      }
   }
}

pub(crate) fn emit_struct_and_default(mir: &AscentMir, include_rel_init_in_default: bool) -> TokenStream {
   let mut relation_fields = vec![];
   let mut field_defaults = vec![];
   for rel in mir.relations_ir_relations.keys().sorted_by_key(|r| &r.name) {
      let name = &rel.name;
      let tuple_ty = tuple_type(&rel.field_types);
      let rel_attrs = &mir.relations_metadata[rel].attributes;
      relation_fields.push(quote! {
         #(#rel_attrs)*
         pub #name: ::std::vec::Vec<#tuple_ty>,
      });
      let init = mir.relations_metadata[rel].initialization.as_deref().cloned();
      let default = match (init, include_rel_init_in_default) {
         (Some(init_expr), true) => quote! { #name: (#init_expr).into_iter().collect(), },
         _ => quote! { #name: ::std::vec::Vec::new(), },
      };
      field_defaults.push(default);
   }

   let ty_signature = &mir.signatures.declaration;
   let (ty_impl_generics, _ty_ty_generics, ty_where_clause) = mir.signatures.split_ty_generics_for_impl();
   let (impl_impl_generics, impl_ty_generics, impl_where_clause) = mir.signatures.split_impl_generics_for_impl();
   let vis = &ty_signature.visibility;
   let struct_name = &ty_signature.ident;
   let struct_attrs = &ty_signature.attrs;

   quote! {
      #(#struct_attrs)*
      #vis struct #struct_name #ty_impl_generics #ty_where_clause {
         #(#relation_fields)*
      }
      impl #impl_impl_generics ::std::default::Default for #struct_name #impl_ty_generics #impl_where_clause {
         fn default() -> Self {
            #struct_name { #(#field_defaults)* }
         }
      }
   }
}

