# DD Backend — Progress Tracker

Opt-in differential-dataflow codegen backend for `ascent!` / `ascent_run!`.
Enabled with an inner attribute:

```rust
ascent! {
   #![backend(dd)]
   relation edge(i32, i32);
   // ...
}
```

The default remains `#![backend(batch)]` (current codegen). The DD
backend dispatches at the end of `ascent_impl` in
`ascent_macro/src/lib.rs`.

## Verification approach

The forked test crate `ascent_tests_dd/` mirrors tests from
`ascent_tests/`. Tests that the backend does not yet support are
annotated `#[ignore = "dd: phase <N> — <reason>"]` rather than deleted
or rewritten. `cargo test -p ascent_tests_dd` therefore prints a
meaningful pass / ignored count that tracks phase progress.

Rule: **never locally rewrite a test to avoid a backend limitation.**
Either fix the backend or document the exclusion.

## Current status

| Phase | Description | Green | Ignored |
|------:|-------------|------:|--------:|
|     0 | Attribute dispatch + stub `run()` | 3 | 2 |
|     1 | Non-recursive rules via `worker.dataflow` | 0 | 1 |
|     2 | Recursive rules via `VecVariable` | 0 | 1 |
|     3 | Conditions, generators, patterns | 0 | 0 |
|     4 | Negation | 0 | 0 |
|     5 | Aggregation | 0 | 0 |
|     6 | Lattices | 0 | 0 |
|     7 | Generics | 0 | 0 |
|     8 | Init / timeout / misc | 0 | 0 |
|     9 | Parallel workers | 0 | 0 |

## Layout

- `ascent_macro/src/ascent_codegen_dd.rs` — DD backend entry
  (`compile_mir_dd`).
- `ascent_macro/src/ascent_hir.rs` — `Backend` enum, `#![backend(…)]`
  attribute parsing (`AscentConfig::BACKEND_ATTR`).
- `ascent_tests_dd/` — forked test suite (kept out of the workspace,
  mirroring `ascent_tests/`).

## Known permanent exclusions

These will stay ignored even when the backend is feature-complete,
because they clash with DD's own invariants. See the design discussion
in the Phase 0 planning conversation.

| Upstream test / feature | Why |
|---|---|
| `test_borrowed_strings*` | DD requires `'static` data in collections. |
| `test_ds_attr` (custom relation DS) | DD picks its own arrangement shape. |
| Rule bodies that capture `&self.*` or share `RefCell` across rules | DD closures must own + be thread-safe. |
