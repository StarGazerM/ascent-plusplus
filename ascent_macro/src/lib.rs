#![allow(clippy::useless_format, clippy::redundant_static_lifetimes, clippy::get_first)]
#![cfg_attr(not(test), deny(unused_crate_dependencies))]
mod tests;
mod ascent_mir;
mod utils;
mod ascent_hir;
mod scratchpad;
mod codegen;
mod ascent_syntax;
mod ascent_sugar;
mod ascent_monotonic;
mod test_errors;
mod syn_utils;

#[macro_use]
extern crate quote;

extern crate proc_macro;
use ascent_monotonic::ascent_check_monotonicity;
use ascent_syntax::{AscentCall, AscentProgram};
use ascent_sugar::desugar_ascent_program;
use proc_macro::TokenStream;
use syn::{punctuated::Punctuated, Result, Token};
use crate::{codegen::mir2rust::compile_mir, ascent_hir::compile_ascent_program_to_hir, ascent_mir::compile_hir_to_mir};

/// The main macro of the ascent library. Allows writing logical inference rules similar to Datalog.
/// 
/// Example:
/// ```
/// # #[macro_use] extern crate ascent_macro;
/// # use ascent::ascent;
/// ascent!{
///   relation edge(i32, i32);
///   relation path(i32, i32);
///   
///   path(x, y) <-- edge(x,y);
///   path(x, z) <-- edge(x,y), path(y, z);
/// }
/// 
/// fn main() {
///   let mut tc_comp = AscentProgram::default();
///   tc_comp.edge = vec![(1,2), (2,3)];
///   tc_comp.run();
///   println!("{:?}", tc_comp.path);
/// }
/// ```
/// this macro creates a type named `AscentProgram` that can be instantiated using `AscentProgram::default()`.
/// The type has a `run()` method, which runs the computation to a fixed point.
#[proc_macro]
pub fn ascent(input: TokenStream) -> TokenStream {
   let res = ascent_impl(input.into(), false, false);
   
   match res {
      Ok(res) => res.into(),
      Err(err) => TokenStream::from(err.to_compile_error()),
   }
}

/// Similar to `ascent`, allows writing logic programs in Rust.
/// 
/// The difference is that `ascent_par` generates parallelized code. 
#[proc_macro]
pub fn ascent_par(input: TokenStream) -> TokenStream {
   let res = ascent_impl(input.into(), false, true);
   
   match res {
      Ok(res) => res.into(),
      Err(err) => TokenStream::from(err.to_compile_error()),
   }
}


/// Like `ascent`, except that the result of an `ascent_run` invocation is a value containing all the relations
/// defined inside the macro body, and computed to a fixed point.
/// 
/// The advantage of `ascent_run` compared to `ascent` is the fact that `ascent_run` has access to local variables
/// in scope:
/// ```
/// # #[macro_use] extern crate ascent;
/// # use ascent::ascent_run;
/// let r = vec![(1,2), (2,3)];
/// let r_tc = ascent_run!{
///    relation tc(i32, i32);
///    tc(x, y) <-- for (x, y) in r.iter();
///    tc(x, z) <-- for (x, y) in r.iter(), tc(y, z);
/// }.tc;
///
/// ```
#[proc_macro]
pub fn ascent_run(input: TokenStream) -> TokenStream {
   let res = ascent_impl(input.into(), true, false);
   
   match res {
      Ok(res) => res.into(),
      Err(err) => TokenStream::from(err.to_compile_error()),
   }
}

/// The parallelized version of `ascent_run`
#[proc_macro]
pub fn ascent_run_par(input: TokenStream) -> TokenStream {
   let res = ascent_impl(input.into(), true, true);
   
   match res {
      Ok(res) => res.into(),
      Err(err) => TokenStream::from(err.to_compile_error()),
   }
}


/// An empty macro expand to nothing
/// this macro is mainly used to share tokens via `export_tokens` in the `macro_magic` crate
/// you don't need to use this macro directly, but import it and use `ascent_export` instead
#[proc_macro]
pub fn ascent_no_expand(_input: TokenStream) -> TokenStream {
   quote! {}.into()
}

/// Export the tokens of an ascent program to be used in another ascent program
/// This macro will stop expansion of annotated ascent program, but it will just export the
///  tokens of the ascent program with a given name and it can be used in another ascent
///  program using the `ascent_use` and `ascent_uses` macro.
/// Example:
/// ```
/// # #[macro_use] extern crate ascent;
/// # use ascent::{ascent_export, ascent_no_expand};
/// #[ascent_export(Foo)]
/// ascent_no_expand! {
///  relation foo(i32);
///  // ....
/// }
/// 
/// ```
/// 
#[proc_macro_attribute]
pub fn ascent_export(attr: TokenStream, input: TokenStream) -> TokenStream {
   let inp: proc_macro2::TokenStream = input.into();
   let attr: syn::Ident = syn::parse(attr).unwrap();
   quote! {
      #[macro_magic::export_tokens(#attr)]
      #inp
   }.into()
}


