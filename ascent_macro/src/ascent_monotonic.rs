#![deny(warnings)]

use crate::ascent_syntax::{AscentProgram, BodyItemNode, HeadItemNode};

pub(crate) fn ascent_check_monotonicity(prog: &AscentProgram) -> Result<(), syn::Error> {
   let mut monotonic_map = std::collections::HashMap::new();

   for rule in &prog.rules {
      for head in &rule.head_clauses {
         if let HeadItemNode::HeadClause(hcl) = head {
            let rel = &hcl.rel;
            if let Some(mono_flag) = monotonic_map.get(rel) {
               if *mono_flag != hcl.delete_flag {
                  return Err(syn::Error::new(
                     hcl.rel.span(),
                     format!("Monotonicity violation for relation {:?}", hcl.rel),
                  ));
               }
            } else {
               monotonic_map.insert(rel, hcl.delete_flag);
            }
         }
      }

      for body in &rule.body_items {
        if let BodyItemNode::Clause(bcl) = body {
            if let Some(mono_flag) = monotonic_map.get(&bcl.rel) {
                if *mono_flag {
                    return Err(syn::Error::new(
                        bcl.rel.span(),
                        format!("Monotonicity violation for relation {:?}", bcl.rel),
                    ));
                }
            }
        }
      }
   }
   Ok(())
}
