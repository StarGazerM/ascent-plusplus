// use crate::fake_vec::FakeVec;

#[doc(hidden)]
#[macro_export]
macro_rules! linear_rel_codegen {
   ( $($tt: tt)* ) => { };
}
pub use linear_rel_codegen as rel_codegen;

#[doc(hidden)]
#[macro_export]
macro_rules! linear_rel {
    ($name: ident, $field_types: ty, $indices: expr, ser, ()) => {
        $crate::fake_vec::FakeVec<$field_types>
     };
     ($name: ident, $field_types: ty, $indices: expr, par, ()) => {
        $crate::fake_vec::FakeVec<$field_types>
     };
}
use ascent::internal::ToRelIndex;
pub use linear_rel as rel;

// shared nothing
#[doc(hidden)]
#[macro_export]
macro_rules! linear_ind_common {
   ($name: ident, $field_types: ty, $indices: expr, ser, ()) => {
      ()
   };
   ($name: ident, $field_types: ty, $indices: expr, par, ()) => {
      ()
   };
}
pub use linear_ind_common as rel_ind_common;

#[doc(hidden)]
#[macro_export]
macro_rules! rel_full_ind {
   ($name: ident, $field_types: ty, $indices: expr, ser, (), $key: ty, $val: ty) => {
      $crate::linear::linear_ind::LinearRelIndexFullType<$key, $val>
   };
   ($name: ident, $field_types: ty, $indices: expr, par, (), $key: ty, $val: ty) => {
      $crate::linear::linear_ind::CLinearRelIndexFull<$key, $val>
   };
}

pub use rel_full_ind;

use super::linear_ind::{LinearRelIndexFullType, LinearRelIndexType};

#[doc(hidden)]
#[macro_export]
macro_rules! rel_ind {
   ($name: ident, $field_types: ty, $indices: expr, ser, (), $ind: expr, $key: ty, $val: ty) => {
      $crate::linear::linear::ToLinearRelIndexType<$key, $val>
   };
   ($name: ident, $field_types: ty, $indices: expr, par, (), [], $key: ty, $val: ty) => {
      $crate::linear::linear_ind::CLinearRelNoIndex<$val>
   };
   ($name: ident, $field_types: ty, $indices: expr, par, (), $ind: expr, $key: ty, $val: ty) => {
      $crate::linear::linear_ind::CLinearRelIndex<$key, $val>
   };
}

pub use rel_ind;

#[derive(Clone)]
pub struct ToLinearRelIndexType<K, V>(pub LinearRelIndexType<K, V>);

impl<K, V> Default for ToLinearRelIndexType<K, V> {
   fn default() -> Self { ToLinearRelIndexType(Default::default()) }
}

impl<K, V, R> ToRelIndex<R> for ToLinearRelIndexType<K, V> {
   type RelIndex<'a>
      = &'a LinearRelIndexType<K, V>
   where
      Self: 'a,
      R: 'a;

   #[inline(always)]
   fn to_rel_index<'a>(&'a self, _rel: &'a R) -> Self::RelIndex<'a> { &self.0 }

   type RelIndexWrite<'a>
      = &'a mut LinearRelIndexType<K, V>
   where
      Self: 'a,
      R: 'a;

   #[inline(always)]
   fn to_rel_index_write<'a>(&'a mut self, _rel: &'a mut R) -> Self::RelIndexWrite<'a> { &mut self.0 }
}

impl<K, V, Rel> ToRelIndex<Rel> for LinearRelIndexFullType<K, V> {
   type RelIndex<'a>
      = &'a Self
   where
      Self: 'a,
      Rel: 'a;
   #[inline(always)]
   fn to_rel_index<'a>(&'a self, _rel: &'a Rel) -> Self::RelIndex<'a> { self }

   type RelIndexWrite<'a>
      = &'a mut Self
   where
      Self: 'a,
      Rel: 'a;

   #[inline(always)]
   fn to_rel_index_write<'a>(&'a mut self, _rel: &'a mut Rel) -> Self::RelIndexWrite<'a> { self }
}

use super::linear_ind::{CLinearNoIndex, CLinearRelIndex, CLinearRelIndexFull};
impl<K, V, Rel> ToRelIndex<Rel> for CLinearRelIndex<K, V> {
   type RelIndex<'a>
      = &'a Self
   where
      Self: 'a,
      Rel: 'a;
   #[inline(always)]
   fn to_rel_index<'a>(&'a self, _rel: &'a Rel) -> Self::RelIndex<'a> { self }

   type RelIndexWrite<'a>
      = &'a mut Self
   where
      Self: 'a,
      Rel: 'a;
   #[inline(always)]
   fn to_rel_index_write<'a>(&'a mut self, _rel: &'a mut Rel) -> Self::RelIndexWrite<'a> { self }
}

impl<K, V, Rel> ToRelIndex<Rel> for CLinearRelIndexFull<K, V> {
   type RelIndex<'a>
      = &'a Self
   where
      Self: 'a,
      Rel: 'a;
   #[inline(always)]
   fn to_rel_index<'a>(&'a self, _rel: &'a Rel) -> Self::RelIndex<'a> { self }

   type RelIndexWrite<'a>
      = &'a mut Self
   where
      Self: 'a,
      Rel: 'a;
   #[inline(always)]
   fn to_rel_index_write<'a>(&'a mut self, _rel: &'a mut Rel) -> Self::RelIndexWrite<'a> { self }
}

impl<V, Rel> ToRelIndex<Rel> for CLinearNoIndex<V> {
   type RelIndex<'a>
      = &'a Self
   where
      Self: 'a,
      Rel: 'a;
   #[inline(always)]
   fn to_rel_index<'a>(&'a self, _rel: &'a Rel) -> Self::RelIndex<'a> { self }

   type RelIndexWrite<'a>
      = &'a mut Self
   where
      Self: 'a,
      Rel: 'a;
   #[inline(always)]
   fn to_rel_index_write<'a>(&'a mut self, _rel: &'a mut Rel) -> Self::RelIndexWrite<'a> { self }
}
