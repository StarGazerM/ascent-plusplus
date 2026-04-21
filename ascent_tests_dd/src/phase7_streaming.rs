//! Streaming tests — feed input as a stream over time, observe output deltas
//! per commit. Batch mode takes a `Vec<T>` upfront; here we push tuples one
//! at a time (from iterators, channels, any producer) through the session
//! API and verify that DD's incremental semantics hold across commits:
//!   • new facts produce positive deltas
//!   • retractions produce negative deltas
//!   • lattice updates produce (-old, +new) pairs per key
//!   • aggregator outputs re-fire as their input changes
//!   • multiple commits compose correctly

use std::sync::mpsc;

use ascent::dd::InputSessionI32Ext;
use ascent::{Dual, ascent};

// ---------------------------------------------------------------------------
// 1. Lattice over time.
// ---------------------------------------------------------------------------

ascent! {
   #![backend(dd)]
   pub struct ShortestPathS;

   relation edge(i32, i32, u32);
   lattice shortest(i32, i32, Dual<u32>);

   shortest(*x, *y, Dual(*w)) <-- edge(x, y, w);
   shortest(*x, *z, Dual(w + l.0)) <-- edge(x, y, w), shortest(y, z, l);
}

/// Streaming a lattice relation: insert edges in order, watch the shortest
/// path value tighten. Each tighter path emits `-old + new` at the lattice
/// key — the hallmark of lattice-aware incremental output.
#[ntest_timeout::timeout(1000)]
#[test]
fn session_lattice_tightens_over_time() {
   let mut s = ShortestPathS::session();

   // t=1: single direct edge.
   s.edge_insert((1, 2, 10));
   s.commit();
   let mut snap = s.shortest_snapshot();
   snap.sort();
   assert_eq!(snap, vec![(1, 2, Dual(10))]);

   // t=2: add a 2-hop bypass that's worse. Shouldn't change (1,2).
   s.edge_insert((1, 3, 50));
   s.edge_insert((3, 2, 5));
   s.commit();
   let mut snap = s.shortest_snapshot();
   snap.sort();
   // (1,2) still best at 10; (1,3)=50, (3,2)=5, plus (1,2) via 1→3→2 = 55 (worse, joined).
   assert!(snap.contains(&(1, 2, Dual(10))));
   assert!(snap.contains(&(1, 3, Dual(50))));
   assert!(snap.contains(&(3, 2, Dual(5))));

   // t=3: a much shorter direct 1→2. The lattice join should tighten (1,2).
   s.edge_insert((1, 2, 3));
   s.commit();
   let mut snap = s.shortest_snapshot();
   snap.sort();
   // (1,2) should now reflect min(10, 3) = 3.
   assert!(snap.contains(&(1, 2, Dual(3))));
   // The OLD (1,2,10) should no longer be the active lattice value at that key.
   assert!(!snap.contains(&(1, 2, Dual(10))));
}

// ---------------------------------------------------------------------------
// 2. Aggregator deltas over time.
// ---------------------------------------------------------------------------

ascent! {
   #![backend(dd)]
   pub struct GroupSumS;

   relation sample(u32, i32);
   relation group_sum(u32, i32);

   group_sum(k, s) <--
      for k in [1u32, 2u32, 3u32],
      agg s = ::ascent::aggregators::sum(v) in sample(k, v);
}

#[ntest_timeout::timeout(1000)]
#[test]
fn session_aggregator_recomputes_on_input_change() {
   let mut s = GroupSumS::session();

   // t=1: two samples in group 1.
   s.sample_insert((1, 10));
   s.sample_insert((1, 20));
   s.commit();
   let mut snap = s.group_sum_snapshot();
   snap.sort();
   // Groups 1=30, 2=0 (empty input → sum yields 0 via Sum impl), 3=0.
   assert!(snap.contains(&(1, 30)));

   // t=2: insert into group 2, observe both delta and snapshot.
   s.sample_insert((2, 100));
   s.sample_insert((2, 200));
   s.commit();
   let snap: Vec<_> = s.group_sum_snapshot();
   assert!(snap.contains(&(2, 300)));
   assert!(snap.contains(&(1, 30)));

   // t=3: retract one sample from group 1 — sum adjusts.
   s.sample_remove((1, 10));
   s.commit();
   let snap: Vec<_> = s.group_sum_snapshot();
   assert!(snap.contains(&(1, 20)));
   // Old value gone.
   assert!(!snap.contains(&(1, 30)));
}

