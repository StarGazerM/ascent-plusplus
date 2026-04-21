//! Manual CSPA using FlowLog-style dataflow shape. Purpose: isolate whether
//! our perf cliff is in the codegen or in the runtime wiring.
//!
//! - Uses `timely::execute_from_args` + `worker.dataflow::<u32, _, _>(...)` directly.
//! - Dataflow body is copy-adapted from `/tmp/flbench_inc/.flowlog_cspa_inc_v2.build/src/main.rs`
//!   (scope.iterative + 3 Variables + 7 join_cores + 5 thresholds).
//! - Inputs: read full Assign.csv / Dereference.csv once on worker 0 via
//!   `InputSession::update(_, 1)`, matching our current Sealer pattern.
//!
//! Invoke:
//!   ASCENT_DD_WORKERS=24 ./cspa_manual <dataset_dir>
//!
//! Compares directly against `flowlog_cspa_inc_v2` — same dataflow shape,
//! same runtime primitives. Any gap left means our ingest is the issue.

use bench_loader::load_2;
use std::env;
use std::time::Instant;

use ascent::dd::Variable;
use ascent::dd::differential_dataflow::input::InputSession;
use ascent::dd::timely;
use ascent::dd::timely::dataflow::ProbeHandle;
use ascent::dd::timely::dataflow::Scope;
use ascent::dd::timely::order::Product;

type Diff = i32;
type Iter = u16;

