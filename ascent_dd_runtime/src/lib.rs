//! Runtime support for the `ascent!` macro's differential-dataflow (DD) backend.
//!
//! Ascent's generated DD code references types and helpers from here via the
//! `::ascent::dd` re-export (enabled by the `dd` feature on the `ascent` crate).
//!
//! # Execution model (Phase 1)
//!
//! One timely worker, a single dataflow, batch semantics:
//! 1. `execute_batch` opens a dataflow scope at timestamp type `u32`.
//! 2. The user build closure creates inputs via [`Sealer::input`] (one session
//!    per relation, pre-loaded at time 0) and attaches output sinks via
//!    [`Sink::attach`] (which also wires the progress probe).
//! 3. After the closure returns, every input session is advanced to `1` and
//!    the worker steps until the probe has cleared `1`.
//!
//! This is enough to cover the Phase 1 gate — non-recursive, non-looping
//! programs — without any iteration scopes, `Variable`s, or semi-naive logic.

pub use {differential_dataflow, ordered_float, timely};
pub use ordered_float::OrderedFloat;

// ---------------------------------------------------------------------------
// AggResult: wrap aggregator outputs so they fit DD's `Data: Ord` requirement.
// ---------------------------------------------------------------------------
//
// Ascent aggregators return arbitrary owned types. DD's Collection value type
// must be `Ord` (for the arrangement's B-tree). Integer / ordered types are
// fine via the blanket impl; `f32` / `f64` fail the `Ord` bound because they
// only implement `PartialOrd`. `AggResult` bridges this by lifting floats
// through `OrderedFloat<T>`, whose total order treats NaN as the largest
// element (NaN == NaN, NaN > all non-NaN). After the DD join, we unwrap back
// to the user's original type before pattern-binding the aggregator output.
pub trait AggResult {
   /// Representation inside a DD Collection — must be `Ord`.
   type Ord: Clone + std::fmt::Debug + std::hash::Hash + Eq + Ord + Send + Sync + 'static;
   fn into_dd(self) -> Self::Ord;
   fn from_dd(v: Self::Ord) -> Self;
}

// Explicit impls — a blanket `impl<T: Ord> AggResult for T` would conflict
// with the `f32` / `f64` specializations under Rust's coherence rules (the
// compiler reserves the right for upstream to one day add `Ord for f64`).
// Users with custom `Ord` types can `impl AggResult for MyType {}` by hand.
macro_rules! agg_result_identity {
   ($($t:ty),* $(,)?) => {
      $(impl AggResult for $t {
         type Ord = $t;
         #[inline] fn into_dd(self) -> $t { self }
         #[inline] fn from_dd(v: $t) -> $t { v }
      })*
   };
}

agg_result_identity!(
   i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize,
   bool, char, String, std::sync::Arc<str>, std::sync::Arc<std::path::Path>,
   std::path::PathBuf, std::time::Duration, std::time::SystemTime
);

impl AggResult for f64 {
   type Ord = OrderedFloat<f64>;
   #[inline] fn into_dd(self) -> OrderedFloat<f64> { OrderedFloat(self) }
   #[inline] fn from_dd(v: OrderedFloat<f64>) -> f64 { v.into_inner() }
}

impl AggResult for f32 {
   type Ord = OrderedFloat<f32>;
   #[inline] fn into_dd(self) -> OrderedFloat<f32> { OrderedFloat(self) }
   #[inline] fn from_dd(v: OrderedFloat<f32>) -> f32 { v.into_inner() }
}

// Tuple impls so aggregators returning `(T, U)` etc. work without boilerplate.
impl AggResult for () {
   type Ord = ();
   #[inline] fn into_dd(self) -> () {}
   #[inline] fn from_dd(_: ()) {}
}

impl<A: AggResult> AggResult for (A,) {
   type Ord = (A::Ord,);
   #[inline] fn into_dd(self) -> (A::Ord,) { (self.0.into_dd(),) }
   #[inline] fn from_dd(v: (A::Ord,)) -> Self { (A::from_dd(v.0),) }
}

impl<A: AggResult, B: AggResult> AggResult for (A, B) {
   type Ord = (A::Ord, B::Ord);
   #[inline] fn into_dd(self) -> (A::Ord, B::Ord) { (self.0.into_dd(), self.1.into_dd()) }
   #[inline] fn from_dd(v: (A::Ord, B::Ord)) -> Self { (A::from_dd(v.0), B::from_dd(v.1)) }
}

