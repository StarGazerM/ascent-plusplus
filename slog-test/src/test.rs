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
      // [(do_length (path h tail)) --> (do_length tail)]
      ,(>? path_571169 . path (h , tail) , >? do_length_516562 . do_length (path_571169) <--
         do_length (path_480031) ,
         let _ = println!("wwwwww {:?}", path_480031),
         path (h , tail) . path_480031 ;)

      [(output y) <-- (input x) (length (do_length x) y)]
   }

   let mut prog = PathLength::default();

   prog.run();
   println!("{:?}", prog.input);
   println!("{:?}", prog.output);
   println!("{:?}", prog.do_length);
   println!("{:?}", prog.path);
}
