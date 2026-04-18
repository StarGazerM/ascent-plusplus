//! `#[plan(variant(delta=N, order=[..]))]` on the batch backend.
//!
//! Plan annotations pass through `compile_hir_rule_to_mir_rules` and
//! control semi-naive expansion for both backends. This file verifies
//! the batch backend accepts the same annotation and produces the same
//! results as an un-annotated rule.

use ascent::ascent;

ascent! {
   pub struct TcDefault;
   relation edge(i32, i32);
   relation path(i32, i32);
   path(x, y) <-- edge(x, y);
   path(x, z) <-- edge(x, y), path(y, z);
}

ascent! {
   pub struct TcPlanned;
   relation edge(i32, i32);
   relation path(i32, i32);
   path(x, y) <-- edge(x, y);

   // Natural order on the only IDB clause — observationally identical
   // to the un-annotated rule.
   #[plan(variant(delta = 1, order = [0, 1]))]
   path(x, z) <-- edge(x, y), path(y, z);
}

#[test]
fn batch_plan_same_as_default() {
   let edges = vec![(1, 2), (2, 3), (3, 4), (1, 5), (5, 6)];

   let mut d = TcDefault::default();
   d.edge = edges.clone();
   d.run();
   let mut dp = d.path;
   dp.sort();

   let mut p = TcPlanned::default();
   p.edge = edges;
   p.run();
   let mut pp = p.path;
   pp.sort();

   assert_eq!(dp, pp);
}