/// Raw-DD usage pattern — bypass every convenience method, drive the
/// worker directly. Mirrors DD's canonical example but WITHOUT the
/// `timely::execute(|worker| { … })` closure — here the worker lives
/// outside in a regular variable, so the control flow isn't trapped in
/// a callback.
#[ntest_timeout::timeout(1000)]
#[test]
fn direct_raw_dd_no_callbacks() {
   let mut s = DirectTcS::session();

   // Load — `edge` is a public InputSession, full DD API available.
   for person in 0i32..10 {
      s.edge.insert((person / 2, person));
   }
   s.advance_all_to(1);

   // Drive the worker directly. `worker` is a public field.
   while s.probe.less_than(&1) {
      s.worker.step();
   }

   // Pull deltas straight from the sink — no `refresh_deltas` or
   // `_state` book-keeping required.
   let mut deltas: Vec<_> = s.path_sink.drain_deltas()
      .into_iter()
      .filter(|(_, d)| *d > 0)
      .map(|(t, _)| t)
      .collect();
   deltas.sort();
   // Binary-tree parent relation has paths (p/2, p) for each p in 0..10.
   // Plus derived: (p/4, p) transitively, etc.
   assert!(deltas.contains(&(0, 0)));   // 0 parents itself (0/2=0)
   assert!(deltas.contains(&(2, 5)));   // 5→2 (5/2=2)
   assert!(deltas.contains(&(0, 5)));   // 5→2→1→0 transitively
}

// ---------------------------------------------------------------------------
// 2.5. Diff-consistency edge cases — retraction cascade correctness.
// ---------------------------------------------------------------------------

ascent! {
   #![backend(dd)]
   pub struct HappyS;

   relation person(i32);
   relation sad(i32);
   relation happy(i32);

   happy(*p) <-- person(p), !sad(p);
}

/// Negation round-trip — adding to the negated relation retracts derived
/// facts; removing from it re-adds them. Exercises DD's antijoin with
/// both positive and negative diffs.
#[ntest_timeout::timeout(1000)]
#[test]
fn session_negation_roundtrip() {
   let mut s = HappyS::session();

   // t=1: seed with three people, nobody sad.
   s.person_insert((1,));
   s.person_insert((2,));
   s.person_insert((3,));
   s.commit();
   let mut snap = s.happy_snapshot();
   snap.sort();
   assert_eq!(snap, vec![(1,), (2,), (3,)]);

   // t=2: person 2 becomes sad → happy(2) must retract.
   s.sad_insert((2,));
   s.commit();
   let mut snap = s.happy_snapshot();
   snap.sort();
   assert_eq!(snap, vec![(1,), (3,)]);
   // The delta for this commit contains a retraction of (2,).
   let retracted: Vec<_> = s.happy_deltas().into_iter().filter(|(_, d)| *d < 0).map(|(t, _)| t).collect();
   assert!(retracted.contains(&(2,)), "expected (2,) retraction, got {retracted:?}");

   // t=3: person 2 cheers up → happy(2) must re-add.
   s.sad_remove((2,));
   s.commit();
   let mut snap = s.happy_snapshot();
   snap.sort();
   assert_eq!(snap, vec![(1,), (2,), (3,)]);
   let added: Vec<_> = s.happy_deltas().into_iter().filter(|(_, d)| *d > 0).map(|(t, _)| t).collect();
   assert!(added.contains(&(2,)), "expected (2,) re-insertion, got {added:?}");
}

/// Retracting an edge whose derived paths have ALTERNATE derivations
/// must NOT retract those paths. Shows reduce's multiplicity-aware
/// semantics: path(1,3) has two derivations (direct via edge(1,3) and
/// 2-hop via 1→2→3); removing edge(1,3) leaves the 2-hop derivation
/// intact, so path(1,3) stays in the snapshot.
#[ntest_timeout::timeout(1000)]
#[test]
fn session_retraction_alt_derivation_survives() {
   let mut s = ChanTcS::session();

   // Triangle: 1→2, 2→3, AND a direct 1→3. path(1,3) has two derivations.
   s.edge_insert((1, 2));
   s.edge_insert((2, 3));
   s.edge_insert((1, 3));
   s.commit();
   let mut snap = s.path_snapshot();
   snap.sort();
   assert_eq!(snap, vec![(1, 2), (1, 3), (2, 3)]);

   // Retract the direct edge. 2-hop derivation of path(1,3) still holds.
   s.edge_remove((1, 3));
   s.commit();
   let mut snap = s.path_snapshot();
   snap.sort();
   // path(1,3) SURVIVES — alternate derivation keeps it alive.
   assert_eq!(snap, vec![(1, 2), (1, 3), (2, 3)]);

   // The delta for this commit should be *empty* for path (no net change)
   // because the cancelled derivation was rebalanced by the other one.
   let net_diff_for_1_3: i32 =
      s.path_deltas().iter().filter(|(t, _)| *t == (1, 3)).map(|(_, d)| *d).sum();
   assert_eq!(net_diff_for_1_3, 0, "path(1,3) shouldn't churn in deltas");
}

