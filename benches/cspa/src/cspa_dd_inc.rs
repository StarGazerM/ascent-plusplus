// Context-Sensitive Pointer Analysis (CSPA) — DD incremental-mode backend.
// Isize diffs, `reduce`-based distinct, `u32` timestamps inside iterative scopes.
use ascent::ascent_par;
use bench_loader::load_2;
use std::env;
use std::time::Instant;

type V = i32;

// FlowLog-parity rule shape — no assign_input/deref_input indirection. The
// user fills `prog.assign`/`prog.deref` directly like FlowLog's Assign /
// Dereference EDBs. Previously we had `assign(x,y) <-- assign_input(x,y)`
// copy rules + two extra relations + an extra non-looping SCC, which made
// the dataflow structurally wider than FlowLog's.
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
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 { eprintln!("Usage: {} <dir>", args[0]); std::process::exit(1); }
    let dir = &args[1];

    let t0 = Instant::now();
    let mut prog = AscentProgram::default();
    for t in load_2(dir, "Assign.csv") { prog.assign.push(t); }
    for t in load_2(dir, "Dereference.csv") { prog.deref.push(t); }
    let load_time = t0.elapsed();
    eprintln!("Loaded in {:?}", load_time);

    let t1 = Instant::now();
    prog.run();
    let run_time = t1.elapsed();

    eprintln!("value_flow: {}", prog.value_flow.len());
    eprintln!("memory_alias: {}", prog.memory_alias.len());
    eprintln!("value_alias: {}", prog.value_alias.len());
    eprintln!("Execution: {:?}", run_time);
    eprintln!("Total: {:?}", load_time + run_time);
}
