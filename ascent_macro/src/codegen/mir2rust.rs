#![deny(warnings)]
use std::collections::HashSet;

use itertools::Itertools;
use proc_macro2::Span;
use syn::spanned::Spanned;
use syn::{Ident, Type};

use crate::ascent_mir::{mir_summary, AscentMir};
use crate::codegen::ds::rel_ds_macro_input;
use crate::codegen::scc::compile_mir_scc;
use crate::codegen::summary::{compile_relation_sizes_body, compile_scc_times_summary_body};
use crate::codegen::update_indices::compile_update_indices_function;
use crate::codegen::util::{
   get_scc_name, lattice_insertion_mutex_var_name, rel_ind_common_type, rel_ind_common_var_name, rel_index_type, rel_type, rule_time_field_name
};

pub(crate) fn compile_mir(mir: &AscentMir, is_ascent_run: bool) -> proc_macro2::TokenStream {
   let par_usings = if mir.is_parallel {
      quote! {
         use ascent::rayon::iter::ParallelBridge;
         use ascent::rayon::iter::ParallelIterator;
         use ascent::internal::CRelIndexRead;
         use ascent::internal::CRelIndexReadAll;
         use ascent::internal::Freezable;
      }
   } else {
      quote! {}
   };

   let more_usings = if !mir.is_parallel {
      quote! {
         use ascent::internal::RelIndexWrite;
      }
   } else {
      quote! {
         use ascent::internal::CRelIndexWrite;
      }
   };

   let run_usings = quote! {
      use core::cmp::PartialEq;
      use ascent::internal::{RelIndexRead, RelIndexReadAll, ToRelIndex0, TupleOfBorrowed};
      #more_usings
      #par_usings
   };

   let mut relation_fields = vec![];
   let mut relation_fields_runtime = vec![];
   let mut field_defaults = vec![];
   let mut field_defaults_runtime = vec![];

   let sorted_relations_ir_relations = mir.relations_ir_relations.iter().sorted_by_key(|(rel, _)| &rel.name);
   for (rel, rel_indices) in sorted_relations_ir_relations {
      if rel.extern_db_name != None {
         continue;
      }
      let name = &rel.name;
      let rel_attrs = &mir.relations_metadata[rel].attributes;
      let sorted_rel_index_names = rel_indices.iter().map(|ind| format!("{}", ind.ir_name())).sorted();
      let rel_indices_comment = format!("\nlogical indices: {}", sorted_rel_index_names.into_iter().join("; "));
      let rel_type = rel_type(rel, mir);
      let rel_ind_common = rel_ind_common_var_name(rel);
      let rel_ind_common_type = rel_ind_common_type(rel, mir);
      relation_fields_runtime.push(quote! {
         #(#rel_attrs)*
         #[doc = #rel_indices_comment]
         pub #name: #rel_type,
         pub #rel_ind_common: #rel_ind_common_type,
      });
      relation_fields.push(quote! {
         #(#rel_attrs)*
         pub #name: #rel_type,
      });
      field_defaults_runtime.push(quote! {#name : Default::default(), #rel_ind_common: Default::default(),});
      field_defaults.push(quote! {#name : Default::default(),});
      if rel.is_lattice && mir.is_parallel {
         let lattice_mutex_name = lattice_insertion_mutex_var_name(rel);
         relation_fields.push(quote! {
            pub #lattice_mutex_name: ::std::vec::Vec<std::sync::Mutex<()>>,
         });
         field_defaults.push(quote! {#lattice_mutex_name: {
            let len = ::ascent::internal::shards_count();
            let mut v = Vec::with_capacity(len); for _ in 0..len {v.push(Default::default())};
            v },
         })
      }
      let sorted_indices = rel_indices.iter().sorted_by_cached_key(|ind| ind.ir_name());
      for ind in sorted_indices {
         let name = &ind.ir_name();
         let rel_index_type = rel_index_type(ind, mir);
         relation_fields_runtime.push(quote! {
            pub #name: #rel_index_type,
         });
         field_defaults_runtime.push(quote! {#name : Default::default(),});
      }
   }

   let sccs_ordered = &mir.sccs;
   let mut rule_time_fields = vec![];
   let mut rule_time_fields_defaults = vec![];
   for i in 0..mir.sccs.len() {
      for (rule_ind, _rule) in mir.sccs[i].rules.iter().enumerate() {
         let name = rule_time_field_name(i, rule_ind);
         rule_time_fields.push(quote! {
            pub #name: std::time::Duration,
         });
         rule_time_fields_defaults.push(quote! {
            #name: std::time::Duration::ZERO,
         });
      }
   }

   let generate_run_timeout = !is_ascent_run && mir.config.generate_run_partial;
   let run_args_decl = if generate_run_timeout {
      quote! {, timeout: ::std::time::Duration}
   } else {
      quote! {}
   };
   let run_args = if generate_run_timeout {
      quote! {timeout}
   } else {
      quote! {}
   };
   let run_extra_prep = if generate_run_timeout {
      quote! {let __start_time = ::ascent::internal::Instant::now();}
   } else {
      quote! {}
   };
   let return_condition = if generate_run_timeout {
      quote! {
         macro_rules! __check_return_conditions {() => {
            if timeout < ::std::time::Duration::MAX && __start_time.elapsed() >= timeout {return false;}
         };}
      }
   } else {
      quote! {
         macro_rules! __check_return_conditions {() => {};}
      }
   };
   let extern_db_args_decl  = mir.extern_dbs.iter()
      .map(|db| {
         let name = &db.db_name;
         let ty = &db.db_type;
         quote!(#name: Rc<RefCell<#ty>>)
      }).collect_vec();
   let extern_db_args = mir.extern_dbs.iter()
      .map(|db| {
         let name = &db.db_name;
         quote!(#name)
      }).collect_vec();
   // generate run arg for every relation
   let input_args_decl = mir.relations_ir_relations.keys()
      .filter(|rel| rel.is_input)
      .map(|rel| {
         let name = &rel.name;
         let ty = rel_type(rel, mir);
         quote!(#name: Rc<RefCell<#ty>>)
      })
      .chain(extern_db_args_decl.into_iter())
      .collect_vec();
   let input_args = mir.relations_ir_relations.keys()
      .filter(|rel| rel.is_input)
      .map(|rel|{
         let name = &rel.name;
         quote!{#name}
      })
      .chain(extern_db_args.into_iter())
      .collect_vec();

   let mut sccs_compiled = vec![];
   let mut sccs_compiled_run = vec![];
   let mut sccs_functions = vec![];
   for (i, _scc) in sccs_ordered.iter().enumerate() {
      let msg = format!("scc {}", i);
      let (scc_pre, scc_loop_body) = compile_mir_scc(mir, i);
      if !is_ascent_run {
         let scc_name = get_scc_name(mir, i);
         let scc_once_name = Ident::new(&format!("{}_exec", scc_name), Span::call_site());
         let scc_func_once = quote! {
            #[allow(unused_assignments, unused_variables, dead_code)]
            pub fn #scc_once_name(&mut self, #(#input_args_decl,)*) -> bool {
               #run_usings
               let _self = self;
               let _scc_start_time = ::ascent::internal::Instant::now();
               #scc_loop_body
               _self.scc_times[#i] += _scc_start_time.elapsed();
               need_break
            }
         };
         let scc_func_body = if mir.sccs[i].is_looping {
            quote! {
               #return_condition
               let _self = self;
               #run_usings
               #scc_pre
               loop {
                  let need_break = _self.#scc_once_name(#(#input_args.clone(),)*);
                  if need_break {break;}
                  __check_return_conditions!();
               }
            }
         } else {
            quote! {
               let _self = self;
               #run_usings
               #scc_pre
               _self.#scc_once_name(#(#input_args,)*);
            }
         };
         sccs_functions.push(scc_func_once);
         let scc_func = quote! {
            #[allow(unused_assignments, unused_variables, dead_code)]
            pub fn #scc_name(&mut self #run_args_decl, #(#input_args_decl,)*) -> bool {
               #run_extra_prep
               ascent::internal::comment(#msg);
               {
                  #scc_func_body
               }
               true
            }
         };
         sccs_functions.push(scc_func);
         let scc_call = quote! {
            let res = _self.#scc_name(#(#input_args.clone(),)* #run_args);
            if !res {return false;}
         };
         sccs_compiled.push(scc_call);
      } else {
         let run_body = if mir.sccs[i].is_looping {
            quote! {
               ascent::internal::comment(#msg);
               {
                  let _scc_start_time = ::ascent::internal::Instant::now();
                  #scc_pre
                  loop {
                     #scc_loop_body
                     if need_break {break;}
                     __check_return_conditions!();
                  }
                  _self.scc_times[#i] += _scc_start_time.elapsed();
               }
            }
         } else {
            quote! {
               ascent::internal::comment(#msg);
               {
                  let _scc_start_time = ::ascent::internal::Instant::now();
                  #scc_pre
                  #scc_loop_body
                  _self.scc_times[#i] += _scc_start_time.elapsed();
               }
            }
         };
         sccs_compiled_run.push(run_body);
      }

      // sccs_compiled.push(quote!{
      //    ascent::internal::comment(#msg);
      //    {
      //       let _scc_start_time = ::ascent::internal::Instant::now();
      //       #scc_compiled
      //       _self.scc_times[#i] += _scc_start_time.elapsed();
      //    }
      // });
   }

   let update_indices_body = compile_update_indices_function(mir);
   let relation_sizes_body = compile_relation_sizes_body(mir);
   let scc_times_summary_body = compile_scc_times_summary_body(mir);

   let mut type_constraints = vec![];
   let mut field_type_names = HashSet::<String>::new();
   let mut lat_field_type_names = HashSet::<String>::new();

   for relation in mir.relations_ir_relations.keys().sorted_by_key(|rel| &rel.name) {
      use crate::quote::ToTokens;
      for (i, field_type) in relation.field_types.iter().enumerate() {
         let is_lat = relation.is_lattice && i == relation.field_types.len() - 1;
         let add = if let Type::Path(path) = field_type {
            let container = if is_lat { &mut lat_field_type_names } else { &mut field_type_names };
            container.insert(path.path.clone().into_token_stream().to_string())
         } else {
            true
         };
         if add {
            let type_constraints_type = if is_lat {
               quote_spanned!(field_type.span()=>LatTypeConstraints)
            } else {
               quote_spanned!(field_type.span()=>TypeConstraints)
            };
            type_constraints.push(quote_spanned! {field_type.span()=>
               let _type_constraints : ascent::internal::#type_constraints_type<#field_type>;
            });
            if mir.is_parallel {
               type_constraints.push(quote_spanned! {field_type.span()=>
                  let _par_constraints : ascent::internal::ParTypeConstraints<#field_type>;
               });
            }
         }
      }
   }

   let mut relation_initializations = vec![];
   for (rel, md) in mir.relations_metadata.iter().sorted_by_key(|(rel, _)| &rel.name) {
      if let Some(ref init) = md.initialization {
         let rel_name = &rel.name;
         relation_initializations.push(quote! {
            _self.#rel_name = #init;
         });
      }
   }
   if !relation_initializations.is_empty() {
      relation_initializations.push(quote! {
         _self.update_indices_priv();
      })
   }

   let swap_input_with_placeholder = mir.relations_ir_relations.keys()
      .filter(|rel| rel.is_input)
      .map(|rel| {
         let name = &rel.name;
         quote! {
            std::mem::swap(&mut _self.#name, #name);
         }
      }).collect_vec();
   
   let mut clear_in_stmts = vec![];
   let mut clear_out_stmts = vec![];
   for i in mir.io.ins.iter() {
      let rels = &mir.relations_ir_relations[i];
      let name = &i.name;
      for rel in rels.iter() {
         // TODO: add clear function on Index trait
         let ir_name = &rel.ir_name();
         let ir_name = if !rel.is_full_index() {
            quote! {#ir_name.0}
         } else { 
            quote! {#ir_name}
         };
         clear_in_stmts.push(quote! {
            _self.#name.clear();
            _self.runtime_total.#ir_name.clear();
            _self.runtime_new.#ir_name.clear();
            _self.runtime_delta.#ir_name.clear();
         });
      }
   }
   for o in mir.io.outs.iter() {
      let rels = &mir.relations_ir_relations[o];
      let name = &o.name;
      for rel in rels.iter() {
         let ir_name = &rel.ir_name();
         let ir_name = if !rel.is_full_index() {
            quote! {#ir_name.0}
         } else { 
            quote! {#ir_name}
         };
         clear_out_stmts.push(quote! {
            _self.#name.clear();
            _self.runtime_total.#ir_name.clear();
            _self.runtime_new.#ir_name.clear();
            _self.runtime_delta.#ir_name.clear();
         });
      }
   }

   let run_func = if is_ascent_run {
      quote! {}
   } else if generate_run_timeout {
      quote! {
         #[doc = "Runs the Ascent program to a fixed point."]
         pub fn run(&mut self, #(#input_args_decl,)* #run_args_decl) -> bool {
            self.run_timeout(#(#input_args,)* ::std::time::Duration::MAX)
         }
      }
   } else {
      quote! {
         #[allow(unused_imports, noop_method_call, suspicious_double_ref_op)]
         #[doc = "Runs the Ascent program to a fixed point."]
         pub fn run(&mut self, #(#input_args_decl,)*) -> bool {
            self.run_with_init_flag(true, #(#input_args,)*)
         }

         pub fn run_with_init_flag(&mut self, init_flag: bool,#(#input_args_decl,)*) -> bool {
            // macro_rules! __check_return_conditions {() => {};}
            // #run_usings
            let _self = self;
            #(#clear_out_stmts)*
            #(#swap_input_with_placeholder)*
            if init_flag { _self.update_indices_priv() };
            #(#sccs_compiled)*
            #(#swap_input_with_placeholder)*
            #(#clear_in_stmts)*
            true
         }
      }
   };
   let run_timeout_func = if !generate_run_timeout {
      quote! {}
   } else {
      quote! {
         #[allow(unused_imports, noop_method_call, suspicious_double_ref_op)]
         #[doc = "Runs the Ascent program to a fixed point or until the timeout is reached. In case of a timeout returns false"]
         pub fn run_timeout(&mut self, #(#input_args_decl,)* #run_args_decl) -> bool {
            let __start_time = ::ascent::internal::Instant::now();
            // #run_usings
            let _self = self;
            #(#clear_out_stmts)*
            #(#swap_input_with_placeholder)*
            _self.update_indices_priv();
            #(#sccs_compiled)*
            #(#swap_input_with_placeholder)*
            true
         }
      }
   };
   let run_code = if !is_ascent_run {
      quote! {}
   } else {
      quote! {
         macro_rules! __check_return_conditions {() => {};}
         #run_usings
         let _self = &mut __run_res;
         #(#clear_out_stmts)*
         #(#relation_initializations)*
         #(#sccs_compiled_run)*
         #(#clear_in_stmts)*
      }
   };

   let relation_initializations_for_default_impl = if is_ascent_run { vec![] } else { relation_initializations };

   let summary = mir_summary(mir);

   let (ty_impl_generics, ty_ty_generics, ty_where_clause) = mir.signatures.split_ty_generics_for_impl();
   let (impl_impl_generics, impl_ty_generics, impl_where_clause) = mir.signatures.split_impl_generics_for_impl();

   let ty_signature = &mir.signatures.declaration;
   if let Some(impl_signature) = &mir.signatures.implementation {
      assert_eq!(ty_signature.ident, impl_signature.ident, "The identifiers of struct and impl must match");
   }

   let ty_ty_generics_str = quote!(#ty_ty_generics).to_string();
   let impl_ty_generics_str = quote!(#impl_ty_generics).to_string();
   assert_eq!(
      ty_ty_generics_str, impl_ty_generics_str,
      "The generic parameters of struct ({ty_ty_generics_str}) and impl ({impl_ty_generics_str}) must match"
   );

   let vis = &ty_signature.visibility;
   let struct_name = &ty_signature.ident;
   let runtime_struct_name = Ident::new(&format!("{}Runtime", struct_name), Span::call_site());
   let struct_attrs = &ty_signature.attrs;
   let summary_fn = if is_ascent_run {
      quote! {
         pub fn summary(&self) -> &'static str {#summary}
      }
   } else {
      quote! {
         pub fn summary() -> &'static str {#summary}
      }
   };
   let rule_time_fields = if mir.config.include_rule_times { rule_time_fields } else { vec![] };
   let rule_time_fields_defaults = if mir.config.include_rule_times { rule_time_fields_defaults } else { vec![] };

   let mut rel_codegens = vec![];
   for rel in mir.relations_ir_relations.keys() {
      let macro_path = &mir.relations_metadata[rel].ds_macro_path;
      let macro_input = rel_ds_macro_input(rel, mir);
      rel_codegens.push(quote_spanned! { macro_path.span()=> #macro_path::rel_codegen!{#macro_input} });
   }

   // generate shared pointer for all external database

   let sccs_count = sccs_ordered.len();
   let res = quote! {
      #(#rel_codegens)*

      #(#struct_attrs)*
      #vis struct #struct_name #ty_impl_generics #ty_where_clause {
         #(#relation_fields)*
         scc_times: [std::time::Duration; #sccs_count],
         scc_iters: [usize; #sccs_count],
         #(#rule_time_fields)*
         pub update_time_nanos: std::sync::atomic::AtomicU64,
         pub update_indices_duration: std::time::Duration,
         pub runtime_total: #runtime_struct_name #ty_ty_generics,
         pub runtime_new: #runtime_struct_name #ty_ty_generics,
         pub runtime_delta: #runtime_struct_name #ty_ty_generics,

         // #(#external_dbs_decl)*
      }
      #vis struct #runtime_struct_name #ty_impl_generics #ty_where_clause  {
         #(#relation_fields_runtime)*
      }
      impl #impl_impl_generics #struct_name #impl_ty_generics #impl_where_clause {
         #(#sccs_functions)*

         #run_func

         #run_timeout_func
         // TODO remove pub update_indices at some point
         #update_indices_body

         #[deprecated = "Explicit call to update_indices not required anymore."]
         pub fn update_indices(&mut self) {
            self.update_indices_priv();
         }
         fn type_constraints() {
            #(#type_constraints)*
         }
         #summary_fn

         pub fn relation_sizes_summary(&self) -> String {
            #relation_sizes_body
         }
         pub fn scc_times_summary(&self) -> String {
            #scc_times_summary_body
         }
      }

      impl #impl_impl_generics Default for #runtime_struct_name #impl_ty_generics #impl_where_clause {
         fn default() -> Self {
            let mut _self = #runtime_struct_name {
               #(#field_defaults_runtime)*
            };
            _self
         }
      }
      impl #impl_impl_generics Default for #struct_name #impl_ty_generics #impl_where_clause {
         fn default() -> Self {
            let mut _self = #struct_name {
               #(#field_defaults)*
               scc_times: [std::time::Duration::ZERO; #sccs_count],
               scc_iters: [0; #sccs_count],
               #(#rule_time_fields_defaults)*
               update_time_nanos: Default::default(),
               update_indices_duration: std::time::Duration::default(),
               runtime_total: Default::default(),
               runtime_new: Default::default(),
               runtime_delta: Default::default(),

               // #(#extern_db_default)*
            };
            #(#relation_initializations_for_default_impl)*
            _self
         }
      }
   };
   if !is_ascent_run {
      res
   } else {
      quote! {
         {
            #res
            let mut __run_res: #struct_name #ty_ty_generics = #struct_name::default();
            #[allow(unused_imports, noop_method_call, suspicious_double_ref_op)]
            {
               ascent::internal::comment("running...");
               #run_code
            }
            __run_res
         }
      }
   }
}
