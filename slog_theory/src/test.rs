

#[test]
fn test_arithm_rel_codegen() {
    use ascent::ascent;
    use crate::arithm::ArithmFn;
    use crate::arithm::eq;
    use smtlib::prelude::*;
    ascent! {
        struct Test;
        #[ds(crate::arithm)]
        relation binary_arithm(ArithmFn, i32, i32);

        relation edge(i32, i32);
        relation path(i32, i32);
        edge(1,2);
        edge(2,3);
        edge(3,4);
        
        path(x, y) <-- edge(x, y);
        path(x, z) <-- path(x, m),
            binary_arithm(eq!(m, n in (m + 1) = (n)), m, n),
            edge(n, z);
    }
    let mut prog = Test::default();
    prog.run();
    println!("path: {:?}", prog.path);
}

#[test]
fn test_invertible() {
    use ascent::ascent;
    use crate::invertible::InvertibleFn;

    ascent! {
        struct Test;
        #[ds(crate::invertible)]
        relation arithm(InvertibleFn, i32, i32);

        relation edge(i32, i32);
        relation path(i32, i32);
        edge(1,2);
        edge(2,3);
        edge(3,4);

        path(x, y) <-- edge(x, y);
        path(x, z) <-- path(x, m),
            arithm(InvertibleFn(|x| x + 1, |x| x - 1), m, n),
            edge(n, z);
    }
    let mut prog = Test::default();
    prog.run();
    println!("path: {:?}", prog.path);
}

