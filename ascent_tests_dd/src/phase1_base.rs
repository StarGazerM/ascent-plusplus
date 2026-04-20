//! Phase 1 target tests: non-recursive rules, simple tuple types.
//!
//! Currently ignored; these flip green when Phase 1 (non-recursive
//! rule lowering via `worker.dataflow`) lands.

use ascent::ascent;

ascent! {
   #![backend(dd)]
   pub struct CrossJoin;

   relation a(i32);
   relation b(i32);
   relation ab(i32, i32);

   ab(x, y) <-- a(x), b(y);
}

#[ntest_timeout::timeout(1000)]
#[test]
fn cross_join_non_recursive() {
   let mut prog = CrossJoin::default();
   prog.a = vec![(1,), (2,)];
   prog.b = vec![(10,), (20,)];
   prog.run();
   let mut ab = prog.ab.clone();
   ab.sort();
   assert_eq!(ab, vec![(1, 10), (1, 20), (2, 10), (2, 20)]);
}

ascent! {
   #![backend(dd)]
   pub struct TwoHop;

   relation edge(i32, i32);
   relation two_hop(i32, i32);

   // Non-recursive one-shared-var join — hits the key=(y,) branch that
   // cross_join doesn't exercise.
   two_hop(x, z) <-- edge(x, y), edge(y, z);
}

#[ntest_timeout::timeout(1000)]
#[test]
fn two_hop_shared_var() {
   let mut prog = TwoHop::default();
   prog.edge = vec![(1, 2), (2, 3), (3, 4), (5, 6)];
   prog.run();
   let mut tw = prog.two_hop.clone();
   tw.sort();
   // 1→2→3, 2→3→4; 3→4 has no continuation; 5→6 dead end.
   assert_eq!(tw, vec![(1, 3), (2, 4)]);
}

ascent! {
   #![backend(dd)]
   pub struct Tc;

   relation edge(i32, i32);
   relation path(i32, i32);

   path(x, y) <-- edge(x, y);
   path(x, z) <-- edge(x, y), path(y, z);
}

#[ntest_timeout::timeout(1000)]
#[test]
fn transitive_closure() {
   let mut prog = Tc::default();
   prog.edge = vec![(1, 2), (2, 3), (3, 4)];
   prog.run();
   let mut path = prog.path.clone();
   path.sort();
   assert_eq!(path, vec![(1, 2), (1, 3), (1, 4), (2, 3), (2, 4), (3, 4)]);
}
