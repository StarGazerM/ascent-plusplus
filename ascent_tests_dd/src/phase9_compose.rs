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

// ---------------------------------------------------------------------------
// Compose with negation. ProgA produces `path` (TC). ProgB takes it as
// `link` and computes `unreachable(x)` — nodes that are NOT reachable
// from any source. Verifies negation in a downstream program works
// against an upstream program's output `Collection`.
// ---------------------------------------------------------------------------

ascent! {
   #![backend(dd)]
   pub struct ProgC;
   #[input] relation node(i32);
   #[input] relation link(i32, i32);
   #[output] relation unreachable(i32);
   // No incoming edge → unreachable. (Non-recursive — keeps the test
   // crisp; we're verifying negation through compose, not recursion
   // through compose, which is already exercised elsewhere.)
   unreachable(x) <-- node(x), ! link(_, x);
}

#[ntest_timeout::timeout(2000)]
#[test]
fn compose_with_negation_downstream() {
   let unreach_sink: Sink<(i32,)> = Sink::new_with_workers(1);
   let unreach_sink_for_build = unreach_sink.clone();

   let edges = vec![(1, 2), (2, 3), (3, 4), (5, 6)];
   let nodes = vec![(1,), (2,), (3,), (4,), (5,), (6,), (7,), (8,)];

   execute_batch_with_workers(1, move |scope, sealer, probe| {
      let wi = sealer.worker_index();
      let edge_coll = sealer.input::<(i32, i32), _>(scope, &edges);
      let node_coll = sealer.input::<(i32,), _>(scope, &nodes);
      // A computes TC of edge → path.
      let a_out = ProgA::build_in_scope(scope, ProgAComposeInputs { edge: edge_coll });
      // C consumes A.path as link, plus a separate node input, computes
      // unreachable via negation.
      let c_out =
         ProgC::build_in_scope(scope, ProgCComposeInputs { node: node_coll, link: a_out.path });
      unreach_sink_for_build.attach(wi, &c_out.unreachable, probe);
   });

   let mut state: ::std::collections::HashMap<(i32,), i32> = Default::default();
   for (t, d) in unreach_sink.drain_deltas() {
      *state.entry(t).or_insert(0) += d;
   }
   let mut got: Vec<_> = state.iter().filter(|(_, c)| **c > 0).map(|(k, _)| *k).collect();
   got.sort();

   // Reachable in path: {2, 3, 4, 6} (each appears as an edge target,
   // also chained from earlier nodes). Unreachable: {1, 5, 7, 8} —
   // 1 and 5 are sources (no incoming), 7 and 8 are isolated.
   assert_eq!(got, vec![(1,), (5,), (7,), (8,)]);
}

// ---------------------------------------------------------------------------
// Compose with aggregation. ProgD takes A's `path` as `link` and counts
// how many destinations each source reaches. Verifies an aggregator
// `agg c = count() in link(x, _)` works against an upstream Collection.
// ---------------------------------------------------------------------------

ascent! {
   #![backend(dd)]
   pub struct ProgD;
   #[input] relation source(i32);
   #[input] relation link(i32, i32);
   #[output] relation reach_count(i32, usize);
   reach_count(x, c) <-- source(x), agg c = ::ascent::aggregators::count() in link(x, _);
}

#[ntest_timeout::timeout(2000)]
#[test]
fn compose_with_aggregation_downstream() {
   let count_sink: Sink<(i32, usize)> = Sink::new_with_workers(1);
   let count_sink_for_build = count_sink.clone();

   let edges = vec![(1, 2), (2, 3), (3, 4), (10, 11), (10, 12)];
   let sources = vec![(1,), (10,)];

   execute_batch_with_workers(1, move |scope, sealer, probe| {
      let wi = sealer.worker_index();
      let edge_coll = sealer.input::<(i32, i32), _>(scope, &edges);
      let src_coll = sealer.input::<(i32,), _>(scope, &sources);
      let a_out = ProgA::build_in_scope(scope, ProgAComposeInputs { edge: edge_coll });
      let d_out =
         ProgD::build_in_scope(scope, ProgDComposeInputs { source: src_coll, link: a_out.path });
      count_sink_for_build.attach(wi, &d_out.reach_count, probe);
   });

   let mut state: ::std::collections::HashMap<(i32, usize), i32> = Default::default();
   for (t, d) in count_sink.drain_deltas() {
      *state.entry(t).or_insert(0) += d;
   }
   let mut got: Vec<_> = state.iter().filter(|(_, c)| **c > 0).map(|(k, _)| *k).collect();
   got.sort();

   // path from source 1: {2, 3, 4} → 3 reachable.
   // path from source 10: {11, 12} → 2 reachable.
   assert_eq!(got, vec![(1, 3), (10, 2)]);
}

