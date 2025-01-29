#![deny(warnings)]
use crate::ascent_hir::IndexValType;
use crate::ascent_mir::{ir_relation_version_var_name, MirRelation, MirRelationVersion};
use crate::codegen::ds::rel_ds_macro_input;
use crate::utils::{tuple, tuple_type};
use crate::{ascent_hir::IrRelation, ascent_mir::AscentMir, ascent_syntax::RelationIdentity};
use itertools::Itertools;
use proc_macro2::{Span, TokenStream};
use syn::{Expr, Ident};
use syn::{parse_quote, parse_quote_spanned, spanned::Spanned, Type};

pub fn rel_ind_common_type(rel: &RelationIdentity, mir: &AscentMir) -> Type {
   if rel.is_lattice {
      parse_quote! { () }
   } else {
      let macro_path = &mir.relations_metadata[rel].ds_macro_path;
      let macro_input = rel_ds_macro_input(rel, mir);
      parse_quote_spanned! { macro_path.span()=> #macro_path::rel_ind_common!(#macro_input) }
   }
}

pub fn rel_index_type(rel: &IrRelation, mir: &AscentMir) -> Type {
   let span = rel.relation.name.span();
   let key_type = rel.key_type();
   let value_type = rel.value_type();

   let is_lat_full_index = rel.relation.is_lattice && &mir.lattices_full_indices[&rel.relation] == rel;

   if rel.relation.is_lattice {
      let res = if !mir.is_parallel {
         if is_lat_full_index {
            quote_spanned! { span=>ascent::internal::RelFullIndexType<#key_type, #value_type> }
         } else {
            quote_spanned! { span=>ascent::internal::LatticeIndexType<#key_type, #value_type> }
         }
      } else {
         // parallel
         if is_lat_full_index {
            quote_spanned! { span=>ascent::internal::CRelFullIndex<#key_type, #value_type> }
         } else if rel.is_no_index() {
            quote_spanned! { span=>ascent::internal::CRelNoIndex<#value_type> }
         } else {
            quote_spanned! { span=>ascent::internal::CRelIndex<#key_type, #value_type> }
         }
      };
      syn::parse2(res).unwrap()
   } else {
      let macro_path = &mir.relations_metadata[&rel.relation].ds_macro_path;
      let span = macro_path.span();
      let macro_input = rel_ds_macro_input(&rel.relation, mir);
      if rel.is_full_index() {
         parse_quote_spanned! {span=> #macro_path::rel_full_ind!(#macro_input, #key_type, #value_type)}
      } else {
         let ind = rel_index_to_macro_input(&rel.indices);
         parse_quote_spanned! {span=> #macro_path::rel_ind!(#macro_input, #ind, #key_type, #value_type)}
      }
   }
}

pub fn rel_type(rel: &RelationIdentity, mir: &AscentMir) -> Type {
   let field_types = tuple_type(&rel.field_types);

   if rel.is_lattice {
      if mir.is_parallel {
         parse_quote! {::ascent::boxcar::Vec<::std::sync::RwLock<#field_types>>}
      } else {
         parse_quote! {::std::vec::Vec<#field_types>}
      }
   } else {
      let macro_path = &mir.relations_metadata[rel].ds_macro_path;
      let macro_input = rel_ds_macro_input(rel, mir);
      parse_quote_spanned! {macro_path.span()=> #macro_path::rel!(#macro_input) }
   }
}

pub fn rel_index_to_macro_input(ind: &[usize]) -> TokenStream {
   let indices = ind.iter().cloned().map(syn::Index::from);
   quote! { [#(#indices),*] }
}

pub fn rule_time_field_name(scc_ind: usize, rule_ind: usize) -> Ident {
   Ident::new(&format!("rule{}_{}_duration", scc_ind, rule_ind), Span::call_site())
}

pub fn get_scc_name(mir: &AscentMir, scc_ind: usize) -> Ident {
   // find if out are in scc contains a stratum relation
   let mut scc_name = Ident::new(&format!("scc_{}", scc_ind), Span::call_site());
   for (rel, _) in &mir.sccs[scc_ind].dynamic_relations {
      if rel.is_hole {
         scc_name = Ident::new(&format!("scc_{}", rel.name), Span::call_site());
      }
   }
   scc_name
}

pub fn lattice_insertion_mutex_var_name(head_relation: &RelationIdentity) -> Ident {
   Ident::new(&format!("__{}_mutex", head_relation.name), head_relation.name.span())
}

pub fn rel_ind_common_var_name(relation: &RelationIdentity) -> Ident {
   Ident::new(&format!("__{}_ind_common", relation.name), relation.name.span())
}


pub fn expr_for_rel(rel: &MirRelation, extern_db_name: &Option<Ident>, mir: &AscentMir) -> proc_macro2::TokenStream {
   fn expr_for_rel_inner(
      ir_name: &Ident, extern_db_name: &Option<Ident>, version: MirRelationVersion, _mir: &AscentMir,
      mir_rel: &MirRelation,
   ) -> (TokenStream, bool) {
      let db = if let Some(db_name) = extern_db_name {
         quote! { #db_name.borrow() }
      } else {
         quote! { _self }
      };
      let var = ir_relation_version_var_name(ir_name, &db, version);
      if mir_rel.relation.is_lattice {
         (quote! { & #var }, true)
      } else {
         let rel_ind_common = ir_relation_version_var_name(&rel_ind_common_var_name(&mir_rel.relation), &db, version);
         (quote! { #var.to_rel_index(& #rel_ind_common) }, false)
      }
   }

   if rel.version == MirRelationVersion::TotalDelta {
      let total_expr = expr_for_rel_inner(&rel.ir_name, extern_db_name, MirRelationVersion::Total, mir, rel).0;
      let delta_expr = expr_for_rel_inner(&rel.ir_name, extern_db_name, MirRelationVersion::Delta, mir, rel).0;
      quote! {
         ascent::internal::RelIndexCombined::new(& #total_expr, & #delta_expr)
      }
   } else {
      let (res, borrowed) = expr_for_rel_inner(&rel.ir_name, extern_db_name, rel.version, mir, rel);
      if !borrowed {
         res
      } else {
         quote! {(#res)}
      }
   }
}

pub fn expr_for_rel_write(mir_rel: &MirRelation, _mir: &AscentMir) -> proc_macro2::TokenStream {
   let var = mir_rel.var_name();
   if mir_rel.relation.is_lattice {
      quote! { #var }
   } else {
      let _self = quote! {_self};
      let rel_ind_common =
         ir_relation_version_var_name(&rel_ind_common_var_name(&mir_rel.relation), &_self, mir_rel.version);
      quote! { #var.to_rel_index_write(&mut #rel_ind_common) }
   }
}

pub fn expr_for_c_rel_write(mir_rel: &MirRelation, _mir: &AscentMir) -> proc_macro2::TokenStream {
   let var = mir_rel.var_name();
   if mir_rel.relation.is_lattice {
      quote! { #var }
   } else {
      let _self = quote! {_self};
      let rel_ind_common =
         ir_relation_version_var_name(&rel_ind_common_var_name(&mir_rel.relation), &_self, mir_rel.version);
      quote! { #var.to_c_rel_index_write(&#rel_ind_common) }
   }
}

pub fn index_get_entry_val_for_insert(rel_ind: &IrRelation, tuple_expr: &Expr, ind_expr: &Expr) -> Expr {
   match &rel_ind.val_type {
      IndexValType::Reference => ind_expr.clone(),
      IndexValType::Direct(inds) => {
         let val = inds
            .iter()
            .map(|&ind| {
               let ind = syn::Index::from(ind);
               parse_quote! {
                  #tuple_expr.#ind.clone()
               }
            })
            .collect_vec();
         tuple(&val)
      }
   }
}
