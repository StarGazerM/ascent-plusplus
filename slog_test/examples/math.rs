use std::hash::{Hash, Hasher};

use slog::*;
prelude!();

use ordered_float::NotNan;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
struct HashableFloat(NotNan<f64>);
impl Hash for HashableFloat {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.into_inner().to_bits().hash(state);
    }
}

impl HashableFloat {
    fn new(f: f64) -> Self {
        HashableFloat(NotNan::new(f).unwrap())
    }
}

type Constant = HashableFloat;
type Symbol = u32;

macro_rules! hf {
    ($x:expr) => {
        HashableFloat::new($x)
    };
}

// define_language! {
//     pub enum Math {
//         "d" = Diff([Id; 2]),
//         "i" = Integral([Id; 2]),

//         "+" = Add([Id; 2]),
//         "-" = Sub([Id; 2]),
//         "*" = Mul([Id; 2]),
//         "/" = Div([Id; 2]),
//         "pow" = Pow([Id; 2]),
//         "ln" = Ln(Id),
//         "sqrt" = Sqrt(Id),

//         "sin" = Sin(Id),
//         "cos" = Cos(Id),

//         Constant(Constant),
//         Symbol(Symbol),
//     }
// }
slog! {
    (struct Math)

    (define eq:ascent_byods_rels::eqrel_canonical usize usize)

    (define diff usize usize)
    (define integral usize usize)
    (define add usize usize)
    (define sub usize usize)
    (define mul usize usize)
    (define div usize usize)
    (define pow usize usize)
    (define ln usize)
    (define sqrt usize)
    (define sin usize)
    (define cos usize)
    // atom of expression
    (define constant Constant)
    (define symbol Symbol)

    (define expression usize)
    
    (expression ?(add x y))
    (expression ?(sub x y))
    (expression ?(mul x y))
    (expression ?(div x y))
    (expression ?(pow x y))
    (expression ?(ln x))
    (expression ?(sqrt x))
    (expression ?(sin x))
    (expression ?(cos x))
    (expression ?(constant c))
    (expression ?(symbol x))
    // reflexive of expression
    [(= e (eq e e)) <-- (expression e)]
    // congruence of eq for expression
    [(eq (add x y) p1) <-- (= p1 (add @x @y))]
    [(eq (sub x y) p1) <-- (= p1 (sub @x @y))]
    [(eq (mul x y) p1) <-- (= p1 (mul @x @y))]
    [(eq (div x y) p1) <-- (= p1 (div @x @y))]
    [(eq (pow x y) p1) <-- (= p1 (pow @x @y))]
    [(eq (ln x) p1) <-- (= p1 (ln @x))]
    [(eq (sqrt x) p1) <-- (= p1 (sqrt @x))]
    [(eq (sin x) p1) <-- (= p1 (sin @x))]
    [(eq (cos x) p1) <-- (= p1 (cos @x))]
    
    // rw!("comm-add";  "(+ ?a ?b)"        => "(+ ?b ?a)"),
    [(= p1 (add @y @x)) --> (eq (add x y) p1)]
    // rw!("comm-mul";  "(* ?a ?b)"        => "(* ?b ?a)"),
    [(= p1 (mul @y @x)) --> (eq (mul x y) p1)]
    // rw!("assoc-add"; "(+ ?a (+ ?b ?c))" => "(+ (+ ?a ?b) ?c)"),
    [(add @(add @x @y) @z) --> (add y z)]
    [(= p1 (add @(add @x @y) @z)) --> (eq (add x @?(add y z)) p1)]

    // rw!("assoc-mul"; "(* ?a (* ?b ?c))" => "(* (* ?a ?b) ?c)"),
    [(mul @a @(mul @b _))--> (mul a b)]
    [(= m1 (mul @a @(mul @b @c))) --> (eq (mul @?(mul a b) c) m1)]
    
    // rw!("distribute"; "(* ?a (+ ?b ?c))"        => "(+ (* ?a ?b) (* ?a ?c))"),
    [(mul @a @(add @b @c)) --> (add (mul a b) (mul a c))]
    [(= m1 (mul @a @(add @b @c))) --> (eq (add @?(mul a b) @?(mul a c)) m1)]

    // rw!("factor"    ; "(+ (* ?a ?b) (* ?a ?c))" => "(* ?a (+ ?b ?c))"),
    [(add @(mul @a @b) @(mul @a @c)) --> (add b c)]
    [(= p1 (add (mul @a @b) @(mul @a @c))) --> (eq (mul a @?(add b c)) p1)]

    // rw!("pow-mul"; "(* (pow ?a ?b) (pow ?a ?c))" => "(pow ?a (+ ?b ?c))"),
    [(mul @(pow @a @b) @(pow a @c)) --> (add b c)]
    [(= m1 (mul @(pow @a @b) @(pow a @c))) --> (eq (pow a @?(add b c)) m1)]

    // rw!("pow0"; "(pow ?x 0)" => "1"
    //     if is_not_zero("?x")),
    [(= p1 (pow _ @(constant ,(hf!(0.0)))))--> (eq (constant ,(hf!(1.0))) p1)]
    // rw!("pow1"; "(pow ?x 1)" => "?x"),
    [(= p1 (pow @x @(constant ,(hf!(1.0))))) --> (eq p1 x)]
    // rw!("pow2"; "(pow ?x 2)" => "(* ?x ?x)"),
    [(= p1 (pow @x @(constant ,(hf!(2.0))))) --> (eq (mul x x) p1)]

    // rw!("pow-recip"; "(pow ?x -1)" => "(/ 1 ?x)"
    //     if is_not_zero("?x")),
    [(= p1 (pow @x @(constant ,(hf!(-1.0))))) --> (eq (div 1 x) p1)]
    // rw!("recip-mul-div"; "(* ?x (/ 1 ?x))" => "1" if is_not_zero("?x")),
    [(= m1 (mul @x @(constant ,(hf!(1.0))))) --> (eq (div 1 x) m1)]

    // rw!("d-variable"; "(d ?x ?x)" => "1" if is_sym("?x")),
    [(= p1 (diff @x @x)) --> (eq (constant ,(hf!(1.0))) p1)]
    // rw!("d-constant"; "(d ?x ?c)" => "0" if is_sym("?x") if is_const_or_distinct_var("?c", "?x")),
    [(= p1 (diff @x @(constant ,(hf!(0.0))))) --> (eq (constant ,(hf!(0.0))) p1)]

    // rw!("d-add"; "(d ?x (+ ?a ?b))" => "(+ (d ?x ?a) (d ?x ?b))"),
    [(diff @x (add @a @b)) --> (diff x a) (diff x b)]
    [(= d1 (diff @x (add @a @b))) --> (eq (add @?(diff x a) @?(diff x b)) d1)]
    // rw!("d-mul"; "(d ?x (* ?a ?b))" => "(+ (* ?a (d ?x ?b)) (* ?b (d ?x ?a)))"),
    // TODO: how to handle this using syntax sugar?
    [(diff @x (mul @a @b)) --> (diff x b) (diff x a)]
    [(diff @x (mul @a @b)) --> (add @?(diff x b) @?(diff x a))]
    [(diff @x (mul @a @b))
     (= d2 (diff x a)) (= dd2 (eq d2 d2))
     (= d3 (diff x b)) (= dd3 (eq d3 d3))
     --> (mul a dd3) (mul b dd2)]
    [(= d1 (diff @x (mul @a @b)))
     (= d2 (diff x a)) (= dd2 (eq d2 d2))
     (= d3 (diff x b)) (= dd3 (eq d3 d3))
     (= m1 (mul b dd2)) (= mm1 (eq m1 m1))
     (= m2 (mul a dd3)) (= mm2 (eq m2 m2))
     --> (eq (add mm1 mm2) d1)]

    // rw!("d-sin"; "(d ?x (sin ?x))" => "(cos ?x)"),
    [(= p1 (diff @x @(sin @x))) --> (eq (cos x) p1)]
    // rw!("d-cos"; "(d ?x (cos ?x))" => "(* -1 (sin ?x))"),
    [(diff @x @(cos @x)) --> (sin x)]
    [(= p1 (diff @x @(cos @x))) --> (eq (mul (constant ,(hf!(-1.0))) @?(sin x)) p1)]

    // rw!("d-ln"; "(d ?x (ln ?x))" => "(/ 1 ?x)" if is_not_zero("?x")),
    [(= d1 (diff @x @(ln x))) --> (eq (div (constant ,(hf!(1.0))) x) d1)]

    // rw!("d-power";
    //     "(d ?x (pow ?f ?g))" =>
    //     "(* (pow ?f ?g)
    //         (+ (* (d ?x ?f)
    //               (/ ?g ?f))
    //            (* (d ?x ?g)
    //               (ln ?f))))"
    //     if is_not_zero("?f")
    //     if is_not_zero("?g")
    // ),

    // rw!("i-one"; "(i 1 ?x)" => "?x"),
    [(= p1 (integral (= @n (constant ,(hf!(1.0)))) @x)) --> (eq x p1)]
    // rw!("i-power-const"; "(i (pow ?x ?c) ?x)" =>
    //     "(/ (pow ?x (+ ?c 1)) (+ ?c 1))" if is_const("?c")),
    [(integral @(pow @x @c) @x) --> (add c (constant ,(hf!(1.0))))]
    // TODO: TBD
    // rw!("i-cos"; "(i (cos ?x) ?x)" => "(sin ?x)"),
    [(= i1 (integral @(cos @x) @x)) --> (eq (sin x) i1)]
    // rw!("i-sin"; "(i (sin ?x) ?x)" => "(* -1 (cos ?x))"),
    [(integral @(sin @x) @x) --> (cos x)]
    [(= i1 (integral @(sin @x) @x)) --> (eq (mul (constant ,(hf!(-1.0))) @?(cos x)) i1)]

    // rw!("i-sum"; "(i (+ ?f ?g) ?x)" => "(+ (i ?f ?x) (i ?g ?x))"),
    [(integral @(add @f @g) @x) --> (integral f x) (integral g x)]
    [(= i1 (integral @(add @f @g) @x)) --> (eq (add @?(integral f x) @?(integral g x)) i1)]
    // rw!("i-dif"; "(i (- ?f ?g) ?x)" => "(- (i ?f ?x) (i ?g ?x))"),
    [(integral @(sub @f @g) @x) --> (integral f x) (integral g x)]
    [(= i1 (integral @(sub @f @g) @x)) --> (eq (sub @?(integral f x) @?(integral g x)) i1)]
    // rw!("i-parts"; "(i (* ?a ?b) ?x)" =>
    //     "(- (* ?a (i ?b ?x)) (i (* (d ?x ?a) (i ?b ?x)) ?x))"),
    // TODO: TBD
}


fn main() {
    let mut prog = Math::default();
    prog.run();
    // println!("{:?}", prog.math);
}