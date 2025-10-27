
use ascent::ascent;


ascent! {
    struct TC;
    relation r(i32, i32);
    relation tc(i32, i32);
    tc(x, y) <-- r(x, y);
    tc(x, z+1) <-- #[heuristic_reordering] tc(x, y), tc(y, z), tc(z, x);
}

fn main() {
   let mut prog = TC::default();
   prog.r = vec![(1, 2), (2, 3), (3, 1)];
   prog.run();
   println!("{:?}", prog.tc);
}
