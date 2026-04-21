//! Minimal sanity test: does our inc-mode `run()` work at all?
use ascent::ascent;

type V = i32;

ascent! {
    #![backend(dd, mode = "incremental")]
    pub struct Tc;
    relation edge(V, V);
    relation path(V, V);
    path(x, y) <-- edge(x, y);
    path(x, z) <-- edge(x, y), path(y, z);
}

fn main() {
    let mut p = Tc::default();
    p.edge.push((1, 2));
    p.edge.push((2, 3));
    p.edge.push((3, 4));
    let t0 = std::time::Instant::now();
    p.run();
    eprintln!("run took {:?}", t0.elapsed());
    let mut out = p.path.clone();
    out.sort();
    eprintln!("path = {:?}", out);
}
