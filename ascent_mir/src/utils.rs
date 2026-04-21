//! Minimal helpers used by moved MIR types.

use std::collections::HashSet;
use std::hash::Hash;

use proc_macro2::{Group, Ident, Span, TokenStream, TokenTree};
use syn::spanned::Spanned;
use syn::{Expr, Pat, Type};

pub fn tuple_type(types: &[Type]) -> Type {
   let res = match types.len() {
      1 => {
         let ty = &types[0];
         quote::quote! { ( #ty, ) }
      },
      _ => quote::quote! { ( #(#types),* ) },
   };
   syn::parse2(res).unwrap()
}

pub fn tuple(exprs: &[Expr]) -> Expr {
   use syn::spanned::Spanned;
   let span = if !exprs.is_empty() { exprs[0].span() } else { Span::call_site() };
   tuple_spanned(exprs, span)
}

pub fn tuple_spanned(exprs: &[Expr], span: Span) -> Expr {
   let res = match exprs.len() {
      1 => {
         let exp = &exprs[0];
         quote::quote_spanned! {span=> ( #exp, ) }
      },
      _ => quote::quote_spanned! {span=> ( #(#exprs),* ) },
   };
   syn::parse2(res).unwrap()
}

pub fn is_wild_card(expr: &Expr) -> bool {
   match expr {
      Expr::Infer(_) => true,
      Expr::Verbatim(ts) => ts.to_string() == "_",
      _ => false,
   }
}

pub fn exp_cloned(exp: &Expr) -> Expr {
   let exp_span = exp.span();
   let res = match exp {
      Expr::Path(_) | Expr::Field(_) | Expr::Paren(_) => quote::quote_spanned! {exp_span=> #exp.clone()},
      _ => quote::quote_spanned! {exp_span=> (#exp).clone()},
   };
   syn::parse2(res).unwrap()
}

/// Rewrite every token in `ts` to carry `span`. Used to retarget a whole
/// generated block at a user-source position (for error messages on
/// macro-expanded code).
pub fn with_span(ts: TokenStream, span: Span) -> TokenStream {
   let mut res = TokenStream::new();
   for tt in ts {
      let tt = match tt {
         TokenTree::Group(g) => {
            let mut new_group = Group::new(g.delimiter(), with_span(g.stream(), span));
            new_group.set_span(span);
            TokenTree::Group(new_group)
         },
         mut other => {
            other.set_span(span);
            other
         },
      };
      res.extend(std::iter::once(tt));
   }
   res
}

pub trait TokenStreamExtensions {
   fn with_span(self, span: Span) -> TokenStream;
}

impl TokenStreamExtensions for TokenStream {
   fn with_span(self, span: Span) -> TokenStream { with_span(self, span) }
}

pub fn expr_to_ident(expr: &Expr) -> Option<Ident> {
   match expr {
      Expr::Path(p) => p.path.get_ident().cloned(),
      _ => None,
   }
}

pub fn pat_to_ident(pat: &Pat) -> Option<Ident> {
   match pat {
      Pat::Ident(ident) => Some(ident.ident.clone()),
      _ => None,
   }
}

pub fn collect_set<T: Eq + Hash>(iter: impl Iterator<Item = T>) -> HashSet<T> { iter.collect() }
pub fn into_set<T: Eq + Hash>(iter: impl IntoIterator<Item = T>) -> HashSet<T> { iter.into_iter().collect() }

pub fn pattern_get_vars(pat: &Pat) -> Vec<Ident> {
   let mut res = vec![];
   match pat {
      Pat::Ident(pat_ident) => {
         res.push(pat_ident.ident.clone());
         if let Some(subpat) = &pat_ident.subpat {
            res.extend(pattern_get_vars(&subpat.1))
         }
      },
      Pat::Lit(_) | Pat::Macro(_) | Pat::Path(_) | Pat::Range(_) | Pat::Rest(_) | Pat::Verbatim(_) | Pat::Wild(_) => {},
      Pat::Or(or_pat) => {
         let cases_vars = or_pat.cases.iter().map(pattern_get_vars).map(into_set);
         let intersection = cases_vars.reduce(|case_vars, accu| collect_set(case_vars.intersection(&accu).cloned()));
         if let Some(intersection) = intersection {
            res.extend(intersection);
         }
      },
      Pat::Reference(ref_pat) => res.extend(pattern_get_vars(&ref_pat.pat)),
      Pat::Slice(slice_pat) =>
         for sub_pat in slice_pat.elems.iter() {
            res.extend(pattern_get_vars(sub_pat));
         },
      Pat::Struct(struct_pat) =>
         for field_pat in struct_pat.fields.iter() {
            res.extend(pattern_get_vars(&field_pat.pat));
         },
      Pat::Tuple(tuple_pat) =>
         for elem_pat in tuple_pat.elems.iter() {
            res.extend(pattern_get_vars(elem_pat));
         },
      Pat::TupleStruct(t) =>
         for elem_pat in t.elems.iter() {
            res.extend(pattern_get_vars(elem_pat));
         },
      Pat::Type(type_pat) => res.extend(pattern_get_vars(&type_pat.pat)),
      _ => {},
   }
   res
}

pub fn intersects<T, I1, I2>(set1: I1, set2: I2) -> bool
where
   T: Hash + Eq,
   I1: IntoIterator<Item = T>,
   I2: IntoIterator<Item = T>,
{
   let mut hs = HashSet::default();
   let mut set1 = set1.into_iter();
   for x in set2 {
      if lazy_contains(&mut hs, &mut set1, x) {
         return true;
      }
   }
   false
}

fn lazy_contains<T: Eq + Hash, It: Iterator<Item = T>>(seen: &mut HashSet<T>, src: &mut It, needle: T) -> bool {
   if seen.contains(&needle) {
      return true;
   }
   for x in src.by_ref() {
      let is_match = x == needle;
      seen.insert(x);
      if is_match {
         return true;
      }
   }
   false
}
