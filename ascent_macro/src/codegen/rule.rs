use std::collections::HashSet;
use std::vec;
use crate::ascent_hir::IrHeadItem;
use crate::utils::{expr_to_ident, tuple_spanned};
use crate::{
   ascent_hir::IndexValType,
   ascent_mir::{
      AscentMir, MirBodyItem,
      MirRelationVersion::{self, *},
      MirRule, MirScc,
   },
   ascent_hir::IrHeadClause,
   utils::{tuple_type, TokenStreamExtensions},
};
use rand::Rng;
use itertools::Itertools;
use syn::{parse2, parse_quote_spanned, spanned::Spanned};
use syn::{parse_quote, Expr, Ident};

use crate::ascent_syntax::{CondClause, EquivClauseNode};
use crate::{
   ascent_mir::{ir_relation_version_var_name, MirRelation},
   utils::{exp_cloned, tuple},
};

use super::util::{
   expr_for_c_rel_write, expr_for_rel, expr_for_rel_write, index_get_entry_val_for_insert,
   lattice_insertion_mutex_var_name,
};

fn compile_cond_clause(cond: &CondClause, body: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
   match cond {
      CondClause::IfLet(if_let_clause) => {
         let pat = &if_let_clause.pattern;
         let expr = &if_let_clause.exp;
         quote_spanned! {if_let_clause.if_keyword.span()=>
            if let #pat = #expr {
               #body
            }
         }
      }
      CondClause::Let(let_clause) => {
         let pat = &let_clause.pattern;
         let expr = &let_clause.exp;
         quote_spanned! {let_clause.let_keyword.span()=>
            let #pat = #expr;
            #body
         }
      }
      CondClause::If(if_clause) => {
         let cond = &if_clause.cond;
         quote_spanned! {if_clause.if_keyword.span()=>
            if #cond {
               #body
            }
         }
      }
   }
}

fn compile_equiv_clause_body(equiv: &EquivClauseNode, body: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
   let src_var = &equiv.left_ident;
   let dst_var = &equiv.right_ident;
   if equiv.representative {
      quote_spanned! {equiv.left_ident.span()=>
         let #dst_var = _self.equiv_ids_.get_dominant_elem(#src_var).unwrap_or(#src_var);
         #body
      }
   } else {
      // gen a random equiv iter_items
      // let iter_items = Ident::new(&format!("__iter_items_{}", rand::thread_rng().gen_range(0..100000)), equiv.left_ident.span());
      quote_spanned! {equiv.left_ident.span()=>
         // let #iter_items: Vec<_> = 
         //    if let Some(set) = _self.equiv_ids_.set_of(#src_var) {
         //        set.cloned().collect()
         //    } else {
         //        vec![#src_var.clone()]
         //    };

        for #dst_var in _self.equiv_ids_.set_of_inc_x(#src_var) {
            #body
        }
      }
   }
}

pub fn compile_mir_rule(rule: &MirRule, scc: &MirScc, mir: &AscentMir) -> proc_macro2::TokenStream {
   let (head_rels_structs_and_vars, head_update_code) = head_clauses_structs_and_update_code(rule, scc, mir);

   const MAX_PAR_ITERS: usize = 2;

   // do parallel iteration up to this clause index (exclusive)
   let par_iter_to_ind = if mir.is_parallel {
      use itertools::FoldWhile::*;
      rule
         .body_items
         .iter()
         .fold_while((0, 0), |(count, i), bi| {
            let new_count = count + matches!(bi, MirBodyItem::Clause(..)) as usize;
            if new_count > MAX_PAR_ITERS {
               Done((new_count, i))
            } else {
               Continue((new_count, i + 1))
            }
         })
         .into_inner()
         .1
   } else {
      0
   };
   let rule_code = compile_mir_rule_inner(rule, scc, mir, par_iter_to_ind, head_update_code, 0);
   quote! {
      #head_rels_structs_and_vars
      #rule_code
   }
}

