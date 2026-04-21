//! Ascent MIR — the non-proc-macro home for MIR data types and the textual
//! grammar that crosses the frontend ↔ backend boundary.
//!
//! # Status
//!
//! Skeleton. Types are being moved from `ascent_macro` in phases. This crate
//! deliberately has no dependency on `ascent_macro` (proc-macro crates can't
//! be depended on anyway), so any type living here is freely consumable by
//! external codegen-backend crates.
//!
//! # Phases
//!
//! 1. Utility helpers used by moved MIR types (done in `utils`).
//! 2. MIR-layer type definitions (next): `AscentMir`, `MirScc`, `MirRule`,
//!    `MirBodyItem`, `MirBodyClause`, `MirRelation`, `MirRelationVersion`.
//! 3. Transitively-referenced types from HIR/syntax layers.
//! 4. Grammar emit/parse impls (currently in `ascent_macro/src/mir_text.rs`).
//!
//! Each phase lands independently with tests green.

pub mod utils;
pub mod syn_utils;
pub mod types;
pub mod text;

pub use types::*;
pub use text::{emit_mir, parse_mir};
