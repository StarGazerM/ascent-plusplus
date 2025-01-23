use ascent::aggregators::count;
use ascent::ascent;
use ascent::lattice;
use ascent::lattice::set::Set;

use std::{
   cell::RefCell,
   collections::BTreeSet,
   hash::{Hash, Hasher},
   rc::Rc,
};

// const *

#[derive(Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Ptr<T>(Rc<RefCell<T>>);

impl<T> Hash for Ptr<T> {
   fn hash<H: Hasher>(&self, state: &mut H) {
      // use the hash value of the pointer
      self.0.as_ptr().hash(state);
   }
}

#[derive(Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct GraphNode {
   id: u32,
   next: BTreeSet<Ptr<GraphNode>>,
   //    prev: BTreeSet<Ptr<GraphNode>>,
}

impl Hash for GraphNode {
   fn hash<H: Hasher>(&self, state: &mut H) { self.id.hash(state); }
}

#[derive(Default, Clone, Hash, PartialEq, Eq)]
struct RelationId {
   id: u32,
   kind: &'static str,
}

macro_rules! ptr {
   ($x:expr) => {
      Ptr(Rc::new(RefCell::new($x)))
   };
}

ascent! {
    struct GraphLift;

    // input relations
    relation edge(u32, u32);

    relation node(u32);
    relation end_node(u32);
    relation start_node(u32);
    lattice lifted_graph_head(Set<Ptr<GraphNode>>);
    relation outages(u32, usize);

    node(x) <-- edge(x, _);
    node(y) <-- edge(_, y);

    start_node(x) <-- node(x), !edge(_, x);
    end_node(x) <-- node(x), !edge(x, _);
    outages(x, n) <-- node(x), agg n = count() in edge(x, _);

    // relation graph_node(u32, Ptr<GraphNode>);

    relation do_lift(u32);
    lattice lift(u32, Set<Ptr<GraphNode>>);

    start_node(x) <-- do_lift(x);

    do_lift(y) <-- do_lift(x), edge(x, y);
    lift(x, Set::singleton(new_node)) <--
        do_lift(x),
        edge(x, y), end_node(y),
        let new_node = ptr!(GraphNode { id: *x, ..Default::default() });

    lift(x, Set::singleton(new_node)) <--
        do_lift(x),
        edge(x, y), lift(y, y_nodes), outages(y, n),
        if &y_nodes.0.len() == n,
        let new_node = ptr!(GraphNode { id: *x, next: y_nodes.0.clone() });

    lifted_graph_head(Set::singleton(new_node)) <-- start_node(x), lift(x, next),
        let new_node = ptr!(GraphNode { id: *x, next: next.0.clone() });
}

#[test]
fn test_graph_lift() {
   let mut prog = GraphLift::default();

   prog.edge = vec![(1, 2), (2, 3), (3, 4), (4, 5)].into_iter().collect();
   prog.run();

   let lifted_graph = GraphNode { id: std::u32::MAX, next: prog.lifted_graph_head[0].0 .0.clone() };
    // println!("{:?}", &lifted_graph);
}
