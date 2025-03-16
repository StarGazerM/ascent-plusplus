#[deny(warnings)]
use std::collections::HashSet;

use itertools::Itertools;
use proc_macro2::Span;
use syn::{parse_quote_spanned, Expr, Ident};

use crate::ascent_mir::AscentMir;
use crate::{
   ascent_hir::IrRelation,
   ascent_syntax::RelationIdentity,
   utils::{tuple_spanned, TokenStreamExtensions},
};

use super::util::{index_get_entry_val_for_insert, rel_ind_common_var_name};

fn compile_update_indices_relation_code(
   r: &RelationIdentity, indices_set: &HashSet<IrRelation>, par: bool, index_insert_fn: &proc_macro2::TokenStream,
   rel_index_write_trait: &proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
   let _ref = if !par {
      quote! {&mut}
   } else {
      quote! {&}
   }
   .with_span(r.name.span());
   let ind_common = rel_ind_common_var_name(r);
   let rel_index_write_trait = rel_index_write_trait.clone().with_span(r.name.span());
   let _self = quote_spanned! {r.name.span().resolved_at(Span::call_site())=> self };
   let to_rel_index_fn = if !par {
      quote! {to_rel_index_write}
   } else {
      quote! {to_c_rel_index_write}
   };
   let to_rel_index = if r.is_lattice {
      quote! {}
   } else {
      quote! {.#to_rel_index_fn(#_ref #_self.runtime_total.#ind_common) }
   };

   let mut update_indices = vec![];
   for ind in indices_set.iter().sorted_by_cached_key(|rel| rel.ir_name()) {
      let ind_name = &ind.ir_name();
      let selection_tuple: Vec<Expr> = ind
         .indices
         .iter()
         .map(|&i| {
            let ind = syn::Index::from(i);
            parse_quote_spanned! {r.name.span()=> tuple.#ind.clone()}
         })
         .collect_vec();
      let selection_tuple = tuple_spanned(&selection_tuple, r.name.span());
      let entry_val = index_get_entry_val_for_insert(
         ind,
         &parse_quote_spanned! {r.name.span()=> tuple},
         &parse_quote_spanned! {r.name.span()=> _i},
      );
      let _pre_ref = if r.is_lattice { quote!() } else { _ref.clone() };
      update_indices.push(quote_spanned! {r.name.span()=>
         let selection_tuple = #selection_tuple;
         let rel_ind = #_ref #_self.runtime_total.#ind_name;
         #rel_index_write_trait::#index_insert_fn(#_pre_ref rel_ind #to_rel_index, selection_tuple, #entry_val);
      });
   }

   let rel_name = &r.name;
   let maybe_lock = if r.is_lattice && par {
      quote_spanned! {r.name.span()=> let tuple = tuple.read().unwrap(); }
   } else {
      quote! {}
   };
   if !par {
      quote_spanned! {r.name.span()=>
         for (_i, tuple) in #_self.#rel_name.iter().enumerate() {
            #maybe_lock
            #(#update_indices)*
         }
      }
   } else {
      quote_spanned! {r.name.span()=>
         (0..#_self.#rel_name.len()).into_par_iter().for_each(|_i| {
            let tuple = &#_self.#rel_name[_i];
            #maybe_lock
            #(#update_indices)*
         });
      }
   }
}

fn update_indices_relation_func_name(r: &RelationIdentity) -> Ident {
   Ident::new(&format!("update_indices_{}", r.name), r.name.span())
}

fn compile_update_indices_relation_function(
   r: &RelationIdentity, indices_set: &HashSet<IrRelation>, mir: &AscentMir,
) -> proc_macro2::TokenStream {
   let par = mir.is_parallel;
   let mut res = vec![];
   if par {
      res.push(quote! { use ascent::rayon::iter::{IntoParallelIterator, ParallelIterator}; })
   }
   let (rel_index_write_trait, index_insert_fn) = if !par {
      (quote! {ascent::internal::RelIndexWrite}, quote! {index_insert})
   } else {
      (quote! {ascent::internal::CRelIndexWrite}, quote! {index_insert})
   };
   res.push(compile_update_indices_relation_code(r, indices_set, par, &index_insert_fn, &rel_index_write_trait));
   let func_name = update_indices_relation_func_name(r);
   let func_body = quote! {
      use ascent::internal::ToRelIndex0;
      use #rel_index_write_trait;
      #(#res)*
   };
   quote! {
      #[allow(noop_method_call, suspicious_double_ref_op)]
      pub fn #func_name(&mut self) {
         #func_body
      }
   }
}

pub fn compile_update_indices_function(mir: &AscentMir) -> proc_macro2::TokenStream {
   let mut update_calls = vec![];
   let mut update_funcs = vec![];

   let sorted_relations_ir_relations = mir.relations_ir_relations.iter().sorted_by_key(|(rel, _)| &rel.name);
   for (r, indices_set) in sorted_relations_ir_relations {
      if r.extern_db_name.is_some() {
         continue;
      }
      let func_name = update_indices_relation_func_name(r);
      update_calls.push(quote! { self.#func_name(); });
      update_funcs.push(compile_update_indices_relation_function(r, indices_set, mir));
   }

   quote! {
      #[allow(noop_method_call, suspicious_double_ref_op)]
      pub fn update_indices_priv(&mut self) {
         let before = ::ascent::internal::Instant::now();
         #(#update_calls)*
         self.update_indices_duration += before.elapsed();
      }

      #(#update_funcs)*
   }
}
