//! equivalence relations with canonicalization as third column for Ascent

#[doc(hidden)]
#[macro_export]
macro_rules! eqrel_canonical_rel_codegen {
   ( $($tt: tt)* ) => {};
}
pub use eqrel_canonical_rel_codegen as rel_codegen;

#[doc(hidden)]
#[macro_export]
macro_rules! eqrel_canonical_rel {
   ($name: ident, ($col1: ty, $col2: ty, $col3: ty), $indices: expr, ser, ()) => {
      // $crate::fake_vec::FakeVec<($col1, $col2, $col3)>
      $crate::fake_vec::VecEqRel
   };
   ($name: ident, ($col1: ty, $col2: ty, $col3: ty), $indices: expr, par, ()) => {
      $crate::fake_vec::FakeVec<($col1, $col2, $col3)>
   };
}
pub use eqrel_canonical_rel as rel;

#[doc(hidden)]
#[macro_export]
macro_rules! eqrel_canonical_rel_full_ind {
    ($name: ident, ($col1: ty, $col2: ty, $col3: ty), $indices: expr, ser, (), $key: ty, $val: ty) => {
        $crate::eqrel_canonical_ind::ToEqRelCanonicalInd0_1_2<$col1>
     };
}
pub use eqrel_canonical_rel_full_ind as rel_full_ind;

#[doc(hidden)]
#[macro_export]
macro_rules! eqrel_canonical_rel_ind {
   ($name: ident, ($col1: ty, $col2: ty, $col3: ty), $indices: expr, ser, (), [], $key: ty, $val: ty) => {
      $crate::eqrel_canonical_ind::ToEqRelCanonicalIndNone<$col1>
   };
   ($name: ident, ($col1: ty, $col2: ty, $col3: ty), $indices: expr, ser, (), [0], $key: ty, $val: ty) => {
      $crate::eqrel_canonical_ind::ToEqRelCanonicalInd0<$col1>
   };
   ($name: ident, ($col1: ty, $col2: ty, $col3: ty), $indices: expr, ser, (), [1], $key: ty, $val: ty) => {
      $crate::eqrel_canonical_ind::ToEqRelCanonicalInd0<$col1>
   };
   ($name: ident, ($col1: ty, $col2: ty, $col3: ty), $indices: expr, ser, (), [2], $key: ty, $val: ty) => {
      $crate::eqrel_canonical_ind::ToEqRelCanonicalInd2<$col1>
   };
   ($name: ident, ($col1: ty, $col2: ty, $col3: ty), $indices: expr, ser, (), [0, 1], $key: ty, $val: ty) => {
      $crate::eqrel_canonical_ind::ToEqRelCanonicalInd0_1<$col1>
   };
   ($name: ident, ($col1: ty, $col2: ty, $col3: ty), $indices: expr, ser, (), [0, 2], $key: ty, $val: ty) => {
      $crate::eqrel_canonical_ind::ToEqRelCanonicalInd0_1<$col1>
   };
   ($name: ident, ($col1: ty, $col2: ty, $col3: ty), $indices: expr, ser, (), [1, 2], $key: ty, $val: ty) => {
      $crate::eqrel_canonical_ind::ToEqRelCanonicalInd0_1<$col1>
   };
   ($name: ident, ($col1: ty, $col2: ty, $col3: ty), $indices: expr, ser, (), [0, 1, 2], $key: ty, $val: ty) => {
   $crate::eqrel_canonical_ind::ToEqRelCanonicalInd0_1_2<$col1>
   };
}
pub use eqrel_canonical_rel_ind as rel_ind;

#[doc(hidden)]
#[macro_export]
macro_rules! eqrel_canonical_rel_ind_common {
    ($name: ident, ($col1: ty, $col2: ty, $col3: ty), $indices: expr, ser, ()) => {
        $crate::eqrel_ind::EqRelIndCommon<$col1>
     };
}
pub use eqrel_canonical_rel_ind_common as rel_ind_common;
