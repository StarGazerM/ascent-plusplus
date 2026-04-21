// Polonius Borrow Checking — DD backend, incremental mode.
// Uses negation (antijoin), which DD batch mode rejects — so we use
// `mode = "incremental"`.
// Input: comma-separated .csv files OR tab-separated .facts files.

use ascent::ascent_par;
use bench_loader::{load_1, load_2, load_3};
use std::env;
use std::time::Instant;

type V = i32;

ascent_par! {
    #![backend(dd, mode = "incremental")]
    pub struct AscentProgram;
    // EDB (input relations)
    relation subset_base(V, V, V);
    relation cfg_edge(V, V);
    relation loan_issued_at(V, V, V);
    relation universal_region(V);
    relation var_used_at(V, V);
    relation loan_killed_at(V, V);
    relation known_placeholder_subset(V, V);
    relation var_dropped_at(V, V);
    relation drop_of_var_derefs_origin(V, V);
    relation var_defined_at(V, V);
    relation child_path(V, V);
    relation path_moved_at_base(V, V);
    relation path_assigned_at_base(V, V);
    relation path_accessed_at_base(V, V);
    relation path_is_var(V, V);
    relation loan_invalidated_at(V, V);
    relation use_of_var_derefs_origin(V, V);

    // IDB (computed relations)
    relation subset(V, V, V);
    relation origin_live_on_entry(V, V);
    relation origin_contains_loan_on_entry(V, V, V);
    relation loan_live_at(V, V);
    relation errors(V, V);
    relation placeholder_origin(V);
    relation subset_error(V, V, V);
    relation cfg_node(V);
    relation var_live_on_entry(V, V);
    relation var_drop_live_on_entry(V, V);
    relation var_maybe_partly_initialized_on_exit(V, V);
    relation var_maybe_partly_initialized_on_entry(V, V);
    relation ancestor_path(V, V);
    relation path_moved_at(V, V);
    relation path_assigned_at(V, V);
    relation path_accessed_at(V, V);
    relation path_begins_with_var(V, V);
    relation path_maybe_initialized_on_exit(V, V);
    relation path_maybe_uninitialized_on_exit(V, V);
    relation move_error(V, V);

    // Basic rules
    subset(o1, o2, p) <-- subset_base(o1, o2, p);
    origin_contains_loan_on_entry(origin, loan, point) <-- loan_issued_at(loan, origin, point);
    placeholder_origin(origin) <-- universal_region(origin);

    known_placeholder_subset(x, z) <--
        known_placeholder_subset(x, y),
        known_placeholder_subset(y, z);

    subset(o1, o3, p) <--
        subset(o1, o2, p),
        subset_base(o2, o3, p),
        if o1 != o3;

    subset(o1, o2, p2) <--
        subset(o1, o2, p1),
        cfg_edge(p1, p2),
        origin_live_on_entry(o1, p2),
        origin_live_on_entry(o2, p2);

    origin_contains_loan_on_entry(o2, loan, point) <--
        origin_contains_loan_on_entry(o1, loan, point),
        subset(o1, o2, point);

    origin_contains_loan_on_entry(origin, loan, p2) <--
        origin_contains_loan_on_entry(origin, loan, p1),
        cfg_edge(p1, p2),
        !loan_killed_at(loan, p1),
        origin_live_on_entry(origin, p2);

    loan_live_at(loan, point) <--
        origin_contains_loan_on_entry(origin, loan, point),
        origin_live_on_entry(origin, point);

    errors(loan, point) <--
        loan_invalidated_at(loan, point),
        loan_live_at(loan, point);

    subset_error(o1, o2, point) <--
        subset(o1, o2, point),
        placeholder_origin(o1),
        placeholder_origin(o2),
        !known_placeholder_subset(o1, o2),
        if o1 != o2;

    cfg_node(p1) <-- cfg_edge(p1, _p2);
    cfg_node(p2) <-- cfg_edge(_p1, p2);

    origin_live_on_entry(origin, point) <--
        cfg_node(point),
        universal_region(origin);

    // Liveness
    var_live_on_entry(var, point) <-- var_used_at(var, point);

    var_maybe_partly_initialized_on_entry(var, p2) <--
        var_maybe_partly_initialized_on_exit(var, p1),
        cfg_edge(p1, p2);

    var_drop_live_on_entry(var, point) <--
        var_dropped_at(var, point),
        var_maybe_partly_initialized_on_entry(var, point);

    origin_live_on_entry(origin, point) <--
        var_drop_live_on_entry(var, point),
        drop_of_var_derefs_origin(var, origin);

    origin_live_on_entry(origin, point) <--
        var_live_on_entry(var, point),
        use_of_var_derefs_origin(var, origin);

    var_live_on_entry(var, p1) <--
        var_live_on_entry(var, p2),
        cfg_edge(p1, p2),
        !var_defined_at(var, p1);

    var_drop_live_on_entry(var, src) <--
        var_drop_live_on_entry(var, tgt),
        cfg_edge(src, tgt),
        !var_defined_at(var, src),
        var_maybe_partly_initialized_on_exit(var, src);

    // Initialization logic
    ancestor_path(x, y) <-- child_path(x, y);
    path_moved_at(x, y) <-- path_moved_at_base(x, y);
    path_assigned_at(x, y) <-- path_assigned_at_base(x, y);
    path_accessed_at(x, y) <-- path_accessed_at_base(x, y);
    path_begins_with_var(x, var) <-- path_is_var(x, var);

    ancestor_path(gp, child) <--
        ancestor_path(parent, child),
        child_path(parent, gp);

    path_moved_at(child, point) <--
        path_moved_at(parent, point),
        ancestor_path(parent, child);

    path_assigned_at(child, point) <--
        path_assigned_at(parent, point),
        ancestor_path(parent, child);

    path_accessed_at(child, point) <--
        path_accessed_at(parent, point),
        ancestor_path(parent, child);

    path_begins_with_var(child, var) <--
        path_begins_with_var(parent, var),
        ancestor_path(parent, child);

    path_maybe_initialized_on_exit(path, point) <--
        path_assigned_at(path, point);

    path_maybe_uninitialized_on_exit(path, point) <--
        path_moved_at(path, point);

    path_maybe_initialized_on_exit(path, p2) <--
        path_maybe_initialized_on_exit(path, p1),
        cfg_edge(p1, p2),
        !path_moved_at(path, p2);

    path_maybe_uninitialized_on_exit(path, p2) <--
        path_maybe_uninitialized_on_exit(path, p1),
        cfg_edge(p1, p2),
        !path_assigned_at(path, p2);

    var_maybe_partly_initialized_on_exit(var, point) <--
        path_maybe_initialized_on_exit(path, point),
        path_begins_with_var(path, var);

    move_error(path, tgt) <--
        path_maybe_uninitialized_on_exit(path, src),
        cfg_edge(src, tgt);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <input_dir>", args[0]);
        std::process::exit(1);
    }
    let dir = &args[1];

    eprintln!("Loading data from {}...", dir);
    let t0 = Instant::now();
    let mut prog = AscentProgram::default();

    macro_rules! load_into {
        ($rel:expr, $data:expr) => { for t in $data { $rel.push(t); } }
    }

    // Loader auto-falls-back to .csv / .facts when extension omitted.
    load_into!(prog.subset_base, load_3(dir, "subset_base"));
    load_into!(prog.cfg_edge, load_2(dir, "cfg_edge"));
    load_into!(prog.loan_issued_at, load_3(dir, "loan_issued_at"));
    load_into!(prog.universal_region, load_1(dir, "universal_region"));
    load_into!(prog.var_used_at, load_2(dir, "var_used_at"));
    load_into!(prog.loan_killed_at, load_2(dir, "loan_killed_at"));
    load_into!(prog.known_placeholder_subset, load_2(dir, "known_placeholder_subset"));
    load_into!(prog.var_dropped_at, load_2(dir, "var_dropped_at"));
    load_into!(prog.drop_of_var_derefs_origin, load_2(dir, "drop_of_var_derefs_origin"));
    load_into!(prog.var_defined_at, load_2(dir, "var_defined_at"));
    load_into!(prog.child_path, load_2(dir, "child_path"));
    load_into!(prog.path_moved_at_base, load_2(dir, "path_moved_at_base"));
    load_into!(prog.path_assigned_at_base, load_2(dir, "path_assigned_at_base"));
    load_into!(prog.path_accessed_at_base, load_2(dir, "path_accessed_at_base"));
    load_into!(prog.path_is_var, load_2(dir, "path_is_var"));
    load_into!(prog.loan_invalidated_at, load_2(dir, "loan_invalidated_at"));
    load_into!(prog.use_of_var_derefs_origin, load_2(dir, "use_of_var_derefs_origin"));

    let load_time = t0.elapsed();
    eprintln!("Loaded in {:?}", load_time);

    let t1 = Instant::now();
    prog.run();
    let run_time = t1.elapsed();

    eprintln!("subset: {}", prog.subset.len());
    eprintln!("origin_live_on_entry: {}", prog.origin_live_on_entry.len());
    eprintln!("origin_contains_loan_on_entry: {}", prog.origin_contains_loan_on_entry.len());
    eprintln!("loan_live_at: {}", prog.loan_live_at.len());
    eprintln!("errors: {}", prog.errors.len());
    eprintln!("Execution: {:?}", run_time);
    eprintln!("Total: {:?}", load_time + run_time);
}
