use std::sync::Mutex;

use egg::{RecExpr, SymbolLang, EGraph, Id as EggId};
use lazy_static::lazy_static;
use slog::*;
prelude!();


macro_rules! rec_expr {
    ($str:expr, $($fmt_arg:tt),*) => {
        EGRAPH.lock().unwrap()
            .add_expr(&format!($str, $($fmt_arg),*)
                .parse::<egg::RecExpr<egg::SymbolLang>>().unwrap())
    };
}

macro_rules! nested_expr {
    ($str:expr, $($fmt_arg:tt),*) => {
        EGRAPH.lock().unwrap()
            .add(SymbolLang::new($str, vec![$(*$fmt_arg),*]))
    }
}

macro_rules! union_expr {
    ($e1:expr, $e2:expr) => {
        EGRAPH.lock().unwrap().union(*$e1, *$e2)
    }
}

// make a global egraph using lazy static
lazy_static! {
    static ref EGRAPH: Mutex<EGraph<SymbolLang, ()>> = Mutex::new(Default::default());
}

slog! {
    (struct Tutorial)
    (define expression usize)
    (define div usize usize)
    (define add usize usize)
    (define mult usize usize)
    (define shl usize usize)
    (define var i64)
    (define lit i64)
    (define egg_expr String EggId)

    (define eq:ascent_byods_rels::eqrel_canonical usize usize)

    // --------------- Type Checking --------------- //
    [(expression ?(div x y)) <-- (expression x) (expression y)]
    [(expression ?(add x y)) <-- (expression x) (expression y)]
    [(expression ?(mult x y)) <-- (expression x) (expression y)]
    [(expression ?(shl x y)) <-- (expression x) (expression y)]
    (expression ?(var v))
    (expression ?(lit n))
    // --------------- Reflexive of eq --------------- //
    [(= e (eq e e)) <-- (expression e)]
    // --------------- Congruence of eq --------------- //
    [(eq e (div ?(eq x x) ?(eq y y))) <-- (= e (div x y))]
    [(eq e (add ?(eq x x) ?(eq y y))) <-- (= e (add x y))]
    [(eq e (mult ?(eq x x) ?(eq y y))) <-- (= e (mult x y))]
    [(eq e (shl ?(eq x x) ?(eq y y))) <-- (= e (shl x y))]

    // --------------- EDB --------------- //
    (div (mult (var 1) (lit 2)) (lit 2))

    // --------------- Rewrite Rules in Egg --------------- //
    [(eq (shl ?(eq x x) (lit 1)) m) <-- (= m (mult x (lit 2)))]
    // bidirectional rewrite rule for div
    // [(eq (div (mult ?(eq x x) ?(eq y y)) ?(eq z z)) m) <--
    //     (= m (mult x (= d (eq d (div y z)))))]
    [(eq (mult ?(eq x x) (div ?(eq y y) ?(eq z z))) m) <--
        (= m (div (= d (eq d (mult x y))) z))]
    [(eq (lit 1) e) <-- (= e (div x x))]
    // [(eq (mult ?(eq e e) (lit 1)) e) <-- (expression e)]
    [(eq ?(eq e e) ?(eq m m)) <-- (= m (mult e n)) (eq n (lit 1))]



    // ----------- convert to equivalent EGraph -------------- //
    // convert to egg rec expr
    [(= e (egg_expr ,(format!("(var {})", x)) egg_id)) <--
        (= e (var x)) ,(let egg_id = rec_expr!("(var {})", x))]
    [(= e (egg_expr ,(format!("(lit {})", x)) egg_id)) <--
        (= e (lit x)) ,(let egg_id = rec_expr!("(lit {})", x))]
    [(= e (egg_expr ,(format!("(mult {} {})", x, y)) egg_id)) <--
        (= e (mult (= e1 (egg_expr x e1_id)) (= e2 (egg_expr y e2_id))))
        // (= e1 (eq _ _)) (= e2 (eq _ _))
        ,(let egg_id = nested_expr!("mult", e1_id, e2_id))]
    [(= e (egg_expr ,(format!("(div {} {})", x, y)) egg_id)) <--
        (= e (div (= e1 (egg_expr x e1_id)) (= e2 (egg_expr y e2_id))))
        // (= e1 (eq _ _)) (= e2 (eq _ _))
        ,(let egg_id = nested_expr!("div", e1_id, e2_id))]
    [(= e (egg_expr ,(format!("(add {} {})", x, y)) egg_id)) <--
        (= e (add (= e1 (egg_expr x e1_id)) (= e2 (egg_expr y e2_id))))
        // (= e1 (eq _ _)) (= e2 (eq _ _))
        ,(let egg_id = nested_expr!("add", e1_id, e2_id))]
    [(= e (egg_expr ,(format!("(shl {} {})", x, y)) egg_id)) <--
        (= e (shl (= e1 (egg_expr x e1_id)) (= e2 (egg_expr y e2_id))))
        // (= e1 (eq _ _)) (= e2 (eq _ _))
        ,(let egg_id = nested_expr!("shl", e1_id, e2_id))]
    // union based on eqrel we computed in previous step
    ,(relation empty(usize);)
    [,(empty(ec)) <--
        (= ec (eq e1 e2))
        (= e1 (egg_expr _ e1_id))
        (= ec (egg_expr _ e2_id))
        ,(let _ = union_expr!(e1_id, e2_id))]
    
    // (define materialize_eq usize usize)
    // [(= id (materialize_eq e1 e2)) <-- (= id (eq e1 e2)) ,(if e1 != e2)]
}

fn main() {
    let mut prog = Tutorial::default();
    prog.run();
    println!("egg_expr: {:?}", prog.egg_expr);
    // println!("materialize_eq: {:?}", prog.materialize_eq);
    {
        EGRAPH.lock().unwrap().rebuild();
    }
    {
        // get target folder
        let target_folder = std::env::current_dir().unwrap().join("target");
        let png_path = target_folder.join("egraph.png");
        let png_path_str = png_path.to_str().unwrap();
        // delete file if it exists
        if png_path.exists() {
            std::fs::remove_file(&png_path).unwrap();
        }
        EGRAPH.lock().unwrap().dot().to_png(png_path_str).unwrap();
        println!("Saved egraph to {}", png_path_str);
    }
    
}