
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

    extern database TC tc;
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

    println!("{:?}", &(sr.reach));
}


ascent! {
    struct Graph;

    relation edge(i32, i32);
    index edge (1);
}

ascent! {
    struct SSSPEager;

    extern database Graph graph;
    relation edge(i32, i32) in graph;

    relation do_length(i32, i32);

    lattice ret(i32);

    ret(1) <-- do_length(x, y), graph.edge(x, y);
    
    ret(ret_val+1) <--
        do_length(x, z),
        graph.edge(y, z),
        let new_do_length = (*x, *y),
        let mut g = SSSPEager::default(),
        let _ = g.graph = _self.graph.clone(),
        let _ = g.do_length = vec![new_do_length],
        let _ = g.run(),
        if g.ret.len() == 1,
        let ret_val = g.ret[0].0;
}

#[test]
fn test_rec_length() {
    let g = Graph::default();
    let g = Rc::new(RefCell::new(g));
    g.borrow_mut().edge = vec![(1, 2), (2, 3), (3, 4), (4, 5)].into_iter().collect();
    g.borrow_mut().run();
    let mut compute_length = SSSPEager::default();
    compute_length.graph = g.clone();
    compute_length.do_length = vec![(1, 5)];
    compute_length.run();

    println!("{:?}", &(compute_length.ret));
}
