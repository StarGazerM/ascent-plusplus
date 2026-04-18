//! Incremental `<Name>Session` API: long-lived worker, push updates, observe
//! deltas or snapshot after each `commit`.

use ascent::ascent;

ascent! {
   #![backend(dd)]
   pub struct TcS;

   relation edge(i32, i32);
   relation path(i32, i32);

   path(x, y) <-- edge(x, y);
   path(x, z) <-- edge(x, y), path(y, z);
}

#[test]
fn session_basic_insert_then_commit() {
   let mut s = TcS::session();

   s.edge_insert((1, 2));
   s.edge_insert((2, 3));
   s.commit();

   let mut p = s.path_snapshot();
   p.sort();
   assert_eq!(p, vec![(1, 2), (1, 3), (2, 3)]);
}

#[test]
fn session_incremental_delta_only() {
   let mut s = TcS::session();

   // Commit 1: seed graph.
   s.edge_insert((1, 2));
   s.edge_insert((2, 3));
   s.commit();

   // Commit 2: extend chain; delta should only contain the *new* paths.
   s.edge_insert((3, 4));
   s.commit();

   let mut delta: Vec<_> = s.path_deltas().into_iter().filter(|(_, d)| *d > 0).map(|(t, _)| t).collect();
   delta.sort();
   // With 3→4 added, the new paths are those ending at 4:
   //   (1,4) via 1→2→3→4, (2,4) via 2→3→4, (3,4) direct.
   assert_eq!(delta, vec![(1, 4), (2, 4), (3, 4)]);

   // Full snapshot reflects everything including the pre-commit-1 set.
   let mut snap = s.path_snapshot();
   snap.sort();
   assert_eq!(snap, vec![(1, 2), (1, 3), (1, 4), (2, 3), (2, 4), (3, 4)]);
}

#[test]
fn session_retraction() {
   let mut s = TcS::session();
   s.edge_insert((1, 2));
   s.edge_insert((2, 3));
   s.edge_insert((3, 4));
   s.commit();

   // Retract the middle link — everything through 2→3 disappears.
   s.edge_remove((2, 3));
   s.commit();

   let mut snap = s.path_snapshot();
   snap.sort();
   // Only direct edges remain reachable: (1,2) and (3,4).
   assert_eq!(snap, vec![(1, 2), (3, 4)]);

   // The deltas from this commit are retractions.
   let retracted: Vec<_> = s.path_deltas().into_iter().filter(|(_, d)| *d < 0).map(|(t, _)| t).collect();
   // At least (2,3), (1,3), (1,4), (2,4) are retracted.
   for t in [(2, 3), (1, 3), (1, 4), (2, 4)] {
      assert!(retracted.contains(&t), "expected retraction of {t:?} in {retracted:?}");
   }
}
