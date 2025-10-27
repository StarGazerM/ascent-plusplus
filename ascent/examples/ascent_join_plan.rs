//! Example of different join plans in Ascent

use ascent::ascent;

ascent! {
    struct JoinBinaryPlan;

    relation foo(i32, i32);
    relation bar(i32, i32);
    relation baz(i32, i32);
    relation foobar(i32, i32, i32);

    // Triangle join
    foobar(x, y, z) <-- foo(x, z), bar(x, y), baz(y, z);
}

ascent! {
    struct SelectJoinPlan;

    relation foo(i32, i32);
    relation bar(i32, i32);
    relation baz(i32, i32);
    relation foobar(i32, i32, i32);

    // Triangle join ∩
    foobar(x, y, z) <--
        foo(x, _), bar(x, _),
        bar(x, y), baz(y, _),
        foo(x, z), baz(y, z);
    // foobar(x, y, z) <--
    //     foo(x, _), foo(x, z),
    //     bar(x, _), bar(x, y),
    //     baz(y, _), baz(y, z);
}

fn main() {
    let mut prog = JoinBinaryPlan::default();

    prog.foo = vec![(1, 2), (1, 3), (2, 3)];
    prog.bar = vec![(1, 2), (1, 3), (2, 3)];
    prog.baz = vec![(1, 2), (1, 3), (2, 3)];

    prog.run();

    println!("foobar: {:?}", prog.foobar);

    let mut prog = SelectJoinPlan::default();

    prog.foo = vec![(1, 2), (1, 3), (2, 3)];
    prog.bar = vec![(1, 2), (1, 3), (2, 3)];
    prog.baz = vec![(1, 2), (1, 3), (2, 3)];

    prog.run();
    println!("foobar: {:?}", prog.foobar);

}


