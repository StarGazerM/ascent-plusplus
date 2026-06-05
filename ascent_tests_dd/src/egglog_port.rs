//! Egglog-on-ascent-dd ports. Each test here is a hand-translation of an
//! egglog program (typically via `scripts/egglog_te.sh`) to
//! `ascent! { #![backend(dd)] … }`. Acceptance criterion: the `(check …)`
//! facts from the original `.egg` source hold in the ported program.
//!
//! See `EGGLOG_PLAN.md` at the repo root for the full phase plan and the
//! six locked design decisions driving these ports.

use ascent::{ascent, Dual};

// ---------------------------------------------------------------------------
// Phase 1: pure-Datalog subset — no `(union …)`, no `(rewrite …)`,
// no `(function … :merge …)`. E-class IDs are trivial singletons, so we
// drop the ID column everywhere and the port is a mechanical 1-to-1 of
// the egglog rules.
//
// Source: others/egglog/tests/web-demo/points-to.egg
// ---------------------------------------------------------------------------

ascent! {
   #![backend(dd)]
   pub struct PointsTo;

   // `(datatype Stmt (New String ClassT) (Assign …) (Store …) (Load …))`
   // becomes one relation per constructor. The `(= x (New a b))` body
   // pattern would bind `x` to the statement's e-class, but `x` is never
   // referenced elsewhere in any rule — with no unions possible, the
   // e-class ID is dead weight, so we drop it entirely.
   //
   // `ClassT = (Class String)` and `FieldT = (Field String)` are single-
   // constructor newtypes over String; in the pure-Datalog encoding we
   // don't distinguish them from raw String.
   #[input] relation new_stmt(String, String);            // New(var, class)
   #[input] relation assign_stmt(String, String);         // Assign(dst, src)
   #[input] relation store_stmt(String, String, String);  // Store(dst, field, src)
   #[input] relation load_stmt(String, String, String);   // Load(dst, src, field)

   // `(relation VarPointsTo (String ClassT))` and
   // `(relation HeapPointsTo (ClassT FieldT ClassT))`.
   #[output] relation var_points_to(String, String);
   #[output] relation heap_points_to(String, String, String);

   // (rule ((= x (New a b))) ((VarPointsTo a b)))
   var_points_to(a.clone(), b.clone()) <-- new_stmt(a, b);

   // (rule ((= x (Assign v1 v2)) (VarPointsTo v2 c2))
   //       ((VarPointsTo v1 c2)))
   var_points_to(v1.clone(), c2.clone())
      <-- assign_stmt(v1, v2), var_points_to(v2, c2);

   // (rule ((= x (Load v1 v2 f)) (VarPointsTo v2 c1) (HeapPointsTo c1 f c2))
   //       ((VarPointsTo v1 c2)))
   var_points_to(v1.clone(), c2.clone())
      <-- load_stmt(v1, v2, f),
          var_points_to(v2, c1),
          heap_points_to(c1, f, c2);

   // (rule ((= x (Store v1 f v2)) (VarPointsTo v1 c1) (VarPointsTo v2 c2))
   //       ((HeapPointsTo c1 f c2)))
   heap_points_to(c1.clone(), f.clone(), c2.clone())
      <-- store_stmt(v1, f, v2),
          var_points_to(v1, c1),
          var_points_to(v2, c2);
}

fn s(x: &str) -> String { x.to_string() }

// ---------------------------------------------------------------------------
// Shared: content-addressed term IDs.
//
// `hash_term(tag, args)` is the universal ID-minting function for the Phase
// 2+ ports. Per design decision #3 in EGGLOG_PLAN.md, an e-class ID is the
// hash of `(constructor_tag, raw_arg_ids)` — deterministic, stateless, gives
// congruence-by-representation.
//
// 64 bits is sufficient: birthday-bound collision probability is below
// 10⁻⁵ at 10⁷ terms, well beyond any realistic eq-sat workload we'll run.
// If we ever need more, swap the mixer for a 128-bit one (two SipHasher
// halves, or BLAKE3 truncated) without changing call sites.
//
// Algorithm: FxHash-style rolling mix. `K` is a large odd constant
// (Knuth's multiplicative-hashing prime variant); `rotate_left(5)` breaks
// linear patterns. Deterministic forever — no library/version sensitivity.
// ---------------------------------------------------------------------------

