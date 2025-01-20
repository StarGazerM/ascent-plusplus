use ascent::{ascent, internal::RelIndexRead};
use ascent_byods_rels::phantom;

ascent! {
    struct TCIncremental;
    #[ds(phantom::rel)]
    relation edge_incremental(i32, i32);
    #[ds(phantom::rel)]
    relation path_incremental(i32, i32);
    relation edge(i32, i32);
    relation path(i32, i32);
    relation outside();

    edge(x, y) <-- edge_incremental(x, y);
    path(x, y) <-- path_incremental(x, y);
    edge_incremental(Default::default(), Default::default()) <-- outside();
    path_incremental(Default::default(), Default::default()) <-- outside();

    path(x, y) <-- delta edge(x, y);
    path(x, z) <-- delta path(x, y), edge(y, z);
    path(x, z) <-- delta edge(y, z), path(x, y);

    path_incremental(x, y) <-- path(x, y);
    edge_incremental(x, y) <-- edge(x, y);
    outside() <-- edge(x, y);
    outside() <-- path(x, y);
}
#[test]
fn run_tc_incremental() {
    let mut tc = TCIncremental::default();
    let data = vec![(1, 2), (2, 3)];
    tc.runtime_total.edge_incremental_indices_none.0 = data.into_iter().map(|p| ((), p)).collect();
    let l = tc.runtime_total.edge_incremental_indices_none.0.len();
    tc.run_with_init_flag(false);

    println!("path_incremental: {:?}", &(tc.path));
}
