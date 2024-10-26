// compute different style of provenance in ascent
#![allow(warnings)]

use ascent::lattice::set::*;
use ascent::ascent;
use ascent::internal::{Freezable, RelIndexReadAll};
use std::path::Path;
use std::rc::Rc;

use crate::{ascent_m_par, ascent_run_m_par};

// why provenance is used trace all possible sources of a output value
// in some literature also called proof tree

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
struct Edge(i32, i32);

#[test]
fn test_tc() {
    let mut tc = ascent_run_m_par! {
        relation edge(i32, i32);
        relation path(i32, i32);

        edge(1, 2);
        edge(2, 3);
        edge(3, 4);
        edge(1, 4);

        path(x, y) <-- edge(x, y);
        path(x, y) <-- path(x, z), edge(z, y);
    };

    // let path_with_cnt = tc.path_indices_0_1.iter().collect::<Vec<_>>();
    tc.path_indices_0_1.freeze();
    let path_with_cnt = tc.path_indices_0_1.
        iter_all().map(|(k, v)|{
            let all_v = v.collect::<Vec<_>>();
            (k, all_v[0].clone())
        }).
        collect::<Vec<_>>();
    println!("path: {:?}", path_with_cnt);
}

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
   tc.edge = vec![(1, 2), (2, 3), (3, 4), (1, 4)];

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
   tc.edge = vec![(1, 2), (2, 3), (3, 4), (1, 4)];

   tc.run();

   println!("path_prov: {:?}", tc.path_prov);
   println!("path_prov size = {}", tc.path_prov.len());
   // assert_eq!(tc.path_prov.len(), 11);
}

// can we get slog style int autoinc id?
#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
struct Tag(&'static str, usize);

ascent_m_par! {
    struct WhySlog;

    macro exists($rel_name: ident, $id: ident, $args: va_list) {
        let $id = !$rel_name($args), $($rel_name)_id($args, $id)
    }
    macro id_rel($rel_name: ident, $args: va_list) {
        $($rel_name)_id($args)
    }
    macro declare_id_rel($rel_name: ident, $args: va_list) {
        relation $rel_name($args);
        relation $($rel_name)_id($args, usize);
    }

    relation edge_raw(i32, i32);
    @declare_id_rel!(edge, i32, i32);
    @declare_id_rel!(path, i32, Tag);

    exists!(edge, id, x, y) <-- edge_raw(x, y);

    exists!(path, new_id, x, nest_id.clone()) <--
        id_rel!(edge, x, y, eid),
        let nest_id = Tag("edge", *eid);

    exists!(path, new_id, x, nest_id.clone()) <--
        edge(x, y),
        id_rel!(path, y, _, pid),
        let nest_id = Tag("edge", *pid);
}

#[test]
fn test_why_slog() {
   let mut tc = WhySlog::default();
   tc.edge_raw = ascent::boxcar::vec![(1, 2), (2, 3), (3, 4), (1, 4),];

   tc.run();

   println!("path_id: {:?}", tc.path_id);
   println!("path_id size = {}", tc.path_id.len());
   println!("edge_id size = {}", tc.edge_id.len());
   println!("edge_id: {:?}", tc.edge_id);
   // assert_eq!(tc.path_prov.len(), 11);
}

// where provenance

ascent_m_par! {
    struct TCWhere;

    relation edge_raw(i32, i32);
    relation edge(i32, i32);
    relation edge_id(i32, i32, usize);
    relation path(i32, i32);
    relation path_id(i32, i32, usize);
    relation provenance(Tag, Tag);

    let eid = !edge(x, y),
    edge_id(x, y, eid) <--
        edge_raw(x, y);

    let new_id = !path(x, y),
    path_id(x, y, new_id),
    provenance(Tag("path", new_id), Tag("edge", *eid)) <--
        edge_id(x, y, eid);

    let new_id = !path(x, z),
    path_id(x, z, new_id),
    // provenance(StructId("path", new_id), StructId("path", *pid)),
    provenance(Tag("path", new_id), Tag("edge", *eid)) <--
        edge_id(x, y, eid),
        path_id(y, z, pid);
}

#[test]
fn test_where() {
   let mut tc = TCWhere::default();
   tc.edge_raw = ascent::boxcar::vec![(1, 2), (2, 3), (3, 4), (1, 4),];

   tc.run();
   println!("provenance size = {}", tc.provenance.len());
   println!("path_id size = {}", tc.path_id.len());
   println!("edge_id size = {}", tc.edge_id.len());
   println!("provenance: {:?}", tc.provenance);
}

ascent! {
    struct Length;
    relation edge_raw(i32, i32);
    relation ID edge(i32, i32);
    relation ID path(i32, Tag);
    relation input(usize);
    // normal TC
    >?id.edge(x, y) <-- edge_raw(x, y);
    >?new_id.path(x, nest_id.clone()) <--
        edge(x, y).eid,
        let nest_id = Tag("edge", *eid);
    >?new_id.path(x, nest_id.clone()) <--
        edge(x, y),
        path(y, _).pid,
        let nest_id = Tag("path", *pid);

    function path_length(Tag) -> usize;
    // length of a path

    %path_length(Tag("path", *pid)) -> ? <--
        input(pid);

    %path_length(?Tag("edge", eid)) -> ret_val <--
        let ret_val = 1;

    %path_length(?Tag("path", pid)) -> ret_val <-- 
        path(x, res).*pid,
        %path_length(res) -> rest_length,
        let ret_val = rest_length + 1;
}

#[test]
fn test_length() {
   let mut tc = Length::default();
   tc.edge_raw = vec![(1, 2), (2, 3), (3, 4), (1, 4)];
   tc.input = vec![(6,)];

   tc.run();
   println!("edge id : {:?}", tc.edge_id);
   println!("path id : {:?}", tc.path_id);
   println!("path_length_do_id : {:?}", tc.path_length_do_id);
   println!("path length : {:?}", tc.path_length);
}
