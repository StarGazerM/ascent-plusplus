
use slog::slog_source;
slog_source! {
   eq_theory (unification_rel):
   unify_eclass(x, x, x) <-- unify_eclass_type(x);
}

use ascent_byods_rels::eqrel_ind::EqRelIndCommon;

pub fn canonicalize_eclass<'a>(
   full: &'a EqRelIndCommon<usize>, delta: &'a EqRelIndCommon<usize>, x: &'a usize,
) -> &'a usize {
   full.combined.get_dominant_elem(x).unwrap_or(delta.combined.get_dominant_elem(x).unwrap_or(x))
}

use std::{hash::Hash, rc::Rc};

pub struct Quotient<T: Clone + Eq + Hash> {
   pub set: Rc<EqRelIndCommon<T>>,
   pub repr: T,
}
