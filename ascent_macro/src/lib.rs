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
use syn::Result;
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


/// An empty macro just used to keep the code
#[proc_macro]
pub fn export_ascent(_input: TokenStream) -> TokenStream {
   quote! {}.into()
}


use macro_magic::import_tokens_attr;
/// Import an external database relation declaration into the ascent program.
#[import_tokens_attr]
#[proc_macro_attribute]
pub fn ascent_use(attr: TokenStream, input: TokenStream) -> TokenStream {

   // let ascent_macro_call = syn::parse_macro_input!(input as syn::Macro);
   // let call_name = ascent_macro_call.path;
   // let ascent_code = ascent_macro_call.tokens;
   // let export_marco_call = syn::parse_macro_input!(attr as syn::Macro);
   // let extern_db_code = export_marco_call.tokens;
   let ascent_code: AscentCall = syn::parse(input).unwrap();
   let run_code = ascent_code.code;
   let ascent_call = ascent_code.ascent_macro;
   let ext_code : AscentCall = syn::parse(attr).unwrap();
   let used_code = ext_code.code;
   // print ascent_code to error
   // if ascent_call is ascent
   let par = ascent_call.to_string() == "ascent_par";
   
   // insert ext_code after the frist ";"
   // let run_code = run_code.to_string();
   // let insert_pos = run_code.find(";");
   // let run_code = run_code[..insert_pos.unwrap()+1].to_string() + &used_code.to_string() + &run_code[insert_pos.unwrap()+1..];
   // // string to tokenstream
   // let run_code = syn::parse_str(&run_code).unwrap();

   // let res = ascent_impl(run_code, ascent_call.to_string() == "ascent_run", par);
   let res = ascent_impl(
      run_code.into_iter().chain(used_code.into_iter()).collect(),
      ascent_call.to_string() == "ascent_run", par);
   
   match res {
      Ok(res) => res.into(),
      Err(err) => TokenStream::from(err.to_compile_error()),
   }
   
   // quote! {
   //    #ascent_call! {
   //       #run_code
   //       #used_code
   //    }
   // }.into()
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
