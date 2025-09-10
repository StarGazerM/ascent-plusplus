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
      use ascent::aggregators::*;
      use ascent::eclass_id;
      use ascent_byods_rels::eqrel;
      use ascent::util::calc_id;
      use slog::equiv_vec_huh;
      use slog::canonicalize;
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

#[proc_macro]
pub fn canonicalize(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
   let input = syn::parse2::<syn::Expr>(input.into()).unwrap();
   let res = quote! {
      _self.runtime_total.__equiv_ind_common.get_dominant_elem(#input).unwrap_or(#input)
   };
   res.into()
}

#[proc_macro]
pub fn equiv_vec_huh(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
   let input = syn::parse2::<syn::Expr>(input.into()).unwrap();
   let res = quote! {
      _self.runtime_total.__equiv_ind_commons.equiv_vec_huh(&#input)
   };
   res.into()
}
