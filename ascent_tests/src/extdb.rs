
use std::{cell::RefCell, rc::Rc};

use ascent::ascent;

ascent! {
    struct TC;

    relation edge(i32, i32);
    relation path(i32, i32);
    index path (0, 1);

    path(x, y) <-- edge(x, y);
    path(x, y) <-- path(x, z), edge(z, y);
}

ascent! {
    struct SingleReach;

    extern TC tc;
    relation path(i32, i32) in tc;

    relation do_reach(i32, i32);
    relation reach(bool);

    reach(true) <-- do_reach(x, y), tc.path(x, y);
}

#[test]
fn test_reach() {
    let tc = TC::default();
    let tc = Rc::new(RefCell::new(tc));
    let mut sr = SingleReach::default();
    sr.tc = tc.clone();

    let input_edges = vec![(1, 2), (2, 3), (3, 4), (4, 5)];
    tc.borrow_mut().edge = input_edges.into_iter().collect();
    tc.borrow_mut().run();
    sr.do_reach = vec![(1, 5)];
    sr.run();

    println!("{:?}", &(sr.do_reach));
    println!("{:?}", &(sr.reach));
}

