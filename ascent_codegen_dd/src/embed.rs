//! DD-level composition (`build_in_scope`) emission.
//!
//! Alongside `::session()` (owning worker, incremental commit loop) and
//! `::run()` (one-shot batch-driver), we emit a third entry point that
//! runs the program's dataflow **inside a caller-provided scope**. Inputs
//! arrive as DD `VecCollection`s from the caller; outputs are returned as
//! `VecCollection`s (no Sink indirection). Timestamps are `u32` — matching
//! session mode — so callers that build on top of `build_session_worker`
//! or any other `u32`-stamped dataflow can compose two programs directly:
//!
//! ```ignore
//! build_session_worker(|scope| {
//!     let edge_input = /* InputSession<u32, (i32,i32), i32>::new().to_collection(scope) */;
//!     let a = ProgA::build_in_scope(scope, ProgAComposeInputs { edge: edge_input });
//!     let b = ProgB::build_in_scope(scope, ProgBComposeInputs { link: a.path });
//!     // ...
//! });
//! ```
//!
//! No new attribute gates this — the codegen emits it alongside the
//! session struct whenever the program is within-envelope (no blocker)
//! and has no generics. Dead code is elided by Rust if the caller only
//! uses `::session()` or `::run()`.

use proc_macro2::{Ident, TokenStream};
use quote::quote;

use ascent_mir::utils::tuple_type;
use ascent_mir::AscentMir;
use itertools::Itertools;

use crate::run_body::emit_closure_body;
use crate::utils::relation_coll_var;

pub(crate) fn compose_inputs_name(base: &Ident) -> Ident {
   Ident::new(&format!("{}ComposeInputs", base), base.span())
}

pub(crate) fn compose_outputs_name(base: &Ident) -> Ident {
   Ident::new(&format!("{}ComposeOutputs", base), base.span())
}

pub(crate) fn emit_compose(mir: &AscentMir) -> TokenStream {
   let ty_signature = &mir.signatures.declaration;
   let base_name = &ty_signature.ident;
   let inputs_name = compose_inputs_name(base_name);
   let outputs_name = compose_outputs_name(base_name);
   let sorted_rels = mir.relations_ir_relations.keys().sorted_by_key(|r| &r.name).collect_vec();
   // Zero-relation programs would produce empty `Inputs<S>` / `Outputs<S>`
   // structs where `S` is never used → E0392 "unused type parameter" and
   // E0283 at the call site from unconstrained type inference. The compose
   // API is meaningless for a program with no relations anyway, so skip.
   if sorted_rels.is_empty() {
      return TokenStream::new();
   }

   // Split inputs / outputs from per-relation `#[input]` / `#[output]`
   // annotations. Unannotated relations are purely internal (neither
   // exposed as caller-seedable nor caller-readable). A relation with
   // BOTH attributes is a caller-seedable derived relation — appears in
   // both structs; the caller's input seed and all derivations `.concat()`
   // into the same accumulator.
   let input_rels: Vec<_> =
      sorted_rels.iter().copied().filter(|r| mir.relations_metadata[*r].is_compose_input).collect();
   let output_rels: Vec<_> =
      sorted_rels.iter().copied().filter(|r| mir.relations_metadata[*r].is_compose_output).collect();

   // Entirely unannotated program = nothing to compose. No Inputs/Outputs
   // structs, no `build_in_scope`. Skip emission — avoids degenerate
   // zero-field `Inputs<S>` / `Outputs<S>` and the E0392 that follows.
   if input_rels.is_empty() && output_rels.is_empty() {
      return TokenStream::new();
   }

   // Per-relation Input/Output field declarations.
   let input_fields: Vec<TokenStream> = input_rels
      .iter()
      .map(|rel| {
         let name = &rel.name;
         let tup_ty = tuple_type(&rel.field_types);
         quote! {
            pub #name: ::ascent::dd::differential_dataflow::VecCollection<S, #tup_ty, i32>,
         }
      })
      .collect();
   let output_fields: Vec<TokenStream> = output_rels
      .iter()
      .map(|rel| {
         let name = &rel.name;
         let tup_ty = tuple_type(&rel.field_types);
         quote! {
            pub #name: ::ascent::dd::differential_dataflow::VecCollection<S, #tup_ty, i32>,
         }
      })
      .collect();

   // Guard against zero-field structs — `S` becomes an unused type parameter
   // (E0392) and callers can't constrain it (E0283). If one side is empty,
   // inject a phantom field to anchor `S`. This can happen when a program
   // has only `#[input]` relations or only `#[output]` relations.
   let (input_fields, output_fields) = {
      let phantom = quote! {
         #[doc(hidden)] pub __phantom: ::std::marker::PhantomData<S>,
      };
      let input_fields = if input_fields.is_empty() {
         vec![phantom.clone()]
      } else {
         input_fields
      };
      let output_fields = if output_fields.is_empty() { vec![phantom] } else { output_fields };
      (input_fields, output_fields)
   };

   let body = emit_compose_body(mir, &sorted_rels, &input_rels, &output_rels, &outputs_name);

   quote! {
      /// Input `Collection` per relation — every relation has one, pass an
      /// empty `Collection` for relations the caller doesn't drive directly.
      /// Rules that derive into a relation will `.concat()` into whatever
      /// Collection you pass here (so the input acts as a seed, not a
      /// replacement).
      pub struct #inputs_name<S: ::ascent::dd::timely::dataflow::Scope<Timestamp = u32>> {
         #(#input_fields)*
      }
      /// Output `Collection` per relation — deduped (via `.threshold` / lattice
      /// `reduce`) and ready to feed into a downstream program or an external
      /// sink.
      pub struct #outputs_name<S: ::ascent::dd::timely::dataflow::Scope<Timestamp = u32>> {
         #(#output_fields)*
      }

      impl #base_name {
         /// Build this program's dataflow INSIDE the caller-provided scope
         /// and return each relation's final `Collection`. Composes with any
         /// other `build_in_scope` program that shares the same scope —
         /// hand one program's `Outputs.<rel>` to another program's
         /// `Inputs.<rel>` and DD wires them directly, no delta handoff.
         ///
         /// Timestamp is `u32` (matching `::session()`); callers using
         /// `build_session_worker` / any `u32`-stamped scope can use this.
         pub fn build_in_scope<S>(
            scope: &mut S,
            inputs: #inputs_name<S>,
         ) -> #outputs_name<S>
         where
            S: ::ascent::dd::timely::dataflow::Scope<Timestamp = u32>
               + ::ascent::dd::differential_dataflow::input::Input,
         {
            #![allow(unused_imports, unused_mut, unused_variables, clippy::all)]
            #body
         }
      }
   }
}