fn hash_term(tag: u64, args: &[u64]) -> u64 {
   const K: u64 = 0x517cc1b727220a95;
   let mut h: u64 = 0xcbf29ce484222325;
   h = (h.rotate_left(5) ^ tag).wrapping_mul(K);
   for &a in args {
      h = (h.rotate_left(5) ^ a).wrapping_mul(K);
   }
   h
}

#[ntest_timeout::timeout(5000)]
#[test]
fn points_to_matches_egglog_checks() {
   // Mirrors the (let …) bindings from points-to.egg:
   //   $l1 = (New "o1" (Class "A"))
   //   $l2 = (New "o2" (Class "B"))
   //   $l3 = (Assign "o3" "o2")
   //   $l4 = (Store "o2" (Field "f") "o1")
   //   $l5 = (Load "r" "o3" (Field "f"))
   let mut p = PointsTo::default();
   p.new_stmt = vec![(s("o1"), s("A")), (s("o2"), s("B"))];
   p.assign_stmt = vec![(s("o3"), s("o2"))];
   p.store_stmt = vec![(s("o2"), s("f"), s("o1"))];
   p.load_stmt = vec![(s("r"), s("o3"), s("f"))];
   p.run();

   // The five (check …) statements from points-to.egg.
   assert!(p.var_points_to.contains(&(s("o1"), s("A"))),
           "check (VarPointsTo \"o1\" $A) failed; got {:?}", p.var_points_to);
   assert!(p.var_points_to.contains(&(s("o2"), s("B"))),
           "check (VarPointsTo \"o2\" $B) failed; got {:?}", p.var_points_to);
   assert!(p.var_points_to.contains(&(s("o3"), s("B"))),
           "check (VarPointsTo \"o3\" $B) failed; got {:?}", p.var_points_to);
   assert!(p.heap_points_to.contains(&(s("B"), s("f"), s("A"))),
           "check (HeapPointsTo $B $f $A) failed; got {:?}", p.heap_points_to);
   assert!(p.var_points_to.contains(&(s("r"), s("A"))),
           "check (VarPointsTo \"r\" $A) failed; got {:?}", p.var_points_to);
}

// ---------------------------------------------------------------------------
// Phase 2: smallest program exercising the full union recipe.
//
// Source: others/egglog/tests/repro-define.egg
//
//   (datatype Nat (S Nat))
//   (constructor ZeroConst () Nat)
//   (let $Zero (ZeroConst))
//   (let $two (S (S $Zero)))
//   (union $two (S (S (S $Zero))))
//   (check (= $two (S (S (S $Zero)))))
//
// In egglog's term-encoded form ($ scripts/egglog_te.sh repro-define.egg) the
// `(union …)` becomes an insertion into `__UF_Nat`, followed by the rebuild
// ruleset propagating equalities through view tables. For the ascent-dd port
// we skip egglog's machinery entirely and rely on the design triple:
//
//   1. Content-addressed IDs (`hash_term`) — every term's e-class ID is a
//      deterministic function of its constructor + arg IDs; two independent
//      derivations of the same term produce the same ID (congruence by
//      construction, no congruence rule needed).
//   2. Monotone `displaced(old, new)` tracks the union; `find` is its
//      transitive closure.
//   3. `*_view` relations derive by routing each raw tuple's ID columns
//      through `find`, giving canonical-ID views automatically. No
//      `(delete …)` actions; DD's diff propagation handles retractions.
//
// The test seeds raw terms and one displaced edge, runs to fixpoint, and
// asserts that $two and S(S(S($Zero))) share a `find` root.
// ---------------------------------------------------------------------------

