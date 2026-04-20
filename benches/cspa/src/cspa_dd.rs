// Context-Sensitive Pointer Analysis (CSPA)
use ascent::ascent_par;
use std::env;
use std::fs;
use std::time::Instant;

type V = i32;

fn load_2(dir: &str, name: &str) -> Vec<(V, V)> {
    let path = format!("{}/{}", dir, name);
    let content = fs::read_to_string(&path).unwrap_or_default();
    let delim = if content.contains('\t') { '\t' } else { ',' };
    content.lines().filter(|l| !l.is_empty())
        .map(|l| { let c: Vec<&str> = l.split(delim).collect(); (c[0].parse().unwrap(), c[1].parse().unwrap()) })
        .collect()
}

ascent_par! {
    #![backend(dd, mode = "batch")]
    pub struct AscentProgram;
    relation assign_input(V, V);
    relation deref_input(V, V);
    relation assign(V, V);
    relation deref(V, V);
    relation value_flow(V, V);
    relation memory_alias(V, V);
    relation value_alias(V, V);

    assign(x, y) <-- assign_input(x, y);
    deref(x, y) <-- deref_input(x, y);

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
    for t in load_2(dir, "Assign.csv") { prog.assign_input.push(t); }
    for t in load_2(dir, "Dereference.csv") { prog.deref_input.push(t); }
    let load_time = t0.elapsed();
    eprintln!("Loaded in {:?}", load_time);

    let t1 = Instant::now();
    prog.run();
    let run_time = t1.elapsed();

    eprintln!("value_flow: {}", prog.value_flow.len());
    eprintln!("memory_alias: {}", prog.memory_alias.len());
    eprintln!("value_alias: {}", prog.value_alias.len());
    eprintln!("Execution: {:?}", run_time);
}
