use ascent::internal::{RelFullIndexRead, RelIndexRead, RelIndexWrite, ToRelIndex};
use crate::util::{empty_rel_index_write, empty_to_rel_index};

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct InvertibleFn(pub fn(i32) -> i32, pub fn(i32) -> i32);

#[derive(Clone, Default)]
pub struct Invertible();
impl Invertible {
   pub fn is_empty(&self) -> bool {
      false
   }
}
impl<'a> RelFullIndexRead<'a> for Invertible {
   type Key = (InvertibleFn, i32, i32);
   fn contains_key(&self, key: &Self::Key) -> bool {
      let (f, op1, op2) = key;
      f.0(*op1) == *op2 && f.1(*op2) == *op1
   }
}
empty_to_rel_index!(Invertible);
empty_rel_index_write!(Invertible, (InvertibleFn, i32, i32), ());

#[derive(Clone, Default)]
pub struct InvertibleInd0(pub Invertible);
impl<'a> RelIndexRead<'a> for InvertibleInd0 {
   type Key = (InvertibleFn, i32);
   type Value = (i32,);
   // type IteratorType = std::iter::Once<Self::Value>;
   fn index_get(&'a self, key: &Self::Key) -> Option<impl Iterator<Item = Self::Value> + Clone + 'a> {
      let (f, op1) = key;
      Some(std::iter::once((f.0(*op1),)))
   }
   fn len_estimate(&self) -> usize {
      1
   }
}
empty_rel_index_write!(InvertibleInd0, (InvertibleFn, i32), (i32,));
empty_to_rel_index!(InvertibleInd0);

pub struct InvertibleInd1(pub Invertible);
impl<'a> RelIndexRead<'a> for InvertibleInd1 {
   type Key = (InvertibleFn, i32);
   type Value = (i32,);
   // type IteratorType = std::iter::Once<Self::Value>;
   fn index_get(&'a self, key: &Self::Key) -> Option<impl Iterator<Item = Self::Value> + Clone + 'a> {
      let (f, op2) = key;
      Some(std::iter::once((f.1(*op2),)))
   }
   fn len_estimate(&self) -> usize {
      1
   }
}
empty_rel_index_write!(InvertibleInd1, (InvertibleFn, i32), (i32,));
empty_to_rel_index!(InvertibleInd1);

// construct BYODs relation from equation
// we don't need to access internals of equation, use a fake data for this
#[doc(hidden)]
#[macro_export]
macro_rules! invertible_rel {
   ($name: ident, $itype: ty, $indices: expr, ser, ()) => {
      ascent_byods_rels::fake_vec::FakeVec<($crate::invertible::InvertibleFn, i32, i32)>
   };
}
pub use invertible_rel as rel;

#[doc(hidden)]
#[macro_export]
macro_rules! invertible_rel_full_ind {
   ($name: ident, ($func: ty, $col1: ty, $col2: ty), $indices: expr, ser, (), $key: ty, $val: ty) => {
      $crate::invertible::Invertible
   };
}
pub use invertible_rel_full_ind as rel_full_ind;

#[doc(hidden)]
#[macro_export]
macro_rules! invertible_rel_ind {
   ($name: ident, $field_types: ty, $indices: expr, ser, (), [0,1], $key: ty, $val: ty) => {
      $crate::invertible::InvertibleInd0
   };
   ($name: ident, $field_types: ty, $indices: expr, ser, (), [0,2], $key: ty, $val: ty) => {
      $crate::invertible::InvertibleInd1
   };
}
pub use invertible_rel_ind as rel_ind;

#[doc(hidden)]
#[macro_export]
macro_rules! invertible_rel_codegen {
   ( $($tt: tt)* ) => {};
}
pub use invertible_rel_codegen as rel_codegen;
#[doc(hidden)]
#[macro_export]
macro_rules! invertible_rel_ind_common {
   ($name: ident, $field_types: ty, $indices: expr, ser, ()) => {
      ()
   };
}
pub use invertible_rel_ind_common as rel_ind_common;