fn compile_mir_rule_inner(
   rule: &MirRule, _scc: &MirScc, mir: &AscentMir, par_iter_to_ind: usize, head_update_code: proc_macro2::TokenStream,
   clause_ind: usize,
) -> proc_macro2::TokenStream {
   if Some(clause_ind) == rule.simple_join_start_index && rule.reorderable {
      let mut rule_cp1 = rule.clone();
      let mut rule_cp2 = rule.clone();
      rule_cp1.reorderable = false;
      rule_cp2.reorderable = false;
      rule_cp2.body_items.swap(clause_ind, clause_ind + 1);
      let rule_cp1_compiled =
         compile_mir_rule_inner(&rule_cp1, _scc, mir, par_iter_to_ind, head_update_code.clone(), clause_ind);
      let rule_cp2_compiled =
         compile_mir_rule_inner(&rule_cp2, _scc, mir, par_iter_to_ind, head_update_code, clause_ind);

      if let [MirBodyItem::Clause(bcl1), MirBodyItem::Clause(bcl2)] = &rule.body_items[clause_ind..clause_ind + 2] {
         let rel1_var_name = expr_for_rel(&bcl1.rel, &bcl1.extern_db_name, mir);
         let rel2_var_name = expr_for_rel(&bcl2.rel, &bcl2.extern_db_name, mir);

         return quote_spanned! {bcl1.rel_args_span=>
            if #rel1_var_name.len() <= #rel2_var_name.len() {
               #rule_cp1_compiled
            } else {
               #rule_cp2_compiled
            }
         };
      } else {
         panic!("unexpected body items in reorderable rule")
      }
   }
   if clause_ind < rule.body_items.len() {
      let bitem = &rule.body_items[clause_ind];
      let doing_simple_join = rule.simple_join_start_index == Some(clause_ind);
      let next_loop = if doing_simple_join {
         compile_mir_rule_inner(rule, _scc, mir, par_iter_to_ind, head_update_code, clause_ind + 2)
      } else {
         compile_mir_rule_inner(rule, _scc, mir, par_iter_to_ind, head_update_code, clause_ind + 1)
      };

      match bitem {
         MirBodyItem::Clause(bclause) => {
            let (clause_ind, bclause) = if doing_simple_join {
               (clause_ind + 1, rule.body_items[clause_ind + 1].unwrap_clause())
            } else {
               (clause_ind, bclause)
            };

            let mut conds_then_next_loop = next_loop;
            for cond in bclause.cond_clauses.iter().rev() {
               conds_then_next_loop = compile_cond_clause(cond, conds_then_next_loop);
            }

            let bclause_rel_name = &bclause.rel.relation.name;
            let selected_args = &bclause.selected_args();
            let pre_clause_vars =
               rule.body_items.iter().take(clause_ind).flat_map(MirBodyItem::bound_vars).collect::<Vec<_>>();

            let clause_vars = bclause.vars();
            let common_vars = clause_vars.iter().filter(|(_i, v)| pre_clause_vars.contains(v)).collect::<Vec<_>>();
            let common_vars_no_indices = common_vars.iter().map(|(_i, v)| v.clone()).collect::<Vec<_>>();

            let cloning_needed = true;

            let matched_val_ident = Ident::new("__val", bclause.rel_args_span);
            let new_vars_binds_and_next = clause_bind_and_continue(
               &bclause.rel,
               clause_vars.iter().filter(|(_i, var)| !common_vars_no_indices.contains(var)).cloned(),
               &matched_val_ident,
               &parse_quote! {_self.#bclause_rel_name},
               cloning_needed,
               mir,
               conds_then_next_loop.clone(),
            );

            let selected_args_cloned = selected_args.iter().map(exp_cloned).collect_vec();
            let selected_args_cloned_canonicalized = if mir.config.egg_mode && bclause.rel.relation.contains_eclass_id {
               selected_args_cloned.iter().enumerate().map(|(i, arg)| {
                  if bclause.rel.relation.is_field_eclass_id(i) {
                     parse_quote! {_self.equiv_ids_.get_dominant_elem(#arg).unwrap_or(#arg)}
                  } else {
                     arg.clone()
                  }
               }).collect_vec()
            } else {
               selected_args_cloned
            };
            let selected_args_tuple = tuple_spanned(&selected_args_cloned_canonicalized, bclause.args_span);
            let rel_version_exp = expr_for_rel(&bclause.rel, &bclause.extern_db_name, mir);

            let span = bclause.rel_args_span;

            let matching_dot_iter = quote_spanned! {bclause.rel_args_span=> __matching};

            let (index_get, iter_all) = if clause_ind < par_iter_to_ind {
               (quote_spanned! {span=> c_index_get}, quote_spanned! {span=> c_iter_all})
            } else {
               (quote_spanned! {span=> index_get}, quote_spanned! {span=> iter_all})
            };

            let def_default_id_code = quote! { let mut __default_id = 0; };

            // The special case where the first clause has indices, but there are no expressions
            // in the args of the first clause
            if doing_simple_join {
               let cl1 = rule.body_items[rule.simple_join_start_index.unwrap()].unwrap_clause();
               let cl2 = bclause;
               let cl1_var_name = expr_for_rel(&cl1.rel, &cl1.extern_db_name, mir);
               let cl2_var_name = expr_for_rel(&cl2.rel, &cl2.extern_db_name, mir);
               let cl1_vars = cl1.vars();

               let cl1_rel_name = &cl1.rel.relation.name;

               let mut cl1_join_vars_assignments = vec![];
               for (tuple_ind, &i) in cl1.rel.indices.iter().enumerate() {
                  let var = expr_to_ident(&cl1.args[i]).unwrap();
                  let tuple_ind = syn::Index { index: tuple_ind as u32, span: var.span() };
                  cl1_join_vars_assignments
                     .push(quote_spanned! {var.span()=> let #var = __cl1_joined_columns.#tuple_ind;});
               }

               let cl1_matched_val_ident = syn::Ident::new("cl1_val", cl1.rel_args_span);
               let cl1_vars_assignments = clause_bind_and_continue(
                  &cl1.rel,
                  cl1_vars.iter().filter(|(i, _var)| !cl1.rel.indices.contains(i)).cloned(),
                  &cl1_matched_val_ident,
                  &parse_quote! {_self.#cl1_rel_name},
                  cloning_needed,
                  mir,
                  // conds_then_next_loop.clone(),
                  quote! {},
               );

               let joined_args_for_cl2_cloned = cl2.selected_args().iter().map(exp_cloned).collect_vec();
               let joined_args_tuple_for_cl2 = tuple_spanned(&joined_args_for_cl2_cloned, cl2.args_span);

               let cl1_tuple_indices_iter = quote_spanned!(cl1.rel_args_span=> __cl1_tuple_indices);

               let mut cl1_conds_then_rest = quote_spanned! {bclause.rel_args_span=>
                  #matching_dot_iter.clone().for_each(|__val|  {
                     // TODO we may be doing excessive cloning
                     let mut __dep_changed = false;
                     #def_default_id_code
                     #new_vars_binds_and_next
                     // #conds_then_next_loop
                  });
               };
               for cond in cl1.cond_clauses.iter().rev() {
                  cl1_conds_then_rest = compile_cond_clause(cond, cl1_conds_then_rest);
               }
               let failed_generate_code = if cl2.is_bang {
                  // if it's bang clause, and the query fails, treat it as a head clause,
                  // generate it here
                  // construct a head clause with the same relation
                  let new_head = IrHeadClause {
                     rel: cl2.rel.relation.clone(),
                     args: cl2.args.clone(),
                     args_span: cl2.args_span,
                     id_name: None,
                     extern_db_name: cl2.extern_db_name.clone(),
                     required_flag: false,
                     delete_flag: false,
                     span: cl2.rel_args_span,
                     inflation_flag: false, // TODO: check if this is correct
                  };
                  let update_code = compile_head_clause(&new_head, _scc, mir);
                  quote_spanned! {cl1.rel_args_span=>
                     else {
                        #update_code
                        return;
                     }
                  }
               } else {
                  quote! {}
               };
               quote_spanned! {cl1.rel_args_span=>
                  #cl1_var_name.#iter_all().for_each(|(__cl1_joined_columns, __cl1_tuple_indices)| {
                     let mut __gen_bang = false;
                     let __cl1_joined_columns = __cl1_joined_columns.tuple_of_borrowed();
                     #(#cl1_join_vars_assignments)*
                     if let Some(__matching) = #cl2_var_name.#index_get(&#joined_args_tuple_for_cl2) {
                        #cl1_tuple_indices_iter.for_each(|cl1_val| {
                           #cl1_vars_assignments
                           #cl1_conds_then_rest
                        });
                     } #failed_generate_code
                  });
               }
            } else {
               quote_spanned! {bclause.rel_args_span=>
                  if let Some(__matching) = #rel_version_exp.#index_get( &#selected_args_tuple) {
                     #matching_dot_iter.for_each(|__val|  {
                        // TODO we may be doing excessive cloning
                        let mut __dep_changed = false;
                        #def_default_id_code
                        #new_vars_binds_and_next
                        // #conds_then_next_loop
                     });
                  }
               }
            }
         }
         MirBodyItem::Equiv(equiv) => {
            compile_equiv_clause_body(equiv, next_loop)
         }
         MirBodyItem::Generator(gen) => {
            let pat = &gen.pattern;
            let expr = &gen.expr;
            quote_spanned! {gen.for_keyword.span()=>
               for #pat in #expr {
                  #next_loop
               }
            }
         }
         MirBodyItem::Cond(cond) => compile_cond_clause(cond, next_loop),
         MirBodyItem::Agg(agg) => {
            let pat = &agg.pat;
            let rel_name = &agg.rel.relation.name;
            let mir_relation = MirRelation::from(agg.rel.clone(), Total);
            // let rel_version_var_name = mir_relation.var_name();
            let rel_expr = expr_for_rel(&mir_relation, &agg.extern_db_name, mir);
            let selected_args = mir_relation.indices.iter().map(|&i| &agg.rel_args[i]);
            let selected_args_cloned = selected_args.map(exp_cloned).collect_vec();
            // In egglog mode, for each selected arg, if it is an eclass_id, we need to use the canonical value
            // TODO: it's not clear how to aggregate over eclass_id, now aggregate over the eclass_id is UB
            let selected_args_canonicalized = if mir.config.egg_mode && mir_relation.relation.contains_eclass_id {
               selected_args_cloned.iter().enumerate().map(|(i, arg)| {
                  if mir_relation.relation.is_field_eclass_id(i) {
                     parse_quote! {_self.equiv_ids_.get_dominant_elem(#arg).unwrap_or(#arg)}
                  } else {
                     arg.clone()
                  }
               }).collect_vec()
            } else {
               selected_args_cloned
            };
            let selected_args_tuple = tuple_spanned(&selected_args_canonicalized, agg.span);
            let agg_args_tuple_indices = agg.bound_args.iter().map(|arg| {
               (
                  agg.rel_args.iter().find_position(|rel_arg| expr_to_ident(rel_arg) == Some(arg.clone())).unwrap().0,
                  arg.clone(),
               )
            });

            let agg_args_tuple =
               tuple_spanned(&agg.bound_args.iter().map(|v| parse_quote! {#v}).collect_vec(), agg.span);

            let vars_assignments = clause_bind_and_continue(
               &MirRelation::from(agg.rel.clone(), MirRelationVersion::Total),
               agg_args_tuple_indices,
               &parse_quote_spanned! {agg.span=> __val},
               &parse_quote! {_self.#rel_name},
               false,
               mir,
               quote! {},
            );

            let agg_func = &agg.aggregator;
            let _self = quote! { _self };
            quote_spanned! {agg.span=>
               let __aggregated_rel = #rel_expr;
               let __matching = __aggregated_rel.index_get( &#selected_args_tuple);
               let __agg_args = __matching.into_iter().flatten().map(|__val| {
                  #vars_assignments
                  #agg_args_tuple
               });
               for #pat in #agg_func(__agg_args) {
                  #next_loop
               }

            }
         }
      }
   } else {
      quote! {
         // let before_update = ::ascent::internal::Instant::now();
         #head_update_code
         // let update_took = before_update.elapsed();
         // _self.update_time_nanos.fetch_add(update_took.as_nanos() as u64, std::sync::atomic::Ordering::Relaxed);
      }
   }
}

fn head_clauses_structs_and_update_code(
   rule: &MirRule, scc: &MirScc, mir: &AscentMir,
) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
   let mut add_rows = vec![];

   for hcl in rule.head_clause.iter() {
      add_rows.push(compile_head_clause_from_item(hcl, scc, mir));
   }
   
   (quote! {}, quote! {#(#add_rows)*})
}

fn compile_head_clause_from_item(
   hitem: &IrHeadItem,
   scc: &MirScc,
   mir: &AscentMir
) -> proc_macro2::TokenStream {
   match hitem {
      IrHeadItem::Clause(hcl) => compile_head_clause(hcl, scc, mir),
      IrHeadItem::Equiv(equiv) => compile_equiv_clause_head(equiv, scc, mir),
   }
}

fn compile_equiv_clause_head(
   equiv: &EquivClauseNode,
   _scc: &MirScc,
   mir: &AscentMir
) -> proc_macro2::TokenStream {
   
   let src_var = equiv.left_ident.clone();
   let dst_var = equiv.right_ident.clone();
   let set_changed_true_code = if !mir.is_parallel {
      quote! { __changed = true; }
   } else {
      quote! { __changed.store(true, std::sync::atomic::Ordering::Relaxed);}
   };
   let src_var_repr_name = Ident::new(&format!("__src_var_repr_{}", src_var), src_var.span());
   let dst_var_repr_name = Ident::new(&format!("__dst_var_repr_{}", dst_var), dst_var.span());
    if equiv.representative {
      let src_var = &equiv.left_ident;
      let dst_var = &equiv.right_ident;
      quote! {
         let #dst_var = _self.equiv_ids_.get_dominant_elem(#src_var).unwrap_or(#src_var);
         #set_changed_true_code
      }
    } else { quote! {
         let #src_var_repr_name = _self.equiv_ids_.get_dominant_elem(&#src_var).unwrap_or(&#src_var);
         let #dst_var_repr_name = _self.equiv_ids_.get_dominant_elem(&#dst_var).unwrap_or(&#dst_var);
         _self.equiv_ids_delta_.add(#src_var_repr_name.clone(), #dst_var_repr_name.clone());
         #set_changed_true_code
      }
   }
}

fn compile_head_clause(
   hcl: &IrHeadClause,
   scc: &MirScc,
   mir: &AscentMir
) -> proc_macro2::TokenStream {
   let set_changed_true_code = if !mir.is_parallel {
      quote! { __changed = true; }
   } else {
      quote! { __changed.store(true, std::sync::atomic::Ordering::Relaxed);}
   };
   let head_rel_name = Ident::new(&hcl.rel.name.to_string(), hcl.span);
   let hcl_args_converted = hcl.args.iter().cloned().map(convert_head_arg).collect_vec();
   let new_row_tuple = tuple_spanned(&hcl_args_converted, hcl.args_span);
   let new_canonical_row_tuple = if mir.config.egg_mode && hcl.rel.contains_eclass_id {
      let mut new_canonical_row_tuple = vec![];
      for i in 0..hcl.args.len() {
         let i_ind = syn::Index::from(i);
         if hcl.rel.is_field_eclass_id(i) {
            // if current arg is eclass_id, we need to use the canonical value
            // by query the hidden union-find relation
            let ith_arg = quote_spanned! {hcl.span=> __new_row.#i_ind};
            let ith_arg_canonical = quote_spanned! {hcl.span=> 
               _self.equiv_ids_.get_dominant_elem(&#ith_arg).unwrap_or(&#ith_arg).clone()
            };
            new_canonical_row_tuple.push(ith_arg_canonical);
         } else {
            // create a syntax node for "i"
            new_canonical_row_tuple.push(quote_spanned! {hcl.span=> __new_row.#i_ind.clone()});
         }
      }
      quote_spanned! {hcl.span=> (#(#new_canonical_row_tuple,)*)}
   } else {
      quote! {}
   };

   let head_relation = &hcl.rel;
   // if None use default name __new_tuple_d
   let new_id_name = match &hcl.id_name {
      Some(new_id) => new_id.clone(),
      None => Ident::new(&format!("__new_{}", head_rel_name), hcl.span),
   };
   let def_id_code = quote_spanned! {hcl.span=> let mut #new_id_name = 0;};

   let row_type = tuple_type(&head_relation.field_types);

   let mut update_indices = vec![];
   let rel_indices = scc.dynamic_relations.get(head_relation);
   let (rel_index_write_trait, index_insert_fn) = if !mir.is_parallel {
      (quote! { ::ascent::internal::RelIndexWrite }, quote! {index_insert})
   } else {
      (quote! { ::ascent::internal::CRelIndexWrite }, quote! {index_insert})
   };
   let (rel_index_write_trait, index_insert_fn) =
      (rel_index_write_trait.with_span(hcl.span), index_insert_fn.with_span(hcl.span));
   let new_ref = if !mir.is_parallel {
      quote! {&mut}
   } else {
      quote! {&}
   };
   let mut used_fields = HashSet::new();
   if let Some(rel_indices) = rel_indices {
      for rel_ind in rel_indices.iter().sorted_by_cached_key(|rel| rel.ir_name()) {
         if rel_ind.is_full_index() {
            continue;
         };
         let var_name = if !mir.is_parallel {
            expr_for_rel_write(&MirRelation::from(rel_ind.clone(), New), mir)
         } else {
            expr_for_c_rel_write(&MirRelation::from(rel_ind.clone(), New), mir)
         };
         let args_tuple: Vec<Expr> = rel_ind
            .indices
            .iter()
            .map(|&i| {
               let i_ind = syn::Index::from(i);
               syn::parse2(quote_spanned! {hcl.span=> __new_row.#i_ind.clone()}).unwrap()
            })
            .collect();
         used_fields.extend(rel_ind.indices.iter().cloned());
         if let IndexValType::Direct(direct) = &rel_ind.val_type {
            used_fields.extend(direct.iter().cloned());
         }
         let args_tuple = tuple(&args_tuple);
         let entry_val = index_get_entry_val_for_insert(
            rel_ind,
            &parse_quote_spanned! {hcl.span=> __new_row},
            &parse_quote_spanned! {hcl.span=> __new_row_ind},
         );
         update_indices.push(quote_spanned! {hcl.span=>
            #rel_index_write_trait::#index_insert_fn(#new_ref #var_name, #args_tuple, #entry_val);
         });
      }
   }

   let head_rel_full_index = &mir.relations_full_indices[head_relation];

   let expr_for_rel_maybe_mut = if mir.is_parallel { expr_for_c_rel_write } else { expr_for_rel_write };
   let head_rel_full_index_expr_new =
      expr_for_rel_maybe_mut(&MirRelation::from(head_rel_full_index.clone(), New), mir);
   let head_rel_full_index_expr_canonical_delta =
      expr_for_rel_maybe_mut(&MirRelation::from(head_rel_full_index.clone(), CanonicalDelta), mir);
   
   // TODO: should we allow adding facts to external relations?
   let head_rel_full_index_expr_delta =
      expr_for_rel(&MirRelation::from(head_rel_full_index.clone(), Delta), &None, mir);
   let head_rel_full_index_expr_total =
      expr_for_rel(&MirRelation::from(head_rel_full_index.clone(), Total), &None, mir);

   let rel_full_index_write_trait = if !mir.is_parallel {
      quote! { ::ascent::internal::RelFullIndexWrite }
   } else {
      quote! { ::ascent::internal::CRelFullIndexWrite }
   }
   .with_span(hcl.span);

   let new_row_to_be_pushed = (0..hcl.rel.field_types.len())
      .map(|i| {
         let ind = syn::Index::from(i);
         let clone = if used_fields.contains(&i) {
            quote! {.clone()}
         } else {
            quote! {}
         };
         if mir.config.egg_mode && hcl.rel.is_field_eclass_id(i) {
            parse_quote_spanned! {hcl.span=> __new_row_canonical.#ind #clone }
         } else {
            parse_quote_spanned! {hcl.span=> __new_row.#ind #clone }
         }
      })
      .collect_vec();
   let new_row_to_be_pushed = tuple_spanned(&new_row_to_be_pushed, hcl.span);

   let db_name = if let Some(db_name) = &hcl.extern_db_name {
      if !mir.is_parallel {
         quote! {#db_name.borrow_mut()}
      } else {
         quote! {#db_name.write().unwrap()}
      }
   } else {
      quote! {_self}
   };
   let compute_id_code = if hcl.id_name.is_some() {
      quote! {
         #db_name.#head_rel_name.push(__new_row_to_be_pushed);
      }
   } else {
      if !mir.is_parallel {
         quote! {
            #new_id_name = #db_name.#head_rel_name.len();
            #db_name.#head_rel_name.push(__new_row_to_be_pushed);
         }
      } else {
         quote! {
            #new_id_name = #db_name.#head_rel_name.push(__new_row_to_be_pushed);
         }
      }
   };
   let push_code = quote! {  
      let __new_row_to_be_pushed = #new_row_to_be_pushed;
      #compute_id_code
      __default_id = #new_id_name;
   };

   let failed_unchanged_code = if !hcl.required_flag || hcl.id_name.is_some() {
      quote! {}
   } else {
      quote! {
         // println!("required flag not satisfied");
         return;
      }
   };
   // TODO: buggy here, make sure always use explict id relation update!
   let update_id_code = if hcl.id_name.is_some() && !hcl.required_flag {
      // update the full and canonical indices
      let id_arg_tuple = (0..hcl.rel.field_types.len())
         .into_iter()
         .flat_map(|i| {
            let i_ind = syn::Index::from(i);
            vec![quote_spanned! {hcl.span=> __new_row.#i_ind.clone()}]
         })
         .collect_vec();
      // find full of head_rel_full_index
      let mut head_rel_id_full_index = mir.get_relation_id(&hcl.rel.name).unwrap();
      head_rel_id_full_index.indices.push(hcl.rel.field_types.len());
      let head_rel_id_full_index_expr =
         expr_for_rel_maybe_mut(&MirRelation::from(head_rel_id_full_index.clone(), New), mir);
      quote_spanned! {hcl.span=>
         let __new_row_id = (#(#id_arg_tuple,)* #new_id_name.clone());
         #rel_full_index_write_trait::insert_if_not_present(#new_ref #head_rel_id_full_index_expr,
            &__new_row_id, ());
      }
   } else {
      quote! {}
   };

   let update_rel_code = if !hcl.delete_flag {
      if let Some(_) = &hcl.extern_db_name {
         quote_spanned! {hcl.span=>
            #push_code
            // #set_changed_true_code
         }
      } else {
         let hname = format!("{}", hcl.rel.name);
         // In egglog mode, Id need to be eclass_id, which is hash value of the canonical value
         let new_row = if mir.config.egg_mode && hcl.rel.contains_eclass_id {
            quote_spanned! {hcl.span=> __new_row_canonical}
         } else {
            quote_spanned! {hcl.span=> __new_row}
         };
         let hash_tuple_code = quote! {
            #new_id_name = {
               use std::hash::{Hash, Hasher};
               let mut hasher = ::std::hash::DefaultHasher::new();
               (#new_row.clone(), #hname).hash(&mut hasher);
               hasher.finish() as usize
            };
         };
         if hcl.inflation_flag || (mir.config.egg_mode && hcl.rel.contains_eclass_id) {
            let canonical_delta_insert_code = if mir.config.egg_mode && hcl.rel.contains_eclass_id {
               quote! {
                  if #rel_full_index_write_trait::insert_if_not_present(#new_ref #head_rel_full_index_expr_canonical_delta,
                     &__new_row_canonical, ()) {
                     #push_code
                  } else {
                     #failed_unchanged_code
                  }
               }
            } else {
               quote! {}
            };
            quote_spanned! {hcl.span=>
               #hash_tuple_code
               if #rel_full_index_write_trait::insert_if_not_present(#new_ref #head_rel_full_index_expr_new,
                  &__new_row, ())
               {
                  #canonical_delta_insert_code ;
                  // keep fixpoint check correct
                  #set_changed_true_code
               } else {
                  #failed_unchanged_code
               }
               ::ascent::internal::comment("inflation!");
               #(#update_indices)*
               // #update_id_code
            }
         } else {
            quote_spanned! {hcl.span=>
               #hash_tuple_code
               if #rel_full_index_write_trait::insert_if_not_present(#new_ref #head_rel_full_index_expr_new,
                  &__new_row, ())
               {
                  #push_code
                  #(#update_indices)*
                  #set_changed_true_code
                  #update_id_code
               } else {
                  #failed_unchanged_code
               }
            }
         }
      }
   } else {
      quote! {}
   };
   if !hcl.rel.is_lattice {
      if hcl.extern_db_name.is_none() {
         if !hcl.inflation_flag && !(mir.config.egg_mode && hcl.rel.contains_eclass_id) {
            quote_spanned! {hcl.span=>
               let __new_row: #row_type = #new_row_tuple;
               #def_id_code

               if !::ascent::internal::RelFullIndexRead::contains_key(&#head_rel_full_index_expr_total, &__new_row) &&
                  !::ascent::internal::RelFullIndexRead::contains_key(&#head_rel_full_index_expr_delta, &__new_row) {
                  #update_rel_code
               } else {
                  #failed_unchanged_code
               }
            }
         } else {
            // if there is inflation, we skip the check for if a a duplicate exists
            if mir.config.egg_mode && hcl.rel.contains_eclass_id {
               // if egglog mode and contains eclass_id, we need to use the canonical value
               // existance check is done on canonical value
               quote_spanned! {hcl.span=>
                  let __new_row: #row_type = #new_row_tuple;
                  let __new_row_canonical = #new_canonical_row_tuple;
                  #def_id_code
                  // if !::ascent::internal::RelFullIndexRead::contains_key(&#head_rel_full_index_expr_total, &__new_row_canonical) &&
                  //    !::ascent::internal::RelFullIndexRead::contains_key(&#head_rel_full_index_expr_delta, &__new_row_canonical) {
                  //    #push_code
                  // } else {
                  //    #failed_unchanged_code
                  // }
                  #update_rel_code
               }
            } else {
               quote_spanned! {hcl.span=>
                  let __new_row: #row_type = #new_row_tuple;
                  #def_id_code
                  if !::ascent::internal::RelFullIndexRead::contains_key(&#head_rel_full_index_expr_total, &__new_row) &&
                     !::ascent::internal::RelFullIndexRead::contains_key(&#head_rel_full_index_expr_delta, &__new_row) {
                     #push_code
                  } else {
                     #failed_unchanged_code
                  }
                  #update_rel_code
               }
            }
         }
      } else {
         quote_spanned! {hcl.span=>
            let __new_row: #row_type = #new_row_tuple;
            #def_id_code
            #update_rel_code
         }
      }
   } else {
      // rel.is_lattice:
      let _self = quote! { _self };
      let lattice_insertion_mutex = lattice_insertion_mutex_var_name(head_relation);
      let head_lat_full_index = &mir.lattices_full_indices[head_relation];
      let head_lat_full_index_var_name_new =
         ir_relation_version_var_name(&head_lat_full_index.ir_name(), &_self, New);
      let head_lat_full_index_var_name_delta =
         ir_relation_version_var_name(&head_lat_full_index.ir_name(), &_self, Delta);
      let head_lat_full_index_var_name_full =
         ir_relation_version_var_name(&head_lat_full_index.ir_name(), &_self, Total);
      let tuple_lat_index = syn::Index::from(hcl.rel.field_types.len() - 1);
      let lattice_key_args: Vec<Expr> = (0..hcl.args.len() - 1)
         .map(|i| {
            let i_ind = syn::Index::from(i);
            syn::parse2(quote_spanned! {hcl.span=> __new_row.#i_ind}).unwrap()
         })
         .map(|e| exp_cloned(&e))
         .collect_vec();
      let lattice_key_tuple = tuple(&lattice_key_args);

      let _self = quote! { _self };
      if !mir.is_parallel {
         quote_spanned! {hcl.span=>
            let __new_row: #row_type = #new_row_tuple;
            let __lattice_key = #lattice_key_tuple;
            if let Some(mut __existing_ind) = #head_lat_full_index_var_name_new.index_get(&__lattice_key)
               .or_else(|| #head_lat_full_index_var_name_delta.index_get(&__lattice_key))
               .or_else(|| #head_lat_full_index_var_name_full.index_get(&__lattice_key))
            {
               let __existing_ind = *__existing_ind.next().unwrap();
               // TODO possible excessive cloning here?
               let __lat_changed = ::ascent::Lattice::join_mut(&mut #_self.#head_rel_name[__existing_ind].#tuple_lat_index, __new_row.#tuple_lat_index.clone());
               if __lat_changed {
                  let __new_row_ind = __existing_ind;
                  #(#update_indices)*
                  #set_changed_true_code
               } else {
                  #failed_unchanged_code
               }
            } else {
               let __new_row_ind = #_self.#head_rel_name.len();
               #(#update_indices)*
               #_self.#head_rel_name.push(#new_row_to_be_pushed);
               #set_changed_true_code
            }
         }
      } else {
         // TODO: fix id with lattice
         quote_spanned! {hcl.span=> // mir.is_parallel:
            let __new_row: #row_type = #new_row_tuple;
            let __lattice_key = #lattice_key_tuple;
            let __existing_ind_in_new = #head_lat_full_index_var_name_new.get_cloned(&__lattice_key);
            let __new_has_ind = __existing_ind_in_new.is_some();
            if let Some(__existing_ind) = __existing_ind_in_new
               .or_else(|| #head_lat_full_index_var_name_delta.get_cloned(&__lattice_key))
               .or_else(|| #head_lat_full_index_var_name_full.get_cloned(&__lattice_key))
            {
               let __lat_changed = ::ascent::Lattice::join_mut(&mut #_self.#head_rel_name[__existing_ind].write().unwrap().#tuple_lat_index,
                                                               __new_row.#tuple_lat_index.clone());
               if __lat_changed && !__new_has_ind{
                  let __new_row_ind = __existing_ind;
                  #(#update_indices)*
                  #set_changed_true_code
               } else {
                  #failed_unchanged_code
               }
            } else {
               let __hash = #head_lat_full_index_var_name_new.hash_usize(&__lattice_key);
               let __lock = #_self.#lattice_insertion_mutex.get(__hash % #_self.#lattice_insertion_mutex.len()).expect("lattice_insertion_mutex index out of bounds").lock().unwrap();
               if let Some(__existing_ind) = #head_lat_full_index_var_name_new.get_cloned(&__lattice_key) {
                  ::ascent::Lattice::join_mut(&mut #_self.#head_rel_name[__existing_ind].write().unwrap().#tuple_lat_index,
                                              __new_row.#tuple_lat_index.clone());
                  #failed_unchanged_code
               } else {
                  let __new_row_ind = #_self.#head_rel_name.push(::std::sync::RwLock::new(#new_row_to_be_pushed));
                  #(#update_indices)*
                  #set_changed_true_code
               }
            }
         }
      }
   }
}

fn convert_head_arg(arg: Expr) -> Expr {
   if let Some(var) = expr_to_ident(&arg) {
      parse2(quote_spanned! {arg.span()=> ascent::internal::Convert::convert(#var)}).unwrap()
   } else {
      arg
   }
}

fn clause_bind_and_continue(
   rel: &MirRelation, vars: impl Iterator<Item = (usize, Ident)>, val_ident: &Ident, relation_expr: &Expr,
   cloning_needed: bool, mir: &AscentMir, next_loop: proc_macro2::TokenStream
) -> proc_macro2::TokenStream {
   let mut assignments = vec![];
   let mut is_eclass_args = vec![];
   let is_delta_rel = rel.version == MirRelationVersion::Delta;

   let mut any_vars = false;
   for (ind_in_tuple, var) in vars {
      let var_type_ascription = {
         let ty = &rel.relation.field_types[ind_in_tuple];
         quote! { : & #ty}
      };

      // In egglog mode, If relation is not delta, we need to iterate over the equivalence
      // class of the value, inflation of delta happens allowing deduplicated tuples to be added
      // into delta relations.
      let need_inflation = !is_delta_rel && mir.config.egg_mode && rel.relation.is_field_eclass_id(ind_in_tuple);
      is_eclass_args.push(need_inflation);
      any_vars = true;
      // In egglog mode, we need to iterate over the equivalence class of the value
      // In non-egglog mode, we can just bind the value
      match &rel.val_type {
         IndexValType::Reference => {
            let ind = syn::Index::from(ind_in_tuple);
            if need_inflation {
               assignments.push(quote! {
                  for #var in _self.equiv_ids_.set_of_inc_x(&__row.#ind)
               })
            } else {
               assignments.push(quote! {
                  let #var #var_type_ascription = &__row.#ind;
               })
            }
         }
         IndexValType::Direct(inds) => {
            let ind = inds.iter().enumerate().find(|(_i, ind)| **ind == ind_in_tuple).unwrap().0;
            let ind = syn::Index::from(ind);
            if need_inflation {
               assignments.push(quote! {
                  for #var in _self.equiv_ids_.set_of_inc_x(&#val_ident.#ind)
               })
            } else {
               assignments.push(quote! {
                  let #var #var_type_ascription = #val_ident.#ind;
               })
            }
         }
      }
   }

   // fold all (assignments, is_eclass_arg) into a single TokenStream
   let assignment_and_next_code = 
      assignments.into_iter().zip(is_eclass_args.into_iter()).rev().fold(
         next_loop,
         |acc, (bind, is_eclass_huh)| {
            // inflate if in egglog mode and operate on a eclass_id
            if is_eclass_huh {
               quote! {
                  #bind {
                     #acc
                  }
               }
            } else {
               quote! {
                  #bind
                  #acc
               }
            }
         }
      );

   let before_bind = if any_vars {
      match &rel.val_type {
         IndexValType::Reference => {
            let maybe_lock = if rel.relation.is_lattice && mir.is_parallel {
               quote! {.read().unwrap()}
            } else {
               quote! {}
            };
            let maybe_clone = if cloning_needed {
               quote! {.clone()}
            } else {
               quote! {}
            }; 
            quote! {
               let __row = &#relation_expr[*#val_ident]#maybe_lock #maybe_clone;
            }
         }
         IndexValType::Direct(_) => {
            quote! {
               let #val_ident = #val_ident.tuple_of_borrowed();
            }
         }
      }
   } else {
      quote! {}
   };

   quote! {
      #before_bind
      #assignment_and_next_code
   }
}
