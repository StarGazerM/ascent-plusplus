#![allow(unused_imports)]
use std::{clone, cmp::max, rc::Rc, cell::RefCell};
use std::ops::Deref;
use std::hash::Hash;
use std::fmt::Debug;
use ascent::ascent;
use slog_utils::eclass_id;
use ascent::aggregators::*;
use ascent::lattice::set::Set;
use ascent::Dual;
use slog_utils::calc_id;
use slog_utils::collect;
use slog_theory::canonicalize_eclass;
use slog_theory::eq_theory::eq_theory as theory_rules;

todo!(());