// ---------------------------------------------------------------------------
// Compose with a LATTICE output. ProgE takes `observation(key, val)` as
// input and exposes a lattice `best(key, val)` (max-per-key) as output.
// Verifies the embed.rs codegen routes lattices through the same final-
// dedup logic as session/run paths.
// ---------------------------------------------------------------------------

ascent! {
   #![backend(dd)]
   pub struct ProgE;
   #[input] relation observation(i32, u32);
   #[output] lattice best(i32, u32);
   best(k, v) <-- observation(k, v);
}

#[ntest_timeout::timeout(2000)]
#[test]
fn compose_with_lattice_output() {
   let best_sink: Sink<(i32, u32)> = Sink::new_with_workers(1);
   let best_sink_for_build = best_sink.clone();

   let obs = vec![(1, 5_u32), (1, 10), (1, 3), (2, 7), (2, 2), (3, 100)];

   execute_batch_with_workers(1, move |scope, sealer, probe| {
      let wi = sealer.worker_index();
      let obs_coll = sealer.input::<(i32, u32), _>(scope, &obs);
      let e_out = ProgE::build_in_scope(scope, ProgEComposeInputs { observation: obs_coll });
      best_sink_for_build.attach(wi, &e_out.best, probe);
   });

   let mut state: ::std::collections::HashMap<(i32, u32), i32> = Default::default();
   for (t, d) in best_sink.drain_deltas() {
      *state.entry(t).or_insert(0) += d;
   }
   let mut got: Vec<_> = state.iter().filter(|(_, c)| **c > 0).map(|(k, _)| *k).collect();
   got.sort();

   // Lattice (u32, join=max): per-key max of observations.
   assert_eq!(got, vec![(1, 10), (2, 7), (3, 100)]);
}

// ---------------------------------------------------------------------------
// `#[plan]` + compose interaction. A rule annotated with a non-default
// `#[plan(order=...)]` must produce identical output whether driven via
// session or via build_in_scope — the plan permutation is applied at
// HIR→MIR lowering, so both emissions see the same MIR. This locks in
// the contract that those features compose without surprises.
// ---------------------------------------------------------------------------

ascent! {
   #![backend(dd)]
   pub struct ProgPlanCompose;
   #[input] relation edge(i32, i32);
   #[output] relation path(i32, i32);
   path(x, y) <-- edge(x, y);
   // Reverse the natural body order — exercises the plan permutation
   // through both session and build_in_scope codegen paths.
   #[plan(variant(delta = 1, order = [1, 0]))]
   path(x, z) <-- edge(x, y), path(y, z);
}

