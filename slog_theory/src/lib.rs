pub use paste::paste;

pub mod eq_theory {
   use ascent::ascent_source;
   ascent_source! {
      eq_theory (unification_rel):
      unify_eclass(parent_a, parent_b) <--
         unify_eclass(child_a, child_b),
         deriv(parent_a, child_a),
         deriv(parent_b, child_b),
         agg sibs_a = collect(sib) in deriv(parent_a, sib),
         agg sibs_b = collect(sib) in deriv(parent_b, sib),
         if total!($unification_rel).combined.equiv_vec_huh(&sibs_a, &sibs_b)
         || delta!($unification_rel).combined.equiv_vec_huh(&sibs_a, &sibs_b)
         ;
   }

   // ascent_source! {
   //    eq_theory_par (unify_total, unify_delta):

   //    #[ds(ascent_byods_rels::eqrel)]
   //    relation unify_eclass(usize, usize);
   //    unify_eclass(Default::default(),Default::default());

   //    unify_eclass(parent_a, parent_b) <--
   //       unify_eclass(child_a, child_b),
   //       deriv(parent_a, child_a),
   //       deriv(parent_b, child_b),
   //       agg sibs_a = collect(sib) in deriv(parent_a, sib),
   //       agg sibs_b = collect(sib) in deriv(parent_b, sib),
   //       if $unify_total.unwrap_frozen().combined.equiv_vec_huh(&sibs_a, &sibs_b)
   //       || $unify_delta.unwrap_frozen().combined.equiv_vec_huh(&sibs_a, &sibs_b)
   //       ;
   // }

   #[macro_export]
   macro_rules! canonicalize_eclass {
      ($h1:tt, $input: tt) => {
         paste! {
            total!($h1).combined.get_dominant_elem($input).unwrap_or(delta!($h1).combined.get_dominant_elem($input).unwrap_or($input))
         }
      };
   }
}