impl<A: AggResult, B: AggResult, C: AggResult> AggResult for (A, B, C) {
   type Ord = (A::Ord, B::Ord, C::Ord);
   #[inline]
   fn into_dd(self) -> (A::Ord, B::Ord, C::Ord) { (self.0.into_dd(), self.1.into_dd(), self.2.into_dd()) }
   #[inline]
   fn from_dd(v: (A::Ord, B::Ord, C::Ord)) -> Self { (A::from_dd(v.0), B::from_dd(v.1), C::from_dd(v.2)) }
}

impl<T: AggResult> AggResult for Option<T> {
   type Ord = Option<T::Ord>;
   #[inline] fn into_dd(self) -> Option<T::Ord> { self.map(T::into_dd) }
   #[inline] fn from_dd(v: Option<T::Ord>) -> Self { v.map(T::from_dd) }
}

impl<T: AggResult> AggResult for Vec<T> {
   type Ord = Vec<T::Ord>;
   #[inline] fn into_dd(self) -> Vec<T::Ord> { self.into_iter().map(T::into_dd).collect() }
   #[inline] fn from_dd(v: Vec<T::Ord>) -> Self { v.into_iter().map(T::from_dd).collect() }
}

// ---------------------------------------------------------------------------
// LatDiff<L>: wrapper that turns an Ascent `Lattice` into a DD `Semigroup`.
// ---------------------------------------------------------------------------
//
// This is how the DD backend handles `lattice foo(K_cols, L)` efficiently:
// the collection lives as `Collection<K_tuple, LatDiff<L>>`, so DD's
// arrangement machinery consolidates via `plus_equals = Lattice::join_mut`
// natively — no per-iteration reduce-over-history, matching Ascent's batch
// backend O(1) per contribution.
//
// `Monoid::zero()` uses `L::default()` (conventionally the lattice bottom).
// `IsZero::is_zero` is hardcoded `false` so the arrangement never discards an
// entry — a lattice value of "bottom" is still a legitimate entry in the
// relation, not an absence marker. (Mirrors Flowlog's semiring templates.)
//
// `Abelian` is intentionally NOT implemented: lattices generally lack a
// negation. This means operators that require `Abelian` (e.g. `negate`,
// `consolidate` in some variants) can't be used directly on a lattice-diff
// Collection — but `arrange_by_key` / `join_core` / `iterate` all work.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct LatDiff<L>(pub L);

impl<L> LatDiff<L> {
   pub fn new(l: L) -> Self { LatDiff(l) }
   pub fn into_inner(self) -> L { self.0 }
}

impl<L: ascent_base::Lattice + Clone> differential_dataflow::difference::IsZero for LatDiff<L> {
   #[inline]
   fn is_zero(&self) -> bool { false }
}

impl<L: ascent_base::Lattice + Clone> differential_dataflow::difference::Semigroup for LatDiff<L> {
   #[inline]
   fn plus_equals(&mut self, rhs: &Self) {
      ascent_base::Lattice::join_mut(&mut self.0, rhs.0.clone());
   }
}

impl<L: ascent_base::Lattice + Clone + Default> differential_dataflow::difference::Monoid for LatDiff<L> {
   #[inline]
   fn zero() -> Self { LatDiff(L::default()) }
}

// Multiply against isize so lattice-diff Collections can be joined against
// regular set-diff (isize) Collections. Ignoring the isize factor is the
// correct semiring action — `isize` counts presence, so any non-zero count
// keeps the lattice value; zero-count shouldn't flow in at this step.
impl<L: Clone> differential_dataflow::difference::Multiply<isize> for LatDiff<L> {
   type Output = LatDiff<L>;
   #[inline]
   fn multiply(self, _rhs: &isize) -> Self::Output { self }
}

// Reverse direction: set-diff × lattice-diff → lattice-diff.
impl<L: Clone> differential_dataflow::difference::Multiply<LatDiff<L>> for isize {
   type Output = LatDiff<L>;
   #[inline]
   fn multiply(self, rhs: &LatDiff<L>) -> Self::Output { rhs.clone() }
}

// Lattice-diff × lattice-diff: for joining two lattice relations. Combine
// via semigroup (join). Tuple (a, b) joined yields their lattice-join.
impl<L: ascent_base::Lattice + Clone> differential_dataflow::difference::Multiply<LatDiff<L>> for LatDiff<L> {
   type Output = LatDiff<L>;
   #[inline]
   fn multiply(mut self, rhs: &LatDiff<L>) -> Self::Output {
      ascent_base::Lattice::join_mut(&mut self.0, rhs.0.clone());
      self
   }
}

use std::sync::{Arc, Mutex};

