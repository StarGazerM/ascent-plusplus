//! Phase 2 coverage: recursion via DD `Variable`, including SCCs with
//! multiple mutually-recursive dynamic relations.

use ascent::ascent;

// ---------------------------------------------------------------------------
// Single-dynamic-rel SCC with user-seeded facts survives the loop.
// ---------------------------------------------------------------------------

ascent! {
   #![backend(dd)]
   pub struct TcWithSeed;

   relation edge(i32, i32);
   relation path(i32, i32);

   path(x, y) <-- edge(x, y);
   path(x, z) <-- edge(x, y), path(y, z);
}

#[test]
fn tc_preserves_seeded_facts() {
   let mut prog = TcWithSeed::default();
   prog.edge = vec![(1, 2), (2, 3)];
   prog.path = vec![(99, 99)]; // seed fact not derivable from edges
   prog.run();
   let mut p = prog.path.clone();
   p.sort();
   assert!(p.contains(&(99, 99)), "user seed survived: {p:?}");
   assert!(p.contains(&(1, 3)), "derived tc still computed: {p:?}");
}

// ---------------------------------------------------------------------------
// Mutual recursion: two dynamic relations in the same SCC, each appearing
// in the other's body. Exercises the tuple-of-leaved-collections path in
// compile_looping_scc.
// ---------------------------------------------------------------------------

ascent! {
   #![backend(dd)]
   pub struct MutualTc;

   relation edge(i32, i32);
   relation back(i32, i32);
   relation ca(i32, i32);
   relation cb(i32, i32);

   // ca and cb are mutually recursive:
   //   each seeds from its own input and then extends via the other.
   ca(x, y) <-- edge(x, y);
   ca(x, z) <-- ca(x, y), cb(y, z);
   cb(x, y) <-- back(x, y);
   cb(x, z) <-- cb(x, y), ca(y, z);
}

#[test]
fn mutual_recursion_two_dyn_rels() {
   let mut prog = MutualTc::default();
   // Linear edge chain: 1→2→3. Back chain: 10→11→12.
   // Plus an inter-graph hop: 3→10 via 'edge' and 12→1 via 'back'.
   prog.edge = vec![(1, 2), (2, 3), (3, 10)];
   prog.back = vec![(10, 11), (11, 12), (12, 1)];
   prog.run();

   // Trace:
   //   ca(3,10) direct edge.
   //   ca(3,11) via ca(3,10) + cb(10,11).
   //   ca(3,12) via ca(3,11) + cb(11,12).   <-- proves ca consumed cb twice.
   //   cb(12,1) direct back.
   //   cb(12,2) via cb(12,1) + ca(1,2).
   //   cb(12,3) via cb(12,2) + ca(2,3).     <-- proves cb consumed ca twice.
   assert!(prog.ca.contains(&(3, 12)), "ca(3,12) should derive via cb twice; got: {:?}", prog.ca);
   assert!(prog.cb.contains(&(12, 3)), "cb(12,3) should derive via ca twice; got: {:?}", prog.cb);

   // ca(1, z) never escapes z=2 because there's no cb(2, _) anywhere — a
   // negative assertion that the closure genuinely respects mutual deps
   // rather than just flood-reaching everything.
   let from_one: Vec<_> = prog.ca.iter().filter(|(x, _)| *x == 1).collect();
   assert_eq!(from_one, vec![&(1, 2)], "ca(1,_) should only be (1,2); got: {from_one:?}");
}