/// Re-insertion after retraction — idempotent state.
#[ntest_timeout::timeout(1000)]
#[test]
fn session_reinsert_after_retract() {
   let mut s = ChanTcS::session();

   s.edge_insert((1, 2));
   s.edge_insert((2, 3));
   s.commit();
   assert_eq!(s.path_snapshot().len(), 3);

   // Remove then re-insert the same edge.
   s.edge_remove((2, 3));
   s.commit();
   let snap = s.path_snapshot();
   assert_eq!(snap, vec![(1, 2)]);

   s.edge_insert((2, 3));
   s.commit();
   let mut snap = s.path_snapshot();
   snap.sort();
   assert_eq!(snap, vec![(1, 2), (1, 3), (2, 3)]);
}

/// Lattice loosens when the tight value is retracted. The shortest-path
/// lattice should fall back to the next-best value once the winning edge
/// is gone — proves reduce-over-lattice handles negative diffs correctly.
#[ntest_timeout::timeout(1000)]
#[test]
fn session_lattice_loosens_on_retract() {
   let mut s = ShortestPathS::session();

   // Both a long (10) and short (3) direct edge from 1→2.
   s.edge_insert((1, 2, 10));
   s.edge_insert((1, 2, 3));
   s.commit();
   assert!(s.shortest_snapshot().contains(&(1, 2, Dual(3))));

   // Retract the short edge. Lattice value must loosen to 10.
   s.edge_remove((1, 2, 3));
   s.commit();
   let snap = s.shortest_snapshot();
   assert!(snap.contains(&(1, 2, Dual(10))),
      "expected lattice to fall back to 10 after retracting 3, got {snap:?}");
   assert!(!snap.contains(&(1, 2, Dual(3))),
      "stale lattice value shouldn't persist: {snap:?}");
}

/// Aggregator group empties → the aggregated row must retract. Refill
/// → new row appears with the new aggregate.
///
/// **DD vs. batch semantic divergence:** batch Ascent calls the aggregator
/// function even for empty groups, so `sum([]) = 0` produces an output
/// row. DD's `reduce` operator is set-semantic — it only fires for keys
/// actually present in the input, so an empty group yields NO row at all
/// (same as `min([])`/`max([])` would in batch). This matches min/max
/// naturally but diverges from sum/count's "identity fires" behavior.
/// Documented; users relying on empty-group-emits-identity must arrange
/// for the group key itself to always be present (e.g. via a base-case
/// contribution).
#[ntest_timeout::timeout(1000)]
#[test]
fn session_aggregator_empty_and_refill() {
   let mut s = GroupSumS::session();

   // t=1: seed group 1.
   s.sample_insert((1, 5));
   s.sample_insert((1, 7));
   s.commit();
   assert!(s.group_sum_snapshot().contains(&(1, 12)));

   // t=2: empty group 1. The aggregated row retracts; no replacement
   // because reduce doesn't fire for an empty key.
   s.sample_remove((1, 5));
   s.sample_remove((1, 7));
   s.commit();
   let snap = s.group_sum_snapshot();
   assert!(!snap.iter().any(|(k, _)| *k == 1),
      "group 1's aggregate should retract, got {snap:?}");

   // t=3: refill with different values. Row re-appears with new aggregate.
   s.sample_insert((1, 100));
   s.commit();
   let snap = s.group_sum_snapshot();
   assert!(snap.contains(&(1, 100)));
}

// ---------------------------------------------------------------------------
// 3. Real channel-driven stream — producer and consumer decoupled.
// ---------------------------------------------------------------------------

ascent! {
   #![backend(dd)]
   pub struct ChanTcS;

   relation edge(i32, i32);
   relation path(i32, i32);

   path(x, y) <-- edge(x, y);
   path(x, z) <-- edge(x, y), path(y, z);
}

enum Event {
   Insert(i32, i32),
   Remove(i32, i32),
   Commit,
}

/// Drive the session from a `Receiver<Event>`. Each `Commit` event flushes
/// the dataflow and collects deltas. Returns all observed path deltas in
/// order of appearance (one Vec per commit).
fn drive_stream(rx: mpsc::Receiver<Event>) -> Vec<Vec<((i32, i32), i32)>> {
   let mut s = ChanTcS::session();
   let mut batches: Vec<Vec<((i32, i32), i32)>> = Vec::new();
   for ev in rx {
      match ev {
         Event::Insert(a, b) => s.edge_insert((a, b)),
         Event::Remove(a, b) => s.edge_remove((a, b)),
         Event::Commit => {
            s.commit();
            batches.push(s.path_deltas());
         },
      }
   }
   batches
}

// ---------------------------------------------------------------------------
// Direct `InputSession` usage — skip `commit()`, drive DD natively.
// ---------------------------------------------------------------------------