use differential_dataflow::difference::Present;
use differential_dataflow::input::{Input, InputSession};
use differential_dataflow::lattice::Lattice;
use differential_dataflow::{ExchangeData, Hashable};
// DD 0.20: `Collection<G, C>` is 2-arg; `VecCollection<G, D, R>` is the
// 3-arg alias over `Vec<(D, G::Timestamp, R)>`. Alias back to `Collection`
// locally so existing signatures (`Collection<G, T, isize>`) compile.
use differential_dataflow::VecCollection as Collection;
use timely::dataflow::scopes::Child;
use timely::dataflow::{ProbeHandle, Scope};
use timely::progress::Timestamp;
use timely::worker::Worker;

/// Diff type used by the **batch** DD codegen path.
///
/// `Present` is a fire-and-forget semigroup — it records "at least one
/// derivation exists" without tracking multiplicity. This matches pure
/// datalog semantics (tuples are present or absent) and lets DD's
/// arrangements skip the multiplicity-accounting overhead that isize
/// incurs. `ThresholdSemigroup` + `threshold_semigroup` replace the
/// `map().reduce().map()` distinct dance; `SemigroupVariable` replaces
/// `Variable` for Present-typed recursive state.
///
/// `Present` cannot represent negative diffs, so it is NOT used on the
/// **session** path (which supports `remove(tuple)` via isize diffs).
pub type BatchDiff = Present;
pub const BATCH_DIFF_ONE: BatchDiff = Present;

/// Re-export for codegen convenience so we don't have to spell the DD
/// module path in generated code.
// DD 0.20: `VecVariable<G, D, R>` is the 3-arg alias over `Variable<G, C>`
// with `C = Vec<(D, G::Timestamp, R)>`. Alias it back to `Variable` /
// `SemigroupVariable` so codegen's existing 3-arg usages compile.
pub use differential_dataflow::operators::iterate::VecVariable as Variable;
pub use differential_dataflow::operators::iterate::VecVariable as SemigroupVariable;

/// Shorthand for the root scope type we hand to user build closures.
/// Dataflow root scope used by `execute_batch`. Parameterised by `Generic`
/// so it covers both single-worker (`Config::thread()`) and multi-worker
/// (`Config::process(N)`) execution paths without type-surgery at every use.
pub type RootScope<'a> = Child<'a, Worker<timely::communication::allocator::Generic>, u32>;

/// Session-mode root scope. Session is single-worker for now (persistent
/// worker across many `commit()` calls with shared-state arrangements).
pub type SessionRootScope<'a> = Child<'a, Worker<timely::communication::allocator::thread::Thread>, u32>;

/// Shorthand for the worker type persisted inside generated `<Name>Session`s.
pub type SessionWorker = Worker<timely::communication::allocator::thread::Thread>;

/// Shorthand for a single-relation input session.
pub type SessionInput<T> = InputSession<u32, T, isize>;

/// Anything that knows how to advance its frontier to a given time and flush.
/// Erased behind `dyn` so a single [`Sealer`] can hold inputs of heterogeneous
/// element types — one session per relation, all at the same `Time`/`Diff`
/// but different element types.
///
/// No `Send` bound: the `Sealer` lives entirely inside the timely worker
/// closure and never crosses threads, and DD's `InputSession` contains `Rc`s
/// that aren't `Send`.
pub trait Sealable {
   fn seal(&mut self, time: u32);
}

impl<D: ExchangeData> Sealable for InputSession<u32, D, isize> {
   fn seal(&mut self, time: u32) {
      self.advance_to(time);
      self.flush();
   }
}

/// Registry of input sessions that `execute_batch` drives to the closed time
/// after the user build closure returns.
///
/// In a multi-worker run, only worker 0 actually inserts tuples — DD's
/// `arrange_by_key` hash-partitions data to the appropriate worker, so
/// loading on a single worker is correct AND cheaper than loading N×.
/// Non-zero workers still create (empty) InputSessions so they advance
/// the frontier in lockstep and the probe reaches the sealed time.
pub struct Sealer {
   sessions: Vec<Box<dyn Sealable + 'static>>,
   worker_index: usize,
}

impl Sealer {
   fn new(worker_index: usize) -> Self { Self { sessions: Vec::new(), worker_index } }

   /// Register a `Vec<T>` as an input relation. Data is inserted at time 0
   /// on worker 0 only; other workers get an empty InputSession. DD's
   /// arrange operators redistribute tuples by key-hash — no data is lost.
   /// The session is sealed to time 1 after dataflow construction.
   pub fn input<T, G>(&mut self, scope: &mut G, data: Vec<T>) -> Collection<G, T, isize>
   where
      T: ExchangeData,
      G: Scope<Timestamp = u32> + Input,
   {
      let mut session: InputSession<u32, T, isize> = InputSession::new();
      let coll = session.to_collection(scope);
      if self.worker_index == 0 {
         for t in data {
            session.insert(t);
         }
      }
      // drop `data` on non-zero workers.
      self.sessions.push(Box::new(session));
      coll
   }

