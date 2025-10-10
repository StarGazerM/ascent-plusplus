use ascent::internal::{RelFullIndexRead, RelIndexRead, RelIndexWrite, ToRelIndex};
use smtlib::{
   Bool, Int, SatResultWithModel, Solver, Sorted, Storage,
   backend::z3_binary::Z3Binary,
   terms::{Const, StaticSorted},
};
use std::hash::{Hash, Hasher};
use std::sync::Arc;

#[derive(Clone)]
pub struct ArithmFn(Arc<dyn for<'a> Fn(Const<'a, Int<'a>>, Const<'a, Int<'a>>) -> Bool<'a> + Send + Sync>);
impl ArithmFn {
   pub fn new<F>(f: F) -> Self
   where
      F: for<'a> Fn(Const<'a, Int<'a>>, Const<'a, Int<'a>>) -> Bool<'a> + Send + Sync + 'static,
   {
      ArithmFn(Arc::new(f))
   }

   pub fn call<'a>(&self, a: Const<'a, Int<'a>>, b: Const<'a, Int<'a>>) -> Bool<'a> {
      (self.0)(a, b)
   }
}
impl PartialEq for ArithmFn {
   fn eq(&self, other: &Self) -> bool {
      Arc::ptr_eq(&self.0, &other.0)
   }
}
impl Eq for ArithmFn {}
impl Hash for ArithmFn {
   fn hash<H: Hasher>(&self, state: &mut H) {
      let ptr = Arc::as_ptr(&self.0) as *const () as usize;
      ptr.hash(state);
   }
}

#[derive(Clone, Default)]
pub struct Equation();
impl Equation {
   pub fn is_empty(&self) -> bool {
      false
   }
}
impl<'a> RelFullIndexRead<'a> for Equation {
   type Key = (ArithmFn, i32, i32);
   fn contains_key(&self, key: &Self::Key) -> bool {
      let (f, op1, op2) = key;
      let st = Storage::new(); // Create a new Storage for each check
      let mut solver = Solver::new(&st, Z3Binary::new("z3").unwrap()).expect("Failed to create Z3 solver");
      let smt_op1 = Int::new_const(&st, "op1");
      let smt_op2 = Int::new_const(&st, "op2");
      solver.assert(smt_op1._eq(*op1 as i64)).expect("Failed to assert constraint");
      solver.assert(smt_op2._eq(*op2 as i64)).expect("Failed to assert constraint");
      solver.assert(f.call(smt_op1, smt_op2)).expect("Failed to assert constraint");
      solver.check_sat().expect("Failed to check satisfiability") == smtlib::SatResult::Sat
   }
}
empty_to_rel_index!(Equation);
empty_rel_index_write!(Equation, (ArithmFn, i32, i32), ());

#[derive(Clone, Default)]
pub struct EquationInd0(pub Equation);
impl<'a> RelIndexRead<'a> for EquationInd0 {
   type Key = (ArithmFn, i32);
   type Value = (i32,);
   type IteratorType = std::iter::Once<Self::Value>;
   fn index_get(&'a self, key: &Self::Key) -> Option<Self::IteratorType> {
      let (f, op1) = key;
      let st = Storage::new();
      let mut solver = Solver::new(&st, Z3Binary::new("z3").unwrap()).expect("Failed to create Z3 solver");
      let smt_op1 = Int::new_const(&st, "op1");
      let smt_op2 = Int::new_const(&st, "op2");
      solver.assert(smt_op1._eq(*op1 as i64)).expect("Failed to assert constraint");
      solver.assert(f.call(smt_op1, smt_op2)).expect("Failed to assert constraint");
      match solver.check_sat_with_model().expect("Failed to check satisfiability") {
         SatResultWithModel::Sat(model) => {
            let op2 = model.eval(smt_op2).unwrap().to_string().parse::<i32>().unwrap();
            Some(std::iter::once((op2,)))
         },
         _ => None,
      }
   }
   fn len_estimate(&self) -> usize {
      1
   }
}
empty_rel_index_write!(EquationInd0, (ArithmFn, i32), (i32,));
empty_to_rel_index!(EquationInd0);

pub struct EquationInd1(pub Equation);
impl<'a> RelIndexRead<'a> for EquationInd1 {
   type Key = (ArithmFn, i32);
   type Value = (i32,);
   type IteratorType = std::iter::Once<Self::Value>;
   fn index_get(&'a self, key: &Self::Key) -> Option<Self::IteratorType> {
      let (f, op2) = key;
      let st = Storage::new();
      let mut solver = Solver::new(&st, Z3Binary::new("z3").unwrap()).expect("Failed to create Z3 solver");
      let smt_op1 = Int::new_const(&st, "op1");
      let smt_op2 = Int::new_const(&st, "op2");
      solver.assert(smt_op2._eq(*op2 as i64)).expect("Failed to assert constraint");
      solver.assert(f.call(smt_op1, smt_op2)).expect("Failed to assert constraint");
      match solver.check_sat_with_model().expect("Failed to check satisfiability") {
         SatResultWithModel::Sat(model) => {
            let op2 = model.eval(smt_op1).unwrap().to_string().parse::<i32>().unwrap();
            Some(std::iter::once((op2,)))
         },
         _ => None,
      }
   }
   fn len_estimate(&self) -> usize {
      1
   }
}
empty_rel_index_write!(EquationInd1, (ArithmFn, i32), (i32,));
empty_to_rel_index!(EquationInd1);

// construct BYODs relation from equation
// we don't need to access internals of equation, use a fake data for this
#[doc(hidden)]
#[macro_export]
macro_rules! arithm_rel {
   ($name: ident, $itype: ty, $indices: expr, ser, ()) => {
      ascent_byods_rels::fake_vec::FakeVec<($crate::arithm::ArithmFn, i32, i32)>
   };
}
pub use arithm_rel as rel;

#[doc(hidden)]
#[macro_export]
macro_rules! arithm_rel_full_ind {
   ($name: ident, ($func: ty, $col1: ty, $col2: ty), $indices: expr, ser, (), $key: ty, $val: ty) => {
      $crate::arithm::Equation
   };
}
pub use arithm_rel_full_ind as rel_full_ind;

#[doc(hidden)]
#[macro_export]
macro_rules! arithm_rel_ind {
   ($name: ident, $field_types: ty, $indices: expr, ser, (), [0,1], $key: ty, $val: ty) => {
      $crate::arithm::EquationInd0
   };
   ($name: ident, $field_types: ty, $indices: expr, ser, (), [0,2], $key: ty, $val: ty) => {
      $crate::arithm::EquationInd1
   };
}
pub use arithm_rel_ind as rel_ind;

#[doc(hidden)]
#[macro_export]
macro_rules! arithm_rel_codegen {
   ( $($tt: tt)* ) => {};
}
pub use arithm_rel_codegen as rel_codegen;
#[doc(hidden)]
#[macro_export]
macro_rules! arithm_rel_ind_common {
   ($name: ident, $field_types: ty, $indices: expr, ser, ()) => {
      ()
   };
}
pub use arithm_rel_ind_common as rel_ind_common;

#[doc(hidden)]
#[macro_export]
macro_rules! eq {
   ($var1: ident, $var2: ident in ($lhs: expr) = ($rhs: expr)) => {
      ArithmFn::new(|$var1, $var2| $lhs._eq($rhs))
   };
}
pub use eq;

use crate::{empty_rel_index_write, empty_to_rel_index};
