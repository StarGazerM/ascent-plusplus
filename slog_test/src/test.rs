use slog::*;

prelude!();

#[test]
fn test_nested_fact() {
   slog! {
      (struct PathLength)
      (define empty usize)
      (define path usize usize)
      (define length usize usize)
      (define do_length usize)
      (define input usize)
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
fn test_foo_bar() {
   slog! {
      (struct Foobar)
      (define foo usize usize)
      (define bar usize usize)
      (define foobar usize usize)

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
      (define add usize usize)
      (define mul usize usize)

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
   }, slog_gen);

   foobar_db!(Foobar1, {
      (foo 1)
      (bar 2)
   });
   foobar_db!(Foobar2, {
      [(foo x) <-- (bar x)  (foo x)]
   });
   let mut from = Foobar1::default();
   from.run();
   let mut to = Foobar2::default();
   pipe_foobar_db!(from, to);
   to.run();
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

#[test]
fn test_ds() {
   slog! {
      (struct DsTest)
      (define tc:ascent_byods_rels::eqrel usize)
      (define tc2 usize usize)
      (define edge usize usize)

      (edge 1 2)
      (edge 2 3)
      (edge 3 4)

      [(= y (tc x)) <-- (edge x y)]
      [(tc2 x y) <-- (= x (tc y))]
   }

   let mut prog = DsTest::default();
   prog.run();
   println!("{:?}", prog.tc2);
}

#[test]
fn test_eq2() {
   slog! {
      (struct EqManual)
      (define foo usize)
      (define bar usize)
      (define foobar usize usize)
      (define top usize)

      (define equiv : ascent_byods_rels::eqrel_canonical
         usize usize)
      ,(relation bar_canonical(usize, usize);)

      [,(equiv(f, f, f)) <-- (= f (foo x))]
      [,(equiv(f, f, f)) <-- (= f (bar x))]
      // congurence of equiv
      [,(equiv(foobar1, foobar2, foobar1)) <--
         (= foobar1 (foobar x y))
         (equiv x x_eq) (equiv y y_eq)
         (= foobar2 (foobar x_eq y_eq))]

      (foobar (foo 1) (bar 1))
      (foobar (bar 1) (bar 2))
      [,(equiv(x, y, x)) <--
         (= x (bar m)) (= y (bar ,(m + 1)))]
      // (bar 1) = (bar 2)
      [(foobar y_canonical y_canonical) <--
         (foobar _ x)
         // ,(equiv(x, x, x_canonical))
         (= x_canonical (equiv x x))
         // ,(let _ = println!("x: {:?}, x_canonical: {:?}", x, x_canonical))
         (= x_canonical (bar m)) (= y (bar ,(m + 1)))
         // ,(equiv(y, y, y_canonical))
         (= y_canonical (equiv y y))]
      // macterialize the eclass of foobar
      [(top f) <-- (= f (foobar x y)), (equiv(_, _, x), equiv(_, _, y))]

      [,(bar_canonical(id, id_canonical)) <-- (= id (bar x)) ,(equiv(id, id, id_canonical))]
   }
   let mut prog = EqManual::default();
   prog.run();
   println!("foo: {:?}", prog.foo);
   println!("bar: {:?}", prog.bar);
   println!("foobar: {:?}", prog.foobar);
   println!("top: {:?}", prog.top);
   println!("bar_canonical: {:?}", prog.bar_canonical);
}

#[test]
fn test_infinity_eq() {
   slog! {
      (struct InfinityEq)
      (define var usize)
      (define num i32)
      (define plus usize usize)
      (define eq: ascent_byods_rels::eqrel_canonical usize usize)
      (define expression usize)
      (define res usize)
      ,(relation eq_materialize(usize, usize);
        relation canonical_expression(usize);)
      [,(eq_materialize(e1, e2)) <-- (eq e1 e2)]
      [,(canonical_expression(e)) <-- (expression e) (= e (eq _ _))]
      [(= e (eq e e)) <-- (expression e)]
      (var 1)

      (plus (plus (var 1) (num 0)) (num 0))
      (expression ?(num n))
      (expression ?(plus p q))
      (expression ?(var v))

      [(= p (eq p p)) <-- (= p (plus x y))]
      
      // congruence of eq
      [(eq (plus ?(eq x x) ?(eq y y)) p1) <--
         (= p1 (plus x y))]
      
      [(eq (plus ?(= ee (eq e e)) ?(num 0)) ee) <-- (expression e)]
      [(plus ?(eq e1 e1) ?(eq n n)) <--
         (= e (plus e1 e2))
         (= n (num 0))]

      [(res 1) <--
         (= ev (var 1))
         (= e1 (plus ev (num 0)))
         (= e1 (eq e1 ev))]
      [(res 2) <--
         (= ev (var 1))
         (= e1 (plus (plus ev (num 0)) (num 0)))
         (= e1 (eq e1 ev))]
   }

   let mut prog = InfinityEq::default();
   prog.run();
   println!("var: {:?}", prog.var);
   println!("num: {:?}", prog.num);
   println!("plus: {:?}", prog.plus);
   println!("canonical_expression: {:?}", prog.canonical_expression);
   println!("res: {:?}", prog.res);
}


#[test]
fn test_eclass() {
   use slog_theory::eq_theory as theory_rules_eclass;
   use slog_utils::ascent_gen;

   local_db!(eclass_test_query,
      (theory (eclass usize unify_eclass ascent_byods_rels::eqrel_canonical)),
   {
      (define foo usize)
      (define bar usize)
      (define foobar @eclass @eclass)
      (define res @eclass)
   }, slog_gen);

   eclass_test_query!(EClassTest, {
      (foobar (foo 1) (bar 1))
      [(unify_eclass foo1 bar1 foo1) <-- (= foo1 (foo x)) (= bar1 (bar x)) ]
   });

   eclass_test_query!(EClassTestRes, {
      [(res x) <-- (foobar x x)]
   });

   let mut q1 = EClassTest::default();
   q1.run();
   let mut q2 = EClassTestRes::default();
   pipe_eclass_test_query!(q1, q2);
   q2.run();
   println!("{:?}", q2.foo);
   println!("{:?}", q2.foobar);
   println!("{:?}", q2.res);
}

