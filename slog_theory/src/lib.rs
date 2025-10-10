
pub mod eq_theory {
   use slog::slog_source;
   slog_source! {
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

   use ascent_byods_rels::eqrel_ind::EqRelIndCommon;

   pub fn canonicalize_eclass<'a>(full: &'a EqRelIndCommon<usize>, delta: &'a EqRelIndCommon<usize>, x: &'a usize) -> &'a usize {
      full.combined.get_dominant_elem(x).unwrap_or(delta.combined.get_dominant_elem(x).unwrap_or(x))
   }
}

pub mod arithm;
pub mod test;
pub mod util;
mod test_compile;
pub mod invertible;
