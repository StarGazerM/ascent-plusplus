

use std::{cell::RefCell, rc::Rc};

use ascent::{ascent, ascent_par};

macro_rules! mono {
    ($x:expr) => {
        $x[0].0
    };
}

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

    extern database TC tc();
    relation path(i32, i32) in tc;

    relation do_reach(i32, i32);
    relation reach(bool);

    reach(true) <-- do_reach(x, y), tc.path(x, y);
}

#[test]
fn test_reach() {
    let mut tc = TC::default();
    let mut sr = SingleReach::default();

    let input_edges = vec![(1, 2), (2, 3), (3, 4), (4, 5)];
    tc.edge = input_edges.into_iter().collect();
    tc.run();
    sr.do_reach = vec![(1, 5)];
    sr.run(Rc::new(RefCell::new(tc)));

    println!("{:?}", &(sr.reach));
}


ascent! {
    struct Graph;

    relation edge(i32, i32);
    index edge (1);
    index edge (0, 1);
}

ascent! {
    struct Printer;

    extern database Graph graph();
    relation edge(i32, i32) in graph;

    relation print(i32, i32);
    relation empty();

    empty() <-- print(x, y), let _ = println!("{} {}", x, y);
}

use ascent::aggregators::sum;

ascent! {
    struct SSSPEager;
    extern database Graph graph();
    extern database Printer pp(graph);

    relation edge(i32, i32) in graph;

    
    relation print(i32, i32) in pp;

    relation do_length(i32, i32);
    relation incomming_length(i32);
    relation ret(i32);

    ret(1) <-- do_length(x, y), graph.edge(x, y);
    
    incomming_length(mono!(g.ret) + 1), pp.print(y, z) <--
        do_length(x, z), graph.edge(y, z),
        do g : SSSPEager {
            do_length : vec![(*x, *y)]
        } (graph.clone(), pp.clone());
    
    ret(total) <-- agg total = sum(x) in incomming_length(x);
}

#[test]
fn test_rec_length() {
    let g = Graph::default();
    let pp = Printer::default();
    let pp = Rc::new(RefCell::new(pp));
    let g = Rc::new(RefCell::new(g));
    // pp.run(Rc::new(RefCell::new(g.clone())));
    g.borrow_mut().edge = vec![(1, 2), (2, 3), (3, 4), (4, 5)].into_iter().collect();
    g.borrow_mut().run();
    let mut compute_length = SSSPEager::default();
    compute_length.do_length = vec![(1, 5)];
    compute_length.run(g.clone(), pp.clone());

    println!("{:?}", &(compute_length.ret));
    println!("{:?}", &(pp.borrow().print));
    pp.borrow_mut().run(g.clone());
}
