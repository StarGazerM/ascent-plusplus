#![cfg(test)]

use quote::quote;
use proc_macro2::TokenStream;

use crate::compile::compile;

// test the syntax of slog

#[test]
fn test_slog_unstructure_compile() {
   let tokens = quote! {
      (struct Foobar)
      (define foo usize usize)
      (define bar usize usize)
      (define foobar usize usize)

      [(foobar a b) <-- (= a (foo x y)) (= b (bar x y))]
   };

   write_to_scratchpad(tokens, quote! {}, false);
}

#[test]
fn test_slog_structure_head_compile() {
   let tokens = quote! {
      (struct Foobar)
      (define foo usize usize)
      (define bar usize usize)
      (define foobar usize usize)

      [(foobar idf (bar x y)) <-- (= idf (foo x y))]
   };

   write_to_scratchpad(tokens, quote! {}, false);
}

#[test]
fn test_slog_structure_body_compile() {
   let tokens = quote! {
      (struct Foobar)
      (define foo usize usize)
      (define bar usize usize)
      (define foobar sexpr sexpr)

      [(foobar idf (bar x y)) <-- (= idf (foo x y)) (bar x y)]

      [(bar x y) <-- (foobar (foo x y) _) (bar x y)]
   };

   write_to_scratchpad(tokens, quote! {}, false);
}


#[test]
fn test_slog_order_compile() {
   let tokens = quote! {
      (struct Foobar)
      (define foo usize usize)
      (define bar usize usize)
      (define foobar sexpr sexpr)

      // query foo before foobar
      [(bar x y) <-- (foobar ?(foo x y) _) (bar x y)]
      [(bar x y) <-- (foobar (foo x y) _) (bar x y)]
   };

   write_to_scratchpad(tokens, quote! {}, false);
}

#[test]
fn test_slog_fact_compile() {
   let tokens = quote! {
      (struct Foobar)
      (define foo usize usize)
      (define bar usize usize)
      (define foobar sexpr sexpr)

      #(foo 1 2)
      #(bar 3 4)
      #(foobar (foo 1 2) (bar 3 4))
   };

   write_to_scratchpad(tokens, quote! {}, false);
}

#[test]
fn test_generative_facts_compile() {
   let tokens = quote! {
      (struct Foobar)
      (define foo usize usize)
      (define bar usize usize)
      (define foobar sexpr sexpr)

      #(foo 1 2)
      #(bar 3 4)
      #(foobar ?(foo x y) ?(bar a b))
   };
   write_to_scratchpad(tokens, quote! {}, false);
}


#[test]
fn test_normal_rule_compile() {
   let tokens = quote! {
      (struct TC)
      (define edge usize usize)
      (define tc usize usize)

      [(tc x y) <-- (edge x y)]
      [(tc x y) <-- (edge x y) (tc x y)]
   };

   write_to_scratchpad(tokens, quote! {}, false);
}


#[test]
fn test_resverse_arrow_rule_compile() {
   let tokens = quote! {
      (struct TC)
      (define edge usize usize)
      (define tc usize usize)

      [(tc x y) --> (edge x y)]
   };

   write_to_scratchpad(tokens, quote! {}, false);
}

#[test]
fn test_escape_syntax_compile1() {
   let tokens = quote! {
      (struct TC)
      (define edge usize usize)
      (define tc usize usize)

      [(tc x yy) <-- (edge x y) ,(let yy = y) ,(if y > &10)]
   };

   write_to_scratchpad(tokens, quote! {}, false);
}

#[test]
fn test_escape_syntax_compile2() {
   let tokens = quote! {
      (struct TC)
      (define edge usize usize)
      (define tc usize usize)

      [(tc ,(x+1) y) <-- (edge x y)]
   };

   write_to_scratchpad(tokens, quote! {}, false);
}

#[test]
fn test_id_unification_compile() {
   let tokens = quote! {
      (struct Foobar)
      (define foo usize usize)
      (define bar usize usize)
      (define foobar sexpr sexpr)

      [(foobar idf (bar x y)) <-- (= idf (foo x y)) (bar x y)]
   };

   write_to_scratchpad(tokens, quote! {}, false);
}

#[test]
fn test_structured_clause_compile() {
   let tokens = quote! {
      (struct Foobar)
      (define foo usize usize)
      (define bar usize usize)
      (define foobar sexpr sexpr)

      [(foobar (foo x y) (bar a b)) <-- (foo x y) (bar a b)]
   };

   write_to_scratchpad(tokens, quote! {}, false);
}

#[test]
fn test_question_mark_compile() {
   let tokens = quote! {
      (struct Foobar)
      (define foo usize usize)
      (define bar usize usize)
      (define foobar sexpr sexpr)

      [(foo ?(bar x z) z) <-- (foo x y)]
   };
   write_to_scratchpad(tokens, quote! {}, false);
}

#[test]
fn test_bang_compile() {
   let tokens = quote! {
      (struct PathLength)
      (define empty usize)
      (define path usize sexpr)
      (define length sexpr usize)
      (define do_length sexpr)

      #(path 1 (path 2 (path 3 ?(nil 0))))

      #(length (do_length ?(nil 0)) 0)
      [(do_length (path h tail)) --> (do_length tail)]
      [(length ?(do_length (path h tail)) ,(l + 1)) <--
         (length (do_length tail) l)]
      [(do_length (path h tail)) --> (do_length (path h tail))]
   };
   write_to_scratchpad(tokens, quote! {}, false);
}

#[test]
fn test_escape_syntax_compile3 () {
   let tokens = quote! {
      (struct Esacape)
      (define empty1 usize)
      (define empty2 usize)
      
      ,(empty1(x) <-- empty2(x);)
   };
   write_to_scratchpad(tokens, quote! {}, false);
}

// #[test]
// fn test_redundant_index() {
//    let tokens = quote! {
//       (struct Foobar)
//       (define foo usize usize)
//       (define bar usize usize)
//       (define foobar sexpr sexpr)
//    };
// }



// a test helper function to write slog to scratchpad
fn write_to_scratchpad(
   tokens: TokenStream, prefix: TokenStream, is_parallel: bool,
) -> TokenStream {
   let code = compile(tokens, is_parallel).unwrap();
   let code = quote! {
      #prefix
      #code
   };
   let template = std::fs::read_to_string("src/scratchpad_template.rs").unwrap();
   let code_str = code.to_string();
   // add `\n` to every line
   let code_str = code_str.replace(";", ";\n");
   let code_in_template = template.replace("todo!(());", &code_str);
   std::fs::write("src/scratchpad.rs", prefix.to_string()).unwrap();
   std::fs::write("src/scratchpad.rs", code_in_template).unwrap();
   std::process::Command::new("rustfmt").args(&["src/scratchpad.rs"]).spawn().unwrap().wait().unwrap();
   code
}