ascent! {
   #![backend(dd)]
   pub struct ReproDefine;

   // Raw constructor tables. Monotonic — nothing retracts these directly;
   // displacement flows through `find` and the `*_view` derivations.
   relation zeroconst_raw(u64);        // ZeroConst()    → eid
   relation s_raw(u64, u64);          // S(arg_eid)     → result_eid

   // Monotone equivalence relation. `displaced(a, b)` means a is no longer
   // canonical; b is its direct parent. Invariant maintained by the caller:
   // `a > b` (u64 natural order) — keeps find acyclic and deterministic.
   relation displaced(u64, u64);

   // All known eids, for find's base case. A row is canonical iff it has no
   // parent in `displaced`.
   relation eid_exists(u64);
   eid_exists(x) <-- zeroconst_raw(x);
   eid_exists(x) <-- s_raw(_, x);
   eid_exists(x) <-- s_raw(x, _);
   eid_exists(x) <-- displaced(x, _);
   eid_exists(x) <-- displaced(_, x);

   relation has_parent(u64);
   has_parent(x) <-- displaced(x, _);

   // find: eid → canonical root. Reflexive on roots, transitive via displaced.
   relation find(u64, u64);
   find(x, x) <-- eid_exists(x), ! has_parent(x);
   find(x, r) <-- displaced(x, y), find(y, r);

   // Canonical views: each raw tuple with every ID column canonicalized.
   // Retractions from `find` diffs automatically retract stale view rows.
   relation zeroconst_view(u64);
   zeroconst_view(c) <-- zeroconst_raw(x), find(x, c);

   relation s_view(u64, u64);
   s_view(ac, xc) <-- s_raw(a, x), find(a, ac), find(x, xc);
}

#[ntest_timeout::timeout(5000)]
#[test]
fn repro_define_union_equates_terms() {
   // Constructor tags — arbitrary but stable.
   const ZEROCONST_TAG: u64 = 0;
   const S_TAG: u64 = 1;

   let zero = hash_term(ZEROCONST_TAG, &[]);
   let s_zero = hash_term(S_TAG, &[zero]);
   let ss_zero = hash_term(S_TAG, &[s_zero]);        // this is $two
   let sss_zero = hash_term(S_TAG, &[ss_zero]);

   let mut p = ReproDefine::default();
   p.zeroconst_raw = vec![(zero,)];
   // All three S-applications that appear in the program.
   p.s_raw = vec![(zero, s_zero), (s_zero, ss_zero), (ss_zero, sss_zero)];

   // (union $two (S (S (S $Zero)))) → one monotone displaced edge,
   // oriented larger → smaller to keep the find chain acyclic.
   let (lo, hi) = if ss_zero < sss_zero { (ss_zero, sss_zero) } else { (sss_zero, ss_zero) };
   p.displaced = vec![(hi, lo)];

   p.run();

   // (check (= $two (S (S (S $Zero))))) — the two terms must share a root.
   let find_of = |eid: u64| -> Option<u64> {
      p.find.iter().find(|(x, _)| *x == eid).map(|(_, r)| *r)
   };
   let root_two = find_of(ss_zero);
   let root_sss = find_of(sss_zero);
   assert!(root_two.is_some(), "find missing for $two (ss_zero={ss_zero:x})");
   assert!(root_sss.is_some(), "find missing for S(S(S($Zero))) (sss_zero={sss_zero:x})");
   assert_eq!(root_two, root_sss,
      "post-union (check (= $two (S (S (S $Zero))))) failed: find(ss_zero)={root_two:?}, find(sss_zero)={root_sss:?}");

   // Canonical view sanity: the canonical form of the two equated terms
   // should land at the same ID in s_view.
   let canon = root_two.unwrap();
   assert!(p.s_view.iter().any(|(_, r)| *r == canon),
      "s_view contains no row landing at the canonical root {canon:x}; got {:?}", p.s_view);
}

// ---------------------------------------------------------------------------
// Phase 4a: first rule-driven rewrite — commutativity of `Add`.
//
// Source: `ascent_tests_dd/tests_egglog_inputs/add_commutative.egg` (a copy
// of the canonical example from egglog's `src/proofs/proof_encoding.md`).
//
//   (sort Math)
//   (constructor Add (i64 i64) Math)
//   (Add 1 2)
//   (rule ((Add a b)) ((union (Add a b) (Add b a))) :name "commutativity")
//   (run 1)
//   (check (= (Add 1 2) (Add 2 1)))
//
// Compared to `ReproDefine` this test exercises TWO new mechanisms:
//   1. A rule fires on pattern-matching `add_view` and *mints new terms*
//      in its head (computes `hash_add(b, a)` for the swapped form).
//   2. The `displaced` relation is populated by the rule, not by the
//      caller — validating that rule heads can correctly inject UF state.
//
// Note: `Add` takes two `i64` *primitives* (not two `Math` e-classes), so
// only the constructor's result e-class column gets `find`'d in the view.
// Primitive arg columns stay as raw `i64` throughout.
// ---------------------------------------------------------------------------

fn hash_add(a: i64, b: i64) -> u64 {
   const ADD_TAG: u64 = 10;
   hash_term(ADD_TAG, &[a as u64, b as u64])
}

