#![allow(unused_imports)]
use std::{clone, cmp::max, rc::Rc, cell::RefCell};
use std::ops::Deref;
use std::hash::Hash;
use std::fmt::Debug;
use ascent::ascent;
use ascent::eclass_id;
use ascent::union_find::EqRel;
use ascent::aggregators::*;
use ascent::lattice::set::Set;
use ascent::Dual;
use ascent::util::calc_id;
use crate::equiv_vec_huh;

macro_rules! canonicalize {
   ($expr:expr) => {
      _self.runtime_total.__equiv_ind_common.combined.get_dominant_elem(#expr).unwrap_or(#expr)
   };
}

todo!(());
