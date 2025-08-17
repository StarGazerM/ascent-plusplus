use quote::quote;

mod syntax;
mod tests;
mod compile;
mod util;

mod scratchpad;

// proc macro compile slog program to ascent program

#[proc_macro]
pub fn prelude(_: proc_macro::TokenStream) -> proc_macro::TokenStream {
   let res = quote! {
      use ascent::*;
      use ascent::eclass_id;
   };
   res.into()
}


#[proc_macro]
pub fn slog(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
   let res = compile::compile(input.into(), false);

   match res {
      Ok(res) => {
         let res = quote! {
             #res
         };
         res.into()
      }
      Err(err) => {
         let err = err.to_compile_error();
         err.into()
      }
   }
}
