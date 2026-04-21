//! Session codegen (incremental API).
//!
//! For each user-declared program `pub struct Tc;` we emit a sibling
//! `pub struct TcSession` that owns a long-lived timely worker and per-
//! relation input sessions + output sinks + running net-multiplicity state.
//! Users drive DD directly from any control flow — sync loop, mpsc, tokio.
//!
//! `commit()` advances every input to the next epoch and steps the worker
//! to that frontier. The dataflow construction inside `Session::new` is the
//! same shape as `run()`'s — the only difference is that the InputSessions
//! and Sinks land on the struct instead of being captured into a one-shot
//! closure.

use proc_macro2::{Ident, TokenStream};
use quote::quote;

use ascent_mir::utils::tuple_type;
use ascent_mir::{AscentMir, RelationIdentity};
use itertools::Itertools;

use crate::run_body::emit_closure_body;
use crate::utils::relation_coll_var;

pub(crate) fn session_struct_name(base: &Ident) -> Ident { Ident::new(&format!("{}Session", base), base.span()) }

/// `run()` as a Session wrapper. Emits the SAME observable behavior as
/// `phase1_run_body` — seed the dataflow with `self.<rel>`, run to fixed
/// point, write final relation contents back to `self` — but delegates
/// the dataflow construction to `Session::new()`. This cuts generated
/// code roughly in half for programs that also emit a session (the
/// common case).
fn run_via_session_body(mir: &AscentMir, target: &TokenStream, session_name: &Ident) -> TokenStream {
   let sorted_rels = mir.relations_ir_relations.keys().sorted_by_key(|r| &r.name).collect_vec();

   // Drain each `self.<rel>: Vec<Tup>` into the session's InputSession.
   // `drain(..)` moves ownership out of self — batch `run()` semantics
   // have always moved input into the dataflow and written it back out.
   let seed_inputs: Vec<TokenStream> = sorted_rels
      .iter()
      .map(|rel| {
         let name = &rel.name;
         quote! {
            for __t in #target.#name.drain(..) {
               __s.#name.insert(__t);
            }
         }
      })
      .collect();

   // After commit, drain each sink directly and call `into_vec` — matches
   // the old `phase1_run_body` exactly (preserves delta-insertion order,
   // which several tests rely on for `assert_eq!` stability). `_snapshot`
   // would round-trip through a `HashMap` and lose ordering.
   let drain_outputs: Vec<TokenStream> = sorted_rels
      .iter()
      .map(|rel| {
         let name = &rel.name;
         let sink_field = Ident::new(&format!("{}_sink", name), name.span());
         quote! {
            #target.#name = ::std::mem::replace(
               &mut __s.#sink_field,
               ::ascent::dd::Sink::new(),
            ).into_vec();
         }
      })
      .collect();

   // Drive the dataflow explicitly — NOT via `commit()`. `commit()` calls
   // `refresh_deltas()` which drains each sink into the `_state` book-
   // keeping map, leaving the sink itself empty. We want to read sinks
   // directly (matches old `run()` semantics), so we advance+step here
   // and leave sinks untouched.
   quote! {
      let mut __s = #session_name::new();
      #(#seed_inputs)*
      __s.advance_all_to(1);
      __s.step_until(1);
      #(#drain_outputs)*
   }
}

