use slog::*;

prelude!();


#[test]
fn test_nested_fact() {
   slog! {
      (struct PathLength)
      (define empty usize)
      (define path usize sexpr)
      (define length sexpr usize)
      (define do_length sexpr)
      (define input sexpr)
      (define output usize)

      (input (path 1 (path 2 (path 3 ?(nil 0)))))

      (length (do_length ?(nil 0)) 0)
      [(do_length (path h tail)) --> (do_length tail)]
      [(length ?(do_length (path h tail)) ,(l + 1)) <--
         (length (do_length tail) l)]
      [(do_length (path h tail)) --> (do_length tail)]
      [(input x) --> (do_length x)]

      [(output y) <-- (input x) (length (do_length x) y)]
   }
   let mut prog = PathLength::default();

   prog.run();
   println!("{:?}", prog.input);
   println!("{:?}", prog.output);
   println!("{:?}", prog.do_length);
}


#[test]
fn test_eclass() {
   use slog_theory::eq_theory::eq_theory as theory_rules_eclass;
   use ascent::{delta, total};
   slog! {
      (struct EClassTest)
      (theory (eclass usize unify_eclass ascent_byods_rels::eqrel))

      (define foo usize)
      (define bar usize)
      (define foobar eclass eclass)
      (define res eclass)

      (foobar (foo 1) (bar 1))
      [(unify_eclass foo1 bar1) <-- (= foo1 (foo x)) (= bar1 (bar x))]
      // (rewrite! (foo x) (bar x))

      [(res x) <-- (foobar x y)
         ,(if x == y)]
   }

   let mut prog = EClassTest::default();
   prog.run();
   println!("{:?}", prog.foo);
   println!("{:?}", prog.foobar);
   println!("{:?}", prog.res);

}


#[test]
fn test_foo_bar() {

   slog! {
      (struct Foobar)
      (define foo usize usize)
      (define bar usize usize)
      (define foobar sexpr sexpr)

      [(foobar idf (bar x y)) <-- (= idf (foo x y)) (bar x y)]
   }

   let mut prog = Foobar::default();
   prog.run();
   println!("{:?}", prog.foo);
   println!("{:?}", prog.bar);
   println!("{:?}", prog.foobar);
}


#[test]
fn test_ast() {

   slog! {
      (struct AST)
      
      (define number i32)
      (define var &'static str)
      (define add eclass eclass)
      (define mul eclass eclass)

      // 2 * (x * 3)
      (mul (number 2) (mul (var "x") (number 3)))
      // 6 * xs
      (mul (number 6) (var "x"))
   }
   let mut prog = AST::default();
   prog.run();
   println!("{:?}", prog.number);
   println!("{:?}", prog.var);
   println!("{:?}", prog.mul);
   println!("{:?}", prog.add);
}

#[test]
fn test_aggregator() {

   slog! {
      (struct AggregatorTest)

      (define foo i32 i32)
      (define bar i32 Vec<i32>)

      (foo 1 2)
      (foo 3 4)
      (foo 1 6)

      [(bar x ,(y.clone())) <-- (foo x _) ,(agg y = collect(v) in foo(x, v, _))]
   }

   let mut prog = AggregatorTest::default();
   prog.run();
   println!("{:?}", prog.foo);
   println!("{:?}", prog.bar);
}

#[test]
fn test_pipe() {
   local_db!(foobar_db, {
      (define foo usize)
      (define bar usize)
   });

   foobar_db!(Foobar1, {
      (foo 1)
      (bar 2)
   });
   
   foobar_db!(Foobar2, {
      [(foo x) <-- (bar x)]
   });
   let mut from = Foobar1::default();
   from.run();
   let mut to = Foobar2::default();
   pipe_foobar_db!(from, to);
   println!("{:?}", to.foo);
   println!("{:?}", to.bar);
}

#[test]
fn tc_reordering() {
   use slog_utils::id_vec;
   slog! {
      (struct TcReordering)
      (define tc usize usize)
      [(tc x y) <-- Δ (tc x y) (tc y z)]
   }
   let mut prog = TcReordering::default();
   prog.tc = id_vec!(calc_id, [(1, 2), (1, 3), (2, 3), (3, 4)]);
   prog.run();
   println!("{:?}", prog.tc);
}
