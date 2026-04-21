//! Phase 3a coverage: rich clause args (wildcards, literals, repeated vars)
//! and inline `if` / `let` conditions.
//!
//! Generators, `if let`, negation, and aggregation are later sub-phases.

use ascent::ascent;

// ---------------------------------------------------------------------------
// Wildcards in body clauses.
// ---------------------------------------------------------------------------

ascent! {
   #![backend(dd)]
   pub struct Wildcards;

   relation edge(i32, i32);
   relation sources(i32);

   // A node that has any outgoing edge.
   sources(x) <-- edge(x, _);
}

#[ntest_timeout::timeout(1000)]
#[test]
fn wildcard_column() {
   let mut p = Wildcards::default();
   p.edge = vec![(1, 2), (1, 3), (2, 4), (5, 6)];
   p.run();
   let mut src = p.sources.clone();
   src.sort();
   src.dedup();
   assert_eq!(src, vec![(1,), (2,), (5,)]);
}

// ---------------------------------------------------------------------------
// Literal constants in body clauses (equality filter).
// ---------------------------------------------------------------------------

ascent! {
   #![backend(dd)]
   pub struct LiteralFilter;

   relation tagged(i32, i32);    // (node, tag)
   relation tag7_nodes(i32);

   // Only pick up nodes tagged with literal 7.
   tag7_nodes(n) <-- tagged(n, 7);
}

#[ntest_timeout::timeout(1000)]
#[test]
fn literal_arg_filters() {
   let mut p = LiteralFilter::default();
   p.tagged = vec![(1, 7), (2, 3), (3, 7), (4, 9)];
   p.run();
   let mut got = p.tag7_nodes.clone();
   got.sort();
   assert_eq!(got, vec![(1,), (3,)]);
}

// ---------------------------------------------------------------------------
// Repeated variable within a single body clause: self-loops on a graph.
// ---------------------------------------------------------------------------

ascent! {
   #![backend(dd)]
   pub struct SelfLoops;

   relation edge(i32, i32);
   relation self_loop(i32);

   self_loop(x) <-- edge(x, x);
}

#[ntest_timeout::timeout(1000)]
#[test]
fn repeated_var_in_clause() {
   let mut p = SelfLoops::default();
   p.edge = vec![(1, 2), (2, 2), (3, 3), (4, 5)];
   p.run();
   let mut got = p.self_loop.clone();
   got.sort();
   assert_eq!(got, vec![(2,), (3,)]);
}

// ---------------------------------------------------------------------------
// Inline `if` condition on bound vars after a clause.
// ---------------------------------------------------------------------------

ascent! {
   #![backend(dd)]
   pub struct IfCond;

   relation num(i32);
   relation big(i32);

   big(x) <-- num(x), if *x > 10;
}

#[ntest_timeout::timeout(1000)]
#[test]
fn if_condition_on_clause() {
   let mut p = IfCond::default();
   p.num = vec![(1,), (5,), (11,), (42,)];
   p.run();
   let mut got = p.big.clone();
   got.sort();
   assert_eq!(got, vec![(11,), (42,)]);
}

// ---------------------------------------------------------------------------
// `let` binding that introduces a new variable from an expression over bound
// vars. Used downstream in the head.
// ---------------------------------------------------------------------------

ascent! {
   #![backend(dd)]
   pub struct LetBinding;

   relation pair(i32, i32);
   relation sum(i32);

   sum(s) <-- pair(a, b), let s = a + b;
}

#[ntest_timeout::timeout(1000)]
#[test]
fn let_binding_extends_bound_vars() {
   let mut p = LetBinding::default();
   p.pair = vec![(1, 2), (10, 5), (3, 4)];
   p.run();
   let mut got = p.sum.clone();
   got.sort();
   assert_eq!(got, vec![(3,), (7,), (15,)]);
}

// ---------------------------------------------------------------------------
// Negation: `!R(args)` desugars to `agg () = not() in R(args)` in HIR.
// Lower as `.antijoin` so this flows through with just the Phase 3b wiring.
// ---------------------------------------------------------------------------

ascent! {
   #![backend(dd)]
   pub struct Negation;

   relation node(i32);
   relation edge(i32, i32);
   relation sink(i32);  // nodes with no outgoing edge

   sink(x) <-- node(x), ! edge(x, _);
}

#[ntest_timeout::timeout(1000)]
#[test]
fn negation_of_any_outgoing_edge() {
   let mut p = Negation::default();
   p.node = vec![(1,), (2,), (3,), (4,)];
   p.edge = vec![(1, 2), (2, 3)];
   p.run();
   let mut got = p.sink.clone();
   got.sort();
   // 1 and 2 have outgoing edges; 3 and 4 are sinks.
   assert_eq!(got, vec![(3,), (4,)]);
}

