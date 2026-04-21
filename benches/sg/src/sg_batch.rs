// Galen EL ontology reasoning — pure Ascent batch.
//   P(x,z) :- PInput(x,z).
//   Q(x,r,z) :- QInput(x,r,z).
//   P(x,z) :- P(x,y), P(y,z).             -- TC
//   Q(x,r,z) :- P(x,y), Q(y,r,z).         -- PropQ
//   P(x,z) :- P(y,w), U(w,r,z), Q(x,r,y). -- Join3a
//   P(x,z) :- C(y,w,z), P(x,w), P(x,y).   -- Join3b
//   Q(x,q,z) :- Q(x,r,z), S(r,q).         -- PropQS
//   Q(x,e,o) :- Q(x,y,z), R(y,u,e), Q(z,u,o). -- Join3c

use ascent::ascent_par;
use bench_loader::{load_2, load_3};
use std::env;
use std::time::Instant;

type V = i32;

ascent_par! {
    relation p_input(V, V);
    relation q_input(V, V, V);
    relation r_input(V, V, V);
    relation c_input(V, V, V);
    relation u_input(V, V, V);
    relation s_input(V, V);

    relation out_p(V, V);
    relation out_q(V, V, V);

    out_p(x, z) <-- p_input(x, z);
    out_q(x, r, z) <-- q_input(x, r, z);
    out_p(x, z) <-- out_p(x, y), out_p(y, z);
    out_q(x, r, z) <-- out_p(x, y), out_q(y, r, z);
    out_p(x, z) <-- out_p(y, w), u_input(w, r, z), out_q(x, r, y);
    out_p(x, z) <-- c_input(y, w, z), out_p(x, w), out_p(x, y);
    out_q(x, q, z) <-- out_q(x, r, z), s_input(r, q);
    out_q(x, e, o) <-- out_q(x, y, z), r_input(y, u, e), out_q(z, u, o);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 { eprintln!("Usage: {} <dir>", args[0]); std::process::exit(1); }
    let dir = &args[1];

    let t0 = Instant::now();
    let mut prog = AscentProgram::default();
    for t in load_2(dir, "P.csv") { prog.p_input.push(t); }
    for t in load_3(dir, "Q.csv") { prog.q_input.push(t); }
    for t in load_3(dir, "R.csv") { prog.r_input.push(t); }
    for t in load_3(dir, "C.csv") { prog.c_input.push(t); }
    for t in load_3(dir, "U.csv") { prog.u_input.push(t); }
    for t in load_2(dir, "S.csv") { prog.s_input.push(t); }
    let load_time = t0.elapsed();
    eprintln!("Loaded in {:?}", load_time);

    let t1 = Instant::now();
    prog.run();
    let run_time = t1.elapsed();

    eprintln!("out_p: {}", prog.out_p.len());
    eprintln!("out_q: {}", prog.out_q.len());
    eprintln!("Execution: {:?}", run_time);
    eprintln!("Total: {:?}", load_time + run_time);
}