fn emit_compose_body(
   mir: &AscentMir,
   sorted_rels: &[&ascent_mir::RelationIdentity],
   input_rels: &[&ascent_mir::RelationIdentity],
   output_rels: &[&ascent_mir::RelationIdentity],
   outputs_name: &Ident,
) -> TokenStream {
   // Per-rel seed:
   //   - Annotated `#[input]`: pull the caller's Collection out of `inputs`.
   //   - Unannotated / `#[output]`-only: seed an empty Collection so the
   //     accumulator starts at zero and derivation rules `.concat()` into it.
   let input_names: ::std::collections::HashSet<String> =
      input_rels.iter().map(|r| r.name.to_string()).collect();

   let mut setup = TokenStream::new();
   for rel in sorted_rels {
      let name = &rel.name;
      let tup_ty = tuple_type(&rel.field_types);
      let coll = relation_coll_var(&rel.name);
      if input_names.contains(&rel.name.to_string()) {
         setup.extend(quote! { let #coll = inputs.#name; });
      } else {
         // Empty seed — derivation rules will concat into this. Use
         // `new_collection_from_raw` with an empty iterator so the
         // Collection has the right diff type (`i32`) and scope.
         setup.extend(quote! {
            let #coll = {
               use ::ascent::dd::differential_dataflow::input::Input;
               let (_seed, coll) = scope.new_collection_from_raw::<#tup_ty, i32, _>(
                  ::std::iter::empty::<(#tup_ty, u32, i32)>(),
               );
               coll
            };
         });
      }
   }

   // Silence unused-variable warning if caller supplies an input relation
   // that the program has no rules consuming (genuinely weird but allowed —
   // composition tolerance for "pre-declared interface, partial impl").
   // Note: this isn't reached today since we always bind `__<rel>_coll`.
   let _ = &input_names;

   // Shared core with a `finalize` that captures each rel's cleaned final
   // Collection into a uniquely-named local (`__<rel>_out`) instead of
   // attaching to a Sink. No probe, no worker_index — this is all inside
   // the caller's scope, which owns those concerns. We capture final
   // expressions for EVERY relation (so we can drop-or-use below) to keep
   // the shared closure body unchanged.
   let (body, hoists, _hoist_count) = emit_closure_body(
      mir,
      sorted_rels,
      |rel| {
         // Mirror session-mode init: rebind as mut for `.concat` accumulation.
         let coll = relation_coll_var(&rel.name);
         quote! { let mut #coll = #coll; }
      },
      |rel, final_expr| {
         let out_var = out_var_name(&rel.name);
         // Prefix unused finals with `_` so internal relations (neither
         // input nor output) don't trigger unused-variable warnings.
         if output_rels.iter().any(|r| r.name == rel.name) {
            quote! { let #out_var = #final_expr; }
         } else {
            let silenced = Ident::new(&format!("_{}", out_var), out_var.span());
            quote! { let #silenced = #final_expr; }
         }
      },
      false,
   );

   // Assemble the Outputs struct literal from `__<rel>_out` locals.
   // If there are no output relations we still need to return an Outputs
   // struct — in that case only the phantom field exists.
   let output_assigns: Vec<TokenStream> = if output_rels.is_empty() {
      vec![quote! { __phantom: ::std::marker::PhantomData, }]
   } else {
      output_rels
         .iter()
         .map(|rel| {
            let name = &rel.name;
            let out_var = out_var_name(&rel.name);
            quote! { #name: #out_var, }
         })
         .collect()
   };

   quote! {
      #setup
      #hoists
      #body
      #outputs_name { #(#output_assigns)* }
   }
}

fn out_var_name(rel: &Ident) -> Ident { Ident::new(&format!("__{}_out", rel), rel.span()) }