fn workers_from_env() -> usize {
    std::env::var("ASCENT_DD_WORKERS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <dir>", args[0]);
        std::process::exit(1);
    }
    let dir = args[1].clone();

    // Load once on the main thread, then clone Arc into workers. FlowLog
    // reads byte-ranges in parallel per worker; we'll match that later if
    // this test shows codegen is the bug.
    let t0 = Instant::now();
    let assign_data: Vec<(i32, i32)> = load_2(&dir, "Assign.csv");
    let deref_data: Vec<(i32, i32)> = load_2(&dir, "Dereference.csv");
    let load_time = t0.elapsed();
    eprintln!(
        "Loaded |assign|={} |deref|={} in {:?}",
        assign_data.len(), deref_data.len(), load_time,
    );

    let workers = workers_from_env();
    let config = if workers == 1 {
        timely::Config::thread()
    } else {
        timely::Config::process(workers)
    };

    let assign_arc = std::sync::Arc::new(assign_data);
    let deref_arc = std::sync::Arc::new(deref_data);

    use ascent::dd::Sink;
    let ma_sink: Sink<(i32, i32)> = Sink::new();
    let va_sink: Sink<(i32, i32)> = Sink::new();
    let vf_sink: Sink<(i32, i32)> = Sink::new();
    let ma_sink_for_closure = ma_sink.clone();
    let va_sink_for_closure = va_sink.clone();
    let vf_sink_for_closure = vf_sink.clone();

    let t1 = Instant::now();
    timely::execute::execute(config, move |worker| {
        let index = worker.index();
        let peers = worker.peers();
        let assign_arc = assign_arc.clone();
        let deref_arc = deref_arc.clone();
        let ma_sink_clone = ma_sink_for_closure.clone();
        let va_sink_clone = va_sink_for_closure.clone();
        let vf_sink_clone = vf_sink_for_closure.clone();

        let (mut hassign, mut hderef, probe) = worker.dataflow::<u32, _, _>(|scope| {
            // Inputs.
            let mut hassign: InputSession<u32, (i32, i32), Diff> = InputSession::new();
            let mut hderef: InputSession<u32, (i32, i32), Diff> = InputSession::new();
            let assign = hassign.to_collection(scope);
            let deref = hderef.to_collection(scope);

            // Non-recursive threshold (FlowLog line 65, 69).
            let assign = assign.threshold(|_, w: &i32| if *w > 0 { 1 } else { 0 });
            let deref = deref.threshold(|_, w: &i32| if *w > 0 { 1 } else { 0 });

            // Non-recursive seed rules (FlowLog line 70-93).
            // value_flow(y,x) <- assign(y,x)  ↦  identity copy (NOT a swap;
            // both head and body have args in the same order).
            let t_swap = assign.clone();
            // value_flow(x,x) <- assign(x,_)  ↦  first col duplicated
            let t_self_first = assign.clone().map(|(x, _y): (i32, i32)| (x, x));
            // value_flow(x,x) <- assign(_,x)  ↦  second col duplicated
            let t_self_second = assign.clone().map(|(_x, y): (i32, i32)| (y, y));
            let value_flow_seed = t_swap
                .concat(t_self_first.clone())
                .concat(t_self_second.clone())
                .threshold(|_, w: &i32| if *w > 0 { 1 } else { 0 });

            // memory_alias(x,x) <- assign(_,x) | <- assign(x,_)
            let memory_alias_seed = t_self_first
                .concat(t_self_second.clone())
                .threshold(|_, w: &i32| if *w > 0 { 1 } else { 0 });

            // Pre-arrange EDBs for joins (FlowLog line 94-117).
            let assign_by_1 = assign.clone().map(|(x, y)| (y, x)).arrange_by_key();
            let deref_by_0 = deref.clone().map(|(x, y)| (x, y)).arrange_by_key();
            let deref_by_0_again = deref.clone().map(|(x, y)| (x, y)).arrange_by_key();

            // Iterative recursion (FlowLog line 118+).
            let (memory_alias, value_alias, value_flow) = scope.iterative::<Iter, _, _>(|inner| {
                let in_memory_alias = memory_alias_seed.enter(inner);
                let in_value_flow = value_flow_seed.enter(inner);
                let in_assign_by_1 = assign_by_1.enter(inner);
                let in_deref_by_0 = deref_by_0.enter(inner);
                let in_deref_by_0_again = deref_by_0_again.enter(inner);

                let (ma_var, ma): (Variable<_, (i32, i32), Diff>, _) =
                    Variable::new(inner, Product::new(Default::default(), 1));
                let (va_var, va): (Variable<_, (i32, i32), Diff>, _) =
                    Variable::new(inner, Product::new(Default::default(), 1));
                let (vf_var, vf): (Variable<_, (i32, i32), Diff>, _) =
                    Variable::new(inner, Product::new(Default::default(), 1));

                let vf_by_1 = vf.clone().map(|(x, y)| (y, x)).arrange_by_key();
                let vf_by_0 = vf.clone().map(|(x, y)| (x, y)).arrange_by_key();
                // Clone the arrangement a few times so downstream uses each own a handle.
                let vf_by_0_a = vf_by_0.clone();
                let vf_by_0_b = vf_by_0.clone();
                let vf_by_0_c = vf_by_0.clone();
                let ma_by_0 = ma.clone().map(|(x, y)| (x, y)).arrange_by_key();
                let va_by_0 = va.clone().map(|(x, y)| (x, y)).arrange_by_key();

                // value_flow(x,y) <- value_flow(x,z), value_flow(z,y)
                let vf_rec = vf_by_1.join_core(vf_by_0_a, |_z, x, y| Some((*x, *y)));
                // value_flow(x,y) <- assign(x,z), memory_alias(z,y)
                let vf_via_ma = in_assign_by_1.join_core(ma_by_0.clone(), |_z, x, y| Some((*x, *y)));

                // memory_alias(x,w) <- deref(y,x), value_alias(y,z), deref(z,w)
                let ma_step1 = in_deref_by_0.join_core(va_by_0, |_y, x, z| Some((*z, *x)));
                let ma_step1_arr = ma_step1.map(|(z, x)| (z, x)).arrange_by_key();
                let ma_rec = ma_step1_arr.join_core(in_deref_by_0_again, |_z, x, w| Some((*x, *w)));

                // value_alias(x,y) <- value_flow(z,x), value_flow(z,y)
                let va_rec1 = vf_by_0.clone().join_core(vf_by_0_b, |_z, x, y| Some((*x, *y)));
                // value_alias(x,y) <- value_flow(z,x), memory_alias(z,w), value_flow(w,y)
                let va_step1 = vf_by_0_c.join_core(ma_by_0, |_z, x, w| Some((*w, *x)));
                let va_step1_arr = va_step1.map(|(w, x)| (w, x)).arrange_by_key();
                let va_rec2 = va_step1_arr.join_core(vf_by_0.clone(), |_w, x, y| Some((*x, *y)));

                let next_vf = vf_rec
                    .concat(vf_via_ma.clone())
                    .concat(in_value_flow.clone())
                    .threshold(|_, w: &i32| if *w > 0 { 1 } else { 0 });
                let next_va = va_rec1
                    .concat(va_rec2.clone())
                    .threshold(|_, w: &i32| if *w > 0 { 1 } else { 0 });
                let next_ma = ma_rec
                    .concat(in_memory_alias.clone())
                    .threshold(|_, w: &i32| if *w > 0 { 1 } else { 0 });

                vf_var.set(next_vf.clone());
                va_var.set(next_va.clone());
                ma_var.set(next_ma.clone());

                (next_ma.leave(), next_va.leave(), next_vf.leave())
            });

            use ascent::dd::Sink;
            use ascent::dd::timely::dataflow::operators::probe::Probe;
            let mut probe = ProbeHandle::new();
            let ma_sink: Sink<(i32, i32)> = ma_sink_clone.clone();
            let va_sink: Sink<(i32, i32)> = va_sink_clone.clone();
            let vf_sink: Sink<(i32, i32)> = vf_sink_clone.clone();
            ma_sink.attach(&memory_alias, &mut probe);
            va_sink.attach(&value_alias, &mut probe);
            vf_sink.attach(&value_flow, &mut probe);

            (hassign, hderef, probe)
        });

        let t_in = std::time::Instant::now();
        for &(x, y) in assign_arc.iter() {
            if ((x as i64).rem_euclid(peers as i64) as usize) == index {
                hassign.update((x, y), 1);
            }
        }
        for &(x, y) in deref_arc.iter() {
            if ((x as i64).rem_euclid(peers as i64) as usize) == index {
                hderef.update((x, y), 1);
            }
        }
        hassign.advance_to(1);
        hassign.flush();
        hderef.advance_to(1);
        hderef.flush();
        let t_ingest = t_in.elapsed();

        let t_step = std::time::Instant::now();
        while probe.less_than(&1) {
            worker.step();
        }
        let t_dataflow = t_step.elapsed();
        if index == 0 {
            use std::sync::atomic::Ordering;
            eprintln!("[w0] ingest={:?} step_loop={:?}", t_ingest, t_dataflow);
        }
    })
    .unwrap()
    .join();
    eprintln!("run+commit took {:?}", t1.elapsed());
    let mut ma_vec = ma_sink.into_vec();
    let mut va_vec = va_sink.into_vec();
    let mut vf_vec = vf_sink.into_vec();
    ma_vec.sort(); ma_vec.dedup();
    va_vec.sort(); va_vec.dedup();
    vf_vec.sort(); vf_vec.dedup();
    eprintln!(
        "distinct: value_flow={} memory_alias={} value_alias={}",
        vf_vec.len(), ma_vec.len(), va_vec.len(),
    );
}
