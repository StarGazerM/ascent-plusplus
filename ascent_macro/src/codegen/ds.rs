use crate::{ascent_mir::AscentMir, ascent_syntax::RelationIdentity, codegen::util::rel_index_to_macro_input, utils::tuple_type};
use itertools::Itertools;
use proc_macro2::TokenStream;
use syn::{parse_quote_spanned, Ident};


pub fn rel_ds_macro_input(rel: &RelationIdentity, mir: &AscentMir) -> TokenStream {
    let span = rel.name.span();
    let field_types = tuple_type(&rel.field_types);
    let indices = mir.relations_ir_relations[rel]
       .iter()
       .sorted_by_key(|r| &r.indices)
       .map(|ir_rel| rel_index_to_macro_input(&ir_rel.indices));
    let args = &mir.relations_metadata[rel].ds_macro_args;
    let par: Ident = if mir.is_parallel {
       parse_quote_spanned! {span=> par}
    } else {
       parse_quote_spanned! {span=> ser}
    };
    let name = Ident::new(&format!("{}_{}", mir.signatures.declaration.ident, rel.name), span);
    quote! {
       #name,
       #field_types,
       [#(#indices),*],
       #par,
       (#args)
    }
 }
 
