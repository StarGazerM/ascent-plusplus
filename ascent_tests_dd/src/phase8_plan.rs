//! `#[plan(variant(delta=N, order=[...]))]` user-supplied compilation plans.
//!
//! A plan forces the semi-naive expansion to emit exactly the variants the
//! user lists, each with the given body-item permutation and delta clause.
//! The result must be observationally equivalent to the un-annotated rule
//! when the plan's variants cover every IDB clause (i.e., the rule remains
//! fully semi-naïve-correct).

use ascent::ascent;

// ---------------------------------------------------------------------------
// 1. Plan is observationally identical to un-planned for full-variant-set.
// ---------------------------------------------------------------------------

ascent! {
   #![backend(dd)]
   pub struct TcDefault;
   relation edge(i32, i32);
   relation path(i32, i32);
   path(x, y) <-- edge(x, y);
   path(x, z) <-- edge(x, y), path(y, z);
}

ascent! {
   #![backend(dd)]
   pub struct TcPlanned;
   relation edge(i32, i32);
   relation path(i32, i32);
   path(x, y) <-- edge(x, y);

   // Natural order with delta on path (the only IDB clause). Equivalent
   // to the auto-generated variant.
   #[plan(variant(delta = 1, order = [0, 1]))]
   path(x, z) <-- edge(x, y), path(y, z);
}

#[ntest_timeout::timeout(1000)]
#[test]
fn plan_matches_default_for_tc() {
   let run = |ed: &[(i32, i32)]| -> Vec<(i32, i32)> {
      let mut prog = TcDefault::default();
      prog.edge = ed.to_vec();
      prog.run();
      let mut p = prog.path;
      p.sort();
      p
   };
   let run_plan = |ed: &[(i32, i32)]| -> Vec<(i32, i32)> {
      let mut prog = TcPlanned::default();
      prog.edge = ed.to_vec();
      prog.run();
      let mut p = prog.path;
      p.sort();
      p
   };
   let edges = vec![(1, 2), (2, 3), (3, 4), (1, 5), (5, 6)];
   assert_eq!(run(&edges), run_plan(&edges));
}

// ---------------------------------------------------------------------------
// 2. Reordering clauses — same semantics, plan picks different seed.
// ---------------------------------------------------------------------------

ascent! {
   #![backend(dd)]
   pub struct TcReversed;
   relation edge(i32, i32);
   relation path(i32, i32);
   path(x, y) <-- edge(x, y);

   // Stream `path` first (order = [1, 0]); edge becomes the arranged side.
   // Delta still points at the `path` clause in the ORIGINAL numbering.
   #[plan(variant(delta = 1, order = [1, 0]))]
   path(x, z) <-- edge(x, y), path(y, z);
}

#[ntest_timeout::timeout(1000)]
#[test]
fn plan_reordered_same_results() {
   let mut d = TcDefault::default();
   d.edge = vec![(1, 2), (2, 3), (3, 4)];
   d.run();
   let mut dp = d.path;
   dp.sort();

   let mut r = TcReversed::default();
   r.edge = vec![(1, 2), (2, 3), (3, 4)];
   r.run();
   let mut rp = r.path;
   rp.sort();

   assert_eq!(dp, rp);
}

// ---------------------------------------------------------------------------
// 3. Rule with two IDB clauses — plan lists BOTH semi-naive variants.
// ---------------------------------------------------------------------------

ascent! {
   #![backend(dd)]
   pub struct ReachDefault;
   relation seed(i32);
   relation reach(i32, i32);
   reach(x, x) <-- seed(x);
   reach(x, z) <-- reach(x, y), reach(y, z);
}

ascent! {
   #![backend(dd)]
   pub struct ReachPlanned;
   relation seed(i32);
   relation reach(i32, i32);
   reach(x, x) <-- seed(x);

   // Both IDB clauses need a delta variant for correctness. Either variant
   // alone would miss derivations.
   #[plan(
      variant(delta = 0, order = [0, 1]),
      variant(delta = 1, order = [1, 0]),
   )]
   reach(x, z) <-- reach(x, y), reach(y, z);
}

#[ntest_timeout::timeout(1000)]
#[test]
fn plan_both_variants_for_two_idb_rule() {
   let mut d = ReachDefault::default();
   d.seed = vec![(1,), (2,)];
   // Chain edges to make reach non-trivial:
   // seed {1, 2} → reach {(1,1), (2,2)} only — reach is transitive over reach itself,
   // which without any (a,b) tuples can't grow.
   d.run();
   let mut dr = d.reach;
   dr.sort();

   let mut p = ReachPlanned::default();
   p.seed = vec![(1,), (2,)];
   p.run();
   let mut pr = p.reach;
   pr.sort();

   assert_eq!(dr, pr);
}

// ---------------------------------------------------------------------------
// 4. `delta=N` is documented as a no-op under DD. Verify: two plans that
//    differ ONLY in `delta` (same `order`) produce identical results,
//    confirming DD ignores the delta hint.
// ---------------------------------------------------------------------------

// Rule `reach(x, z) <-- reach(x, y), reach(y, z)` has TWO dynamic atoms,
// so delta can legitimately be 0 OR 1. Under batch these would expand to
// TWO different MIR rules (one delta'd per atom). Under DD both plans must
// produce the same output — DD collapses all deltas into a single join_core
// that sees deltas from either side automatically.

ascent! {
   #![backend(dd)]
   pub struct ReachDeltaA;
   relation seed(i32, i32);
   relation reach(i32, i32);
   reach(x, y) <-- seed(x, y);
   #[plan(variant(delta = 0, order = [0, 1]))]
   reach(x, z) <-- reach(x, y), reach(y, z);
}

ascent! {
   #![backend(dd)]
   pub struct ReachDeltaB;
   relation seed(i32, i32);
   relation reach(i32, i32);
   reach(x, y) <-- seed(x, y);
   // Same order, different delta. Under DD this must still match A.
   #[plan(variant(delta = 1, order = [0, 1]))]
   reach(x, z) <-- reach(x, y), reach(y, z);
}

#[ntest_timeout::timeout(1000)]
#[test]
fn plan_delta_is_noop_under_dd() {
   let seeds = vec![(1, 2), (2, 3), (3, 4), (5, 6)];

   let mut a = ReachDeltaA::default();
   a.seed = seeds.clone();
   a.run();
   let mut ar = a.reach;
   ar.sort();

   let mut b = ReachDeltaB::default();
   b.seed = seeds;
   b.run();
   let mut br = b.reach;
   br.sort();

   assert_eq!(ar, br);
}
