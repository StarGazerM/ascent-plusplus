# Egglog on ascent-dd — implementation plan

## Goal

Implement an egglog-compatible equality-saturation surface on top of the
ascent-dd backend, proving differential dataflow is a viable execution
engine for egglog programs without adding new DD runtime primitives. The
acceptance target is observational equivalence with upstream egglog on a
representative slice of its test suite — start with pure-Datalog
programs, work up through full rewrite-and-union semantics.

The upstream egglog sources live at [others/egglog/](others/egglog) and
are consulted as the semantic reference (see
`memory/reference_egglog.md`).

## Scope anchors

**In scope.** Rule bodies and heads (match / apply / union / delete),
`datatype` / `sort` / `constructor`, `function :merge` (when the merge
is commutative-associative-idempotent), `let` / globals,
`rewrite` / `birewrite` / `rule`, `run` / `run-schedule`, `check`.
Observational equivalence on `check` statements is the acceptance
criterion for every phase.

**Out of scope for now.** Proof tracking; cost-based extraction
(`:cost`, `extract`); scheduler tuning beyond naive fixpoint; egglog
container primitives (`Map`, `Set`, `Vec`, `MultiSet`) beyond what maps
trivially to Rust stdlib; parser / surface syntax (we reuse egglog's
own CLI); egglog-experimental plugins.

**Out of scope permanently.** Anything that requires mutable
in-iteration state DD can't express. None identified yet; document as
encountered.

## Design decisions (locked)

These are the architectural commitments the design conversation
converged on. Any future deviation is a deliberate re-decision, not
default drift.

1. **Input format = egglog's `--term-encoding --mode desugar` output.**
   The term-encoding pass (documented at
   [others/egglog/src/proofs/proof_encoding.md](others/egglog/src/proofs/proof_encoding.md))
   removes all `(union …)` from the program and emits pure rule-based
   egglog using only `(delete …)`, relation insertion, and `(set …)`.
   We take THAT as the lowering input. No parser work, no surface-syntax
   design. See `memory/project_egglog_term_encoding.md`.

2. **Retraction via DD diff propagation through `find`, not via
   staleness predicates.** No `(delete …)` as a head action in the
   generated ascent program. Visible relations are derived as
   `*_view(canon_ids…) <-- *_raw(raw_ids…), find(id, canon_id) …`.
   When `displaced` grows, `find` emits `-1`/`+1` diffs, and DD's
   join arithmetic propagates them through every `*_view` automatically
   — that IS the negative stratum. The earlier "staleness predicate"
   formulation (`visible = raw ∧ ¬stale`) turned out to be a
   reimplementation of what DD already gives for free; validated in
   Phase 2 where the Phase 1 sketch's explicit `stale_*` rules dropped
   out without changing semantics. Keep staleness predicates in reserve
   *only* for patterns DD's join diffs can't cover (none observed yet;
   document if encountered).

3. **IDs are content-addressed — no minting, no DD primitive
   changes.** An e-class ID is `hash(constructor_tag, raw_arg_ids)`
   via a 64-bit mixer (see `hash_term` in
   `ascent_tests_dd/src/egglog_port.rs`). u64 birthday-bound
   collision probability is < 10⁻⁵ at 10⁷ terms — well beyond
   realistic eq-sat workloads. No interning table; no stratified
   ID allocator. This is *after-hash* (hash raw arg IDs, not
   `find()`-canonicalized args) — giving stable, immutable IDs.
   Congruence falls out of representation: identical structure ⇒
   identical tuple ⇒ DD set semantics collapse. If we ever hit a
   workload where u64 collisions bite, upgrade the mixer to a
   128-bit one (SipHasher halves, or BLAKE3 truncated) without
   touching call sites.

4. **`displaced(old_eid, new_eid)` is the only UF state, and it is
   monotone.** Egglog never un-unions, so `displaced` only grows — DD's
   happy place. `find(eid, Dual(root))` is a recursive
   `Dual<u64>`-lattice (u64::min under the Dual reversal) that
   converges to the smallest reachable eid per equivalence class. The
   lattice form keeps the entire `displaced ↔ find ↔ view ↔ rule`
   SCC monotone and works whether `displaced` is user-input or
   rule-derived (the naïve `find(x, x) <-- !has_parent(x)` form only
   stratifies when displaced is user-input). All canonicalization is
   a derived view unwrapping `find(_, Dual(r))` on each e-class column.

5. **Default: one relation per constructor (non-flattened).** Pattern
   flattening is an optimization, not the primary encoding. We flatten
   a specific nested pattern only when (a) intermediate terms are
   provably never unioned / cross-referenced / extracted, or (b)
   measured query-time win justifies the extra relation. Phase 1–4
   ports use non-flattened.

