//! Thin shim: re-export generic syn helpers from `ascent_mir::syn_utils`,
//! plus the proc-macro-only `ResTokenStream2Ext` that can't live in a
//! non-proc-macro crate.

#![deny(warnings)]

pub use ascent_mir::syn_utils::*;

/// Helper trait for turning `syn::Result<TokenStream>` into the proc-macro
/// `TokenStream`, surfacing parse errors as `compile_error!(…)` tokens.
///
/// Lives here (not in `ascent_mir`) because `proc_macro::TokenStream` is
/// only available in proc-macro crates.
pub trait ResTokenStream2Ext {
   fn into_token_stream(self) -> proc_macro::TokenStream;
}

impl ResTokenStream2Ext for syn::Result<proc_macro2::TokenStream> {
   fn into_token_stream(self) -> proc_macro::TokenStream {
      match self {
         Ok(res) => res.into(),
         Err(err) => proc_macro::TokenStream::from(err.to_compile_error()),
      }
   }
}
