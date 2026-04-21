//! Per-rule free-fn extraction.
//!
//! Motivation: rustc places free `fn` items into codegen units
//! independently. When the generated program has hundreds of rules all
//! bodied inline inside one `run()` fn, the whole body ends up in a
//! single codegen unit — LLVM can't parallelize backend optimization.
//! Extracting each rule body to a free `fn` at module level lets rustc
//! split work across codegen units, scaling compile time with rule count.
//!
//! This first slice covers the simplest emission path: **non-looping SCC
//! single-clause fusion** (i.e. rules like `head(y, x) <-- rel(y, x);`
//! with no cond clauses). Future slices extend to multi-clause join
//! chains and recursive rule bodies inside `scope.iterative`.
//!
//! ## Fn shape
//!
//! For a rule of the form `head(...) <-- rel(...)` with input tuple type
//! `InTy` and head tuple type `OutTy`, we emit:
//!
//! ```ignore
//! pub(crate) fn <fn_name><G>(
//!     coll: &::ascent::dd::differential_dataflow::VecCollection<G, InTy, i32>,
//! ) -> ::ascent::dd::differential_dataflow::VecCollection<G, OutTy, i32>
//! where
//!     G: ::ascent::dd::timely::dataflow::Scope,
//!     G::Timestamp: ::ascent::dd::differential_dataflow::lattice::Lattice + Ord,
//! {
//!     coll.clone().map(move |<destructure>| -> OutTy { <var_rebinds> <head> })
//! }
//! ```
//!
//! The fn is generic over scope `G` so it works in both the top-level
//! `run()` path (`G = Child<Worker<Allocator>, u32>`) and, if lifted later,
//! inside `scope.iterative<Iter, _>` (`G = Child<_, Product<u32, Iter>>`).
//! rustc monomorphizes per call site and can place each monomorphization
//! in its own codegen unit.

use std::cell::RefCell;

use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::Type;

/// Per-macro-invocation context collecting rule fn items to emit at
/// module level. Thread-local because proc-macros run single-threaded
/// per invocation, and threading a `&mut` through every `compile_*` call
/// would touch ~8 functions in 4 files for a crosscutting feature.
///
/// Lifecycle:
///   - `reset(program_name)` at the top of `compile_mir_dd`.
///   - Code paths that extract a rule call `push(fn_item)` to accumulate.
///   - `take()` at the end of `compile_mir_dd` returns the collected
///     items to splice into the emitted module-level tokens.
///
/// Nested `ascent!` invocations shouldn't nest here — proc-macro calls
/// are serialized by rustc, so `reset` on each outer call resets cleanly.
#[derive(Default)]
struct Ctx {
   items: Vec<TokenStream>,
   counter: usize,
   program_name: Option<Ident>,
   enabled: bool,
}

thread_local! {
   static CTX: RefCell<Ctx> = RefCell::new(Ctx::default());
}

/// Enable/disable rule-fn extraction for this macro invocation. Call at
/// the start of `compile_mir_dd` with the program's struct name (used
/// for scoping fn idents so multiple `ascent!` in one module don't
/// collide).
pub(crate) fn reset(program_name: Ident, enabled: bool) {
   CTX.with(|c| {
      let mut c = c.borrow_mut();
      c.items.clear();
      c.counter = 0;
      c.program_name = Some(program_name);
      c.enabled = enabled;
   });
}

/// Is rule-fn extraction enabled for the current invocation? Callers use
/// this to fall back to the inline emission path when disabled (e.g. in
/// modes that aren't yet compatible).
pub(crate) fn enabled() -> bool {
   CTX.with(|c| c.borrow().enabled)
}

/// Allocate the next fn ident for this program. Monotonic counter avoids
/// name collisions between rules that hash to the same body shape.
pub(crate) fn next_ident() -> Ident {
   CTX.with(|c| {
      let mut c = c.borrow_mut();
      let prog = c.program_name.clone().unwrap_or_else(|| Ident::new("P", Span::call_site()));
      let idx = c.counter;
      c.counter += 1;
      rule_fn_ident(&prog, idx)
   })
}

/// Record a rule fn item for emission at module level.
pub(crate) fn push(item: TokenStream) {
   CTX.with(|c| c.borrow_mut().items.push(item));
}

/// Consume and return all recorded rule fn items. Call at end of codegen.
pub(crate) fn take() -> TokenStream {
   CTX.with(|c| {
      let mut c = c.borrow_mut();
      let items: Vec<TokenStream> = std::mem::take(&mut c.items);
      quote! { #(#items)* }
   })
}

/// A single-clause rule fn: `fn f<G>(coll: &Collection<G, InTy, i32>) -> Collection<G, OutTy, i32>`.
///
/// `body` is the `move |destruct| -> OutTy { rebinds; head }` closure
/// already built by the caller's emission helpers. We just plug it into
/// a `coll.clone().map(body)` inside the fn item.
pub(crate) struct SingleClauseRuleFn {
   pub name: Ident,
   pub in_ty: TokenStream,
   pub out_ty: Type,
   pub projection_body: TokenStream,
}

impl SingleClauseRuleFn {
   /// Emit the fn item declaration (to place at module level).
   pub fn fn_item(&self) -> TokenStream {
      let name = &self.name;
      let in_ty = &self.in_ty;
      let out_ty = &self.out_ty;
      let body = &self.projection_body;
      quote! {
         #[doc(hidden)]
         #[allow(non_snake_case, clippy::too_many_arguments, unused_parens, unused_variables, clippy::all)]
         pub(crate) fn #name<G>(
            __coll: &::ascent::dd::differential_dataflow::VecCollection<G, #in_ty, i32>,
         ) -> ::ascent::dd::differential_dataflow::VecCollection<G, #out_ty, i32>
         where
            G: ::ascent::dd::timely::dataflow::Scope,
            G::Timestamp: ::ascent::dd::differential_dataflow::lattice::Lattice + Ord,
         {
            #body
         }
      }
   }

   /// Emit the call site expression: `<name>(&<coll_expr>)`.
   pub fn call(&self, coll_ident: &Ident) -> TokenStream {
      let name = &self.name;
      quote! { #name(&#coll_ident) }
   }
}

/// Generate a unique rule-fn ident scoped to the current MIR program.
/// Caller ensures `idx` is unique across all extracted rule fns in this
/// program. The ident embeds the program name (via `program_name`) so
/// multiple `ascent!` invocations in one module don't collide.
pub(crate) fn rule_fn_ident(program_name: &Ident, idx: usize) -> Ident {
   Ident::new(&format!("__ascent_dd_rule_{}_{}", program_name, idx), Span::call_site())
}