6. **DD fuses egglog's match/apply/rebuild rounds into one fixpoint.**
   Egglog's `(run-schedule (saturate rebuilding))` injection between
   rule rounds collapses — DD propagates diffs through `displaced`
   continuously, keeping everything canonical. This is a semantic
   divergence worth tracking for programs that depend on observable
   rebuild-round boundaries (none observed yet).

## Architecture — lowering pipeline

```
┌───────────────┐  egglog --term-encoding --mode desugar  ┌──────────────────┐
│ user.egg      │ ───────────────────────────────────────▶│ user.te.egg      │
│ (egglog src)  │                                         │ (union-free;     │
│               │                                         │  only delete/    │
│               │                                         │  insert/set)     │
└───────────────┘                                         └────────┬─────────┘
                                                                   │
                                                  manual port      │
                                                  (Phase 1–4) or   │
                                                  codegen (Ph. 5)  ▼
                                                           ┌─────────────────────┐
                                                           │ user.rs             │
                                                           │ ascent! {           │
                                                           │   #![backend(dd)]   │
                                                           │   …                 │
                                                           │ }                   │
                                                           └────────┬────────────┘
                                                                    │ ascent_codegen_dd
                                                                    ▼
                                                           ┌─────────────────────┐
                                                           │ DD dataflow         │
                                                           │ (ascent_dd_runtime) │
                                                           └─────────────────────┘
```

The key property: each arrow is mechanical. Correctness follows from
(a) the term-encoding being union-free, (b) the dual-direction
stratification being semantically equivalent to delete+insert, (c) the
content-addressed IDs giving automatic congruence.

## The lowering recipe, spelled out

For each sort `S` in the term-encoded input, emit:

```rust
// Monotone equivalence relation, with invariant first > second (larger →
// smaller). Populated by user rule heads whose body ends with `(union a b)`.
relation displaced_s(u64, u64);

// All known eids (base case for the find lattice).
relation eid_exists_s(u64);
eid_exists_s(e) <-- /* every column of a sort-S _raw relation */;
eid_exists_s(x) <-- displaced_s(x, _);
eid_exists_s(x) <-- displaced_s(_, x);

// find is a Dual<u64>-lattice that picks the MIN eid reachable via
// displaced* (reflexive). Dual<u64>::join is u64::min, so the lattice
// naturally converges to the canonical representative per equivalence class.
//
// Why a lattice and not `find(x, x) <-- !has_parent(x)`: when displaced is
// user-input (Phase 2 style), the negation form stratifies cleanly. But as
// soon as a user rule derives displaced (Phase 4a and beyond), find →
// !has_parent → displaced → view → rule → find creates a cycle through
// negation, which ascent can't stratify. The Dual-lattice form keeps the
// entire SCC monotone; it works whether displaced is rule-derived or not.
// (Phase 4a validated this; see phase6_lattice::ShortestPath for the same
// pattern on graph distance.)
lattice find_s(u64, Dual<u64>);
find_s(x, Dual(*x)) <-- eid_exists_s(x);
find_s(x, Dual(l.0)) <-- displaced_s(x, z), find_s(z, l);
```

For each constructor `F(A, B) : S`, emit:

```rust
// Raw: stores (a_eid, b_eid, f_eid) where f_eid = hash(F_TAG, a_eid, b_eid).
// Grows monotonically; never retract directly.
relation f_raw(u64, u64, u64);

// Visible / canonical view — unwrap the Dual on each find, project every
// e-class column. Primitive arg columns (i64, String, etc.) pass through
// unchanged. DD's diff propagation handles retractions for free: when
// displaced grows, find emits new Dual values, and the join passes those
// as retractions + insertions of view rows.
relation f_view(u64, u64, u64);
f_view(la.0, lb.0, le.0) <--
    f_raw(a, b, e),
    find_a(a, la), find_b(b, lb), find_s(e, le);
```

**For user rules that emit `displaced` (i.e., any rewrite), match body
atoms against `*_raw` — NOT `*_view`.** Matching on view creates a
self-destructive SCC: the rule's body depends on `find`; firing the
rule adds a `displaced` edge; the edge changes `find`; `find`'s change
retracts the body match; the body's retraction retracts the
`displaced` edge; loop. DD oscillates forever.

Raw-match keeps the body premise monotone (`*_raw` only grows). The
`displaced` edges it emits stay stable. Correctness caveat: raw-match
doesn't fire "modulo equivalence" — if `f_raw(a, _)` and `f_raw(b, _)`
exist with `a ≡ b` via an unrelated union, a structural join on `f_raw`
only fires for each literal row, not for cross-equivalence
combinations. That gap is filled by explicit congruence rules (one
per constructor) that derive `displaced(res_a, res_b)` when
`f_raw(a, res_a)` and `f_raw(b, res_b)` share a canonical arg.