pub(crate) fn emit_session(mir: &AscentMir) -> TokenStream {
   let ty_signature = &mir.signatures.declaration;
   let base_name = &ty_signature.ident;
   let session_name = session_struct_name(base_name);
   let sorted_rels = mir.relations_ir_relations.keys().sorted_by_key(|r| &r.name).collect_vec();

   // Per-relation field declarations / init / methods.
   let mut input_fields = vec![];
   let mut sink_fields = vec![];
   let mut state_fields = vec![];
   let mut last_delta_fields = vec![];
   let mut per_rel_methods = vec![];
   let mut input_advances = vec![]; // inside commit()
   let mut sink_drains = vec![]; // inside commit()

   for rel in &sorted_rels {
      let name = &rel.name;
      let tup_ty = tuple_type(&rel.field_types);
      // Public input field named after the relation itself. Users drive DD
      // directly — `sess.edge.insert(...)`, `sess.edge.advance_to(5)`,
      // `sess.edge.flush()`. Full `InputSession` API available.
      let input_field = name.clone();
      let sink_field = Ident::new(&format!("{}_sink", name), name.span());
      let state_field = Ident::new(&format!("{}_state", name), name.span());
      let last_delta_field = Ident::new(&format!("{}_last_deltas", name), name.span());
      let insert_m = Ident::new(&format!("{}_insert", name), name.span());
      let remove_m = Ident::new(&format!("{}_remove", name), name.span());
      let snapshot_m = Ident::new(&format!("{}_snapshot", name), name.span());
      let deltas_m = Ident::new(&format!("{}_deltas", name), name.span());

      input_fields.push(quote! {
         pub #input_field: ::ascent::dd::SessionInput<#tup_ty>,
      });
      sink_fields.push(quote! {
         /// Raw DD output sink for this relation. Call
         /// `sess.<rel>_sink.drain_deltas()` for direct access to the
         /// accumulated `Vec<(T, isize)>` — bypasses the `_deltas()` /
         /// `_snapshot()` / `refresh_deltas()` book-keeping.
         pub #sink_field: ::ascent::dd::Sink<#tup_ty>,
      });
      state_fields.push(quote! {
         #state_field: ::std::collections::HashMap<#tup_ty, i32>,
      });
      last_delta_fields.push(quote! {
         #last_delta_field: ::std::vec::Vec<(#tup_ty, i32)>,
      });

      per_rel_methods.push(quote! {
         /// Shorthand for `sess.<rel>.update(tuple, +1)` — DD's `InputSession`
         /// method. Use the field directly for `advance_to`, `flush`,
         /// `update_at`, `time`, etc. (`.insert` exists only on `isize`-diff
         /// sessions; FlowLog-parity i32-diff needs explicit `+1` / `-1`.)
         pub fn #insert_m(&mut self, tuple: #tup_ty) { self.#input_field.update(tuple, 1); }
         /// Shorthand for `sess.<rel>.update(tuple, -1)` — retract one.
         pub fn #remove_m(&mut self, tuple: #tup_ty) { self.#input_field.update(tuple, -1); }
         /// Snapshot of net-positive tuples, built from the running state
         /// map. Only reflects deltas that have been drained from the sink
         /// (via `commit()` or `refresh_deltas()`).
         pub fn #snapshot_m(&self) -> ::std::vec::Vec<#tup_ty> {
            self.#state_field.iter().filter(|(_, c)| **c > 0).map(|(t, _)| t.clone()).collect()
         }
         /// Deltas captured on the last `commit()` / `refresh_deltas()`.
         /// Positive diff = insertion, negative = retraction. Reads are
         /// non-destructive and return a clone.
         pub fn #deltas_m(&self) -> ::std::vec::Vec<(#tup_ty, i32)> {
            self.#last_delta_field.clone()
         }
      });

      input_advances.push(quote! {
         self.#input_field.advance_to(next);
         self.#input_field.flush();
      });
      // Drain sink → update running state + cache for `_deltas()` reads.
      sink_drains.push(quote! {
         {
            let deltas = self.#sink_field.drain_deltas();
            for (t, d) in &deltas {
               *self.#state_field.entry(t.clone()).or_insert(0) += d;
            }
            self.#last_delta_field = deltas;
         }
      });
   }

   // Build-dataflow body used inside `Session::new`. Mirrors `phase1_run_body`
   // but with inputs/sinks declared inline (not via Sealer) so they live on
   // the returned tuple.
   let build_body = emit_session_build_body(mir, &sorted_rels);

   // Tuple type of build-closure return value: ((input_a, input_b, ...), (sink_a, sink_b, ...), probe).
   let input_field_idents: Vec<Ident> = sorted_rels.iter().map(|r| r.name.clone()).collect();
   let sink_field_idents: Vec<Ident> =
      sorted_rels.iter().map(|r| Ident::new(&format!("{}_sink", r.name), r.name.span())).collect();

   // State + last-delta field initializers (all default/empty).
   let state_inits: Vec<TokenStream> = sorted_rels
      .iter()
      .flat_map(|r| {
         let s = Ident::new(&format!("{}_state", r.name), r.name.span());
         let d = Ident::new(&format!("{}_last_deltas", r.name), r.name.span());
         [
            quote! { #s: ::std::collections::HashMap::new(), },
            quote! { #d: ::std::vec::Vec::new(), },
         ]
      })
      .collect();

   let input_init_moves: Vec<TokenStream> =
      input_field_idents.iter().map(|i| quote! { #i: handles.inputs.#i, }).collect();
   let sink_init_moves: Vec<TokenStream> =
      sink_field_idents.iter().map(|i| quote! { #i: handles.sinks.#i, }).collect();

   // Anonymous struct "handles" with named fields — simpler than deep tuples.
   // Use a private sub-module per session to scope the helper struct.

   quote! {
      /// Live incremental session for the program.
      ///
      /// # Convenience path
      /// ```ignore
      /// let mut s = MyProgram::session();
      /// s.my_rel_insert((1, 2));
      /// s.commit();
      /// let out = s.my_out_deltas();
      /// ```
      ///
      /// # Raw DD path (no closures — worker lives outside any `dataflow(|scope| …)`)
      /// ```ignore
      /// let mut s = MyProgram::session();
      ///
      /// // InputSession is exposed directly — full DD API.
      /// for p in 0..n { s.manages.insert((p / 2, p)); }
      /// s.advance_all_to(1);          // bump every input's logical time + flush
      /// s.step_until(1);              // drive worker.step() until probe passes
      ///
      /// // Sinks are public: pull deltas without the snapshot book-keeping.
      /// for (tup, diff) in s.output_sink.drain_deltas() {
      ///     println!("{:?} {:+}", tup, diff);
      /// }
      ///
      /// // Need a DD primitive we didn't wrap? The worker is public too:
      /// s.worker.step_or_park_timeout(Some(std::time::Duration::from_millis(10)));
      /// ```
      ///
      /// Public fields: `<rel>` (InputSession), `<rel>_sink` (Sink), `worker`,
      /// `probe`, `epoch`. Drive from any control flow — sync loop, `mpsc::Receiver`,
      /// a tokio task — DD doesn't care.
      pub struct #session_name {
         /// The underlying timely worker. Public so users can call DD
         /// primitives we don't wrap (e.g. `step_or_park_timeout`).
         pub worker: ::ascent::dd::SessionWorker,
         /// Output frontier probe. Cheap to clone; use `.less_than(&t)` to
         /// check whether an epoch has cleared.
         pub probe: ::ascent::dd::timely::dataflow::ProbeHandle<u32>,
         /// Epoch counter used by the convenience `commit()` — advanced by 1
         /// each call. Read-only for external observers; modify only via
         /// `commit()` or by driving inputs directly.
         pub epoch: u32,
         #(#input_fields)*
         #(#sink_fields)*
         #(#state_fields)*
         #(#last_delta_fields)*
      }

      impl #session_name {
         pub fn new() -> Self {
            struct __Handles {
               inputs: __Inputs,
               sinks: __Sinks,
               probe: ::ascent::dd::timely::dataflow::ProbeHandle<u32>,
            }
            #[allow(non_camel_case_types)]
            struct __Inputs { #(#input_fields)* }
            #[allow(non_camel_case_types)]
            struct __Sinks { #(#sink_fields)* }

            let (worker, handles): (_, __Handles) = ::ascent::dd::build_session_worker(|scope| {
               #build_body
            });

            Self {
               worker,
               probe: handles.probe,
               epoch: 0,
               #( #input_init_moves )*
               #( #sink_init_moves )*
               #( #state_inits )*
            }
         }

         /// Advance all inputs to the next logical epoch, flush, drive the
         /// worker to that frontier, and drain sinks. Convenience for the
         /// simple case — for fine-grained control, use the `InputSession`
         /// handles directly + `step_until()` / `refresh_deltas()`.
         ///
         /// After this returns, `<rel>_snapshot` reflects the new fixed
         /// point and `<rel>_deltas` holds the change set.
         pub fn commit(&mut self) {
            let next = self.epoch + 1;
            #(#input_advances)*
            self.step_until(next);
            self.epoch = next;
            self.refresh_deltas();
         }

         /// Advance EVERY input's logical time to `time` and flush its
         /// buffer. Required before `step_until(time)` will make progress —
         /// DD's output frontier is the join of input frontiers, so any
         /// input still at an earlier time stalls everything downstream.
         /// Use this even for derived relations you never `.insert()` into;
         /// they still have an `InputSession` handle whose time must advance.
         pub fn advance_all_to(&mut self, time: u32) {
            #(
               self.#input_field_idents.advance_to(time);
               self.#input_field_idents.flush();
            )*
         }

         /// Step the underlying timely worker once. Use when driving DD
         /// manually (you've flushed inputs yourself and want to make
         /// progress without advancing the fused epoch).
         pub fn step(&mut self) { self.worker.step(); }

         /// Drive the worker until the output frontier has passed `time`.
         /// Cheap no-op if already ahead.
         pub fn step_until(&mut self, time: u32) {
            while self.probe.less_than(&time) {
               self.worker.step();
            }
         }

         /// Drive the worker until outputs reflect every input's current
         /// epoch. Call after advancing+flushing inputs at mixed times to
         /// let DD catch up without picking a specific frontier.
         pub fn step_until_idle(&mut self) {
            let __max: u32 = [#(*self.#input_field_idents.time()),*]
               .into_iter()
               .max()
               .unwrap_or(0);
            self.step_until(__max);
         }

         /// Read-only access to the output probe — useful for
         /// `probe.less_than(t)` / `probe.with_frontier(|f| …)` checks.
         pub fn probe(&self) -> &::ascent::dd::timely::dataflow::ProbeHandle<u32> {
            &self.probe
         }

         /// Drain every relation's sink into its running-state map + cache
         /// the drained vec for `_deltas()` reads. Call after stepping
         /// manually if you've bypassed `commit()`.
         pub fn refresh_deltas(&mut self) {
            #(#sink_drains)*
         }

         #(#per_rel_methods)*
      }

      impl ::std::default::Default for #session_name {
         fn default() -> Self { Self::new() }
      }
   }
}

/// Emit the body of the closure passed to `build_session_worker`. Creates
/// inputs + sinks + probe as local lets, builds the dataflow like the batch
/// backend, and returns a `__Handles` struct.
fn emit_session_build_body(mir: &AscentMir, sorted_rels: &[&RelationIdentity]) -> TokenStream {
   let mut setup = TokenStream::new();

   // Per-relation: create InputSession + initial (non-mut) coll + Sink.
   // `emit_closure_body` will later re-bind `__<rel>_coll` as `let mut`
   // so the `.concat` accumulation pattern works.
   for rel in sorted_rels {
      let name = &rel.name;
      let tup_ty = tuple_type(&rel.field_types);
      let input = name.clone();
      let sink = Ident::new(&format!("{}_sink", name), name.span());
      let coll = relation_coll_var(name);
      setup.extend(quote! {
         let mut #input: ::ascent::dd::SessionInput<#tup_ty> = ::ascent::dd::differential_dataflow::input::InputSession::new();
         let #coll = #input.to_collection(scope);
         let #sink: ::ascent::dd::Sink<#tup_ty> = ::ascent::dd::Sink::new();
      });
   }

   // Probe must be in scope before `emit_closure_body` emits the sink
   // `attach(&mut probe)` calls.
   setup.extend(quote! {
      let mut probe: ::ascent::dd::timely::dataflow::ProbeHandle<u32> = ::ascent::dd::timely::dataflow::ProbeHandle::new();
   });

   // Shared core: imports, mut coll rebind, unit scope, SCC bodies, attach.
   // Session path is FnOnce (single-worker, persistent). No hoist rebinds
   // needed — the build closure runs exactly once inside `worker.dataflow`,
   // inner DD operators can safely move hoisted Vecs without the Fn trap.
   let (body, hoists, _hoist_count) = emit_closure_body(
      mir,
      sorted_rels,
      |rel| {
         // Session init: rebind the per-rel `__<rel>_coll` (already created
         // above from the InputSession) as `let mut` for rule accumulation.
         let coll = relation_coll_var(&rel.name);
         quote! { let mut #coll = #coll; }
      },
      |rel| {
         let sink = Ident::new(&format!("{}_sink", rel.name), rel.name.span());
         quote! { #sink }
      },
      quote! { &mut probe },
      false,
      // Session is single-worker: literal 0 (Sink has only one slot here).
      quote! { 0usize },
   );
   setup.extend(hoists);
   setup.extend(body);

   // Assemble `__Handles { inputs, sinks, probe }` for `Session::new()` to
   // unpack into struct fields.
   let input_assigns: Vec<TokenStream> = sorted_rels
      .iter()
      .map(|r| {
         let n = r.name.clone();
         quote! { #n, }
      })
      .collect();
   let sink_assigns: Vec<TokenStream> = sorted_rels
      .iter()
      .map(|r| {
         let n = Ident::new(&format!("{}_sink", r.name), r.name.span());
         quote! { #n, }
      })
      .collect();

   setup.extend(quote! {
      __Handles {
         inputs: __Inputs { #(#input_assigns)* },
         sinks: __Sinks { #(#sink_assigns)* },
         probe,
      }
   });

   setup
}
