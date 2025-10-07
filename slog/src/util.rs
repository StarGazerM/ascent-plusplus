use proc_macro2::{Group, Span, TokenStream, TokenTree};
use rand::Rng;
use syn::{braced, parenthesized, parse::{Parse, ParseBuffer, ParseStream}, parse2, punctuated::Punctuated, Ident, Result, Token};
use quote::{quote, quote_spanned};

use crate::{compile::compile, syntax::{kw_slog, ShareDbInput, SlogGenInput, SlogSourceInput, SlogType}};

// create a new ident with the given name + a random suffix
pub fn new_ident(name: &str) -> Ident {
   let suffix = rand::rng().random_range(0..1000000);
   Ident::new(&format!("{}_{}", name, suffix), Span::call_site())
}


pub fn share_db_impl(input: proc_macro::TokenStream, local_scope: bool) -> proc_macro::TokenStream {
   let ShareDbInput { db_name, theory, content, kont_macro } = parse2(input.into()).unwrap();
   let theory_code = if theory.uses.is_empty() {
      quote! {}
   } else {
      quote! {
         #theory
      }
   };
   let rel_decls = content
      .iter()
      .map(|rel_decl| {
         let rel_name = rel_decl.rel_name.clone();
         let arg_types = rel_decl
            .arg_types
            .iter()
            .map(|arg_type| match arg_type {
               SlogType::Theory(th_name) => {
                  quote! { @#th_name }
               },
               SlogType::Rust(ty) => {
                  quote! { #ty }
               },
            })
            .collect::<Vec<_>>();
         quote! {
            (define #rel_name #(#arg_types)*)
         }
      })
      .collect::<Vec<_>>();
   let rel_decls_par = rel_decls.clone();
   let pipe_ident = Ident::new(&format!("pipe_{}", db_name), db_name.span());
   let rel_name_assigns = content
      .iter()
      .map(|rel_decl| {
         let rel_name = rel_decl.rel_name.clone();
         quote_spanned! { rel_decl.rel_name.span() =>
            $to.#rel_name = $from.#rel_name;
         }
      })
      .collect::<Vec<_>>();

   let theory_name_assigns = theory.uses.iter().map(|th| {
      let rel_ind_common_var_name = Ident::new(&format!("__{}_ind_common", th.unify_rel), th.unify_rel.span());
      quote_spanned! { th.unify_rel.span() =>
         $to.#rel_ind_common_var_name = $from.#rel_ind_common_var_name;
      }
   }).collect::<Vec<_>>();

   // let  theory
   let export_code = if local_scope {
      quote! {
         #[macro_export]
      }
   } else {
      quote! {}
   };
   // let db_code_name = Ident::new(&format!("{}Code", db_name), db_name.span());
   quote! {
      #export_code
      macro_rules! #db_name {
         ($name:ident, { $($x:tt)* }) => {
             #kont_macro!($name, {
                 #theory_code
                 #(#rel_decls)*
             }, {
                $($x)*
             }, slog)
         };
         ($name:ident, par, { $($x:tt)* }) => {
            #kont_macro!($name, {
                #theory_code
                #(#rel_decls_par)*
            }, {
                $($x)*
            }, slog_par)
        };
     }
     
     #export_code
     macro_rules! #pipe_ident {
        ($from:ident, $to:ident) => {
            #(#rel_name_assigns)*
            #(#theory_name_assigns)*
        };
     }
   }
   .into()
}

pub fn slog_gen_impl(input: proc_macro::TokenStream) -> Result<TokenStream> {
   let SlogGenInput { struct_name, prev_code, new_code, slog_macro } = parse2(input.into()).unwrap();
   // let prev_code = unify_span(prev_code,  new_code.span());
   let code = quote! {
      // #slog_macro! {
         (struct #struct_name)
         #prev_code
         #new_code
      // }
   };
   compile(code.into(), false)
   // Ok(code.into())
}


pub fn slog_source_impl(input: proc_macro::TokenStream) -> Result<TokenStream> {
   let SlogSourceInput { struct_name, opt_args, content } = parse2(input.into())?;
   let opt_args_tokens = opt_args.iter().map(|arg| quote_spanned! {
      arg.span()=> $#arg:ident
   }).collect::<Vec<_>>();
   Ok(quote! {
      #[macro_export]
      macro_rules! #struct_name {
         ((#(#opt_args_tokens),*), ($used_struct_name:ident), { $macro_call:ident },
          { $($before:tt)* },
          { $($after:tt)* }
         ) => {
            ascent_gen! (
               $used_struct_name,
               { #content $($before)* },
               { #content $($after)* },
               $macro_call
            )
         };
      }
   }
   .into())
}

// Recursively sets the span of every token in a stream to `Span::call_site()`.
pub fn respan_to_call_site(stream: TokenStream) -> TokenStream {
   stream.into_iter().map(|token| {
       match token {
           TokenTree::Group(group) => {
               // The token is a group (e.g., `(...)`, `{...}`).
               // We need to recursively respan the tokens *inside* the group.
               let new_stream = respan_to_call_site(group.stream());
               let mut new_group = Group::new(group.delimiter(), new_stream);
               new_group.set_span(Span::call_site());
               TokenTree::Group(new_group)
           }
           mut other_token => {
               // The token is an Ident, Punct, or Literal.
               // We can just set its span directly.
               other_token.set_span(Span::call_site());
               other_token
           }
       }
   }).collect()
}