ascent! {
   #![backend(dd)]
   pub struct DirectTcS;

   relation edge(i32, i32);
   relation path(i32, i32);

   path(x, y) <-- edge(x, y);
   path(x, z) <-- edge(x, y), path(y, z);
}

/// Demonstrates multi-epoch batching: push several logical timesteps worth
/// of data before a single step + refresh. DD handles all epochs together
/// (batches get consolidated) — lower overhead than commit-per-epoch.
///
/// Note: every relation (including derived `path`) has an `InputSession`;
/// the output frontier is the meet of all input frontiers, so you must
/// advance ALL of them to the target time. `advance_all_to(t)` does this.
#[ntest_timeout::timeout(1000)]
#[test]
fn direct_input_multi_epoch_pipelined() {
   let mut s = DirectTcS::session();

   // Epoch 1: seed the graph.
   s.edge.insert((1, 2));
   s.edge.insert((2, 3));
   // Epoch 2: retract the middle edge, add a new edge.
   s.edge.advance_to(1);
   s.edge.insert((3, 4));
   s.edge.advance_to(2);
   s.edge.remove((2, 3));
   s.edge.advance_to(3);

   // Bring ALL inputs (including derived `path`) to the target epoch and
   // flush in one shot, then let outputs catch up.
   s.advance_all_to(3);
   s.step_until(3);
   s.refresh_deltas();

   // Final state: edges 1→2 and 3→4 remain; paths only via them.
   let mut snap = s.path_snapshot();
   snap.sort();
   assert_eq!(snap, vec![(1, 2), (3, 4)]);
}

/// Show `step_until_idle` — don't name a time, just let the worker catch
/// up to whatever each input's current time is.
#[ntest_timeout::timeout(1000)]
#[test]
fn direct_input_step_until_idle() {
   let mut s = DirectTcS::session();

   s.edge.insert((1, 2));
   s.edge.insert((2, 3));
   // Advance + flush every input to time 1 so the probe can make progress.
   s.advance_all_to(1);

   s.step_until_idle();
   s.refresh_deltas();

   let mut snap = s.path_snapshot();
   snap.sort();
   assert_eq!(snap, vec![(1, 2), (1, 3), (2, 3)]);
}

/// Demonstrates checking the probe directly for frontier observation.
#[ntest_timeout::timeout(1000)]
#[test]
fn direct_input_probe_access() {
   let mut s = DirectTcS::session();

   s.edge.insert((1, 2));
   s.advance_all_to(5);

   // Probe hasn't made progress yet.
   assert!(s.probe().less_than(&5));

   // Step manually until it has.
   while s.probe().less_than(&5) {
      s.step();
   }
   s.refresh_deltas();

   assert_eq!(s.path_snapshot(), vec![(1, 2)]);
}

#[ntest_timeout::timeout(1000)]
#[test]
fn session_mpsc_producer_consumer() {
   let (tx, rx) = mpsc::channel::<Event>();

   // Producer — could be another thread, a timer, tokio stream, etc. Here
   // we batch events inline: inserts then a commit marker, repeat.
   tx.send(Event::Insert(1, 2)).unwrap();
   tx.send(Event::Insert(2, 3)).unwrap();
   tx.send(Event::Commit).unwrap();

   tx.send(Event::Insert(3, 4)).unwrap();
   tx.send(Event::Commit).unwrap();

   tx.send(Event::Remove(2, 3)).unwrap();
   tx.send(Event::Commit).unwrap();
   drop(tx);

   let batches = drive_stream(rx);
   assert_eq!(batches.len(), 3);

   // Batch 1: deltas for initial (1,2), (2,3), plus derived (1,3).
   let b1: Vec<_> =
      batches[0].iter().filter(|(_, d)| *d > 0).map(|(t, _)| *t).collect::<std::collections::HashSet<_>>().into_iter().collect();
   for t in [(1, 2), (2, 3), (1, 3)] {
      assert!(b1.contains(&t), "batch 1 missing {t:?}: {b1:?}");
   }

   // Batch 2: (3,4) direct, (2,4) via 2→3→4, (1,4) via 1→2→3→4.
   let b2: std::collections::HashSet<_> =
      batches[1].iter().filter(|(_, d)| *d > 0).map(|(t, _)| *t).collect();
   for t in [(3, 4), (2, 4), (1, 4)] {
      assert!(b2.contains(&t), "batch 2 missing {t:?}: {b2:?}");
   }

   // Batch 3: retracted the 2→3 middle link → all paths through it vanish.
   let retracted: std::collections::HashSet<_> =
      batches[2].iter().filter(|(_, d)| *d < 0).map(|(t, _)| *t).collect();
   for t in [(2, 3), (1, 3), (2, 4), (1, 4)] {
      assert!(retracted.contains(&t), "batch 3 missing retraction of {t:?}: {retracted:?}");
   }
}