#[ntest_timeout::timeout(2000)]
#[test]
fn compose_plus_plan_consistent_across_emissions() {
   let edges = vec![(1, 2), (2, 3), (3, 4), (5, 6)];
   let expected: Vec<(i32, i32)> = vec![(1, 2), (1, 3), (1, 4), (2, 3), (2, 4), (3, 4), (5, 6)];

   // build_in_scope path
   let scope_out: Vec<(i32, i32)> = {
      let sink: Sink<(i32, i32)> = Sink::new_with_workers(1);
      let sink_for_build = sink.clone();
      let edges_clone = edges.clone();
      execute_batch_with_workers(1, move |scope, sealer, probe| {
         let wi = sealer.worker_index();
         let edge_coll = sealer.input::<(i32, i32), _>(scope, &edges_clone);
         let out = ProgPlanCompose::build_in_scope(
            scope,
            ProgPlanComposeComposeInputs { edge: edge_coll },
         );
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
   assert_eq!(scope_out, expected, "build_in_scope + #[plan] produced wrong output");

   // session path — same program, same plan, must match
   let session_out: Vec<(i32, i32)> = {
      let mut s = ProgPlanCompose::session();
      for e in &edges {
         s.edge_insert(*e);
      }
      s.commit();
      let mut p = s.path_snapshot();
      p.sort();
      p
   };
   assert_eq!(session_out, expected, "session + #[plan] produced wrong output");
   assert_eq!(scope_out, session_out, "plan emissions diverged across compose vs session");
}

// ---------------------------------------------------------------------------
// Edge case: program with ONLY `#[output]` (no `#[input]`). The
// ComposeInputs struct has zero "real" fields — the codegen injects a
// phantom field to anchor the `S` type parameter and avoid E0392
// "unused type parameter". This test exercises that path.
//
// The program seeds itself via a literal in the rule body — no caller-
// provided collections needed. ComposeInputs is empty-but-anchored.
// ---------------------------------------------------------------------------

ascent! {
   #![backend(dd)]
   pub struct ProgF;
   #[output] relation singleton(i32);
   // Generator with no body atoms — pure self-seeding.
   singleton(x) <-- for x in 0..3;
}

#[ntest_timeout::timeout(2000)]
#[test]
fn compose_output_only_no_inputs() {
   let sink: Sink<(i32,)> = Sink::new_with_workers(1);
   let sink_for_build = sink.clone();
   execute_batch_with_workers(1, move |scope, sealer, probe| {
      let wi = sealer.worker_index();
      // ComposeInputs has only the phantom field — fill it to anchor `S`.
      let out = ProgF::build_in_scope(
         scope,
         ProgFComposeInputs { __phantom: ::std::marker::PhantomData },
      );
      sink_for_build.attach(wi, &out.singleton, probe);
   });

   let mut state: ::std::collections::HashMap<(i32,), i32> = Default::default();
   for (t, d) in sink.drain_deltas() {
      *state.entry(t).or_insert(0) += d;
   }
   let mut got: Vec<_> = state.iter().filter(|(_, c)| **c > 0).map(|(k, _)| *k).collect();
   got.sort();
   assert_eq!(got, vec![(0,), (1,), (2,)]);
}

// ---------------------------------------------------------------------------
// Three-stage pipeline composition: A → B → C all wired together inside a
// single timely scope. Verifies build_in_scope handles arbitrary chaining
// — each program receives `Collection`s from upstream programs and
// returns `Collection`s consumed downstream, with no Vec round-trips.
//
// Pipeline semantics:
//   A: edges → path (TC)
//   B: link (= A.path) → reach (TC over the TC, i.e. still TC)
//   C: source + link (= B.reach) → reach_count (per-source count)
// ---------------------------------------------------------------------------

#[ntest_timeout::timeout(2000)]
#[test]
fn three_stage_pipeline_chained_compose() {
   let count_sink: Sink<(i32, usize)> = Sink::new_with_workers(1);
   let count_sink_for_build = count_sink.clone();

   let edges = vec![(1, 2), (2, 3), (3, 4), (10, 11), (10, 12)];
   let sources = vec![(1,), (10,)];

   execute_batch_with_workers(1, move |scope, sealer, probe| {
      let wi = sealer.worker_index();
      let edge_coll = sealer.input::<(i32, i32), _>(scope, &edges);
      let src_coll = sealer.input::<(i32,), _>(scope, &sources);

      let a_out = ProgA::build_in_scope(scope, ProgAComposeInputs { edge: edge_coll });
      // B consumes A.path; B.reach goes into C.
      let b_out = ProgB::build_in_scope(scope, ProgBComposeInputs { link: a_out.path });
      // C consumes B.reach + a separate source input.
      let c_out =
         ProgD::build_in_scope(scope, ProgDComposeInputs { source: src_coll, link: b_out.reach });

      count_sink_for_build.attach(wi, &c_out.reach_count, probe);
   });

   let mut state: ::std::collections::HashMap<(i32, usize), i32> = Default::default();
   for (t, d) in count_sink.drain_deltas() {
      *state.entry(t).or_insert(0) += d;
   }
   let mut got: Vec<_> = state.iter().filter(|(_, c)| **c > 0).map(|(k, _)| *k).collect();
   got.sort();

   // Same expected output as the 2-stage aggregation test — TC is
   // idempotent, so A→B→C same as A→C for these inputs.
   assert_eq!(got, vec![(1, 3), (10, 2)]);
}