use macro_magic::import_tokens_attr;

/// This macro is used to import code from another exported ascent program
/// Example:
/// ```
/// # #[macro_use] extern crate ascent;
/// # use ascent::{ascent_export, ascent_no_expand, ascent_use, ascent_uses};
/// #[ascent_export]
/// ascent!{
/// relation foo(i32);
/// }
/// 
/// #[ascent_use(Foo)]
/// ascent!{
/// relation bar(i32);
/// }
/// 
/// ```
/// 
/// 
#[import_tokens_attr]
#[proc_macro_attribute]
pub fn ascent_use(attr: TokenStream, input: TokenStream) -> TokenStream {
   let ascent_call = syn::parse(input);
   if ascent_call.is_err() {
      return TokenStream::from(ascent_call.unwrap_err().to_compile_error())
   }
   let ascent_call: AscentCall = ascent_call.unwrap();
   let ascent_other_uses = ascent_call.used_paths;
   let ascent_code = ascent_call.code;
   let macro_invocation = ascent_call.ascent_macro;
   let attr_call = syn::parse(attr);
   if attr_call.is_err() {
      return TokenStream::from(attr_call.unwrap_err().to_compile_error())
   }
   let attr_call: AscentCall = attr_call.unwrap();
   let attr_code = attr_call.code;
   quote! {
      #[ascent_uses]
      #macro_invocation! {
         use [#ascent_other_uses];
         #ascent_code
         #attr_code
      }
   }.into()
}


/// This macro is used to import multiple code snippets from other exported ascent programs
/// After annotated a ascent program with this macro, you can use the `use` keyword in the
/// first line of your ascent program to import the exported tokens from other ascent programs
/// Example:
/// ```
/// # #[macro_use] extern crate ascent;
/// # use ascent::{ascent_export, ascent_no_expand, ascent_use, ascent_uses};
/// #[ascent_export(Foo)]
/// ascent!{
/// relation foo(i32);
/// }
/// 
/// #[ascent_export(Bar)]
/// ascent!{
/// relation bar(i32);
/// }
/// 
/// #[ascent_uses]
/// ascent!{
/// use [Foo, Bar];
/// relation baz(i32);
/// }
/// 
/// ```
/// 
#[proc_macro_attribute]
pub fn ascent_uses(_attr: TokenStream ,input: TokenStream) -> TokenStream {
   let ascent_code = syn::parse(input);
   if ascent_code.is_err() {
      return TokenStream::from(ascent_code.unwrap_err().to_compile_error())
   }
   let ascent_code: AscentCall = ascent_code.unwrap();
   if ascent_code.used_paths.len() == 0 {
      let ascent_call = ascent_code.ascent_macro;
      let res = ascent_impl(
         ascent_code.code,
         ascent_call.to_string() == "ascent_run",
         ascent_call.to_string() == "ascent_par");
         if res.is_err() {
            return TokenStream::from(res.unwrap_err().to_compile_error());
         }
      let res = res.unwrap();
      res.into()
   } else {
      let head_path = &ascent_code.used_paths[0];
      let other_paths : Punctuated<syn::ExprPath, Token![,]> = ascent_code.used_paths.iter().skip(1).cloned().collect();
      let macro_invocation = ascent_code.ascent_macro;
      let code = ascent_code.code;
      quote! {
         #[ascent_use(#head_path)]
         #macro_invocation! {
            use [#other_paths];
            #code
         }
      }.into()
   }
}

pub(crate) fn ascent_impl(input: proc_macro2::TokenStream, is_ascent_run: bool, is_parallel: bool) -> Result<proc_macro2::TokenStream> {
   let prog: AscentProgram = syn::parse2(input)?;
   // println!("prog relations: {}", prog.relations.len());
   // println!("parse res: {} relations, {} rules", prog.relations.len(), prog.rules.len());

   let prog = desugar_ascent_program(prog)?;

   ascent_check_monotonicity(&prog)?;
   
   let hir = compile_ascent_program_to_hir(&prog, is_parallel)?;
   // println!("hir relations: {}", hir.relations_ir_relations.keys().map(|r| &r.name).join(", "));

   let mir = compile_hir_to_mir(&hir)?;

   // println!("mir relations: {}", mir.relations_ir_relations.keys().map(|r| &r.name).join(", "));

   let code = compile_mir(&mir, is_ascent_run);

   Ok(code)
}
