//! Sanity check: run CSPA on a synthetic tiny dataset (no file I/O).
//! Tells us if the program itself completes + correctness, independent
//! of the file-load path.

use ascent::ascent_par;
use std::time::Instant;

type V = i32;

ascent_par! {
    #![backend(dd)]
    #![dd(mode = incremental)]
    pub struct AscentProgram;
    relation assign(V, V);
    relation deref(V, V);
    relation value_flow(V, V);
    relation memory_alias(V, V);
    relation value_alias(V, V);

    value_flow(y, x) <-- assign(y, x);
    value_flow(x, x) <-- assign(x, _);
    value_flow(x, x) <-- assign(_, x);

    memory_alias(x, x) <-- assign(_, x);
    memory_alias(x, x) <-- assign(x, _);

    value_flow(x, y) <-- value_flow(x, z), value_flow(z, y);
    value_flow(x, y) <-- assign(x, z), memory_alias(z, y);

    value_alias(x, y) <-- value_flow(z, x), value_flow(z, y);
    value_alias(x, y) <-- value_flow(z, x), memory_alias(z, w), value_flow(w, y);

    memory_alias(x, w) <-- deref(y, x), value_alias(y, z), deref(z, w);
}

fn main() {
    let n: i32 = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(20);
    let mut p = AscentProgram::default();
    // Chain: 0→1→2→…→n-1 via assign; 0→n, 1→n+1, … via deref.
    for i in 0..n - 1 { p.assign.push((i, i + 1)); }
    for i in 0..n { p.deref.push((i, i + n)); }
    eprintln!("n={} |assign|={} |deref|={}", n, p.assign.len(), p.deref.len());
    let t0 = Instant::now();
    p.run();
    let dt = t0.elapsed();
    eprintln!("run took {:?}", dt);
    eprintln!("value_flow={} memory_alias={} value_alias={}",
        p.value_flow.len(), p.memory_alias.len(), p.value_alias.len());
}
