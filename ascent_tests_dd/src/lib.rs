#![allow(confusable_idents)]
//! Test-suite fork that exercises the `#![backend(dd)]` codegen.
//!
//! Keep one module per feature cluster so `cargo test -p ascent_tests_dd`
//! has a readable output. Tests that the backend does not yet support are
//! annotated `#[ignore = "dd: phase <N> — <reason>"]` — never deleted,
//! never locally rewritten to avoid the limitation. That way the
//! green/ignored counts are a meaningful progress metric.
//!
//! # Upstream test fork
//!
//! The goal is that every test in `ascent_tests` passes verbatim under the
//! DD backend too. To avoid maintaining a parallel copy, we mirror each
//! upstream file here with *one* change: `use ascent::{ascent, ascent_run}`
//! becomes `use ascent_tests_dd::{ascent_dd as ascent, ascent_run_dd as
//! ascent_run}`. The shim macros below just inject `#![backend(dd)]` at the
//! top of the macro body and forward to `::ascent::ascent!`/`ascent_run!`.
//! Nothing else in the test body changes.

#[macro_export]
macro_rules! ascent_dd {
   ($($tt:tt)*) => {
      ::ascent::ascent! { #![backend(dd)] $($tt)* }
   };
}

#[macro_export]
macro_rules! ascent_run_dd {
   ($($tt:tt)*) => {
      ::ascent::ascent_run! { #![backend(dd)] $($tt)* }
   };
}

// Scaffolding that upstream ascent_tests expect: `ascent_maybe_par` (the
// `ascent_m_par!` / `ascent_run_m_par!` macros + `lat_to_vec`), `utils` (the
// `rels_equal` + `assert_rels_eq!` helpers). Kept as siblings so forked test
// files compile verbatim.
pub mod ascent_maybe_par;
pub mod utils;

#[cfg(test)]
mod smoke;
#[cfg(test)]
mod phase1_base;
#[cfg(test)]
mod phase2_recursion;
#[cfg(test)]
mod phase2_session;
#[cfg(test)]
mod phase3_clauses;
#[cfg(test)]
mod phase6_lattice;
#[cfg(test)]
mod phase7_streaming;
#[cfg(test)]
mod phase8_plan;
#[cfg(test)]
mod phase9_compose;
#[cfg(test)]
mod egglog_port;
// Upstream `ascent_tests/src/tests.rs` forked verbatim (see
// `upstream_tests.rs.hold`). Many tests there use types (`LambdaCalcExpr`,
// `TNode`) without `Ord`/`Serialize`/`Deserialize` derives, which the DD
// backend fundamentally requires. Reactivating this module requires either
// (a) per-test exclusion attributes or (b) user-type derive annotations in
// the datalog. Tracked in status; reactivated piecewise as support lands.
// Upstream fork held — use upstream_subset for surgical per-test enablement.
// #[cfg(test)]
// mod upstream_tests;
#[cfg(test)]
mod upstream_subset;