   fn seal_all(&mut self, time: u32) {
      for s in self.sessions.iter_mut() {
         s.seal(time);
      }
   }
}

/// Captures deltas emitted by a DD `Collection` into a `Vec<T>`.
///
/// Always clone the sink *before* moving it into the `execute_batch` build
/// closure; keep the outer clone to drain results via [`Sink::into_vec`] after
/// `execute_batch` returns.
pub struct Sink<T: Clone + Send + 'static> {
   buf: Arc<Mutex<Vec<(T, isize)>>>,
}

impl<T: Clone + Send + 'static> Default for Sink<T> {
   fn default() -> Self { Self::new() }
}

impl<T: Clone + Send + 'static> Clone for Sink<T> {
   fn clone(&self) -> Self { Self { buf: self.buf.clone() } }
}

impl<T: Clone + Send + 'static> Sink<T> {
   pub fn new() -> Self { Self { buf: Arc::new(Mutex::new(Vec::new())) } }

   /// Hook this sink onto `collection` and wire the progress probe.
   pub fn attach<G>(&self, collection: &Collection<G, T, isize>, probe: &mut ProbeHandle<G::Timestamp>)
   where
      G: Scope,
      G::Timestamp: Lattice + Timestamp,
      T: ExchangeData + Hashable,
   {
      let buf = self.buf.clone();
      // DD 0.20: `.inspect` takes self by value — clone the `&Collection`.
      collection
         .clone()
         .inspect(move |(d, _t, r)| {
            buf.lock().unwrap().push((d.clone(), *r));
         })
         .probe_with(probe);
   }

   /// Drain into a flat `Vec<T>`, expanding positive diffs and dropping rows
   /// with net-negative multiplicity.
   pub fn into_vec(self) -> Vec<T> {
      let buf = std::mem::take(&mut *self.buf.lock().unwrap());
      let mut out = Vec::with_capacity(buf.len());
      for (t, diff) in buf {
         if diff > 0 {
            for _ in 0..diff {
               out.push(t.clone());
            }
         }
      }
      out
   }

   /// Drain the raw delta log — `(value, diff)` pairs — since the last call
   /// to any drain method. Positive diff = insertion, negative = retraction.
   ///
   /// This is the incremental-output primitive: after a `Session::commit()`
   /// this returns only the changes since the prior commit, not the full
   /// collection state.
   pub fn drain_deltas(&self) -> Vec<(T, isize)> { std::mem::take(&mut *self.buf.lock().unwrap()) }
}

/// Runs a single-worker, single-dataflow batch computation to completion.
///
/// The build closure receives:
/// - `scope`: the root dataflow scope (timestamp type `u32`)
/// - `sealer`: registry for input relations (call `sealer.input(scope, data)`)
/// - `probe`: progress probe to attach to every output via `Sink::attach`
///
/// All inputs are sealed to time 1 after the closure returns, then the worker
/// steps until the probe has cleared that time.
///
/// Note: we construct `Worker` inline via `Worker::new` rather than going
/// through `timely::execute_directly` so the build closure is free of the
/// `Send + Sync + 'static` bounds `execute_directly` imposes. This lets user
/// Ascent programs capture non-'static borrows (e.g. `edges: &[(i32,i32)]`
/// in a test fn) — matching what batch Ascent supports.
/// Collect an iterator into `Vec<T>` regardless of whether its `Item` is
/// `T` (owned) or `&T` (ref). Bridges the batch-Ascent / DD mismatch where
/// a user's `for pat in some.iter()` yields refs in batch but DD requires
/// owned values to store in a `Collection` (which needs `'static` data).
///
/// Works via `Borrow<T>`: `T: Borrow<T>` (blanket) and `&T: Borrow<T>` both
/// satisfy the bound, so a single call handles both `r.iter()` (refs) and
/// `0..n` (owned) identically, producing `Vec<T>` in either case.
pub fn collect_owned<T, I>(iter: I) -> Vec<T>
where
   T: Clone,
   I: IntoIterator,
   I::Item: std::borrow::Borrow<T>,
{
   use std::borrow::Borrow;
   iter.into_iter().map(|i| Borrow::<T>::borrow(&i).clone()).collect()
}

/// Coerce a value of type `T` or `&T` into owned `T` by cloning. Used by
/// DD codegen at `let` condition boundaries so the resulting pattern value
/// is always owned (the new Collection tuple element must be `'static`).
///
/// Call site: `clone_borrow(&(expr))`. For `expr: T` this gives `&T`; for
/// `expr: &T` this gives `&&T` which Rust deref-coerces to `&T`. Either way
/// the parameter lands as `&T` and we clone it.
#[inline]
pub fn clone_borrow<T: Clone>(x: &T) -> T { x.clone() }

