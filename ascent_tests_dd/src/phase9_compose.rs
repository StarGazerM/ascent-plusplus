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
use ascent::dd::differential_dataflow::input::InputSession;
use ascent::dd::timely::dataflow::ProbeHandle;
use ascent::dd::{Sink, build_session_worker, execute_batch_with_workers, scope_input_session};

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
      let (mut edge_input, edge_coll) = scope_input_session::<_, (i32, i32)>(scope);

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
      let (mut edge_input, edge_coll) = scope_input_session::<_, (i32, i32)>(scope);
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
      let (mut vals_in, vals_coll) = scope_input_session::<_, (i32,)>(scope);
      let (mut dsrc_in, dsrc_coll) = scope_input_session::<_, (i32,)>(scope);

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

// ---------------------------------------------------------------------------
// Multi-worker compose: drive `build_in_scope` inside `execute_batch` with
// N=4 workers. Verifies DD's cross-worker exchange correctly handles
// arrangements BUILT BY ONE PROGRAM AND CONSUMED BY ANOTHER within the
// same dataflow — i.e., that the compose API isn't quietly assuming
// single-worker semantics anywhere.
//
// Key is that `Sealer::input` hash-partitions the seed data across
// workers at ingest, and DD's `join_core` across the compose boundary
// exchanges tuples to the correct worker before joining. If anything
// were single-worker-implicit (e.g. caching a `worker_index=0`
// assumption), this test would produce wrong or partial output.
// ---------------------------------------------------------------------------

#[ntest_timeout::timeout(5000)]
#[test]
fn compose_under_execute_batch_multi_worker() {
   const WORKERS: usize = 4;
   // Match Sink slot count to execute-batch worker count — the two MUST
   // agree or `attach`'s per-worker indexed write panics OOB. `Sink::new()`
   // (no args) reads from the env var and would mismatch our explicit 4.
   let reach_sink: Sink<(i32, i32)> = Sink::new_with_workers(WORKERS);
   let reach_sink_for_build = reach_sink.clone();

   // Enough rows that hash-partition will distribute across all 4 workers —
   // a single-worker bug would show up as missing pairs, not as skew.
   let edges: Vec<(i32, i32)> =
      vec![(1, 2), (2, 3), (3, 4), (4, 5), (5, 6), (6, 7), (10, 11), (11, 12), (12, 13)];

   execute_batch_with_workers(WORKERS, move |scope, sealer, probe| {
      let wi = sealer.worker_index();
      let edge_coll = sealer.input::<(i32, i32), _>(scope, &edges);
      let a_out = ProgA::build_in_scope(scope, ProgAComposeInputs { edge: edge_coll });
      let b_out = ProgB::build_in_scope(scope, ProgBComposeInputs { link: a_out.path });
      reach_sink_for_build.attach(wi, &b_out.reach, probe);
   });

   let mut state: ::std::collections::HashMap<(i32, i32), i32> = Default::default();
   for (t, d) in reach_sink.drain_deltas() {
      *state.entry(t).or_insert(0) += d;
   }
   let mut snap: Vec<_> = state.iter().filter(|(_, c)| **c > 0).map(|(k, _)| *k).collect();
   snap.sort();

   // Expected TC-of-TC on the two chains {1..7} and {10..13}:
   //   chain (1→2→3→4→5→6→7) → all (i, j) with 1≤i<j≤7 except those crossing chains.
   //   chain (10→11→12→13)   → all (i, j) with 10≤i<j≤13.
   let mut expected: Vec<(i32, i32)> = Vec::new();
   for i in 1..=7 {
      for j in (i + 1)..=7 {
         expected.push((i, j));
      }
   }
   for i in 10..=13 {
      for j in (i + 1)..=13 {
         expected.push((i, j));
      }
   }
   expected.sort();

   assert_eq!(snap, expected, "multi-worker compose missing or extra pairs — worker topology bug?");
}

// ---------------------------------------------------------------------------
// Scale smoke: session-mode compose with a non-trivial workload. Catches
// pathological slowdowns (e.g. a future regression that reintroduces the
// Sealer-style operator-graph bug fixed in commit ffb88e1, which caused
// fixpoint divergence at n~1025+ in the `execute_batch` path).
//
// N=100 → 5k TC pairs, comfortable under both debug (~5 s) and release
// (~0.05 s). Catches a hang regression without chewing 30 s of CI time.
// Bigger N values are valid (release: N=1000 in ~15 s) but make the test
// fragile under load — bump only if you have a specific scale concern.
// ---------------------------------------------------------------------------

