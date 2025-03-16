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

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc;
use std::thread;

fn check_exit_tc(recv_signal: &Option<(i32, i32)>, exit: &Rc<RefCell<bool>>) {
    println!("Received {:?}", recv_signal);
    if recv_signal.is_none() {
        *exit.borrow_mut() = true;
    }
}

ascent! {
    struct StreamTC;
    extern arguement &mpsc::Receiver<Option<(i32, i32)>> edge_source;
    extern arguement Rc<RefCell<bool>> exit;
    relation edge(i32, i32);
    relation path(i32, i32);
    lattice outside(usize);
    outside(0);
    path(x, y) <-- edge(x, y);
    path(x, z) <-- delta path(x, y), edge(y, z);
    path(x, z) <-- delta edge(x, y), path(y, z);
    outside(cnt+1) <-- delta outside(cnt), path(_, _), if !exit.borrow().clone();
    edge(src_x, src_y) <--
        let res = edge_source.try_recv(),
        if res.is_ok(), let recv_data = res.unwrap(),
        let _ = check_exit_tc(&recv_data, &exit),
        if let Some((src_x, src_y)) = recv_data,
        outside(x);
}
#[test]
fn run_stream_tc() {
    let (tx, rx): (mpsc::Sender<Option<(i32, i32)>>, mpsc::Receiver<Option<(i32, i32)>>) = mpsc::channel();
    let tc_thread = thread::spawn(move || {
        let mut tc = StreamTC::default();
        println!("TC thread started");
        let fixed = Rc::new(RefCell::new(false));
        loop {
            tc.run(&rx, fixed.clone());
            if fixed.borrow().clone() {
                println!("Fixed point reached!");
                break;
            }
        }
        tc
    });
    let edge_thread = thread::spawn(move || {
        let data = vec![(1, 2), (2, 3), (3, 4), (4, 5), (1, 5)];
        println!("Edge thread started");
        for d in data {
            let _ = tx.send(Some(d));
            println!("Sent {:?}", d);
        }
        tx.send(None).unwrap();
    });
    edge_thread.join().unwrap();
    let tc = tc_thread.join().unwrap();
    println!("path: {:?}", &(tc.path));
}
