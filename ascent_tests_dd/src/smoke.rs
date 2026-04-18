//! Phase 0 smoke tests: confirms the `#![backend(dd)]` attribute parses,
//! dispatches to the DD codegen path, and emits compilable code.
//!
//! These tests deliberately do NOT assert anything about fixed-point
//! semantics — Phase 0 `run()` is a no-op. They only gate "the plumbing
//! works end-to-end."

use ascent::ascent;

ascent! {
   #![backend(dd)]
   pub struct Empty;
}

ascent! {
   #![backend(dd)]
   pub struct RelsOnly;

   relation edge(i32, i32);
   relation path(i32, i32);
}

#[test]
fn smoke_empty_program_builds_and_runs() {
   let mut prog = Empty::default();
   prog.run();
}

#[test]
fn smoke_program_with_relations_exposes_fields() {
   let mut prog = RelsOnly::default();
   prog.edge.push((1, 2));
   prog.edge.push((2, 3));
   prog.run();
   // Phase 0: `run()` is a no-op, so `edge` is untouched and `path` is empty.
   assert_eq!(prog.edge, vec![(1, 2), (2, 3)]);
   assert!(prog.path.is_empty());
}

#[test]
fn smoke_ascent_run_returns_struct() {
   let res = ascent::ascent_run! {
      #![backend(dd)]
      relation r(i32);
   };
   // `ascent_run!` returns the struct; Phase 0 does not populate anything.
   assert!(res.r.is_empty());
}
