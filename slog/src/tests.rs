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

      [(foobar a b) <- (= a (foo x y)) (= b (bar x y))]
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

      [(foobar idf (bar x y)) <- (= idf (foo x y)) (bar x y)]

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

      // [(foobar idf (bar x y)) <- (= idf (foo x y)) (bar x y)]

      [(bar x y) <-- (foobar ?(foo x y) _) (bar x y)]
   };

   write_to_scratchpad(tokens, quote! {}, false);
}

#[test]
fn test_redundant_index() {
   let tokens = quote! {
      (struct Foobar)
      (define foo usize usize)
      (define bar usize usize)
      (define foobar sexpr sexpr)

      
   };


}

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
