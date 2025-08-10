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

      #(input (path 1 (path 2 (path 3 ?(nil 0)))))

      #(length (do_length ?(nil 0)) 0)
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
   println!("{:?}", prog.path_id);
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

      edge(1, 2);
      edge(2, 3);
      edge(3, 4);
      edge(4, 5);

      x <=> y, path(x, y) <-- edge(x, y);

      x <=> z,
      path(x, z) <--
         path(x, y),
         y <=> y_inflate,
         edge(y_inflate, z),
         z <=>? z_rep;
   }

   let mut prog = TCEq::default();
   prog.run();
   println!("{:?}", prog.edge);
   println!("{:?}", prog.path);
}
