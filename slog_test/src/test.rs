use slog::*;

prelude!();


#[test]
fn test_nested_fact() {
   use slog_eq_theory::eq_theory::eq_theory as theory_rules;
   slog! {
      (struct PathLength)
      (define empty usize)
      (define path usize sexpr)
      (define length sexpr usize)
      (define do_length sexpr)
      (define input sexpr)
      (define output usize)

      // ,(include_source!(theory_rules, __unify_ind_common_total, __unify_ind_common_delta );)

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
   use slog_eq_theory::eq_theory::eq_theory as theory_rules;
   slog! {
      (struct EClassTest)

      (define foo usize)
      (define bar usize)
      (define foobar eclass eclass)
      (define res eclass)

      (foobar (foo 1) (bar 1))
      [(union foo1 bar1) <-- (= foo1 (foo x)) (= bar1 (bar x))]
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
   use slog_eq_theory::eq_theory::eq_theory as theory_rules;

   slog! {
      (struct Foobar)
      (define foo usize usize)
      (define bar usize usize)
      (define foobar sexpr sexpr)

      // ,(include_source!(theory_rules, __unify_ind_common_total, __unify_ind_common_delta);)

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
   use slog_eq_theory::eq_theory::eq_theory as theory_rules;

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
   use slog_eq_theory::eq_theory::eq_theory as theory_rules;

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