/// DD worker count.
///
/// Default is **1** — `prog.run()` on a small input shouldn't silently
/// spawn N OS threads. Batch parallelism is opt-in via the env var.
///
/// When to raise it: benchmarks / big-graph workloads where join
/// arrangement partitioning actually amortizes the thread-spawn cost.
/// Rule of thumb: input ≥ 10⁴ tuples per worker.
///
/// Why not default-to-parallel: spawning `available_parallelism()` OS
/// threads per `run()` call has two bad failure modes:
///   1. Small inputs: thread-setup dominates, single-worker is faster.
///   2. Parallel test harnesses: N tests × M workers = N·M threads,
///      tests thrash and miss timeouts. See `ascent_tests_dd`.
///
/// Resolution order:
///   1. `ASCENT_DD_WORKERS` env var (any integer ≥ 1).
///   2. Fall back to `1`.
pub fn dd_worker_count() -> usize {
   if let Ok(s) = std::env::var("ASCENT_DD_WORKERS") {
      if let Ok(n) = s.parse::<usize>() {
         if n >= 1 {
            return n;
         }
      }
   }
   1
}

// ---------------------------------------------------------------------------
// Batch runtime (Present-typed): used by generated `run()` for DD backend.
// ---------------------------------------------------------------------------

/// Present-typed input handle. Each worker gets its own, but only
/// worker 0 feeds data (DD's arrange hash-partitions downstream).
///
/// Outer timestamp is `()` — FlowLog's trick. `Product<(), Iter>` is
/// `TotalOrder`, which `ThresholdTotal::threshold_semigroup` requires.
/// `u32` outer (session path) gives `Product<u32, Iter>` — NOT total —
/// so batch-mode threshold wouldn't compile.
pub type BatchSessionInput<T> = InputSession<(), T, BatchDiff>;

trait BatchSealable {
   fn seal(self: Box<Self>);
}
impl<D: ExchangeData> BatchSealable for InputSession<(), D, BatchDiff> {
   fn seal(mut self: Box<Self>) {
      // `()` timestamp: nothing to advance to — flush buffered inserts and
      // drop the session so the probe's frontier can retire past `()`.
      // Without the drop, the session keeps the outer frontier open and
      // `step_while(probe.less_than(&()))` would spin forever.
      self.flush();
      drop(self);
   }
}

/// Batch-mode sealer: mirrors `Sealer` but returns `Collection`s with
/// `Present` diff over `()` timestamp. Same worker-0-loads-only semantics.
pub struct BatchSealer {
   sessions: Vec<Box<dyn BatchSealable + 'static>>,
   worker_index: usize,
}

impl BatchSealer {
   fn new(worker_index: usize) -> Self { Self { sessions: Vec::new(), worker_index } }

   /// Worker index. Exposed so generated code can route per-worker sink writes.
   pub fn worker_index(&self) -> usize { self.worker_index }

   pub fn input<T, G>(&mut self, scope: &mut G, data: Vec<T>) -> Collection<G, T, BatchDiff>
   where
      T: ExchangeData,
      G: Scope<Timestamp = ()> + Input,
   {
      // Use `scope.new_collection_from_raw(empty)` instead of
      // `InputSession::new() + to_collection()`. The InputSession::new()
      // path creates a DIFFERENT timely operator graph that breaks DD's
      // fixpoint convergence under `Present` diff at scale (confirmed via
      // hand-written side-by-side: `new_collection_from_raw` scales
      // linearly to n=10000+; `InputSession::new()+to_collection()` hangs
      // at n=1025). Root cause: `scope.new_input()` vs `scope.input_from()`
      // register the input differently with the scope's progress tracker.
      let (mut session, coll) = scope.new_collection_from_raw::<T, BatchDiff, _>(
         ::std::iter::empty::<(T, (), BatchDiff)>(),
      );
      if self.worker_index == 0 {
         for t in data {
            session.update(t, BATCH_DIFF_ONE);
         }
      }
      self.sessions.push(Box::new(session));
      coll
   }

   fn seal_all(&mut self) {
      // `std::mem::take` drains the Vec so we can pass ownership of each
      // Box to `seal()`, which drops the session after flushing.
      for s in std::mem::take(&mut self.sessions) {
         s.seal();
      }
   }
}

