
#[macro_export]
macro_rules! eq_canonicalize {
   ($h1:tt, $h2:tt, $input: tt) => {
      $h1.combined.get_dominant_elem($input).unwrap_or($h2.combined.get_dominant_elem($input).unwrap_or($input))
   };

   ($h:tt, $input: tt) => {
      $h.combined.get_dominant_elem($input).unwrap_or($input)
   };
}

pub use eq_canonicalize as canonicalize;



pub mod eq_theory {
   use ascent::ascent_source;
   ascent_source! {
      eq_theory (unify_total, unify_delta):

      unify(parent_a, parent_b) <--
         unify(child_a, child_b),
         deriv(parent_a, child_a),
         deriv(parent_b, child_b),
         agg sibs_a = collect(sib) in deriv(parent_a, sib),
         agg sibs_b = collect(sib) in deriv(parent_b, sib),
         if $unify_total.combined.equiv_vec_huh(&sibs_a, &sibs_b)
         || $unify_delta.combined.equiv_vec_huh(&sibs_a, &sibs_b)
         ;
   }
}

// ascent_source! {
//    empty_theory:
//    unify(0,0);
// }