Congruence rule template (single-arg constructor F, validated in Phase
4c `CongruenceFull`):

```rust
displaced(hi, lo)
   <-- f_raw(arg_a, result_a),
       f_raw(arg_b, result_b),
       find(arg_a, canon),          // shared Dual<u64> var = join on
       find(arg_b, canon),          //   same canonical
       if *result_a != *result_b,
       let hi = if *result_a > *result_b { *result_a } else { *result_b },
       let lo = if *result_a > *result_b { *result_b } else { *result_a };
```

For N-arg constructors, extend with one `find(arg_k_a, canon_k)` +
`find(arg_k_b, canon_k)` pair per argument position. Body stability:
`f_raw` monotone, `find` monotone-reducing (Dual lattice). Head
stability: `result_a`, `result_b` from raw (immutable once computed).

For user rules that only **add** new raw rows (no `displaced`), the
body can match on either raw or view — both work. Use view when you
want "match modulo equivalence" for free; use raw when you want a
strictly structural match. AddCommutative matches on view and is
stable because its head tuple values come from raw primitive-arg
columns (i64), which don't flow through find.

For each user rule (term-encoded form), translate body atoms against
the appropriate relation per the above guidance, translate insertion
heads to `_raw` insertions with `hash(…)` computed IDs, translate
`(delete X)` heads into `displaced_*` insertions (for union-triggered
deletes) or explicit
`stale_*` predicates (for user `(delete …)` commands).

## Phases

### Phase 0 — toolchain & cross-check harness

Deliverables:
- Shell script `scripts/egglog_te.sh <file>` producing `<file>.te.egg`
  and capturing egglog's `(check …)` outcomes as a reference.
- Rust test harness under `ascent_tests_dd/src/egglog_port/` that runs
  egglog and the ascent port against the same input and asserts
  observational equivalence on `(check …)` facts.

Done when: the script produces a reference snapshot for
`tests/web-demo/points-to.egg`; the harness diffs two fact-sets and
reports mismatches.

### Phase 1 — pure-Datalog port (no unions)

Scope: programs with only `relation` + `rule` + insert-only heads. No
`(union …)`, no `rewrite`, no `(function … :merge …)`. No e-class IDs
needed; drop the `_raw`/`_view`/`stale_*` triple and use flat
relations.

Reference: `points-to.egg` — translation sketched in conversation.

Deliverables:
- `ascent_tests_dd/src/egglog_port/points_to.rs` — hand translation.
- Test asserting all 5 `(check …)` statements from `points-to.egg`
  hold.

Done when: test passes. Baseline artifact.

### Phase 2 — smallest rewrite example (ID + displaced + view) — **landed**

Ported `others/egglog/tests/repro-define.egg` as
`egglog_port::repro_define_union_equates_terms` in
[ascent_tests_dd/src/egglog_port.rs](ascent_tests_dd/src/egglog_port.rs).
Cross-check: `egglog repro-define.egg` exits 0; our port has
`find(ss_zero) == find(sss_zero)` post-union.

Two landings worth remembering:
1. The `stale_*` predicates from the original plan sketch were not
   needed — `*_view` derived via `*_raw + find(…)` gives canonical
   views with retractions flowing through DD's join diffs for free.
   See design decision #2.
2. Picked u64 IDs over u128 (better memory / cache behavior;
   collision probability negligible for realistic workloads). See
   design decision #3.

`hash_term(tag: u64, args: &[u64]) -> u64` lives inline in
`egglog_port.rs` for now; factor out to a utility module once a
third port reuses it.

Done when: observational equivalence with egglog on `(check …)`.
This proves the core recipe works on a non-trivial program.

### Phase 3 — UF-under-DD benchmarking & decision memo

Scope: measure whether DD's incremental `find` keeps up with egglog's
explicit rebuild on a union-heavy workload.