ascent! {
   #![backend(dd)]
   pub struct AddCommutative;

   // (constructor Add (i64 i64) Math) — two i64 primitives, one Math e-class.
   relation add_raw(i64, i64, u64);

   // Monotone UF, populated by the commutativity rule. Invariant maintained
   // by the rule: `first > second` (larger → smaller), so find always
   // converges to the minimum reachable eid.
   relation displaced(u64, u64);

   // All known eids — base case for find.
   relation eid_exists(u64);
   eid_exists(e) <-- add_raw(_, _, e);
   eid_exists(x) <-- displaced(x, _);
   eid_exists(x) <-- displaced(_, x);

   // find as a Dual<u64>-lattice: picks the MIN eid reachable from each x via
   // displaced* (reflexively). Dual<u64>::join is u64::min, so the lattice
   // automatically converges to the smallest reachable — that's the canonical.
   //
   // Why a lattice, not the `! has_parent` negation we used in ReproDefine:
   // here `displaced` is rule-derived, not user-input. `find` depends on
   // displaced; displaced depends on `add_view`; add_view depends on find.
   // The negation form (!has_parent) puts find in a cycle through negation,
   // which isn't stratifiable. The lattice form keeps the whole SCC
   // monotone and works inside a recursive fixpoint (see phase6_lattice's
   // shortest_path for the same trick on graph distance).
   lattice find(u64, Dual<u64>);
   find(x, Dual(*x)) <-- eid_exists(x);
   find(x, Dual(l.0)) <-- displaced(x, z), find(z, l);

   // Canonical view — unwrap the Dual for downstream rules.
   relation add_view(i64, i64, u64);
   add_view(a, b, l.0) <-- add_raw(a, b, eid), find(eid, l);

   // === "commutativity" rule ===
   // Body: (Add a b) matches against the canonical view.
   // Head: (union (Add a b) (Add b a)) — mint the swapped term, union it.
   // Split into two ascent rules (shared body) — ascent doesn't have block
   // heads.

   // Head action 1: materialize the swapped raw term.
   add_raw(*b, *a, swapped)
      <-- add_view(a, b, _),
          let swapped = hash_add(*b, *a);

   // Head action 2: emit the union edge. Oriented larger → smaller to
   // preserve the acyclicity invariant on `displaced`; skipped when a == b
   // (self-swap — no edge needed).
   displaced(hi, lo)
      <-- add_view(a, b, forward),
          let swapped = hash_add(*b, *a),
          if *forward != swapped,
          let hi = if *forward > swapped { *forward } else { swapped },
          let lo = if *forward > swapped { swapped } else { *forward };
}

#[ntest_timeout::timeout(5000)]
#[test]
fn add_commutative_rewrite() {
   let mut p = AddCommutative::default();
   let e12 = hash_add(1, 2);
   // (Add 1 2) at top level → seed the raw relation.
   p.add_raw = vec![(1, 2, e12)];
   p.run();

   let e21 = hash_add(2, 1);
   let find_of = |x: u64| -> Option<u64> {
      p.find.iter().find(|(e, _)| *e == x).map(|(_, r)| r.0)
   };
   let root_ab = find_of(e12);
   let root_ba = find_of(e21);
   assert!(root_ab.is_some(), "find missing for Add(1,2) (e12={e12:x})");
   assert!(root_ba.is_some(), "find missing for Add(2,1) (e21={e21:x}); rule likely didn't fire");
   assert_eq!(root_ab, root_ba,
      "(check (= (Add 1 2) (Add 2 1))) failed: find(e12)={root_ab:?}, find(e21)={root_ba:?}");

   // Rule-minted Add(2,1) must appear in add_raw (proves the rule's new-term
   // head action executed, not just the displaced head).
   assert!(p.add_raw.iter().any(|(a, b, _)| *a == 2 && *b == 1),
      "commutativity rule did not mint Add(2,1); add_raw = {:?}", p.add_raw);
}