ascent! {
   #![backend(dd)]
   pub struct NegationWithLit;

   relation candidate(i32);
   relation rejected(i32, i32);   // (node, reason_code)
   relation approved(i32);

   // Approved iff the candidate is not rejected for reason 7.
   approved(x) <-- candidate(x), ! rejected(x, 7);
}

#[ntest_timeout::timeout(1000)]
#[test]
fn negation_with_literal_arg() {
   let mut p = NegationWithLit::default();
   p.candidate = vec![(1,), (2,), (3,)];
   p.rejected = vec![(1, 5), (2, 7), (3, 9)];
   p.run();
   let mut got = p.approved.clone();
   got.sort();
   // 1 rejected for reason 5 (not 7) → still approved.
   // 2 rejected for reason 7 → NOT approved.
   // 3 rejected for reason 9 → still approved.
   assert_eq!(got, vec![(1,), (3,)]);
}

// ---------------------------------------------------------------------------
// Batch-mode negation (FlowLog-parity pos/neg/normalize antijoin).
//
// `Present` diff isn't `Abelian`, so DD's `.antijoin()` can't be used
// directly. Codegen instead re-encodes both sides as `i32` (+1 for R, -1
// for R⋈L), concat's them, then `threshold_semigroup` filters back to
// `Present` where the net weight is positive.
// ---------------------------------------------------------------------------

ascent! {
   #![backend(dd)]
    #![dd(mode = batch)]
   pub struct NegationBatch;

   relation node(i32);
   relation edge(i32, i32);
   relation sink(i32);

   sink(x) <-- node(x), ! edge(x, _);
}

#[ntest_timeout::timeout(5000)]
#[test]
fn negation_batch_of_any_outgoing_edge() {
   let mut p = NegationBatch::default();
   p.node = vec![(1,), (2,), (3,), (4,)];
   p.edge = vec![(1, 2), (2, 3)];
   p.run();
   let mut got = p.sink.clone();
   got.sort();
   assert_eq!(got, vec![(3,), (4,)]);
}

ascent! {
   #![backend(dd)]
    #![dd(mode = batch)]
   pub struct NegationBatchWithLit;

   relation candidate(i32);
   relation rejected(i32, i32);
   relation approved(i32);

   approved(x) <-- candidate(x), ! rejected(x, 7);
}

#[ntest_timeout::timeout(5000)]
#[test]
fn negation_batch_with_literal_arg() {
   let mut p = NegationBatchWithLit::default();
   p.candidate = vec![(1,), (2,), (3,)];
   p.rejected = vec![(1, 5), (2, 7), (3, 9)];
   p.run();
   let mut got = p.approved.clone();
   got.sort();
   assert_eq!(got, vec![(1,), (3,)]);
}

// ---------------------------------------------------------------------------
// Batch-mode aggregation.
//
// `reduce` requires `R2: Abelian`; `Present` isn't Abelian. Codegen wraps
// the reduce in a `Present → i32 → reduce → Present` round-trip so the
// inner reduce sees an Abelian diff, and the result rejoins the batch
// dataflow as a normal Present-diff Collection.
// ---------------------------------------------------------------------------

ascent! {
   #![backend(dd)]
    #![dd(mode = batch)]
   pub struct AggBatchCount;

   relation edge(u32, u32);
   relation path(u32, u32);
   relation num_paths(usize);

   path(a, b) <-- edge(a, b);
   path(a, c) <-- path(a, b), edge(b, c);
   num_paths(n) <-- agg n = ::ascent::aggregators::count() in path(_, _);
}

#[ntest_timeout::timeout(5000)]
#[test]
fn agg_batch_count() {
   let mut p = AggBatchCount::default();
   p.edge = vec![(1, 2), (2, 3), (3, 4)];
   p.run();
   // Paths: (1,2) (2,3) (3,4) (1,3) (2,4) (1,4) = 6.
   assert_eq!(p.num_paths, vec![(6,)]);
}

ascent! {
   #![backend(dd)]
    #![dd(mode = batch)]
   pub struct AggBatchMinGrouped;

   relation bar(i32, i32, i32);
   relation grp(i32, i32);
   relation min_per_group(i32, i32, i32);

   // `grp` dedupes (x, y); batch's Present diff is set-semantic so the
   // projection drops the multiplicity. One row per group avoids the
   // accum-fanout that would multiply the agg output.
   grp(x, y) <-- bar(x, y, _);
   min_per_group(x, y, z) <--
      grp(x, y),
      agg z = ::ascent::aggregators::min(v) in bar(x, y, v);
}

#[ntest_timeout::timeout(5000)]
#[test]
fn agg_batch_min_grouped() {
   let mut p = AggBatchMinGrouped::default();
   p.bar = vec![(1, 1, 9), (1, 1, 3), (1, 1, 5), (1, 2, 7), (1, 2, 2)];
   p.run();
   let mut got = p.min_per_group.clone();
   got.sort();
   assert_eq!(got, vec![(1, 1, 3), (1, 2, 2)]);
}

