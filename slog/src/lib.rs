use quote::quote;

mod syntax;
mod tests;
mod compile;
mod util;


// proc macro compile slog program to ascent program
#[proc_macro]
pub fn prelude(_: proc_macro::TokenStream) -> proc_macro::TokenStream {
   let res = quote! {
      use ascent::*;
      use ascent::aggregators::*;
      use slog_utils::eclass_id;
      use ascent_byods_rels::eqrel;
      use slog_utils::calc_id;
      use slog_utils::collect;
      // use slog_theory::canonicalize;
      // use slog_theory::unification_ds;
      // use slog_theory::theory_propagation;
   };
   res.into()
}

#[proc_macro]
pub fn slog(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
   let res = compile::compile(input.into(), false);

   match res {
      Ok(res) => {
         res.into()
      }
      Err(err) => {
         let err = err.to_compile_error();
         err.into()
      }
   }
}

#[proc_macro]
pub fn slog_par(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
   let res = compile::compile(input.into(), true);

   match res {
      Ok(res) => {
         res.into()
      }
      Err(err) => {
         let err = err.to_compile_error();
         err.into()
      }
   }
}


#[proc_macro]
pub fn local_db(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
   util::share_db_impl(input, false)
}

#[proc_macro]
pub fn share_db(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
   util::share_db_impl(input, true)
}

#[proc_macro]
pub fn slog_gen(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
   let res = util::slog_gen_impl(input);

   match res {
      Ok(res) => {
         res.into()
      },
      Err(err) => {
         let err = err.to_compile_error();
         err.into()
      }
   }
}

#[proc_macro]
pub fn slog_source(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
   let res = util::slog_source_impl(input);

   match res {
      Ok(res) => {
         res.into()
      },
      Err(err) => {
         let err = err.to_compile_error();
         err.into()
      }
   }
}
