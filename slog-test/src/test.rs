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
fn test_inflation_ascent() {
   ascent! {
      struct Inflation;
      relation edge(usize, usize);
      relation path(usize, usize);

      edge(1, 2);
      edge(2, 3);
      edge(3, 4);
      edge(4, 5);
      edge(1, 3);
      path(1, 2);

      path(x, y) <-- edge(x, y);
      inflate path(x, z) <-- path(x, y), let _ = println!("path({:?}, {:?})", x, y), edge(y, z);
   }

   let mut prog = Inflation::default();

   prog.run();
   println!("{:?}", prog.edge);
   println!("{:?}", prog.path);
}

#[test]
fn test_equality_ascent() {
   ascent! {
      #![egglog_mode]
      struct TCEq;

      relation edge(usize, usize);
      relation path(usize, usize);
      relation path_rep(usize, usize);

      edge(1, 2);
      edge(2, 3);
      edge(3, 4);
      edge(4, 5);
      edge(6, 7);
      edge(7, 8);

      x <=> y, path(x, y) <-- edge(x, y);

      x_rep <=> z_rep,
      path(x_rep.clone(), z_rep.clone()) <--
         path(x, y),
         y <=> y_inflate,
         edge(y_inflate, z),
         z <=>! z_rep,
         x <=>! x_rep;

      path_rep(x, rep_x) <-- path(x, _), x <=>! rep_x;
      path_rep(y, rep_y) <-- path(_, y), y <=>! rep_y;
   }

   let mut prog = TCEq::default();
   prog.run();
   println!("{:?}", prog.edge);
   println!("{:?}", prog.path_rep);
}

#[test]
fn test_eclass() {
   slog! {
      (struct EClassTest)

      (define foo usize)
      (define bar usize)
      (define foobar eclass eclass)
      (define res eclass)

      (foobar (foo 1) (bar 1))
      [(union foo1 bar1) <-- (= foo1 (foo x)) (= bar1 (bar x))]
      // (rewrite! (foo x) (bar x))

      [(res x) <-- (foobar x x)]
   }

   let mut prog = EClassTest::default();
   prog.run();
   println!("{:?}", prog.foo);
   println!("{:?}", prog.foobar);
   println!("{:?}", prog.res);
}

#[test]
fn test_eqrel() {
   use ascent::union_find::EqRel;

   let mut eq_rel = EqRel::default();

   eq_rel.add(1, 2);
   eq_rel.add(2, 3);

   let res = eq_rel.set_of(&1);
   println!("{:?}", res);
   let res_rep_3 = eq_rel.get_dominant_elem(&3);
   println!("{:?}", res_rep_3);
   let res_rep_2 = eq_rel.get_dominant_elem(&2);
   println!("{:?}", res_rep_2);
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
