//! `ProgA::build_in_scope(...)` — DD-level composition across two programs.
//!
//! Verifies the `build_in_scope` entry point wires one program's output
//! `Collection` into another's input `Collection` inside a single timely
//! dataflow — no delta round-trip through Rust Vecs.
//!
//! Relations are opted into the compose API via `#[input]` (seeded by the
//! caller) and `#[output]` (read by the caller). Unannotated relations are
//! internal to the program and don't appear in either struct — type-check
//! is the enforcement mechanism: forgetting a required input field is a
//! compile error, and wiring mismatched tuple types fails to typecheck.

use ascent::ascent;
use ascent::dd::build_session_worker;
use ascent::dd::differential_dataflow::input::{Input, InputSession};
use ascent::dd::timely::dataflow::ProbeHandle;
use ascent::dd::Sink;

// ---------------------------------------------------------------------------
// Two programs. A takes `edge` as input and exposes `path` (TC) as output.
// B takes `link` and exposes `reach`. `path` is unannotated in A so it's
// internal — BUT we want to expose it, so it's marked `#[output]`. `link`
// in B is marked `#[input]`. Compose: A.path → B.link.
// ---------------------------------------------------------------------------

ascent! {
   #![backend(dd)]
   pub struct ProgA;
   #[input] relation edge(i32, i32);
   #[output] relation path(i32, i32);
   path(x, y) <-- edge(x, y);
   path(x, z) <-- edge(x, y), path(y, z);
}

ascent! {
   #![backend(dd)]
   pub struct ProgB;
   #[input] relation link(i32, i32);
   #[output] relation reach(i32, i32);
   reach(x, y) <-- link(x, y);
   reach(x, z) <-- link(x, y), reach(y, z);
}

#[ntest_timeout::timeout(2000)]
#[test]
fn compose_two_programs_via_build_in_scope() {
   let reach_sink: Sink<(i32, i32)> = Sink::new();
   let reach_sink_for_build = reach_sink.clone();

   let (mut worker, (mut edge_input, probe)) = build_session_worker(move |scope| {
      let mut edge_input: InputSession<u32, (i32, i32), i32> = InputSession::new();
      let edge_coll = edge_input.to_collection(scope);

      // No `path_empty` / `link_empty` ceremony any more — only the
      // `#[input]` fields appear in ComposeInputs. `path` on A is
      // `#[output]`-only, not in Inputs; `reach` on B is `#[output]`-only,
      // not in Inputs.
      let a_out = ProgA::build_in_scope(scope, ProgAComposeInputs { edge: edge_coll });
      let b_out = ProgB::build_in_scope(scope, ProgBComposeInputs { link: a_out.path });

      let mut probe: ProbeHandle<u32> = ProbeHandle::new();
      reach_sink_for_build.attach(0, &b_out.reach, &mut probe);
      (edge_input, probe)
   });

   edge_input.update((1, 2), 1);
   edge_input.update((2, 3), 1);
   edge_input.update((3, 4), 1);
   edge_input.advance_to(1);
   edge_input.flush();
   while probe.less_than(&1) {
      worker.step();
   }

   let mut state: ::std::collections::HashMap<(i32, i32), i32> = Default::default();
   for (t, d) in reach_sink.drain_deltas() {
      *state.entry(t).or_insert(0) += d;
   }
   let mut snap: Vec<_> = state.iter().filter(|(_, c)| **c > 0).map(|(k, _)| *k).collect();
   snap.sort();

   let expected: Vec<(i32, i32)> = vec![(1, 2), (1, 3), (1, 4), (2, 3), (2, 4), (3, 4)];
   assert_eq!(snap, expected);
}

// ---------------------------------------------------------------------------
// Retraction through the composed pipeline: edge retraction → A.path
// retractions → B.link retractions → B.reach retractions, all at DD level
// in a single dataflow. Nothing marshals through Rust in between.
// ---------------------------------------------------------------------------

