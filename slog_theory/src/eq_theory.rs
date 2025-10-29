
use slog::slog_source;
slog_source! {
   eq_theory (unification_rel):
   unify_eclass(x, x, x) <-- unify_eclass_type(x);
}

use ascent_byods_rels::{eqrel_ind::EqRelIndCommon, union_find::EqRel};

pub fn canonicalize_eclass<'a>(
   full: &'a EqRelIndCommon<usize>, delta: &'a EqRelIndCommon<usize>, x: &'a usize,
) -> &'a usize {
   full.combined.get_dominant_elem(x).unwrap_or(delta.combined.get_dominant_elem(x).unwrap_or(x))
}

use std::{fmt::Formatter, fmt::Debug, hash::Hash, rc::Rc};

pub struct Quotient<T: Clone + Hash + Eq + PartialOrd> {
   pub set: Rc<EqRel<T>>,
   pub repr: T,
}

impl<T: Clone + Hash + Eq + PartialOrd> PartialEq for Quotient<T> {
   fn eq(&self, other: &Self) -> bool {
      // check if use the same disjoint set
      Rc::ptr_eq(&self.set, &other.set) && self.set.contains(&self.repr, &other.repr)
   }
}

impl<T: Clone + Hash + Eq + PartialOrd> Eq for Quotient<T> {}

impl<T: Clone + Hash + Eq + PartialOrd> Debug for Quotient<T> {
   fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
      // only print the repr
      write!(f, "{:?}", self.repr)
   }
}
