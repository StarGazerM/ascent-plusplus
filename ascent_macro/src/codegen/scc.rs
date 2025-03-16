#![deny(warnings)]
use itertools::{Either, Itertools};
use crate::ascent_mir::{mir_rule_summary, MirBodyItem, MirRelationVersion::*, MirScc};

use crate::codegen::rule::compile_mir_rule;
use crate::codegen::util::{expr_for_rel, rule_time_field_name};
use crate::{ascent_mir::{ir_relation_version_var_name, AscentMir, MirRelation, MirRelationVersion}, codegen::util::{expr_for_rel_write, rel_ind_common_type, rel_ind_common_var_name, rel_index_type}};


pub fn compile_mir_scc(mir: &AscentMir, scc_ind: usize) -> 
   (proc_macro2::TokenStream, proc_macro2::TokenStream) {

   let scc = &mir.sccs[scc_ind];
   let mut move_total_to_delta = vec![];
   let mut shift_delta_to_total_new_to_delta = vec![];
   // let mut move_total_to_field = vec![];
   let mut freeze_code = vec![];
   // let mut def_relation_cnt = vec![];
   let mut unfreeze_code = vec![];

   let _self = quote! { _self };

   use std::iter::once;
   let sorted_dynamic_relations = scc.dynamic_relations.iter()
      .filter(|(rel, _)| rel.extern_db_name.is_none())
      .sorted_by_cached_key(|(rel, _)| rel.name.clone());
   for rel in sorted_dynamic_relations.flat_map(|(rel, indices)| {
      once(Either::Left(rel)).chain(indices.iter().sorted_by_cached_key(|rel| rel.ir_name()).map(Either::Right))
   }) {
      let (ir_name, _ty) = match rel {
        Either::Left(rel) => (rel_ind_common_var_name(rel), rel_ind_common_type(rel, mir)),
        Either::Right(rel_ind) => (rel_ind.ir_name(), rel_index_type(rel_ind, mir)),
      };
      let delta_var_name = ir_relation_version_var_name(&ir_name, &_self, MirRelationVersion::Delta);
      let total_var_name = ir_relation_version_var_name(&ir_name, &_self, MirRelationVersion::Total);
      let new_var_name = ir_relation_version_var_name(&ir_name, &_self, MirRelationVersion::New);
      // let counter_name = ir_relation_counter(&ir_name);
      let total_field = &ir_name;
      // move_total_to_delta.push(quote_spanned! {ir_name.span()=>
      //    let mut #delta_var_name: #ty = ::std::mem::take(&mut #_self.#total_field);
      //    let mut #total_var_name : #ty = Default::default();
      //    let mut #new_var_name : #ty = Default::default();
      // });
      move_total_to_delta.push(quote_spanned! {ir_name.span()=>
         #delta_var_name = ::std::mem::take(&mut #_self.runtime_total.#total_field);
         #total_var_name = Default::default();
         #new_var_name = Default::default();
      });

      match rel {
         Either::Left(rel_ind_common) => {
            if rel_ind_common.extern_db_name.is_some() {
               continue;
            }
            shift_delta_to_total_new_to_delta.push(quote_spanned!{ir_name.span()=>
               ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(&mut #new_var_name, &mut #delta_var_name, &mut #total_var_name);
            });
            move_total_to_delta.push(quote_spanned! {ir_name.span()=>
               ::ascent::internal::RelIndexMerge::init(&mut #new_var_name, &mut #delta_var_name, &mut #total_var_name);
            });
         },
         Either::Right(ir_rel) => {
            if ir_rel.relation.extern_db_name.is_some() {
               continue;
            }
            let delta_expr = expr_for_rel_write(&MirRelation::from(ir_rel.clone(), Delta), mir);
            let total_expr = expr_for_rel_write(&MirRelation::from(ir_rel.clone(), Total), mir);
            let new_expr = expr_for_rel_write(&MirRelation::from(ir_rel.clone(), New), mir);

            shift_delta_to_total_new_to_delta.push(quote_spanned!{ir_name.span()=>
               ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(&mut #new_expr, &mut #delta_expr, &mut #total_expr);
            });
            move_total_to_delta.push(quote_spanned! {ir_name.span()=>
               ::ascent::internal::RelIndexMerge::init(&mut #new_expr, &mut #delta_expr, &mut #total_expr);
            });
         },
      }
      
      // move_total_to_field.push(quote_spanned!{ir_name.span()=>
      //    #_self.#total_field = std::mem::take(&mut #total_var_name);
      // });

      if mir.is_parallel {
         freeze_code.push(quote_spanned!{ir_name.span()=>
            #total_var_name.freeze();
            #delta_var_name.freeze();
         });

         unfreeze_code.push(quote_spanned!{ir_name.span()=>
            #total_var_name.unfreeze();
            #delta_var_name.unfreeze();
         });
      }

      // if mir.is_parallel {
      //    def_relation_cnt.push(quote_spanned!{ir_name.span()=>
      //       let #counter_name = std::sync::atomic::AtomicU64::new(#_self.#total_field.len() as u64);
      //    });
      // } else {
      //    def_relation_cnt.push(quote_spanned!{ir_name.span()=>
      //       let mut #counter_name = #_self.#total_field.len();
      //    });
      
      // }
   }
   let sorted_body_only_relations = scc.body_only_relations.iter()
      .filter(|(rel, _)| rel.extern_db_name.is_none())
      .sorted_by_cached_key(|(rel, _)| rel.name.clone());
   for rel in sorted_body_only_relations.flat_map(|(rel, indices)| {
      let sorted_indices = indices.iter().sorted_by_cached_key(|rel| rel.ir_name());
      once(Either::Left(rel)).chain(sorted_indices.map(Either::Right))
   }) {
      let (ir_name, _ty) = match rel {
         Either::Left(rel) => (rel_ind_common_var_name(rel), rel_ind_common_type(rel, mir)),
         Either::Right(rel_ind) => (rel_ind.ir_name(), rel_index_type(rel_ind, mir)),
      };
      // let total_var_name = ir_relation_version_var_name(&ir_name, &_self, MirRelationVersion::Total);
      let total_field = &ir_name;

      if mir.is_parallel {
         move_total_to_delta.push(quote_spanned!{ir_name.span()=>
            #_self.runtime_total.#total_field.freeze();
         });
      }

      // move_total_to_delta.push(quote_spanned! {ir_name.span()=>
      //    let #total_var_name: #ty = std::mem::take(&mut #_self.#total_field);
      // });
      // move_total_to_delta.push(quote_spanned! {ir_name.span()=>
      //    #total_var_name = std::mem::take(&mut #_self.#total_field);
      // });

      // move_total_to_field.push(quote_spanned!{ir_name.span()=>
      //    #_self.#total_field = std::mem::take(&mut #total_var_name);
      // });
   }
   
   let rule_parallelism = mir.config.inter_rule_parallelism && mir.is_parallel;

   let mut evaluate_rules = vec![];

   for (i, rule) in scc.rules.iter().enumerate() {
      let msg = mir_rule_summary(rule);
      let rule_compiled = compile_mir_rule(rule, scc, mir);
      let rule_time_field = rule_time_field_name(scc_ind, i);
      let (before_rule_var, update_rule_time_field) = if mir.config.include_rule_times {
         (quote! {let before_rule = ::ascent::internal::Instant::now();}, 
          quote!{_self.#rule_time_field += before_rule.elapsed();})
      } else {(quote!{}, quote!{})};
      let size_check_code_list = rule.body_items.iter().filter_map(|bi| {
         if let MirBodyItem::Clause(clause) = bi {
            if clause.rel.version == MirRelationVersion::Delta {
            let rel = expr_for_rel(&clause.rel, &clause.extern_db_name, mir);
               Some(quote! {
                  #rel.len() > 0
               })
            } else {None}
         } else {None}
      });
      // connect the size check code with logical and
      let size_check_code = size_check_code_list.reduce(|a, b| quote!{#a && #b}).unwrap_or(quote!{true});
      evaluate_rules.push(if rule_parallelism { quote! {
         ascent::internal::comment(#msg);
         if #size_check_code {
            __scope.spawn(|_| {
               #before_rule_var
               #rule_compiled
               #update_rule_time_field
            });
         }
      }} else { quote! {
         #before_rule_var
         ascent::internal::comment(#msg);
         if #size_check_code {
            #rule_compiled
         }
         #update_rule_time_field
      }});
   }
   
   let evaluate_rules = if rule_parallelism {
      quote! {
         ascent::rayon::scope(|__scope| {
            #(#evaluate_rules)*
         }); 
      }
   } else {
      quote! { #(#evaluate_rules)* }
   };

   let eval_ext_dbs = generated_ext_dbs(scc, mir);

   let changed_var_def_code = if !mir.is_parallel {
      quote! { let mut __changed = false; }
   } else {
      quote! { let __changed = std::sync::atomic::AtomicBool::new(false); }
   };
   let check_changed_code = if !mir.is_parallel {
      quote! {__changed}
   } else {
      quote! {__changed.load(std::sync::atomic::Ordering::Relaxed)}
   };

   let eval_once = if scc.is_looping {
      quote! {
         #changed_var_def_code

         #(#freeze_code)*
         // evaluate rules
         #evaluate_rules

         #(#unfreeze_code)*
         #(#shift_delta_to_total_new_to_delta)*
         #(#eval_ext_dbs)*
         _self.scc_iters[#scc_ind] += 1;
         // if !#check_changed_code {break;}
         let need_break = !#check_changed_code;
         // __check_return_conditions!();
      }
   } else {
      quote! {
         // let mut __changed = false;
         #changed_var_def_code
         let mut __default_id = 0;
         #(#freeze_code)*

         #evaluate_rules

         #(#unfreeze_code)*

         #(#shift_delta_to_total_new_to_delta)*
         #(#shift_delta_to_total_new_to_delta)*
         #(#eval_ext_dbs)*
         _self.scc_iters[#scc_ind] += 1;
         let need_break = true;
         // __check_return_conditions!();
      }
   };
   // quote! {
   //    // define variables for delta and new versions of dynamic relations in the scc
   //    // move total versions of dynamic indices to delta
   //    #(#move_total_to_delta)*

   //    #evaluate_rules_loop

   //    #(#move_total_to_field)*
   // }
   (quote! {#(#move_total_to_delta)*},
    quote! {#eval_once})
}

fn generated_ext_dbs(scc: &MirScc, mir: &AscentMir) -> Vec<proc_macro2::TokenStream> {
   let used_db_set = scc.dynamic_relations.iter().filter_map(|(rel, _)| rel.extern_db_name.clone()).collect::<std::collections::HashSet<_>>();
   used_db_set.iter().map(
      |ext_db_name| {
         let ext_db = mir.extern_dbs.iter().find(|db| db.db_name == *ext_db_name).unwrap();
         let args = &ext_db.db_args;
         quote! {
            #ext_db_name.borrow_mut().run(
               #(#args.clone()),*
            );
         }
      }
   ).collect_vec()
}
