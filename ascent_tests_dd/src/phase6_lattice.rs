//! Phase 6a coverage: lattice relations via `reduce(lattice-join)` at
//! materialization points.
//!
//! Approach: a `lattice foo(K_cols..., L)` is stored as a regular
//! `Collection<(K_cols..., L), isize>`. At `sink.attach` and
//! `Variable::set` sites, instead of `.distinct()` we emit a `.reduce`
//! that groups by `K_cols` and lattice-joins the `L` values.
//!
//! Known perf limitation: DD's `reduce` re-runs the closure over the full
//! input multiset per group whenever that group's input changes. Lattice
//! relations with hot keys accumulating many contributions across iterations
//! are super-linear. Correctness first; Option B (Semigroup-as-diff) is the
//! performance follow-up.

use ascent::ascent;

// ---------------------------------------------------------------------------
// Trivial non-recursive lattice: take the max u32 observed per key.
// ---------------------------------------------------------------------------

ascent! {
   #![backend(dd)]
   pub struct MaxPerKey;

   relation observation(i32, u32);
   lattice best(i32, u32);

   // `u32` already has a `Lattice` impl (join = max).
   best(k, v) <-- observation(k, v);
}

#[ntest_timeout::timeout(1000)]
#[test]
fn lattice_max_per_key() {
   let mut p = MaxPerKey::default();
   p.observation = vec![(1, 5), (1, 10), (1, 3), (2, 7), (2, 2), (3, 100)];
   p.run();
   let mut b = p.best.clone();
   b.sort();
   assert_eq!(b, vec![(1, 10), (2, 7), (3, 100)]);
}

// ---------------------------------------------------------------------------
// Recursive lattice: shortest path on a DAG using `Dual<u32>` (which
// reverses the lattice order — max of Dual(u32) = min of u32).
// ---------------------------------------------------------------------------

use ascent::Dual;

ascent! {
   #![backend(dd)]
   pub struct ShortestPath;

   relation edge(i32, i32, u32);
   lattice shortest_path(i32, i32, Dual<u32>);

   shortest_path(x, y, Dual(*w)) <-- edge(x, y, w);
   shortest_path(x, z, Dual(w + l.0)) <-- edge(x, y, w), shortest_path(y, z, l);
}

#[ntest_timeout::timeout(1000)]
#[test]
fn lattice_shortest_path_dag() {
   let mut p = ShortestPath::default();
   // Graph: 1→2(30), 2→3(50), 1→3(40). Shortest 1→3 = 40 (direct), not 80.
   p.edge = vec![(1, 2, 30), (2, 3, 50), (1, 3, 40)];
   p.run();
   let mut sp = p.shortest_path.clone();
   sp.sort();
   // Expected:
   //   1→2 = 30
   //   1→3 = 40 (min of 40 and 30+50)
   //   2→3 = 50
   assert_eq!(sp, vec![(1, 2, Dual(30)), (1, 3, Dual(40)), (2, 3, Dual(50))]);
}
