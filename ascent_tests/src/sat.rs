

// Simulate a 3-SAT solver using Datalog
use ascent::ascent;

// Trace is an assignment of variables to true or false
#[derive(Default, Debug)]
struct Trace {
    trace: Vec<bool>,
}

// This ascent program used to do unit propagation
ascent! {
    struct UnitPropagation;

    relation clause(usize, i32, i32, i32);
    relation var(i32);
    
    // clause A and B share literal x
    relation connected_clause(usize, usize, i32);

    
}

