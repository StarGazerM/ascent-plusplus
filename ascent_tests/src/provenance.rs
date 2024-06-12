
// compute different style of provenance in ascent
#![allow(warnings)]

use ascent::*;
use std::rc::Rc;
use ascent::lattice::set::*;

// why provenance is used trace all possible sources of a output value
// in some literature also called proof tree

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
struct Edge(i32, i32);

ascent! {
    struct TCWhyExpensive;

    relation edge(i32, i32);
    relation path(i32, i32);

    // mark each edge with a unique id, here I use a Edge struct
    relation edge_prov(i32, i32, Rc<Edge>);
    // each path will now pair with Rc<Edge> to mark the edge contributing to the path
    relation path_prov(i32, i32, Rc<Edge>);

    edge_prov(x, y, e) <-- edge(x, y), let e = Rc::new(Edge(*x, *y));

    path_prov(x, y, e) <-- edge_prov(x, y, e);

    path_prov(x, y, e1),
    path_prov(x, y, e2) <--
        path_prov(x, z, e1),
        edge_prov(z, y, e2);

    path(x, y) <-- path_prov(x, y, _);
}


#[test]
fn test_why_expensive() {
    let mut tc = TCWhyExpensive::default();
    tc.edge = vec![
        (1, 2),
        (2, 3),
        (3, 4),
        (1, 4),
    ];

    tc.run();

    println!("path_prov: {:?}", tc.path_prov);
    println!("path_prov size = {}", tc.path_prov.len());
    assert_eq!(tc.path_prov.len(), 11);
}

// above version is flatten version
// a more memory efficient version of the above
ascent! {
    struct TCWhyLattice;

    relation edge(i32, i32);

    relation path(i32, i32);
    relation edge_prov(i32, i32, Rc<Edge>);

    lattice path_prov(i32, i32, Set<Rc<Edge>>);

    edge_prov(x, y, e) <-- edge(x, y), let e = Rc::new(Edge(*x, *y));

    path(x, y), path_prov(x, y, Set::singleton(e.clone())) <--
        edge_prov(x, y, e);

    path(x, y),
    path_prov(x, y, Set::singleton(e2.clone())),
    path_prov(x, y, e_prev) <--
        path_prov(x, z, e_prev),
        edge_prov(z, y, e2);
}


#[test]
fn test_why_lattice() {
    let mut tc = TCWhyLattice::default();
    tc.edge = vec![
        (1, 2),
        (2, 3),
        (3, 4),
        (1, 4),
    ];

    tc.run();

    println!("path_prov: {:?}", tc.path_prov);
    println!("path_prov size = {}", tc.path_prov.len());
    // assert_eq!(tc.path_prov.len(), 11);
}

// can we get slog style int autoinc id?
#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
struct StructId(&'static str, usize);

ascent_par!{
    struct WhySlog;

    relation edge_raw(i32, i32);
    relation edge(i32, i32);
    relation edge_id(i32, i32, usize);
    relation path(i32, StructId);
    relation path_id(i32, StructId, usize);

    let eid = !edge(x, y), edge_id(x, y, eid) <--
        edge_raw(x, y);
    
    let new_id = !path(x, nest_id.clone()),
    path_id(x, nest_id, new_id) <--
        edge_id(x, y, eid),
        let nest_id = StructId("edge", *eid);
    
    let new_id = !path(x, nest_id.clone()),
    path_id(x, nest_id, new_id) <--
        edge(x, y),
        path_id(y, _, pid),
        let nest_id = StructId("edge", *pid);

}

#[test]
fn test_why_slog() {
    let mut tc = WhySlog::default();
    tc.edge_raw = ascent::boxcar::vec![
        (1, 2),
        (2, 3),
        (3, 4),
        (1, 4),
    ];

    tc.run();

    println!("path_id: {:?}", tc.path_id);
    println!("path_id size = {}", tc.path_id.len());
    // assert_eq!(tc.path_prov.len(), 11);
}

// where provenance


ascent_par! {
    struct TCWhere;

    relation edge_raw(i32, i32);
    relation edge(i32, i32);
    relation edge_id(i32, i32, usize);
    relation path(i32, i32);
    relation path_id(i32, i32, usize);
    relation provenance(StructId, StructId);

    // let <id> = ... will allow you to extract the length of the relation when
    // head clasue tuple is created. `!` operator will enforce all head clause
    // to be generated after this clause to be generated after this clause
    // **succeed** (If a tuple get deduplicated, it means failed).
    // by combine `let` and `!` in head clauses you can get the slog style autoinc id
    let eid = !edge(x, y), edge_id(x, y, eid) <--
        edge_raw(x, y);

    let new_id = !path(x, y),
    path_id(x, y, new_id),
    provenance(StructId("path", new_id), StructId("edge", *eid)) <--
        edge_id(x, y, eid);
    
    let new_id = !path(x, z),
    path_id(x, z, new_id),
    // provenance(StructId("path", new_id), StructId("path", *pid)),
    provenance(StructId("path", new_id), StructId("edge", *eid)) <--
        edge_id(x, y, eid),
        path_id(y, z, pid);
}

#[test]
fn test_where() {
    let mut tc = TCWhere::default();
    tc.edge_raw = ascent::boxcar::vec![
        (1, 2),
        (2, 3),
        (3, 4),
        (1, 4),
    ];

    tc.run();
    println!("provenance size = {}", tc.provenance.len());
    println!("path_id size = {}", tc.path_id.len());
    println!("edge_id size = {}", tc.edge_id.len());
    println!("provenance: {:?}", tc.provenance);
}
