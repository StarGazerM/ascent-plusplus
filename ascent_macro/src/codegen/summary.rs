#![deny(warnings)]
use itertools::Itertools;

use crate::{
   ascent_mir::{mir_rule_summary, AscentMir},
   codegen::util::rule_time_field_name,
};

pub fn compile_relation_sizes_body(mir: &AscentMir) -> proc_macro2::TokenStream {
   let mut write_sizes = vec![];
   for r in mir.relations_ir_relations.keys().sorted_by_key(|r| &r.name) {
      if r.extern_db_name.is_some() {
         continue;
      }
      let rel_name = &r.name;
      let rel_name_str = r.name.to_string();
      write_sizes.push(quote! {
         writeln!(&mut res, "{} size: {}", #rel_name_str, self.#rel_name.len()).unwrap();
      });
   }
   quote! {
      use std::fmt::Write;
      let mut res = String::new();
      #(#write_sizes)*
      res
   }
}

pub fn compile_scc_times_summary_body(mir: &AscentMir) -> proc_macro2::TokenStream {
   let mut res = vec![];
   for i in 0..mir.sccs.len() {
      let i_str = format!("{}", i);
      res.push(quote!{
         writeln!(&mut res, "scc {}: iterations: {}, time: {:?}", #i_str, self.scc_iters[#i], self.scc_times[#i]).unwrap();
      });
      if mir.config.include_rule_times {
         let mut sum_of_rule_times = quote! { std::time::Duration::ZERO };
         for (rule_ind, _rule) in mir.sccs[i].rules.iter().enumerate() {
            let rule_time_field = rule_time_field_name(i, rule_ind);
            sum_of_rule_times = quote! { #sum_of_rule_times + self.#rule_time_field};
         }
         res.push(quote! {
            let sum_of_rule_times = #sum_of_rule_times;
            writeln!(&mut res, "  sum of rule times: {:?}", sum_of_rule_times).unwrap();
         });
         for (rule_ind, rule) in mir.sccs[i].rules.iter().enumerate() {
            let rule_time_field = rule_time_field_name(i, rule_ind);
            let rule_summary = mir_rule_summary(rule);
            res.push(quote! {
               writeln!(&mut res, "  rule {}\n    time: {:?}", #rule_summary, self.#rule_time_field).unwrap();
            });
         }
         res.push(quote! {
            writeln!(&mut res, "-----------------------------------------").unwrap();
         });
      }
   }
   let update_indices_time_code = quote! {
      writeln!(&mut res, "update_indices time: {:?}", self.update_indices_duration).unwrap();
   };
   quote! {
      use std::fmt::Write;
      let mut res = String::new();
      #update_indices_time_code
      #(#res)*
      res
   }
}