#[ntest_timeout::timeout(10000)]
#[test]
fn compose_session_scale_smoke() {
   let reach_sink: Sink<(i32, i32)> = Sink::new();
   let reach_sink_for_build = reach_sink.clone();

   let (mut worker, (mut edge_input, probe)) = build_session_worker(move |scope| {
      let (mut edge_input, edge_coll) = scope_input_session::<_, (i32, i32)>(scope);
      let a_out = ProgA::build_in_scope(scope, ProgAComposeInputs { edge: edge_coll });
      let b_out = ProgB::build_in_scope(scope, ProgBComposeInputs { link: a_out.path });
      let mut probe: ProbeHandle<u32> = ProbeHandle::new();
      reach_sink_for_build.attach(0, &b_out.reach, &mut probe);
      (edge_input, probe)
   });

   const N: i32 = 100;
   for i in 0..N {
      edge_input.update((i, i + 1), 1);
   }
   edge_input.advance_to(1);
   edge_input.flush();
   while probe.less_than(&1) {
      worker.step();
   }

   let mut state: ::std::collections::HashMap<(i32, i32), i32> = Default::default();
   for (t, d) in reach_sink.drain_deltas() {
      *state.entry(t).or_insert(0) += d;
   }
   let reached_count = state.values().filter(|c| **c > 0).count();
   // TC of a length-N chain is N*(N+1)/2 pairs. TC-of-TC is the same set.
   assert_eq!(reached_count, (N as usize) * (N as usize + 1) / 2, "missing pairs at scale");
}

// ---------------------------------------------------------------------------
// Three-emission coexistence: a single program annotated for compose
// (`#[input]` / `#[output]`) must still work via `.run()` and `::session()`
// — the three emissions share MIR but produce independent code paths,
// and a regression in one mustn't silently break the others.
//
// Asserts identical output across all three for the same input.
// ---------------------------------------------------------------------------

ascent! {
   #![backend(dd)]
   pub struct TriProg;
   #[input] relation edge(i32, i32);
   #[output] relation path(i32, i32);
   path(x, y) <-- edge(x, y);
   path(x, z) <-- edge(x, y), path(y, z);
}

#[ntest_timeout::timeout(5000)]
#[test]
fn three_emissions_produce_identical_output() {
   let edges = vec![(1, 2), (2, 3), (3, 4), (5, 6)];
   let expected: Vec<(i32, i32)> = vec![(1, 2), (1, 3), (1, 4), (2, 3), (2, 4), (3, 4), (5, 6)];

   // 1. .run() — uses execute_batch under the hood, multi-worker capable
   let run_path: Vec<(i32, i32)> = {
      let mut p = TriProg::default();
      p.edge = edges.clone();
      p.run();
      let mut path = p.path;
      path.sort();
      path.dedup();
      path
   };
   assert_eq!(run_path, expected, ".run() produced wrong output");

   // 2. ::session() + commit() — incremental, single-worker
   let session_path: Vec<(i32, i32)> = {
      let mut s = TriProg::session();
      for e in &edges {
         s.edge_insert(*e);
      }
      s.commit();
      let mut path = s.path_snapshot();
      path.sort();
      path
   };
   assert_eq!(session_path, expected, "::session() produced wrong output");

   // 3. build_in_scope inside execute_batch — composable path
   let scope_path: Vec<(i32, i32)> = {
      let sink: Sink<(i32, i32)> = Sink::new_with_workers(1);
      let sink_for_build = sink.clone();
      let edges_for_build = edges.clone();
      execute_batch_with_workers(1, move |scope, sealer, probe| {
         let wi = sealer.worker_index();
         let edge_coll = sealer.input::<(i32, i32), _>(scope, &edges_for_build);
         let out = TriProg::build_in_scope(scope, TriProgComposeInputs { edge: edge_coll });
         sink_for_build.attach(wi, &out.path, probe);
      });
      let mut state: ::std::collections::HashMap<(i32, i32), i32> = Default::default();
      for (t, d) in sink.drain_deltas() {
         *state.entry(t).or_insert(0) += d;
      }
      let mut path: Vec<_> = state.iter().filter(|(_, c)| **c > 0).map(|(k, _)| *k).collect();
      path.sort();
      path
   };
   assert_eq!(scope_path, expected, "build_in_scope produced wrong output");
}