// ---------------------------------------------------------------------------
// Phase 4b: nested body pattern — `(rewrite (A (A x)) x)` (A is an involution).
//
// Source: `ascent_tests_dd/tests_egglog_inputs/a_involution.egg`.
//
//   (sort Math)
//   (constructor Leaf () Math)
//   (constructor A (Math) Math)
//   (rewrite (A (A x)) x)
//   (A (A (Leaf)))
//   (run 1)
//   (check (= (A (A (Leaf))) (Leaf)))
//
// New mechanism exercised here: the rule body is `(A (A x))` — two instances
// of the same constructor nested, sharing an e-class variable at the
// boundary. After term-encoding this becomes a self-join on the view
// relation with a shared variable on the arg-eid column. All arg columns
// are e-class IDs (not primitives), so every body/head atom routes through
// `find` on every e-class position.
// ---------------------------------------------------------------------------

fn hash_leaf() -> u64 {
   const LEAF_TAG: u64 = 20;
   hash_term(LEAF_TAG, &[])
}
fn hash_a(arg: u64) -> u64 {
   const A_TAG: u64 = 21;
   hash_term(A_TAG, &[arg])
}

ascent! {
   #![backend(dd)]
   pub struct AInvolution;

   // (constructor Leaf () Math) — nullary.
   relation leaf_raw(u64);                 // just the result eid

   // (constructor A (Math) Math) — unary over Math.
   relation a_raw(u64, u64);               // (arg_eid, result_eid)

   // Monotone UF.
   relation displaced(u64, u64);

   // All known eids.
   relation eid_exists(u64);
   eid_exists(e) <-- leaf_raw(e);
   eid_exists(arg) <-- a_raw(arg, _);
   eid_exists(e) <-- a_raw(_, e);
   eid_exists(x) <-- displaced(x, _);
   eid_exists(x) <-- displaced(_, x);

   // find lattice: min reachable eid.
   lattice find(u64, Dual<u64>);
   find(x, Dual(*x)) <-- eid_exists(x);
   find(x, Dual(l.0)) <-- displaced(x, z), find(z, l);

   // Canonical views.
   relation leaf_view(u64);
   leaf_view(l.0) <-- leaf_raw(e), find(e, l);

   relation a_view(u64, u64);
   a_view(la.0, lr.0) <-- a_raw(arg, e), find(arg, la), find(e, lr);

   // === rewrite (A (A x)) x ===
   // Body matches `(A (A x))` via a self-join on `a_raw` (not `a_view`):
   //   inner row: a_raw(x_eid, mid)     <- inner A: A(x) = mid
   //   outer row: a_raw(mid,   outer)   <- outer A: A(mid) = outer
   // Head: union(outer, x_eid).
   //
   // IMPORTANT — why raw, not view:
   // The rule participates in the recursive SCC
   //   a_raw/a_view → find → displaced → find → a_view → rule → displaced.
   // If the body matches `a_view` (canonical), firing the rule adds a
   // displaced edge that changes `find`, which changes `a_view`, which
   // retracts the body match, which retracts `displaced`. DD oscillates
   // forever — a self-destructive rule. Matching the raw relation keeps
   // the body premise monotone (raw only grows), so emitted `displaced`
   // tuples are stable.
   //
   // Cost: we don't "match modulo equivalence" — if `mid` equals some
   // other eid via union, the structural join won't find it. That's what
   // an explicit congruence rule would add, but this test's program
   // doesn't require it. When we port programs that need congruence,
   // we'll add a separate congruence rule per constructor.
   displaced(hi, lo)
      <-- a_raw(x_eid, mid),
          a_raw(mid, outer),
          if *x_eid != *outer,
          let hi = if *outer > *x_eid { *outer } else { *x_eid },
          let lo = if *outer > *x_eid { *x_eid } else { *outer };
}

#[ntest_timeout::timeout(5000)]
#[test]
fn a_involution_nested_pattern() {
   let leaf = hash_leaf();
   let a_leaf = hash_a(leaf);
   let aa_leaf = hash_a(a_leaf);

   let mut p = AInvolution::default();
   p.leaf_raw = vec![(leaf,)];
   p.a_raw = vec![(leaf, a_leaf), (a_leaf, aa_leaf)];
   p.run();

   let find_of = |x: u64| -> Option<u64> {
      p.find.iter().find(|(e, _)| *e == x).map(|(_, r)| r.0)
   };
   let root_leaf = find_of(leaf);
   let root_aa = find_of(aa_leaf);
   assert!(root_leaf.is_some(), "find missing for Leaf (leaf={leaf:x})");
   assert!(root_aa.is_some(), "find missing for A(A(Leaf)) (aa_leaf={aa_leaf:x})");
   assert_eq!(root_leaf, root_aa,
      "(check (= (A (A (Leaf))) (Leaf))) failed: find(leaf)={root_leaf:?}, find(aa_leaf)={root_aa:?}");

   // A(Leaf) should NOT be unioned with either Leaf or A(A(Leaf)) by this
   // single-rule program — it's an independent e-class. Guard against a
   // regression where an over-eager union collapses everything.
   let root_a = find_of(a_leaf);
   assert_ne!(root_a, root_leaf,
      "A(Leaf) was incorrectly unioned with Leaf; find(a_leaf)={root_a:?}, find(leaf)={root_leaf:?}");
}