#[ntest_timeout::timeout(2000)]
#[test]
fn compose_retraction_propagates_through_pipeline() {
   let reach_sink: Sink<(i32, i32)> = Sink::new();
   let reach_sink_for_build = reach_sink.clone();

   let (mut worker, (mut edge_input, mut probe)) = build_session_worker(move |scope| {
      let mut edge_input: InputSession<u32, (i32, i32), i32> = InputSession::new();
      let edge_coll = edge_input.to_collection(scope);
      let a_out = ProgA::build_in_scope(scope, ProgAComposeInputs { edge: edge_coll });
      let b_out = ProgB::build_in_scope(scope, ProgBComposeInputs { link: a_out.path });
      let mut probe: ProbeHandle<u32> = ProbeHandle::new();
      reach_sink_for_build.attach(0, &b_out.reach, &mut probe);
      (edge_input, probe)
   });

   let mut step_to = |edge: &mut InputSession<u32, (i32, i32), i32>,
                      probe: &mut ProbeHandle<u32>,
                      worker: &mut ::ascent::dd::timely::worker::Worker<_>,
                      epoch: &mut u32| {
      let next = *epoch + 1;
      edge.advance_to(next);
      edge.flush();
      while probe.less_than(&next) {
         worker.step();
      }
      *epoch = next;
   };

   edge_input.update((1, 2), 1);
   edge_input.update((2, 3), 1);
   edge_input.update((3, 4), 1);
   let mut epoch: u32 = 0;
   step_to(&mut edge_input, &mut probe, &mut worker, &mut epoch);
   let _ = reach_sink.drain_deltas();

   edge_input.update((2, 3), -1);
   step_to(&mut edge_input, &mut probe, &mut worker, &mut epoch);

   let retracted: ::std::collections::HashSet<(i32, i32)> =
      reach_sink.drain_deltas().into_iter().filter(|(_, d)| *d < 0).map(|(t, _)| t).collect();
   for t in [(1, 3), (1, 4), (2, 3), (2, 4)] {
      assert!(retracted.contains(&t), "expected retraction of {t:?} after pruning 2→3, got {retracted:?}");
   }
}

// ---------------------------------------------------------------------------
// Mixed relation (`#[input] #[output]`): caller seeds values AND rules
// derive more into the same relation. Both flows land in the same
// accumulator; the output reflects the union.
// ---------------------------------------------------------------------------

ascent! {
   #![backend(dd)]
   pub struct MixedProg;
   // `vals` is a bus: caller can seed, rules can also derive into it.
   #[input] #[output] relation vals(i32);
   #[input] relation double_source(i32);
   // Derive even-doubles into vals.
   vals(x * 2) <-- double_source(x);
}

#[ntest_timeout::timeout(2000)]
#[test]
fn mixed_input_output_relation() {
   let vals_sink: Sink<(i32,)> = Sink::new();
   let vals_sink_for_build = vals_sink.clone();

   let (mut worker, (mut vals_input, mut dsrc_input, probe)) = build_session_worker(move |scope| {
      let mut vals_in: InputSession<u32, (i32,), i32> = InputSession::new();
      let mut dsrc_in: InputSession<u32, (i32,), i32> = InputSession::new();
      let vals_coll = vals_in.to_collection(scope);
      let dsrc_coll = dsrc_in.to_collection(scope);

      let out = MixedProg::build_in_scope(
         scope,
         MixedProgComposeInputs { vals: vals_coll, double_source: dsrc_coll },
      );

      let mut probe: ProbeHandle<u32> = ProbeHandle::new();
      vals_sink_for_build.attach(0, &out.vals, &mut probe);
      (vals_in, dsrc_in, probe)
   });

   // Seed vals with 100, 200. double_source with 3, 4 → rule adds 6, 8.
   vals_input.update((100,), 1);
   vals_input.update((200,), 1);
   dsrc_input.update((3,), 1);
   dsrc_input.update((4,), 1);
   vals_input.advance_to(1);
   vals_input.flush();
   dsrc_input.advance_to(1);
   dsrc_input.flush();
   while probe.less_than(&1) {
      worker.step();
   }

   let mut state: ::std::collections::HashMap<(i32,), i32> = Default::default();
   for (t, d) in vals_sink.drain_deltas() {
      *state.entry(t).or_insert(0) += d;
   }
   let mut snap: Vec<_> = state.iter().filter(|(_, c)| **c > 0).map(|(k, _)| *k).collect();
   snap.sort();
   assert_eq!(snap, vec![(6,), (8,), (100,), (200,)]);
}