/// Present-typed output sink with PER-WORKER slots — each worker writes
/// to `bufs[worker_index]`, no cross-worker Mutex contention on the hot
/// path. Prior design used one `Arc<Mutex<Vec>>` shared across workers;
/// at 24 workers pushing millions of tuples that lock dominated runtime
/// and killed parallel scaling (we stayed at 55s from 4→24 workers while
/// FlowLog scaled to 23s). Per-worker slot Mutexes are uncontended.
pub struct BatchSink<T: Clone + Send + 'static> {
   bufs: Arc<Vec<Mutex<Vec<T>>>>,
}

impl<T: Clone + Send + 'static> Default for BatchSink<T> {
   fn default() -> Self { Self::new() }
}

impl<T: Clone + Send + 'static> Clone for BatchSink<T> {
   fn clone(&self) -> Self { Self { bufs: self.bufs.clone() } }
}

impl<T: Clone + Send + 'static> BatchSink<T> {
   pub fn new() -> Self {
      // Pre-allocate one slot per worker. `dd_worker_count()` resolves the
      // env var / CPU-count fallback that `execute_batch_present` will use.
      let n = dd_worker_count();
      let mut bufs = Vec::with_capacity(n);
      for _ in 0..n {
         bufs.push(Mutex::new(Vec::new()));
      }
      Self { bufs: Arc::new(bufs) }
   }

   pub fn attach<G>(
      &self, worker_index: usize, collection: &Collection<G, T, BatchDiff>,
      probe: &mut ProbeHandle<G::Timestamp>,
   )
   where
      G: Scope,
      G::Timestamp: Lattice + Timestamp,
      T: ExchangeData + Hashable,
   {
      let bufs = self.bufs.clone();
      collection
         .clone()
         .inspect(move |(d, _t, _present)| {
            // Only this worker writes to this slot — Mutex is uncontested,
            // lock/unlock are near-free on modern pthreads (~10ns).
            bufs[worker_index].lock().unwrap().push(d.clone());
         })
         .probe_with(probe);
   }

   /// Drain all per-worker slots, concat into a single Vec.
   pub fn into_vec(self) -> Vec<T> {
      // `Arc::try_unwrap` lets us consume the inner Vec<Mutex<Vec<T>>>
      // without cloning. Should succeed since we're consuming self and
      // all closures hold their own clones that are dropped by now.
      match Arc::try_unwrap(self.bufs) {
         Ok(vec) => {
            let mut out = Vec::new();
            for m in vec {
               out.append(&mut m.into_inner().unwrap());
            }
            out
         }
         Err(arc) => {
            // Fallback: outstanding clones still exist — drain via lock.
            let mut out = Vec::new();
            for m in arc.iter() {
               out.append(&mut std::mem::take(&mut *m.lock().unwrap()));
            }
            out
         }
      }
   }
}

/// Present-typed parallel batch runner. Mirrors `execute_batch` but uses
/// `Present` diffs so arrangements / joins skip multiplicity accounting,
/// and `()` outer timestamp so `Product<(), Iter>` satisfies `TotalOrder`
/// (required by `threshold_semigroup` inside iterative scopes).
/// Used by generated `run()` in the DD backend's batch mode.
pub fn execute_batch_present<F>(build: F)
where F: for<'a> Fn(&mut BatchRootScope<'a>, &mut BatchSealer, &mut ProbeHandle<()>) + Send + Sync + 'static {
   let workers = dd_worker_count();
   let config = if workers == 1 { timely::Config::thread() } else { timely::Config::process(workers) };
   let build = Arc::new(build);
   timely::execute::execute(config, move |worker| {
      let build = build.clone();
      let worker_index = worker.index();
      let (mut sealer, probe) = worker.dataflow::<(), _, _>(|scope| {
         let mut sealer = BatchSealer::new(worker_index);
         let mut probe = ProbeHandle::new();
         build(scope, &mut sealer, &mut probe);
         (sealer, probe)
      });
      // Drop sessions (frontier retires past `()`), then step to quiescence.
      // For `()` timestamps, `probe.less_than(&())` is already false at the
      // start (frontier `[()]` — `()` not `< ()`), so the incremental-style
      // frontier-chasing loop would 0-step. Correct batch idiom: drive
      // until `worker.step()` reports no pending activations.
      sealer.seal_all();
      while worker.step() {}
      let _ = probe;
   })
   .unwrap()
   .join();
}

/// Outer-scope alias for batch mode: `()` timestamp. Mirrors `RootScope` but
/// with `TotalOrder`-friendly timestamp for `threshold_semigroup`.
pub type BatchRootScope<'a> = timely::dataflow::scopes::Child<
   'a,
   timely::worker::Worker<timely::communication::Allocator>,
   (),
>;