// ---------------------------------------------------------------------------
// Phase 4c: demonstrating when a congruence rule is required.
//
// Source: `ascent_tests_dd/tests_egglog_inputs/congruence.egg`.
//
//   (datatype Math (A) (B) (F Math))
//   (F (A))
//   (F (B))
//   (union (A) (B))
//   (run 1)
//   (check (= (F (A)) (F (B))))
//
// In egglog, once A ≡ B, congruence implies F(A) ≡ F(B) — the e-graph's
// rebuild pass handles this automatically. There's no user-written rule
// that does it. In our DD encoding we have to add that closure explicitly.
//
// Two programs below illustrate the gap:
//   - `CongruenceNaive` — no congruence rule. Test asserts that WITHOUT
//     it, F(A) and F(B) have distinct find roots (i.e., the port fails
//     to match egglog's check).
//   - `CongruenceFull`  — adds a congruence rule for F. Test asserts
//     the expected `F(A) ≡ F(B)` holds.
//
// The congruence rule is the template for all single-arg constructors:
// raw-match on two F rows, join through `find` on the shared arg
// canonical, emit a `displaced` edge between the two results. Body
// premises are all monotone (raw + find), so the emission is stable.
// ---------------------------------------------------------------------------

fn hash_aconst() -> u64 {
   const A_TAG: u64 = 30;
   hash_term(A_TAG, &[])
}
fn hash_bconst() -> u64 {
   const B_TAG: u64 = 31;
   hash_term(B_TAG, &[])
}
fn hash_f(arg: u64) -> u64 {
   const F_TAG: u64 = 32;
   hash_term(F_TAG, &[arg])
}

// --- Naive port: NO congruence rule. Demonstrates the gap. ---
ascent! {
   #![backend(dd)]
   pub struct CongruenceNaive;

   relation a_const_raw(u64);
   relation b_const_raw(u64);
   relation f_raw(u64, u64);      // (arg_eid, result_eid)
   relation displaced(u64, u64);

   relation eid_exists(u64);
   eid_exists(e) <-- a_const_raw(e);
   eid_exists(e) <-- b_const_raw(e);
   eid_exists(arg) <-- f_raw(arg, _);
   eid_exists(e) <-- f_raw(_, e);
   eid_exists(x) <-- displaced(x, _);
   eid_exists(x) <-- displaced(_, x);

   lattice find(u64, Dual<u64>);
   find(x, Dual(*x)) <-- eid_exists(x);
   find(x, Dual(l.0)) <-- displaced(x, z), find(z, l);

   relation f_view(u64, u64);
   f_view(la.0, lr.0) <-- f_raw(arg, e), find(arg, la), find(e, lr);

   // NO rules here. displaced is only what the user seeds. Congruence
   // would require a rule we haven't written.
}

#[ntest_timeout::timeout(5000)]
#[test]
fn congruence_naive_fails() {
   let a = hash_aconst();
   let b = hash_bconst();
   let fa = hash_f(a);
   let fb = hash_f(b);

   let mut p = CongruenceNaive::default();
   p.a_const_raw = vec![(a,)];
   p.b_const_raw = vec![(b,)];
   p.f_raw = vec![(a, fa), (b, fb)];
   // The user's (union (A) (B)) — seed directly.
   let (lo_ab, hi_ab) = if a < b { (a, b) } else { (b, a) };
   p.displaced = vec![(hi_ab, lo_ab)];
   p.run();

   let find_of = |x: u64| -> Option<u64> {
      p.find.iter().find(|(e, _)| *e == x).map(|(_, r)| r.0)
   };
   let root_a = find_of(a).unwrap();
   let root_b = find_of(b).unwrap();
   assert_eq!(root_a, root_b,
      "direct union failed to equate A and B; find(a)={root_a:x}, find(b)={root_b:x}");

   // But F(A) and F(B) are NOT equated — congruence rule missing.
   // This demonstrates the gap: egglog would pass the check here, we don't.
   let root_fa = find_of(fa).unwrap();
   let root_fb = find_of(fb).unwrap();
   assert_ne!(root_fa, root_fb,
      "unexpected: F(A) and F(B) are equivalent WITHOUT a congruence rule — \
       did the encoding accidentally cover it? find(fa)={root_fa:x}, find(fb)={root_fb:x}");
}

