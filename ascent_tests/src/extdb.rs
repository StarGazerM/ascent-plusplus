

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
    struct TestExtArg;
    extern arguement i32 test_arg;

    relation test(i32, i32);
    relation foo(i32);

    test(x, z) <-- foo(x), let z = test_arg;

    foo(1);
}

#[test]
fn test_ext_arg() {
    let mut test = TestExtArg::default();
    test.run(1);
    assert!(test.test.contains(&(1, 1)));
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

    await print;

    empty() <-- print(x, y), let _ = println!("{} {}", x, y);
}

use ascent::aggregators::sum;

ascent! {
    struct LongestPath;
    extern database Graph graph();
    extern database Printer pp(graph);
    extern arguement (i32, i32) do_length;

    relation edge(i32, i32) in graph;
    relation print(i32, i32) in pp;

    relation incomming_length(i32);
    relation ret(i32);

    ret(1) <-- let (x, y) = do_length, graph.edge(x, y);
    
    incomming_length(mono!(g.ret) + 1), pp.print(y, z) <--
        let (x, z) = do_length, graph.edge(y, z),
        do g : LongestPath {} (graph.clone(), pp.clone(), (x, *y));
    
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
    let mut compute_length = LongestPath::default();
    compute_length.run(g.clone(), pp.clone(), (1, 5));

    println!("{:?}", &(compute_length.ret));
    println!("{:?}", &(pp.borrow().print));
}