ascent! {
   #![backend(dd)]
    #![dd(mode = batch)]
   pub struct AggBatchSum;

   relation foo(i32, i32);
   relation foo_keys(i32);
   relation total(i32, i32);

   foo_keys(x) <-- foo(x, _);
   total(x, s) <--
      foo_keys(x),
      agg s = ::ascent::aggregators::sum(v) in foo(x, v);
}

#[ntest_timeout::timeout(5000)]
#[test]
fn agg_batch_sum() {
   let mut p = AggBatchSum::default();
   p.foo = vec![(1, 10), (1, 20), (1, 30), (2, 5), (2, 7)];
   p.run();
   let mut got = p.total.clone();
   got.sort();
   assert_eq!(got, vec![(1, 60), (2, 12)]);
}

// ---------------------------------------------------------------------------
// Generators: `for pat in expr` iterates an arbitrary Rust iterator inside a
// rule body, binding each item as a new variable. This is the idiomatic way
// Ascent programs seed relations from outer-scope data.
// ---------------------------------------------------------------------------

#[ntest_timeout::timeout(1000)]
#[test]
fn generator_from_outer_vec() {
   let data = vec![(1, 10), (2, 20), (3, 30)];
   let res = ascent::ascent_run! {
      #![backend(dd)]
      relation scaled(i32, i32);
      // For each (k, v) in the outer Vec, derive (k, v * 2).
      scaled(k, v * 2) <-- for (k, v) in data.iter().copied();
   };
   let mut s = res.scaled.clone();
   s.sort();
   assert_eq!(s, vec![(1, 20), (2, 40), (3, 60)]);
}

#[ntest_timeout::timeout(1000)]
#[test]
fn generator_with_prior_binding() {
   let range_end: i32 = 4;
   let res = ascent::ascent_run! {
      #![backend(dd)]
      relation seed(i32);
      relation expanded(i32, i32);
      // Generator uses a bound var (`x`) from a prior clause as its iterator source.
      expanded(x, y) <-- seed(x), for y in 1..range_end;
   };
   let mut got = res.expanded;
   got.sort();
   assert_eq!(got.iter().filter(|(x, _)| *x == 1).count(), 0); // no seed yet
   // Seed is empty above; this tests compilability and the join-with-generator path.
   assert!(got.is_empty());
}

#[ntest_timeout::timeout(1000)]
#[test]
fn generator_extends_via_prior_var() {
   let mut prog = AscentPrep::default();
   prog.base = vec![(10,), (20,)];
   prog.run();
   let mut got = prog.expanded.clone();
   got.sort();
   // For each base(n), generate (n, n+k) for k in 1..3 — so {11, 12, 21, 22}.
   assert_eq!(got, vec![(10, 11), (10, 12), (20, 21), (20, 22)]);
}

ascent! {
   #![backend(dd)]
   pub struct AscentPrep;

   relation base(i32);
   relation expanded(i32, i32);

   expanded(n, n + k) <-- base(n), for k in 1..3i32;
}

// ---------------------------------------------------------------------------
// `if let`: pattern-match inside the body; on match, bind and continue.
// ---------------------------------------------------------------------------

ascent! {
   #![backend(dd)]
   pub struct IfLetCond;

   relation maybe(i32, Option<i32>);
   relation present(i32, i32);

   present(k, v) <-- maybe(k, opt), if let Some(v) = opt;
}

#[ntest_timeout::timeout(1000)]
#[test]
fn if_let_filters_and_binds() {
   let mut p = IfLetCond::default();
   p.maybe = vec![(1, Some(10)), (2, None), (3, Some(30))];
   p.run();
   let mut got = p.present.clone();
   got.sort();
   assert_eq!(got, vec![(1, 10), (3, 30)]);
}

// ---------------------------------------------------------------------------
// Combination: join + wildcard + if + let, all in one rule.
// ---------------------------------------------------------------------------

ascent! {
   #![backend(dd)]
   pub struct Combined;

   relation edge(i32, i32);
   relation weight(i32, i32);   // (edge_from, weight)
   relation heavy_doubled(i32, i32);

   // Pick edges whose weight is > 2, emit (from, weight*2).
   heavy_doubled(from, dw) <-- edge(from, _), weight(from, w), if *w > 2, let dw = w * 2;
}

#[ntest_timeout::timeout(1000)]
#[test]
fn combined_wildcard_join_if_let() {
   let mut p = Combined::default();
   p.edge = vec![(1, 100), (2, 200), (3, 300)];
   p.weight = vec![(1, 1), (2, 5), (3, 10)];
   p.run();
   let mut got = p.heavy_doubled.clone();
   got.sort();
   assert_eq!(got, vec![(2, 10), (3, 20)]);
}