// --- Full port: adds an explicit congruence rule for F. ---
ascent! {
   #![backend(dd)]
   pub struct CongruenceFull;

   relation a_const_raw(u64);
   relation b_const_raw(u64);
   relation f_raw(u64, u64);
   relation displaced(u64, u64);

   relation eid_exists(u64);
   eid_exists(e) <-- a_const_raw(e);
   eid_exists(e) <-- b_const_raw(e);
   eid_exists(arg) <-- f_raw(arg, _);
   eid_exists(e) <-- f_raw(_, e);
   eid_exists(x) <-- displaced(x, _);
   eid_exists(x) <-- displaced(_, x);

   lattice find(u64, Dual<u64>);
   find(x, Dual(*x)) <-- eid_exists(x);
   find(x, Dual(l.0)) <-- displaced(x, z), find(z, l);

   relation f_view(u64, u64);
   f_view(la.0, lr.0) <-- f_raw(arg, e), find(arg, la), find(e, lr);

   // === Congruence rule for F ===
   // If two raw F rows have args with the same canonical, their
   // results must be in the same e-class. Body is raw-match + find-join
   // on the shared canon (which is a lattice-valued shared var — joins
   // on the Dual<u64> value). Head values (fa, fb) are from raw (stable),
   // so the emission is stable under SCC iteration.
   displaced(hi, lo)
      <-- f_raw(arg_a, result_a),
          f_raw(arg_b, result_b),
          find(arg_a, canon),      // shared canon → join modulo equiv
          find(arg_b, canon),
          if *result_a != *result_b,
          let hi = if *result_a > *result_b { *result_a } else { *result_b },
          let lo = if *result_a > *result_b { *result_b } else { *result_a };
}

// ---------------------------------------------------------------------------
// Phase 4d: stitching the two poles — demonstrating that DD's diff
// propagation IS the negative program. Uses session mode so we can commit in
// stages and inspect `s_view` deltas directly: +1 for insertions, -1 for
// retractions.
//
// Shape:
//   Commit 1 — seed ZeroConst, S(Zero), S(S(Zero)), S(S(S(Zero))). No
//              displaced. View reflects the raw chain verbatim.
//   Commit 2 — user asserts union of S(S(Zero)) with S(S(S(Zero))) by
//              inserting a displaced edge. No new raw rows.
//
// What we assert after commit 2:
//   * `s_view_deltas()` contains BOTH a `-1` diff (the stale row pointing at
//     the now-non-canonical id) AND a `+1` diff (the same raw row
//     re-projected through the updated find).
//   * `s_view_snapshot()` has the stale row gone and the canonical row
//     present.
//
// That pair of diffs is the two-pole stitch: the "positive" program adds
// the canonical row; the "negative" program retracts the stale one; both
// fall out of DD's join arithmetic on the `s_raw + find` derivation when
// find's value for one of the arg columns changes. No delete actions, no
// staleness predicates, no extra rule wiring — just the view's defining
// join.
// ---------------------------------------------------------------------------

