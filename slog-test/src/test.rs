use slog::*;

prelude!();

#[test]
fn test_nested_fact() {
   slog! {
      (struct PathLength)
      (define edge usize usize)
      (define path usize sexpr)
      (define length sexpr usize)
      (define do_length sexpr)

      #(path 1 (path 2 (path 3 ?(nil 0))))
   }

   let mut prog = PathLength::default();

   prog.run();
   println!("{:?}", prog.nil_id );
   println!("{:?}", prog.path_id);
}