// ---------------------------------------------------------------------------
// Session runtime (isize-typed): used by `<Name>Session::commit()`.
// ---------------------------------------------------------------------------

pub fn execute_batch<F>(build: F)
where F: for<'a> Fn(&mut RootScope<'a>, &mut Sealer, &mut ProbeHandle<u32>) + Send + Sync + 'static {
   let workers = dd_worker_count();
   let config = if workers == 1 { timely::Config::thread() } else { timely::Config::process(workers) };
   let build = Arc::new(build);
   // Each worker runs `build` independently; shared state (Sink's
   // Arc<Mutex<Vec>>) is captured by reference. Inputs are re-`.clone()`d
   // per worker call — DD's arrange operators hash-partition keys across
   // workers, so redundant loading is correct and the real join work
   // distributes.
   timely::execute::execute(config, move |worker| {
      let build = build.clone();
      let worker_index = worker.index();
      let (mut sealer, probe) = worker.dataflow::<u32, _, _>(|scope| {
         let mut sealer = Sealer::new(worker_index);
         let mut probe = ProbeHandle::new();
         build(scope, &mut sealer, &mut probe);
         (sealer, probe)
      });
      sealer.seal_all(1);
      while probe.less_than(&1) {
         worker.step();
      }
   })
   .unwrap()
   .join();
}

// ---------------------------------------------------------------------------
// Session: persistent worker for incremental updates
// ---------------------------------------------------------------------------
//
// `execute_batch` constructs + tears down a worker per call — fine for one-shot
// `run()` semantics. Incremental use needs the worker to live across many
// `insert/remove/commit` cycles so that DD's arrangements stay warm and
// subsequent `commit`s produce only deltas.
//
// The machinery below is thin: generated `<Name>Session` structs own a
// `Worker<Thread>` plus `InputSession`s plus output `Sink`s plus a probe,
// constructed by `build_session_worker` below. Users call `.commit()` to
// advance time and drive the worker to the new frontier.

/// Builds a single-worker inline (no thread spawn) and runs the user build
/// closure inside `worker.dataflow`. Returns both the worker and the value
/// produced by `build` (typically a tuple of input sessions, sinks, and a
/// probe). The worker is kept alive and returned to the caller, who is
/// responsible for driving it via `worker.step()` after each commit.
pub fn build_session_worker<T, F>(
   build: F,
) -> (timely::worker::Worker<timely::communication::allocator::thread::Thread>, T)
where F: FnOnce(&mut SessionRootScope<'_>) -> T {
   let alloc = timely::communication::allocator::thread::Thread::default();
   let mut worker = timely::worker::Worker::new(
      timely::WorkerConfig::default(),
      alloc,
      Some(std::time::Instant::now()),
   );
   let out = worker.dataflow::<u32, _, _>(|scope| build(scope));
   (worker, out)
}

#[cfg(test)]
mod tests {
   // DD 0.20: `Join` / `Threshold` / `Iterate` are not standalone traits any
   // more — `.join`, `.distinct`, `.concat`, etc. are inherent methods on
   // `Collection` / `VecCollection`. So no trait imports needed.
   use differential_dataflow::VecCollection;
   use timely::dataflow::ProbeHandle;
   use timely::dataflow::scopes::Scope;
   use timely::order::Product;

   // `Variable` here refers to the crate-level re-export
   // (`VecVariable`, 3-arg — `<G, D, R>`), not `differential_dataflow::operators::iterate::Variable`
   // (the newer 2-arg type in DD 0.20).
   use super::*;

   #[test]
   fn sink_captures_cross_join() {
      // Mirrors the shape of Ascent's `ab(x, y) <-- a(x), b(y)` lowering.
      let sink: Sink<(i32, i32)> = Sink::new();
      let sink_for_closure = sink.clone();

      execute_batch(move |scope, sealer, probe| {
         let a = sealer.input::<i32, _>(scope, vec![1, 2]).map(|x| ((), x));
         let b = sealer.input::<i32, _>(scope, vec![10, 20]).map(|x| ((), x));
         let ab = a.join(b).map(|(_, (x, y))| (x, y));
         sink_for_closure.attach(&ab, probe);
      });

      let mut got = sink.into_vec();
      got.sort();
      assert_eq!(got, vec![(1, 10), (1, 20), (2, 10), (2, 20)]);
   }