#[ntest_timeout::timeout(5000)]
#[test]
fn view_deltas_stitch_positive_and_negative_poles() {
   const ZEROCONST_TAG: u64 = 0;
   const S_TAG: u64 = 1;
   let zero = hash_term(ZEROCONST_TAG, &[]);
   let s_zero = hash_term(S_TAG, &[zero]);
   let ss_zero = hash_term(S_TAG, &[s_zero]);
   let sss_zero = hash_term(S_TAG, &[ss_zero]);

   let mut sess = ReproDefine::session();

   // --- Commit 1: seed four terms. No union yet. ---
   sess.zeroconst_raw_insert((zero,));
   sess.s_raw_insert((zero, s_zero));
   sess.s_raw_insert((s_zero, ss_zero));
   sess.s_raw_insert((ss_zero, sss_zero));
   sess.commit();

   let mut view_before: Vec<_> = sess.s_view_snapshot();
   view_before.sort();
   let mut expected_before = vec![(zero, s_zero), (s_zero, ss_zero), (ss_zero, sss_zero)];
   expected_before.sort();
   assert_eq!(view_before, expected_before,
      "before union, s_view should reflect raw chain verbatim");

   // --- Commit 2: assert the union (ss_zero ≡ sss_zero). ---
   // Preserve the `a > b` invariant on displaced so find stays acyclic.
   let (lo, hi) = if ss_zero < sss_zero { (ss_zero, sss_zero) } else { (sss_zero, ss_zero) };
   sess.displaced_insert((hi, lo));
   sess.commit();

   // Inspect deltas on s_view from commit 2.
   let deltas = sess.s_view_deltas();
   let retracted: ::std::collections::HashSet<(u64, u64)> =
      deltas.iter().filter(|(_, d)| *d < 0).map(|(t, _)| *t).collect();
   let inserted: ::std::collections::HashSet<(u64, u64)> =
      deltas.iter().filter(|(_, d)| *d > 0).map(|(t, _)| *t).collect();

   // The raw row s_raw(ss_zero, sss_zero) has BOTH columns in the displaced
   // pair — one is hi (displaced to lo), the other is lo (canonical).
   // Old canonical projection: (ss_zero, sss_zero). New canonical projection:
   // (find(ss_zero), find(sss_zero)) = (lo, lo) regardless of orientation,
   // because whichever is hi resolves to lo.
   //
   // That's the stitch in action: one -1 on the stale, one +1 on the canonical.
   assert!(retracted.contains(&(ss_zero, sss_zero)),
      "expected -1 on stale view row (ss_zero={ss_zero:x}, sss_zero={sss_zero:x}); got retracted={retracted:?}");
   assert!(inserted.contains(&(lo, lo)),
      "expected +1 on canonical view row (lo, lo)=({lo:x},{lo:x}); got inserted={inserted:?}");

   // The (s_zero, ss_zero) row also changes IF ss_zero = hi (i.e., it's the
   // one being displaced). Assert the orientation-dependent case too.
   if ss_zero == hi {
      assert!(retracted.contains(&(s_zero, ss_zero)),
         "expected -1 on (s_zero, ss_zero) when ss_zero is displaced; retracted={retracted:?}");
      assert!(inserted.contains(&(s_zero, lo)),
         "expected +1 on (s_zero, lo) when ss_zero is displaced; inserted={inserted:?}");
   }

   // Snapshot confirms: stale gone, canonical present, unaffected rows untouched.
   let view_after: ::std::collections::HashSet<(u64, u64)> =
      sess.s_view_snapshot().into_iter().collect();
   assert!(!view_after.contains(&(ss_zero, sss_zero)),
      "stale view row (ss_zero, sss_zero) still present after union");
   assert!(view_after.contains(&(lo, lo)),
      "canonical view row (lo, lo) missing after union");
   // The zero→s_zero row doesn't touch the displaced pair, always stable.
   assert!(view_after.contains(&(zero, s_zero)),
      "untouched view row (zero, s_zero) missing");
}

#[ntest_timeout::timeout(5000)]
#[test]
fn congruence_with_rule_works() {
   let a = hash_aconst();
   let b = hash_bconst();
   let fa = hash_f(a);
   let fb = hash_f(b);

   let mut p = CongruenceFull::default();
   p.a_const_raw = vec![(a,)];
   p.b_const_raw = vec![(b,)];
   p.f_raw = vec![(a, fa), (b, fb)];
   let (lo_ab, hi_ab) = if a < b { (a, b) } else { (b, a) };
   p.displaced = vec![(hi_ab, lo_ab)];
   p.run();

   let find_of = |x: u64| -> Option<u64> {
      p.find.iter().find(|(e, _)| *e == x).map(|(_, r)| r.0)
   };
   let root_a = find_of(a).unwrap();
   let root_b = find_of(b).unwrap();
   assert_eq!(root_a, root_b, "direct union failed for A, B");

   // Now F(A) and F(B) must be equated by the congruence rule.
   let root_fa = find_of(fa).unwrap();
   let root_fb = find_of(fb).unwrap();
   assert_eq!(root_fa, root_fb,
      "(check (= (F (A)) (F (B)))) failed: find(fa)={root_fa:x}, find(fb)={root_fb:x}");
}
