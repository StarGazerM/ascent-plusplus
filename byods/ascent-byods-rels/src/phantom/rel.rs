#[doc(hidden)]
#[macro_export]
macro_rules! phantom_rel_codegen {
   ( $($tt: tt)* ) => {};
}
pub use phantom_rel_codegen as rel_codegen;

#[doc(hidden)]
#[macro_export]
macro_rules! phantom_rel {
    ($name: ident, $field_types: ty, $indices: expr, ser, ()) => {
        $crate::phantom::phantom_vec::PhantomVec<$field_types>
    };
    ($name: ident, $field_types: ty, $indices: expr, par, ()) => {
        $crate::phantom::phantom_vec::rel::PhantomVec<$field_types>
    };
}
use ascent::internal::ToRelIndex;
pub use phantom_rel as rel;

// shared nothing
#[doc(hidden)]
#[macro_export]
macro_rules! phantom_ind_common {
   ($name: ident, $field_types: ty, $indices: expr, ser, ()) => {
      ()
   };
   ($name: ident, $field_types: ty, $indices: expr, par, ()) => {
      ()
   };
}
pub use phantom_ind_common as rel_ind_common;

#[doc(hidden)]
#[macro_export]
macro_rules! phantom_full_ind {
   ($name: ident, $field_types: ty, $indices: expr, ser, (), $key: ty, $val: ty) => {
      $crate::phantom::ind::PhantomRelIndexFullType<$key, $val>
   };
   ($name: ident, $field_types: ty, $indices: expr, par, (), $key: ty, $val: ty) => {
      $crate::phantom::ind::PhantomRelIndexFullType<$key, $val>
   };
}
pub use phantom_full_ind as rel_full_ind;

use super::ind::{PhantomRelIndexFullType, PhantomRelIndexType};

#[doc(hidden)]
#[macro_export]
macro_rules! phantom_rel_ind {
    ($name: ident, $field_types: ty, $indices: expr, ser, (), $ind: expr, $key: ty, $val: ty) => {
        $crate::phantom::rel::ToPhantomRelIndexType<$key, $val>
    };
    ($name: ident, $field_types: ty, $indices: expr, par, (), [], $key: ty, $val: ty) => {
        $crate::phantom::ind::PhantomRelNoIndex<$val>
    };
    ($name: ident, $field_types: ty, $indices: expr, par, (), $ind: expr, $key: ty, $val: ty) => {
        $crate::phantom::ind::PhantomRelIndex<$val>
    };
}

pub use phantom_rel_ind as rel_ind;

#[derive(Clone)]
pub struct ToPhantomRelIndexType<K, V>(pub PhantomRelIndexType<K, V>);

impl<K, V> Default for ToPhantomRelIndexType<K, V> {
   fn default() -> Self { ToPhantomRelIndexType(Default::default()) }
}

impl<K, V, R> ToRelIndex<R> for ToPhantomRelIndexType<K, V> {
   type RelIndex<'a>
      = &'a PhantomRelIndexType<K, V>
   where
      Self: 'a,
      R: 'a;

   #[inline(always)]
   fn to_rel_index<'a>(&'a self, _rel: &'a R) -> Self::RelIndex<'a> { &self.0 }

   type RelIndexWrite<'a>
      = &'a mut PhantomRelIndexType<K, V>
   where
      Self: 'a,
      R: 'a;

   #[inline(always)]
   fn to_rel_index_write<'a>(&'a mut self, _rel: &'a mut R) -> Self::RelIndexWrite<'a> { &mut self.0 }
}

impl<K, V, Rel> ToRelIndex<Rel> for PhantomRelIndexFullType<K, V> {
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

use super::ind::{CPhantomNoIndex, CPhantomRelIndex, CPhantomRelIndexFull};
impl<K, V, Rel> ToRelIndex<Rel> for CPhantomRelIndex<K, V> {
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

impl<K, V, Rel> ToRelIndex<Rel> for CPhantomRelIndexFull<K, V> {
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

impl<V, Rel> ToRelIndex<Rel> for CPhantomNoIndex<V> {
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