   /// PoC: long-lived worker doing transitive closure, with live inserts
   /// and retractions between commits. Mirrors what the codegen-emitted
   /// `TcSession` is going to look like.
   #[test]
   fn session_tc_incremental() {
      let path_sink: Sink<(i32, i32)> = Sink::new();
      let path_sink_for_build = path_sink.clone();

      let (mut worker, (mut edge, mut probe)) = build_session_worker(move |scope| {
         let mut edge: InputSession<u32, (i32, i32), isize> = InputSession::new();
         let edge_coll = edge.to_collection(scope);
         let mut probe: ProbeHandle<u32> = ProbeHandle::new();

         let path = scope.iterative::<u32, _, _>(|inner| {
            let edge_inner = edge_coll.enter(inner);
            // DD 0.20: `Variable::new` returns `(Variable, VecCollection)`.
            // The Variable itself is consumed by `.set(...)`; we read the
            // companion Collection directly (no `*var` deref any more).
            let (path_var, path_coll): (
               Variable<_, (i32, i32), isize>,
               VecCollection<_, (i32, i32), isize>,
            ) = Variable::new(inner, Product::new(Default::default(), 1));

            let rule1 = edge_inner.clone();
            let rule2 = edge_inner
               .map(|(x, y)| (y, x))
               .join(path_coll.map(|(y, z)| (y, z)))
               .map(|(_y, (x, z))| (x, z));

            // 0.20: `.concat()` consumes both sides; `.set(...)` consumes the
            // Variable and takes an owned Collection (returns `()`). Keep a
            // clone for `.leave()` to carry the final Collection out of scope.
            let next = rule1.concat(rule2).distinct();
            path_var.set(next.clone());
            next.leave()
         });

         path_sink_for_build.attach(&path, &mut probe);
         (edge, probe)
      });

      // Helper: advance all inputs to `next_epoch` and drive the worker to
      // that frontier, then return the deltas the path sink saw.
      let mut epoch: u32 = 0;
      let mut step_to = |edge: &mut InputSession<u32, (i32, i32), isize>,
                         probe: &mut ProbeHandle<u32>,
                         worker: &mut timely::worker::Worker<_>,
                         epoch: &mut u32| {
         let next = *epoch + 1;
         edge.advance_to(next);
         edge.flush();
         while probe.less_than(&next) {
            worker.step();
         }
         *epoch = next;
      };

      // Commit 1: insert 1→2, 2→3. Expect path {(1,2), (2,3), (1,3)}.
      edge.insert((1, 2));
      edge.insert((2, 3));
      step_to(&mut edge, &mut probe, &mut worker, &mut epoch);

      // Running net multiplicity state (what Session::*_snapshot builds on).
      let mut path_state: std::collections::HashMap<(i32, i32), isize> = Default::default();
      for (t, d) in path_sink.drain_deltas() {
         *path_state.entry(t).or_insert(0) += d;
      }
      let mut snap: Vec<_> = path_state.iter().filter(|(_, c)| **c > 0).map(|(k, _)| *k).collect();
      snap.sort();
      assert_eq!(snap, vec![(1, 2), (1, 3), (2, 3)]);

      // Commit 2: add 3→4. Incrementally expect (1,4), (2,4), (3,4) added.
      edge.insert((3, 4));
      step_to(&mut edge, &mut probe, &mut worker, &mut epoch);
      let incremental = path_sink.drain_deltas();
      for (t, d) in &incremental {
         *path_state.entry(*t).or_insert(0) += *d;
      }
      let mut added: Vec<_> = incremental.iter().filter(|(_, d)| *d > 0).map(|(t, _)| *t).collect();
      added.sort();
      assert_eq!(added, vec![(1, 4), (2, 4), (3, 4)]);

      // Commit 3: retract 2→3. Expect (1,3), (2,3), (2,4) all retracted, (1,4) survives via 1→2→…→4.
      edge.remove((2, 3));
      step_to(&mut edge, &mut probe, &mut worker, &mut epoch);
      for (t, d) in path_sink.drain_deltas() {
         *path_state.entry(t).or_insert(0) += d;
      }
      let mut snap3: Vec<_> = path_state.iter().filter(|(_, c)| **c > 0).map(|(k, _)| *k).collect();
      snap3.sort();
      // After retracting 2→3: remaining edges {1→2, 3→4}. Paths: (1,2), (3,4).
      // Note: 1→2 has no onward because 2→3 is gone.
      assert_eq!(snap3, vec![(1, 2), (3, 4)]);
   }

   #[test]
   fn sink_captures_passthrough() {
      // Degenerate case: `out(x) <-- src(x)`; just a map + sink.
      let sink: Sink<i32> = Sink::new();
      let s2 = sink.clone();
      execute_batch(move |scope, sealer, probe| {
         let out = sealer.input::<i32, _>(scope, vec![7, 8, 9]);
         s2.attach(&out, probe);
      });
      let mut got = sink.into_vec();
      got.sort();
      assert_eq!(got, vec![7, 8, 9]);
   }
}