Rationale: egglog switches between incremental and full rebuild via a
heuristic
([core-relations/src/table/rebuild.rs:272-278](others/egglog/core-relations/src/table/rebuild.rs#L272-L278)).
DD is always incremental and may have arrangement-maintenance
overhead. Decision data needed before scaling the port.

Deliverables:
- Bench in `benches/egglog_uf.rs` running a union-heavy program under
  (a) egglog directly, (b) the Phase 2 port. Wall-clock, allocations,
  DD arrangement memory.
- Decision memo saved as `memory/project_egglog_dd_uf_perf.md`:
  - Within 3× egglog on representative union-heavy workloads → continue.
  - 10×+ → redesign `find` (alternatives: eager canonicalization,
    batched UF closure, flattened pattern relations as primary
    encoding).

Done when: benchmark runs under `cargo bench`; decision memo saved.

### Phase 4 — scale up the corpus

Scope: port 5–10 additional egglog tests of increasing complexity.
Surface blockers before committing to codegen.

Candidates (rough order):
- `calc.egg` — rewrite-heavy group theory (explored in conversation).
- `bitwise.egg` — uses i64 primitives.
- `complex-merge-func.egg` — tests `:merge` semantics; maps onto
  ascent's `#[lattice]` feature where merge is CAI.
- `fibonacci-demand.egg` — demand-driven computation.
- `container-rebuild.egg` — container-type canonicalization (may be a
  Phase 5+ problem).

Deliverables:
- One ported test per candidate in `ascent_tests_dd/src/egglog_port/`.
- Running blocker list in the "Known gaps" section below
  (append-only).

Done when: either all targeted tests pass, or each failure has a
documented root cause and a decision (defer / redesign /
out-of-scope).

### Phase 5 — mechanical codegen (gated on Phases 1–4 success)

Scope: write a lowering from term-encoded egglog → ascent source,
replacing manual ports. Can live as a separate binary
(`egglog-to-ascent`) consuming `egglog --term-encoding --mode desugar`
output.

Deliverables (if this phase activates):
- Parser for the subset of term-encoded egglog we actually use.
- Lowering passes: (a) per-sort `displaced`/`find`, (b) per-constructor
  raw+view+stale triple, (c) per-rule body/head translation.
- Regenerate all Phase 1–4 hand ports from codegen; round-trip
  equivalence test.

Done when: all hand-ported tests regenerate-and-pass from the codegen
path.

## Known gaps / open questions

Append-only. Each entry is either resolved (struck through with a
pointer to the fix) or still open.

- **Scheduler semantics.** Egglog's `(run-schedule (saturate R1 R2))`
  fuses into one DD fixpoint under our design. Are there egglog
  programs that depend on `(run N)` bounded iterations to produce
  observably-different output? If so, we need a bounded-iteration
  mode. Revisit during Phase 4.
- **`:merge` semantics — reclassified as translation recipe, not a
  gap.** Empirical scan of all 48 `:merge` uses in
  [others/egglog/tests/](others/egglog/tests/) shows every
  non-side-effecting merge in practice is expressible as a lattice
  operation, once we allow content-addressed tie-breakers:
  - CAI value merges (`max`, `min`, `set-union`, `or`, etc.) — 30/48
    cases — map to `#[lattice]` directly.
  - `:merge old` — 4 uses, all write-once semantics in practice —
    lower to "min by content-addressed insertion tag." Content IDs
    are already u64 hashes (design decision #3), so the tie-breaker
    is free.
  - `:merge new` — 7 uses, all "rules produce the same value; pick
    any" in practice — lower as "max by insertion tag." Gives a
    deterministic but user-invariant-dependent answer.
  - Abelian-group merges (`+`, `*`) — 1-2 uses — either reduce with
    the group op (simple) or custom DD `Diff` type (performant).
  Side-effecting merges (actions inside `:merge`) — not observed in
  the core test corpus; desugar into an ordinary rule before the
  port if ever encountered.
- **Primitive library.** `bigint`, `bigrat`, `f64`, `Vec`, `Map`,
  `Set`, `MultiSet`. Most Rust-native (i64, String); some need
  DD-side support. Triage per-candidate.
- **Hash-collision assertion.** In debug builds, add an invariant
  check that flags two structurally-distinct `_raw` rows sharing an
  ID. Upgrade to u256 or interned IDs if we hit real collisions.
- **Cycle avoidance in `displaced`.** Egglog's
  `ordering-max`/`ordering-min` ensures the UF is acyclic. Our
  recursive `find` would loop if a cycle were introduced. Invariant
  to enforce: `displaced(a, b) ⟹ a < b` in the canonical u64
  ordering. Codegen-level check when we emit a `displaced` insertion.

## Directory layout

```
ascent_tests_dd/src/egglog_port/
├── lib.rs            # hash_term util, displaced/find macros
├── points_to.rs      # Phase 1
├── min_rewrite.rs    # Phase 2
└── …                 # Phase 4 tests

scripts/
└── egglog_te.sh      # emit term-encoded form

benches/
└── egglog_uf.rs      # Phase 3

others/egglog/        # reference clone (already present)

EGGLOG_PLAN.md        # this file
```

## Non-goals for this document

- Schedule estimates. No dates. Phases complete when their
  deliverables land.
- Upstream contribution plans (ascent or egglog). Defer until after
  Phase 4.
- Paper / publication plans. Same.
