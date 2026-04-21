// Andersen's pointer analysis — DD batch-mode backend.

use ascent::ascent_par;
use bench_loader::load_2;
use std::env;
use std::time::Instant;

type V = i32;

ascent_par! {
    #![backend(dd)]
    #![dd(mode = batch)]
    pub struct AscentProgram;
    relation address_of(V, V);
    relation assign(V, V);
    relation load(V, V);
    relation store(V, V);
    relation points_to(V, V);

    points_to(y, x) <-- address_of(y, x);
    points_to(y, x) <-- points_to(z, x), assign(y, z);
    points_to(y, w) <-- points_to(x, z), load(y, x), points_to(z, w);
    points_to(z, w) <-- points_to(y, z), store(y, x), points_to(x, w);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <dir>", args[0]);
        std::process::exit(1);
    }
    let dir = &args[1];

    let t0 = Instant::now();
    let mut prog = AscentProgram::default();
    for t in load_2(dir, "addressOf.csv") { prog.address_of.push(t); }
    for t in load_2(dir, "assign.csv") { prog.assign.push(t); }
    for t in load_2(dir, "load.csv") { prog.load.push(t); }
    for t in load_2(dir, "store.csv") { prog.store.push(t); }
    let load_time = t0.elapsed();
    eprintln!("Loaded in {:?}", load_time);

    let t1 = Instant::now();
    prog.run();
    let run_time = t1.elapsed();

    eprintln!("points_to: {}", prog.points_to.len());
    eprintln!("Execution: {:?}", run_time);
    eprintln!("Total: {:?}", load_time + run_time);
}
