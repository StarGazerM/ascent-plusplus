#![allow(unused_imports)]
use std::clone;
use std::cmp::max;
use std::fmt::Debug;
use std::hash::Hash;
use std::ops::Deref;
use std::rc::Rc;

#[allow(dead_code)]
pub trait Atom:
   From<usize> + Into<usize> + Copy + Clone + std::fmt::Debug + Eq + Ord + Hash + Sync + Send + 'static
{
   fn index(self) -> usize;
}

#[allow(dead_code)]
pub trait FactTypes: Copy + Clone + Debug {
   type Origin: Atom;
   type Loan: Atom;
   type Point: Atom;
   type Variable: Atom;
   type Path: Atom;
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum List<T> {
   Cons(T, Rc<List<T>>),
   Nil,
}

macro_rules! cons {
   ($h: expr, $t: expr) => {
        Rc::new(List::Cons($h, $t))
    };
}

macro_rules! nil {
   () => {
        Rc::new(List::Nil)
    };
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct Res(&'static str);

#[warn(warnings)]
#[allow(unused_imports)]
#[allow(dead_code)]
#[allow(redundant_semicolons)]
#[cfg(test)]
fn _test<T: FactTypes>() {
   use ascent::aggregators::*;
   use ascent::lattice::set::Set;
   use ascent::{Dual, rel as custom_ds};
   ::ascent::rel::rel_codegen! { AscentProgram_a , (i32 , i32) , [[0 , 1] , [1]] , ser , () }
   ::ascent::rel::rel_codegen! { AscentProgram_b , (i32 , i32) , [[] , [0] , [0 , 1] , [1]] , ser , () }
   ::ascent::rel::rel_codegen! { AscentProgram_c , (i32 , i32) , [[] , [0] , [0 , 1]] , ser , () }
   pub struct AscentProgram {
      #[doc = "\nlogical indices: a_indices_0_1; a_indices_1"]
      pub a: ::ascent::rel::rel!(AscentProgram_a, (i32, i32), [[0, 1], [1]], ser, ()),
      __a_ind_common: ::ascent::rel::rel_ind_common!(AscentProgram_a, (i32, i32), [[0, 1], [1]], ser, ()),
      a_indices_0_1: ::ascent::rel::rel_full_ind!(AscentProgram_a, (i32, i32), [[0, 1], [1]], ser, (), (i32, i32), ()),
      a_indices_1: ::ascent::rel::rel_ind!(AscentProgram_a, (i32, i32), [[0, 1], [1]], ser, (), [1], (i32,), (i32,)),
      #[doc = "\nlogical indices: b_indices_0; b_indices_0_1; b_indices_1; b_indices_none"]
      pub b: ::ascent::rel::rel!(AscentProgram_b, (i32, i32), [[], [0], [0, 1], [1]], ser, ()),
      __b_ind_common: ::ascent::rel::rel_ind_common!(AscentProgram_b, (i32, i32), [[], [0], [0, 1], [1]], ser, ()),
      b_indices_0:
         ::ascent::rel::rel_ind!(AscentProgram_b, (i32, i32), [[], [0], [0, 1], [1]], ser, (), [0], (i32,), (i32,)),
      b_indices_0_1:
         ::ascent::rel::rel_full_ind!(AscentProgram_b, (i32, i32), [[], [0], [0, 1], [1]], ser, (), (i32, i32), ()),
      b_indices_1:
         ::ascent::rel::rel_ind!(AscentProgram_b, (i32, i32), [[], [0], [0, 1], [1]], ser, (), [1], (i32,), (i32,)),
      b_indices_none:
         ::ascent::rel::rel_ind!(AscentProgram_b, (i32, i32), [[], [0], [0, 1], [1]], ser, (), [], (), (i32, i32)),
      #[doc = "\nlogical indices: c_indices_0; c_indices_0_1; c_indices_none"]
      pub c: ::ascent::rel::rel!(AscentProgram_c, (i32, i32), [[], [0], [0, 1]], ser, ()),
      __c_ind_common: ::ascent::rel::rel_ind_common!(AscentProgram_c, (i32, i32), [[], [0], [0, 1]], ser, ()),
      c_indices_0:
         ::ascent::rel::rel_ind!(AscentProgram_c, (i32, i32), [[], [0], [0, 1]], ser, (), [0], (i32,), (i32,)),
      c_indices_0_1:
         ::ascent::rel::rel_full_ind!(AscentProgram_c, (i32, i32), [[], [0], [0, 1]], ser, (), (i32, i32), ()),
      c_indices_none:
         ::ascent::rel::rel_ind!(AscentProgram_c, (i32, i32), [[], [0], [0, 1]], ser, (), [], (), (i32, i32)),
      scc_times: [std::time::Duration; 1usize],
      scc_iters: [usize; 1usize],
      update_time_nanos: std::sync::atomic::AtomicU64,
      update_indices_duration: std::time::Duration,
   }
   impl AscentProgram {
      #[allow(unused_imports, noop_method_call, suspicious_double_ref_op)]
      #[doc = "Runs the Ascent program to a fixed point."]
      pub fn run(&mut self) {
         #![allow(clippy::all)]
         macro_rules! __check_return_conditions {
            () => { };
         }
         use core::cmp::PartialEq;

         use ascent::internal::{RelIndexRead, RelIndexReadAll, RelIndexWrite, ToRelIndex0, TupleOfBorrowed};
         self.update_indices_priv();
         let _self = self;
         ascent::internal::comment("scc 0");
         {
            let _scc_start_time = ::ascent::internal::Instant::now();
            let mut __a_ind_common_delta: ::ascent::rel::rel_ind_common!(
               AscentProgram_a,
               (i32, i32),
               [[0, 1], [1]],
               ser,
               ()
            ) = ::std::mem::take(&mut _self.__a_ind_common);
            let mut __a_ind_common_total: ::ascent::rel::rel_ind_common!(
               AscentProgram_a,
               (i32, i32),
               [[0, 1], [1]],
               ser,
               ()
            ) = Default::default();
            let mut __a_ind_common_new: ::ascent::rel::rel_ind_common!(
               AscentProgram_a,
               (i32, i32),
               [[0, 1], [1]],
               ser,
               ()
            ) = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut __a_ind_common_new, &mut __a_ind_common_delta, &mut __a_ind_common_total,
            );
            let mut a_indices_0_1_delta: ::ascent::rel::rel_full_ind!(
               AscentProgram_a,
               (i32, i32),
               [[0, 1], [1]],
               ser,
               (),
               (i32, i32),
               ()
            ) = ::std::mem::take(&mut _self.a_indices_0_1);
            let mut a_indices_0_1_total: ::ascent::rel::rel_full_ind!(
               AscentProgram_a,
               (i32, i32),
               [[0, 1], [1]],
               ser,
               (),
               (i32, i32),
               ()
            ) = Default::default();
            let mut a_indices_0_1_new: ::ascent::rel::rel_full_ind!(
               AscentProgram_a,
               (i32, i32),
               [[0, 1], [1]],
               ser,
               (),
               (i32, i32),
               ()
            ) = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut a_indices_0_1_new.to_rel_index_write(&mut __a_ind_common_new),
               &mut a_indices_0_1_delta.to_rel_index_write(&mut __a_ind_common_delta),
               &mut a_indices_0_1_total.to_rel_index_write(&mut __a_ind_common_total),
            );
            let mut a_indices_1_delta: ::ascent::rel::rel_ind!(
               AscentProgram_a,
               (i32, i32),
               [[0, 1], [1]],
               ser,
               (),
               [1],
               (i32,),
               (i32,)
            ) = ::std::mem::take(&mut _self.a_indices_1);
            let mut a_indices_1_total: ::ascent::rel::rel_ind!(
               AscentProgram_a,
               (i32, i32),
               [[0, 1], [1]],
               ser,
               (),
               [1],
               (i32,),
               (i32,)
            ) = Default::default();
            let mut a_indices_1_new: ::ascent::rel::rel_ind!(
               AscentProgram_a,
               (i32, i32),
               [[0, 1], [1]],
               ser,
               (),
               [1],
               (i32,),
               (i32,)
            ) = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut a_indices_1_new.to_rel_index_write(&mut __a_ind_common_new),
               &mut a_indices_1_delta.to_rel_index_write(&mut __a_ind_common_delta),
               &mut a_indices_1_total.to_rel_index_write(&mut __a_ind_common_total),
            );
            let mut __b_ind_common_delta: ::ascent::rel::rel_ind_common!(
               AscentProgram_b,
               (i32, i32),
               [[], [0], [0, 1], [1]],
               ser,
               ()
            ) = ::std::mem::take(&mut _self.__b_ind_common);
            let mut __b_ind_common_total: ::ascent::rel::rel_ind_common!(
               AscentProgram_b,
               (i32, i32),
               [[], [0], [0, 1], [1]],
               ser,
               ()
            ) = Default::default();
            let mut __b_ind_common_new: ::ascent::rel::rel_ind_common!(
               AscentProgram_b,
               (i32, i32),
               [[], [0], [0, 1], [1]],
               ser,
               ()
            ) = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut __b_ind_common_new, &mut __b_ind_common_delta, &mut __b_ind_common_total,
            );
            let mut b_indices_0_delta: ::ascent::rel::rel_ind!(
               AscentProgram_b,
               (i32, i32),
               [[], [0], [0, 1], [1]],
               ser,
               (),
               [0],
               (i32,),
               (i32,)
            ) = ::std::mem::take(&mut _self.b_indices_0);
            let mut b_indices_0_total: ::ascent::rel::rel_ind!(
               AscentProgram_b,
               (i32, i32),
               [[], [0], [0, 1], [1]],
               ser,
               (),
               [0],
               (i32,),
               (i32,)
            ) = Default::default();
            let mut b_indices_0_new: ::ascent::rel::rel_ind!(
               AscentProgram_b,
               (i32, i32),
               [[], [0], [0, 1], [1]],
               ser,
               (),
               [0],
               (i32,),
               (i32,)
            ) = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut b_indices_0_new.to_rel_index_write(&mut __b_ind_common_new),
               &mut b_indices_0_delta.to_rel_index_write(&mut __b_ind_common_delta),
               &mut b_indices_0_total.to_rel_index_write(&mut __b_ind_common_total),
            );
            let mut b_indices_0_1_delta: ::ascent::rel::rel_full_ind!(
               AscentProgram_b,
               (i32, i32),
               [[], [0], [0, 1], [1]],
               ser,
               (),
               (i32, i32),
               ()
            ) = ::std::mem::take(&mut _self.b_indices_0_1);
            let mut b_indices_0_1_total: ::ascent::rel::rel_full_ind!(
               AscentProgram_b,
               (i32, i32),
               [[], [0], [0, 1], [1]],
               ser,
               (),
               (i32, i32),
               ()
            ) = Default::default();
            let mut b_indices_0_1_new: ::ascent::rel::rel_full_ind!(
               AscentProgram_b,
               (i32, i32),
               [[], [0], [0, 1], [1]],
               ser,
               (),
               (i32, i32),
               ()
            ) = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut b_indices_0_1_new.to_rel_index_write(&mut __b_ind_common_new),
               &mut b_indices_0_1_delta.to_rel_index_write(&mut __b_ind_common_delta),
               &mut b_indices_0_1_total.to_rel_index_write(&mut __b_ind_common_total),
            );
            let mut b_indices_1_delta: ::ascent::rel::rel_ind!(
               AscentProgram_b,
               (i32, i32),
               [[], [0], [0, 1], [1]],
               ser,
               (),
               [1],
               (i32,),
               (i32,)
            ) = ::std::mem::take(&mut _self.b_indices_1);
            let mut b_indices_1_total: ::ascent::rel::rel_ind!(
               AscentProgram_b,
               (i32, i32),
               [[], [0], [0, 1], [1]],
               ser,
               (),
               [1],
               (i32,),
               (i32,)
            ) = Default::default();
            let mut b_indices_1_new: ::ascent::rel::rel_ind!(
               AscentProgram_b,
               (i32, i32),
               [[], [0], [0, 1], [1]],
               ser,
               (),
               [1],
               (i32,),
               (i32,)
            ) = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut b_indices_1_new.to_rel_index_write(&mut __b_ind_common_new),
               &mut b_indices_1_delta.to_rel_index_write(&mut __b_ind_common_delta),
               &mut b_indices_1_total.to_rel_index_write(&mut __b_ind_common_total),
            );
            let mut b_indices_none_delta: ::ascent::rel::rel_ind!(
               AscentProgram_b,
               (i32, i32),
               [[], [0], [0, 1], [1]],
               ser,
               (),
               [],
               (),
               (i32, i32)
            ) = ::std::mem::take(&mut _self.b_indices_none);
            let mut b_indices_none_total: ::ascent::rel::rel_ind!(
               AscentProgram_b,
               (i32, i32),
               [[], [0], [0, 1], [1]],
               ser,
               (),
               [],
               (),
               (i32, i32)
            ) = Default::default();
            let mut b_indices_none_new: ::ascent::rel::rel_ind!(
               AscentProgram_b,
               (i32, i32),
               [[], [0], [0, 1], [1]],
               ser,
               (),
               [],
               (),
               (i32, i32)
            ) = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut b_indices_none_new.to_rel_index_write(&mut __b_ind_common_new),
               &mut b_indices_none_delta.to_rel_index_write(&mut __b_ind_common_delta),
               &mut b_indices_none_total.to_rel_index_write(&mut __b_ind_common_total),
            );
            let mut __c_ind_common_delta: ::ascent::rel::rel_ind_common!(
               AscentProgram_c,
               (i32, i32),
               [[], [0], [0, 1]],
               ser,
               ()
            ) = ::std::mem::take(&mut _self.__c_ind_common);
            let mut __c_ind_common_total: ::ascent::rel::rel_ind_common!(
               AscentProgram_c,
               (i32, i32),
               [[], [0], [0, 1]],
               ser,
               ()
            ) = Default::default();
            let mut __c_ind_common_new: ::ascent::rel::rel_ind_common!(
               AscentProgram_c,
               (i32, i32),
               [[], [0], [0, 1]],
               ser,
               ()
            ) = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut __c_ind_common_new, &mut __c_ind_common_delta, &mut __c_ind_common_total,
            );
            let mut c_indices_0_delta: ::ascent::rel::rel_ind!(
               AscentProgram_c,
               (i32, i32),
               [[], [0], [0, 1]],
               ser,
               (),
               [0],
               (i32,),
               (i32,)
            ) = ::std::mem::take(&mut _self.c_indices_0);
            let mut c_indices_0_total: ::ascent::rel::rel_ind!(
               AscentProgram_c,
               (i32, i32),
               [[], [0], [0, 1]],
               ser,
               (),
               [0],
               (i32,),
               (i32,)
            ) = Default::default();
            let mut c_indices_0_new: ::ascent::rel::rel_ind!(
               AscentProgram_c,
               (i32, i32),
               [[], [0], [0, 1]],
               ser,
               (),
               [0],
               (i32,),
               (i32,)
            ) = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut c_indices_0_new.to_rel_index_write(&mut __c_ind_common_new),
               &mut c_indices_0_delta.to_rel_index_write(&mut __c_ind_common_delta),
               &mut c_indices_0_total.to_rel_index_write(&mut __c_ind_common_total),
            );
            let mut c_indices_0_1_delta: ::ascent::rel::rel_full_ind!(
               AscentProgram_c,
               (i32, i32),
               [[], [0], [0, 1]],
               ser,
               (),
               (i32, i32),
               ()
            ) = ::std::mem::take(&mut _self.c_indices_0_1);
            let mut c_indices_0_1_total: ::ascent::rel::rel_full_ind!(
               AscentProgram_c,
               (i32, i32),
               [[], [0], [0, 1]],
               ser,
               (),
               (i32, i32),
               ()
            ) = Default::default();
            let mut c_indices_0_1_new: ::ascent::rel::rel_full_ind!(
               AscentProgram_c,
               (i32, i32),
               [[], [0], [0, 1]],
               ser,
               (),
               (i32, i32),
               ()
            ) = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut c_indices_0_1_new.to_rel_index_write(&mut __c_ind_common_new),
               &mut c_indices_0_1_delta.to_rel_index_write(&mut __c_ind_common_delta),
               &mut c_indices_0_1_total.to_rel_index_write(&mut __c_ind_common_total),
            );
            let mut c_indices_none_delta: ::ascent::rel::rel_ind!(
               AscentProgram_c,
               (i32, i32),
               [[], [0], [0, 1]],
               ser,
               (),
               [],
               (),
               (i32, i32)
            ) = ::std::mem::take(&mut _self.c_indices_none);
            let mut c_indices_none_total: ::ascent::rel::rel_ind!(
               AscentProgram_c,
               (i32, i32),
               [[], [0], [0, 1]],
               ser,
               (),
               [],
               (),
               (i32, i32)
            ) = Default::default();
            let mut c_indices_none_new: ::ascent::rel::rel_ind!(
               AscentProgram_c,
               (i32, i32),
               [[], [0], [0, 1]],
               ser,
               (),
               [],
               (),
               (i32, i32)
            ) = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut c_indices_none_new.to_rel_index_write(&mut __c_ind_common_new),
               &mut c_indices_none_delta.to_rel_index_write(&mut __c_ind_common_delta),
               &mut c_indices_none_total.to_rel_index_write(&mut __c_ind_common_total),
            );
            #[allow(unused_assignments, unused_variables)]
            loop {
               let mut __changed = false;
               ascent::internal::comment(
                  "a, b, c <-- a_indices_1_delta, b_indices_0_total+delta, c_indices_0_total+delta [SIMPLE JOIN]",
               );
               {
                  let any_rel_empty = a_indices_1_delta.to_rel_index(&__a_ind_common_delta).is_empty()
                     || ascent::internal::RelIndexCombined::new(
                        &b_indices_0_total.to_rel_index(&__b_ind_common_total),
                        &b_indices_0_delta.to_rel_index(&__b_ind_common_delta),
                     )
                     .is_empty()
                     || ascent::internal::RelIndexCombined::new(
                        &c_indices_0_total.to_rel_index(&__c_ind_common_total),
                        &c_indices_0_delta.to_rel_index(&__c_ind_common_delta),
                     )
                     .is_empty();
                  if !any_rel_empty {
                     if a_indices_1_delta.to_rel_index(&__a_ind_common_delta).len_estimate()
                        <= ascent::internal::RelIndexCombined::new(
                           &b_indices_0_total.to_rel_index(&__b_ind_common_total),
                           &b_indices_0_delta.to_rel_index(&__b_ind_common_delta),
                        )
                        .len_estimate()
                     {
                        a_indices_1_delta.to_rel_index(&__a_ind_common_delta).iter_all().for_each(
                           |(__cl1_joined_columns, __cl1_tuple_indices)| {
                              let __cl1_joined_columns = __cl1_joined_columns.tuple_of_borrowed();
                              let y = __cl1_joined_columns.0;
                              if let Some(__matching) = ascent::internal::RelIndexCombined::new(
                                 &b_indices_0_total.to_rel_index(&__b_ind_common_total),
                                 &b_indices_0_delta.to_rel_index(&__b_ind_common_delta),
                              )
                              .index_get(&(y.clone(),))
                              {
                                 __cl1_tuple_indices.for_each(|cl1_val| {
                                    let cl1_val = cl1_val.tuple_of_borrowed();
                                    let x: &i32 = cl1_val.0;
                                    __matching.clone().for_each(|__val| {
                                       let __val = __val.tuple_of_borrowed();
                                       let z: &i32 = __val.0;
                                       if let Some(__matching) = ascent::internal::RelIndexCombined::new(
                                          &c_indices_0_total.to_rel_index(&__c_ind_common_total),
                                          &c_indices_0_delta.to_rel_index(&__c_ind_common_delta),
                                       )
                                       .index_get(&(z.clone(),))
                                       {
                                          __matching.for_each(|__val| {
                                             let __val = __val.tuple_of_borrowed();
                                             let w: &i32 = __val.0;
                                             let __new_row: (i32, i32) = (
                                                ascent::internal::Convert::convert(y),
                                                ascent::internal::Convert::convert(z),
                                             );
                                             if !::ascent::internal::RelFullIndexRead::contains_key(
                                                &a_indices_0_1_total.to_rel_index(&__a_ind_common_total),
                                                &__new_row,
                                             ) && !::ascent::internal::RelFullIndexRead::contains_key(
                                                &a_indices_0_1_delta.to_rel_index(&__a_ind_common_delta),
                                                &__new_row,
                                             ) {
                                                if ::ascent::internal::RelFullIndexWrite::insert_if_not_present(
                                                   &mut a_indices_0_1_new.to_rel_index_write(&mut __a_ind_common_new),
                                                   &__new_row,
                                                   (),
                                                ) {
                                                   let __new_row_ind = _self.a.len();
                                                   _self.a.push((__new_row.0.clone(), __new_row.1.clone()));
                                                   ::ascent::internal::RelIndexWrite::index_insert(
                                                      &mut a_indices_1_new.to_rel_index_write(&mut __a_ind_common_new),
                                                      (__new_row.1.clone(),),
                                                      (__new_row.0.clone(),),
                                                   );
                                                   __changed = true;
                                                }
                                             }
                                             let __new_row: (i32, i32) = (
                                                ascent::internal::Convert::convert(z),
                                                ascent::internal::Convert::convert(w),
                                             );
                                             if !::ascent::internal::RelFullIndexRead::contains_key(
                                                &b_indices_0_1_total.to_rel_index(&__b_ind_common_total),
                                                &__new_row,
                                             ) && !::ascent::internal::RelFullIndexRead::contains_key(
                                                &b_indices_0_1_delta.to_rel_index(&__b_ind_common_delta),
                                                &__new_row,
                                             ) {
                                                if ::ascent::internal::RelFullIndexWrite::insert_if_not_present(
                                                   &mut b_indices_0_1_new.to_rel_index_write(&mut __b_ind_common_new),
                                                   &__new_row,
                                                   (),
                                                ) {
                                                   let __new_row_ind = _self.b.len();
                                                   _self.b.push((__new_row.0.clone(), __new_row.1.clone()));
                                                   ::ascent::internal::RelIndexWrite::index_insert(
                                                      &mut b_indices_0_new.to_rel_index_write(&mut __b_ind_common_new),
                                                      (__new_row.0.clone(),),
                                                      (__new_row.1.clone(),),
                                                   );
                                                   ::ascent::internal::RelIndexWrite::index_insert(
                                                      &mut b_indices_1_new.to_rel_index_write(&mut __b_ind_common_new),
                                                      (__new_row.1.clone(),),
                                                      (__new_row.0.clone(),),
                                                   );
                                                   ::ascent::internal::RelIndexWrite::index_insert(
                                                      &mut b_indices_none_new
                                                         .to_rel_index_write(&mut __b_ind_common_new),
                                                      (),
                                                      (__new_row.0.clone(), __new_row.1.clone()),
                                                   );
                                                   __changed = true;
                                                }
                                             }
                                             let __new_row: (i32, i32) = (
                                                ascent::internal::Convert::convert(x),
                                                ascent::internal::Convert::convert(y),
                                             );
                                             if !::ascent::internal::RelFullIndexRead::contains_key(
                                                &c_indices_0_1_total.to_rel_index(&__c_ind_common_total),
                                                &__new_row,
                                             ) && !::ascent::internal::RelFullIndexRead::contains_key(
                                                &c_indices_0_1_delta.to_rel_index(&__c_ind_common_delta),
                                                &__new_row,
                                             ) {
                                                if ::ascent::internal::RelFullIndexWrite::insert_if_not_present(
                                                   &mut c_indices_0_1_new.to_rel_index_write(&mut __c_ind_common_new),
                                                   &__new_row,
                                                   (),
                                                ) {
                                                   let __new_row_ind = _self.c.len();
                                                   _self.c.push((__new_row.0.clone(), __new_row.1.clone()));
                                                   ::ascent::internal::RelIndexWrite::index_insert(
                                                      &mut c_indices_0_new.to_rel_index_write(&mut __c_ind_common_new),
                                                      (__new_row.0.clone(),),
                                                      (__new_row.1.clone(),),
                                                   );
                                                   ::ascent::internal::RelIndexWrite::index_insert(
                                                      &mut c_indices_none_new
                                                         .to_rel_index_write(&mut __c_ind_common_new),
                                                      (),
                                                      (__new_row.0.clone(), __new_row.1.clone()),
                                                   );
                                                   __changed = true;
                                                }
                                             }
                                          });
                                       }
                                    });
                                 });
                              }
                           },
                        );
                     } else {
                        ascent::internal::RelIndexCombined::new(
                           &b_indices_0_total.to_rel_index(&__b_ind_common_total),
                           &b_indices_0_delta.to_rel_index(&__b_ind_common_delta),
                        )
                        .iter_all()
                        .for_each(|(__cl1_joined_columns, __cl1_tuple_indices)| {
                           let __cl1_joined_columns = __cl1_joined_columns.tuple_of_borrowed();
                           let y = __cl1_joined_columns.0;
                           if let Some(__matching) =
                              a_indices_1_delta.to_rel_index(&__a_ind_common_delta).index_get(&(y.clone(),))
                           {
                              __cl1_tuple_indices.for_each(|cl1_val| {
                                 let cl1_val = cl1_val.tuple_of_borrowed();
                                 let z: &i32 = cl1_val.0;
                                 __matching.clone().for_each(|__val| {
                                    let __val = __val.tuple_of_borrowed();
                                    let x: &i32 = __val.0;
                                    if let Some(__matching) = ascent::internal::RelIndexCombined::new(
                                       &c_indices_0_total.to_rel_index(&__c_ind_common_total),
                                       &c_indices_0_delta.to_rel_index(&__c_ind_common_delta),
                                    )
                                    .index_get(&(z.clone(),))
                                    {
                                       __matching.for_each(|__val| {
                                          let __val = __val.tuple_of_borrowed();
                                          let w: &i32 = __val.0;
                                          let __new_row: (i32, i32) = (
                                             ascent::internal::Convert::convert(y),
                                             ascent::internal::Convert::convert(z),
                                          );
                                          if !::ascent::internal::RelFullIndexRead::contains_key(
                                             &a_indices_0_1_total.to_rel_index(&__a_ind_common_total),
                                             &__new_row,
                                          ) && !::ascent::internal::RelFullIndexRead::contains_key(
                                             &a_indices_0_1_delta.to_rel_index(&__a_ind_common_delta),
                                             &__new_row,
                                          ) {
                                             if ::ascent::internal::RelFullIndexWrite::insert_if_not_present(
                                                &mut a_indices_0_1_new.to_rel_index_write(&mut __a_ind_common_new),
                                                &__new_row,
                                                (),
                                             ) {
                                                let __new_row_ind = _self.a.len();
                                                _self.a.push((__new_row.0.clone(), __new_row.1.clone()));
                                                ::ascent::internal::RelIndexWrite::index_insert(
                                                   &mut a_indices_1_new.to_rel_index_write(&mut __a_ind_common_new),
                                                   (__new_row.1.clone(),),
                                                   (__new_row.0.clone(),),
                                                );
                                                __changed = true;
                                             }
                                          }
                                          let __new_row: (i32, i32) = (
                                             ascent::internal::Convert::convert(z),
                                             ascent::internal::Convert::convert(w),
                                          );
                                          if !::ascent::internal::RelFullIndexRead::contains_key(
                                             &b_indices_0_1_total.to_rel_index(&__b_ind_common_total),
                                             &__new_row,
                                          ) && !::ascent::internal::RelFullIndexRead::contains_key(
                                             &b_indices_0_1_delta.to_rel_index(&__b_ind_common_delta),
                                             &__new_row,
                                          ) {
                                             if ::ascent::internal::RelFullIndexWrite::insert_if_not_present(
                                                &mut b_indices_0_1_new.to_rel_index_write(&mut __b_ind_common_new),
                                                &__new_row,
                                                (),
                                             ) {
                                                let __new_row_ind = _self.b.len();
                                                _self.b.push((__new_row.0.clone(), __new_row.1.clone()));
                                                ::ascent::internal::RelIndexWrite::index_insert(
                                                   &mut b_indices_0_new.to_rel_index_write(&mut __b_ind_common_new),
                                                   (__new_row.0.clone(),),
                                                   (__new_row.1.clone(),),
                                                );
                                                ::ascent::internal::RelIndexWrite::index_insert(
                                                   &mut b_indices_1_new.to_rel_index_write(&mut __b_ind_common_new),
                                                   (__new_row.1.clone(),),
                                                   (__new_row.0.clone(),),
                                                );
                                                ::ascent::internal::RelIndexWrite::index_insert(
                                                   &mut b_indices_none_new.to_rel_index_write(&mut __b_ind_common_new),
                                                   (),
                                                   (__new_row.0.clone(), __new_row.1.clone()),
                                                );
                                                __changed = true;
                                             }
                                          }
                                          let __new_row: (i32, i32) = (
                                             ascent::internal::Convert::convert(x),
                                             ascent::internal::Convert::convert(y),
                                          );
                                          if !::ascent::internal::RelFullIndexRead::contains_key(
                                             &c_indices_0_1_total.to_rel_index(&__c_ind_common_total),
                                             &__new_row,
                                          ) && !::ascent::internal::RelFullIndexRead::contains_key(
                                             &c_indices_0_1_delta.to_rel_index(&__c_ind_common_delta),
                                             &__new_row,
                                          ) {
                                             if ::ascent::internal::RelFullIndexWrite::insert_if_not_present(
                                                &mut c_indices_0_1_new.to_rel_index_write(&mut __c_ind_common_new),
                                                &__new_row,
                                                (),
                                             ) {
                                                let __new_row_ind = _self.c.len();
                                                _self.c.push((__new_row.0.clone(), __new_row.1.clone()));
                                                ::ascent::internal::RelIndexWrite::index_insert(
                                                   &mut c_indices_0_new.to_rel_index_write(&mut __c_ind_common_new),
                                                   (__new_row.0.clone(),),
                                                   (__new_row.1.clone(),),
                                                );
                                                ::ascent::internal::RelIndexWrite::index_insert(
                                                   &mut c_indices_none_new.to_rel_index_write(&mut __c_ind_common_new),
                                                   (),
                                                   (__new_row.0.clone(), __new_row.1.clone()),
                                                );
                                                __changed = true;
                                             }
                                          }
                                       });
                                    }
                                 });
                              });
                           }
                        });
                     }
                  }
               }
               ascent::internal::comment(
                  "a, b, c <-- b_indices_1_delta, c_indices_0_total+delta, a_indices_1_total [SIMPLE JOIN]",
               );
               {
                  let any_rel_empty = b_indices_1_delta.to_rel_index(&__b_ind_common_delta).is_empty()
                     || ascent::internal::RelIndexCombined::new(
                        &c_indices_0_total.to_rel_index(&__c_ind_common_total),
                        &c_indices_0_delta.to_rel_index(&__c_ind_common_delta),
                     )
                     .is_empty()
                     || a_indices_1_total.to_rel_index(&__a_ind_common_total).is_empty();
                  if !any_rel_empty {
                     if b_indices_1_delta.to_rel_index(&__b_ind_common_delta).len_estimate()
                        <= ascent::internal::RelIndexCombined::new(
                           &c_indices_0_total.to_rel_index(&__c_ind_common_total),
                           &c_indices_0_delta.to_rel_index(&__c_ind_common_delta),
                        )
                        .len_estimate()
                     {
                        b_indices_1_delta.to_rel_index(&__b_ind_common_delta).iter_all().for_each(
                           |(__cl1_joined_columns, __cl1_tuple_indices)| {
                              let __cl1_joined_columns = __cl1_joined_columns.tuple_of_borrowed();
                              let z = __cl1_joined_columns.0;
                              if let Some(__matching) = ascent::internal::RelIndexCombined::new(
                                 &c_indices_0_total.to_rel_index(&__c_ind_common_total),
                                 &c_indices_0_delta.to_rel_index(&__c_ind_common_delta),
                              )
                              .index_get(&(z.clone(),))
                              {
                                 __cl1_tuple_indices.for_each(|cl1_val| {
                                    let cl1_val = cl1_val.tuple_of_borrowed();
                                    let y: &i32 = cl1_val.0;
                                    __matching.clone().for_each(|__val| {
                                       let __val = __val.tuple_of_borrowed();
                                       let w: &i32 = __val.0;
                                       if let Some(__matching) =
                                          a_indices_1_total.to_rel_index(&__a_ind_common_total).index_get(&(y.clone(),))
                                       {
                                          __matching.for_each(|__val| {
                                             let __val = __val.tuple_of_borrowed();
                                             let x: &i32 = __val.0;
                                             let __new_row: (i32, i32) = (
                                                ascent::internal::Convert::convert(y),
                                                ascent::internal::Convert::convert(z),
                                             );
                                             if !::ascent::internal::RelFullIndexRead::contains_key(
                                                &a_indices_0_1_total.to_rel_index(&__a_ind_common_total),
                                                &__new_row,
                                             ) && !::ascent::internal::RelFullIndexRead::contains_key(
                                                &a_indices_0_1_delta.to_rel_index(&__a_ind_common_delta),
                                                &__new_row,
                                             ) {
                                                if ::ascent::internal::RelFullIndexWrite::insert_if_not_present(
                                                   &mut a_indices_0_1_new.to_rel_index_write(&mut __a_ind_common_new),
                                                   &__new_row,
                                                   (),
                                                ) {
                                                   let __new_row_ind = _self.a.len();
                                                   _self.a.push((__new_row.0.clone(), __new_row.1.clone()));
                                                   ::ascent::internal::RelIndexWrite::index_insert(
                                                      &mut a_indices_1_new.to_rel_index_write(&mut __a_ind_common_new),
                                                      (__new_row.1.clone(),),
                                                      (__new_row.0.clone(),),
                                                   );
                                                   __changed = true;
                                                }
                                             }
                                             let __new_row: (i32, i32) = (
                                                ascent::internal::Convert::convert(z),
                                                ascent::internal::Convert::convert(w),
                                             );
                                             if !::ascent::internal::RelFullIndexRead::contains_key(
                                                &b_indices_0_1_total.to_rel_index(&__b_ind_common_total),
                                                &__new_row,
                                             ) && !::ascent::internal::RelFullIndexRead::contains_key(
                                                &b_indices_0_1_delta.to_rel_index(&__b_ind_common_delta),
                                                &__new_row,
                                             ) {
                                                if ::ascent::internal::RelFullIndexWrite::insert_if_not_present(
                                                   &mut b_indices_0_1_new.to_rel_index_write(&mut __b_ind_common_new),
                                                   &__new_row,
                                                   (),
                                                ) {
                                                   let __new_row_ind = _self.b.len();
                                                   _self.b.push((__new_row.0.clone(), __new_row.1.clone()));
                                                   ::ascent::internal::RelIndexWrite::index_insert(
                                                      &mut b_indices_0_new.to_rel_index_write(&mut __b_ind_common_new),
                                                      (__new_row.0.clone(),),
                                                      (__new_row.1.clone(),),
                                                   );
                                                   ::ascent::internal::RelIndexWrite::index_insert(
                                                      &mut b_indices_1_new.to_rel_index_write(&mut __b_ind_common_new),
                                                      (__new_row.1.clone(),),
                                                      (__new_row.0.clone(),),
                                                   );
                                                   ::ascent::internal::RelIndexWrite::index_insert(
                                                      &mut b_indices_none_new
                                                         .to_rel_index_write(&mut __b_ind_common_new),
                                                      (),
                                                      (__new_row.0.clone(), __new_row.1.clone()),
                                                   );
                                                   __changed = true;
                                                }
                                             }
                                             let __new_row: (i32, i32) = (
                                                ascent::internal::Convert::convert(x),
                                                ascent::internal::Convert::convert(y),
                                             );
                                             if !::ascent::internal::RelFullIndexRead::contains_key(
                                                &c_indices_0_1_total.to_rel_index(&__c_ind_common_total),
                                                &__new_row,
                                             ) && !::ascent::internal::RelFullIndexRead::contains_key(
                                                &c_indices_0_1_delta.to_rel_index(&__c_ind_common_delta),
                                                &__new_row,
                                             ) {
                                                if ::ascent::internal::RelFullIndexWrite::insert_if_not_present(
                                                   &mut c_indices_0_1_new.to_rel_index_write(&mut __c_ind_common_new),
                                                   &__new_row,
                                                   (),
                                                ) {
                                                   let __new_row_ind = _self.c.len();
                                                   _self.c.push((__new_row.0.clone(), __new_row.1.clone()));
                                                   ::ascent::internal::RelIndexWrite::index_insert(
                                                      &mut c_indices_0_new.to_rel_index_write(&mut __c_ind_common_new),
                                                      (__new_row.0.clone(),),
                                                      (__new_row.1.clone(),),
                                                   );
                                                   ::ascent::internal::RelIndexWrite::index_insert(
                                                      &mut c_indices_none_new
                                                         .to_rel_index_write(&mut __c_ind_common_new),
                                                      (),
                                                      (__new_row.0.clone(), __new_row.1.clone()),
                                                   );
                                                   __changed = true;
                                                }
                                             }
                                          });
                                       }
                                    });
                                 });
                              }
                           },
                        );
                     } else {
                        ascent::internal::RelIndexCombined::new(
                           &c_indices_0_total.to_rel_index(&__c_ind_common_total),
                           &c_indices_0_delta.to_rel_index(&__c_ind_common_delta),
                        )
                        .iter_all()
                        .for_each(|(__cl1_joined_columns, __cl1_tuple_indices)| {
                           let __cl1_joined_columns = __cl1_joined_columns.tuple_of_borrowed();
                           let z = __cl1_joined_columns.0;
                           if let Some(__matching) =
                              b_indices_1_delta.to_rel_index(&__b_ind_common_delta).index_get(&(z.clone(),))
                           {
                              __cl1_tuple_indices.for_each(|cl1_val| {
                                 let cl1_val = cl1_val.tuple_of_borrowed();
                                 let w: &i32 = cl1_val.0;
                                 __matching.clone().for_each(|__val| {
                                    let __val = __val.tuple_of_borrowed();
                                    let y: &i32 = __val.0;
                                    if let Some(__matching) =
                                       a_indices_1_total.to_rel_index(&__a_ind_common_total).index_get(&(y.clone(),))
                                    {
                                       __matching.for_each(|__val| {
                                          let __val = __val.tuple_of_borrowed();
                                          let x: &i32 = __val.0;
                                          let __new_row: (i32, i32) = (
                                             ascent::internal::Convert::convert(y),
                                             ascent::internal::Convert::convert(z),
                                          );
                                          if !::ascent::internal::RelFullIndexRead::contains_key(
                                             &a_indices_0_1_total.to_rel_index(&__a_ind_common_total),
                                             &__new_row,
                                          ) && !::ascent::internal::RelFullIndexRead::contains_key(
                                             &a_indices_0_1_delta.to_rel_index(&__a_ind_common_delta),
                                             &__new_row,
                                          ) {
                                             if ::ascent::internal::RelFullIndexWrite::insert_if_not_present(
                                                &mut a_indices_0_1_new.to_rel_index_write(&mut __a_ind_common_new),
                                                &__new_row,
                                                (),
                                             ) {
                                                let __new_row_ind = _self.a.len();
                                                _self.a.push((__new_row.0.clone(), __new_row.1.clone()));
                                                ::ascent::internal::RelIndexWrite::index_insert(
                                                   &mut a_indices_1_new.to_rel_index_write(&mut __a_ind_common_new),
                                                   (__new_row.1.clone(),),
                                                   (__new_row.0.clone(),),
                                                );
                                                __changed = true;
                                             }
                                          }
                                          let __new_row: (i32, i32) = (
                                             ascent::internal::Convert::convert(z),
                                             ascent::internal::Convert::convert(w),
                                          );
                                          if !::ascent::internal::RelFullIndexRead::contains_key(
                                             &b_indices_0_1_total.to_rel_index(&__b_ind_common_total),
                                             &__new_row,
                                          ) && !::ascent::internal::RelFullIndexRead::contains_key(
                                             &b_indices_0_1_delta.to_rel_index(&__b_ind_common_delta),
                                             &__new_row,
                                          ) {
                                             if ::ascent::internal::RelFullIndexWrite::insert_if_not_present(
                                                &mut b_indices_0_1_new.to_rel_index_write(&mut __b_ind_common_new),
                                                &__new_row,
                                                (),
                                             ) {
                                                let __new_row_ind = _self.b.len();
                                                _self.b.push((__new_row.0.clone(), __new_row.1.clone()));
                                                ::ascent::internal::RelIndexWrite::index_insert(
                                                   &mut b_indices_0_new.to_rel_index_write(&mut __b_ind_common_new),
                                                   (__new_row.0.clone(),),
                                                   (__new_row.1.clone(),),
                                                );
                                                ::ascent::internal::RelIndexWrite::index_insert(
                                                   &mut b_indices_1_new.to_rel_index_write(&mut __b_ind_common_new),
                                                   (__new_row.1.clone(),),
                                                   (__new_row.0.clone(),),
                                                );
                                                ::ascent::internal::RelIndexWrite::index_insert(
                                                   &mut b_indices_none_new.to_rel_index_write(&mut __b_ind_common_new),
                                                   (),
                                                   (__new_row.0.clone(), __new_row.1.clone()),
                                                );
                                                __changed = true;
                                             }
                                          }
                                          let __new_row: (i32, i32) = (
                                             ascent::internal::Convert::convert(x),
                                             ascent::internal::Convert::convert(y),
                                          );
                                          if !::ascent::internal::RelFullIndexRead::contains_key(
                                             &c_indices_0_1_total.to_rel_index(&__c_ind_common_total),
                                             &__new_row,
                                          ) && !::ascent::internal::RelFullIndexRead::contains_key(
                                             &c_indices_0_1_delta.to_rel_index(&__c_ind_common_delta),
                                             &__new_row,
                                          ) {
                                             if ::ascent::internal::RelFullIndexWrite::insert_if_not_present(
                                                &mut c_indices_0_1_new.to_rel_index_write(&mut __c_ind_common_new),
                                                &__new_row,
                                                (),
                                             ) {
                                                let __new_row_ind = _self.c.len();
                                                _self.c.push((__new_row.0.clone(), __new_row.1.clone()));
                                                ::ascent::internal::RelIndexWrite::index_insert(
                                                   &mut c_indices_0_new.to_rel_index_write(&mut __c_ind_common_new),
                                                   (__new_row.0.clone(),),
                                                   (__new_row.1.clone(),),
                                                );
                                                ::ascent::internal::RelIndexWrite::index_insert(
                                                   &mut c_indices_none_new.to_rel_index_write(&mut __c_ind_common_new),
                                                   (),
                                                   (__new_row.0.clone(), __new_row.1.clone()),
                                                );
                                                __changed = true;
                                             }
                                          }
                                       });
                                    }
                                 });
                              });
                           }
                        });
                     }
                  }
               }
               ascent::internal::comment(
                  "a, b, c <-- c_indices_0_delta, b_indices_1_total, a_indices_1_total [SIMPLE JOIN]",
               );
               {
                  let any_rel_empty = c_indices_0_delta.to_rel_index(&__c_ind_common_delta).is_empty()
                     || b_indices_1_total.to_rel_index(&__b_ind_common_total).is_empty()
                     || a_indices_1_total.to_rel_index(&__a_ind_common_total).is_empty();
                  if !any_rel_empty {
                     if c_indices_0_delta.to_rel_index(&__c_ind_common_delta).len_estimate()
                        <= b_indices_1_total.to_rel_index(&__b_ind_common_total).len_estimate()
                     {
                        c_indices_0_delta.to_rel_index(&__c_ind_common_delta).iter_all().for_each(
                           |(__cl1_joined_columns, __cl1_tuple_indices)| {
                              let __cl1_joined_columns = __cl1_joined_columns.tuple_of_borrowed();
                              let z = __cl1_joined_columns.0;
                              if let Some(__matching) =
                                 b_indices_1_total.to_rel_index(&__b_ind_common_total).index_get(&(z.clone(),))
                              {
                                 __cl1_tuple_indices.for_each(|cl1_val| {
                                    let cl1_val = cl1_val.tuple_of_borrowed();
                                    let w: &i32 = cl1_val.0;
                                    __matching.clone().for_each(|__val| {
                                       let __val = __val.tuple_of_borrowed();
                                       let y: &i32 = __val.0;
                                       if let Some(__matching) =
                                          a_indices_1_total.to_rel_index(&__a_ind_common_total).index_get(&(y.clone(),))
                                       {
                                          __matching.for_each(|__val| {
                                             let __val = __val.tuple_of_borrowed();
                                             let x: &i32 = __val.0;
                                             let __new_row: (i32, i32) = (
                                                ascent::internal::Convert::convert(y),
                                                ascent::internal::Convert::convert(z),
                                             );
                                             if !::ascent::internal::RelFullIndexRead::contains_key(
                                                &a_indices_0_1_total.to_rel_index(&__a_ind_common_total),
                                                &__new_row,
                                             ) && !::ascent::internal::RelFullIndexRead::contains_key(
                                                &a_indices_0_1_delta.to_rel_index(&__a_ind_common_delta),
                                                &__new_row,
                                             ) {
                                                if ::ascent::internal::RelFullIndexWrite::insert_if_not_present(
                                                   &mut a_indices_0_1_new.to_rel_index_write(&mut __a_ind_common_new),
                                                   &__new_row,
                                                   (),
                                                ) {
                                                   let __new_row_ind = _self.a.len();
                                                   _self.a.push((__new_row.0.clone(), __new_row.1.clone()));
                                                   ::ascent::internal::RelIndexWrite::index_insert(
                                                      &mut a_indices_1_new.to_rel_index_write(&mut __a_ind_common_new),
                                                      (__new_row.1.clone(),),
                                                      (__new_row.0.clone(),),
                                                   );
                                                   __changed = true;
                                                }
                                             }
                                             let __new_row: (i32, i32) = (
                                                ascent::internal::Convert::convert(z),
                                                ascent::internal::Convert::convert(w),
                                             );
                                             if !::ascent::internal::RelFullIndexRead::contains_key(
                                                &b_indices_0_1_total.to_rel_index(&__b_ind_common_total),
                                                &__new_row,
                                             ) && !::ascent::internal::RelFullIndexRead::contains_key(
                                                &b_indices_0_1_delta.to_rel_index(&__b_ind_common_delta),
                                                &__new_row,
                                             ) {
                                                if ::ascent::internal::RelFullIndexWrite::insert_if_not_present(
                                                   &mut b_indices_0_1_new.to_rel_index_write(&mut __b_ind_common_new),
                                                   &__new_row,
                                                   (),
                                                ) {
                                                   let __new_row_ind = _self.b.len();
                                                   _self.b.push((__new_row.0.clone(), __new_row.1.clone()));
                                                   ::ascent::internal::RelIndexWrite::index_insert(
                                                      &mut b_indices_0_new.to_rel_index_write(&mut __b_ind_common_new),
                                                      (__new_row.0.clone(),),
                                                      (__new_row.1.clone(),),
                                                   );
                                                   ::ascent::internal::RelIndexWrite::index_insert(
                                                      &mut b_indices_1_new.to_rel_index_write(&mut __b_ind_common_new),
                                                      (__new_row.1.clone(),),
                                                      (__new_row.0.clone(),),
                                                   );
                                                   ::ascent::internal::RelIndexWrite::index_insert(
                                                      &mut b_indices_none_new
                                                         .to_rel_index_write(&mut __b_ind_common_new),
                                                      (),
                                                      (__new_row.0.clone(), __new_row.1.clone()),
                                                   );
                                                   __changed = true;
                                                }
                                             }
                                             let __new_row: (i32, i32) = (
                                                ascent::internal::Convert::convert(x),
                                                ascent::internal::Convert::convert(y),
                                             );
                                             if !::ascent::internal::RelFullIndexRead::contains_key(
                                                &c_indices_0_1_total.to_rel_index(&__c_ind_common_total),
                                                &__new_row,
                                             ) && !::ascent::internal::RelFullIndexRead::contains_key(
                                                &c_indices_0_1_delta.to_rel_index(&__c_ind_common_delta),
                                                &__new_row,
                                             ) {
                                                if ::ascent::internal::RelFullIndexWrite::insert_if_not_present(
                                                   &mut c_indices_0_1_new.to_rel_index_write(&mut __c_ind_common_new),
                                                   &__new_row,
                                                   (),
                                                ) {
                                                   let __new_row_ind = _self.c.len();
                                                   _self.c.push((__new_row.0.clone(), __new_row.1.clone()));
                                                   ::ascent::internal::RelIndexWrite::index_insert(
                                                      &mut c_indices_0_new.to_rel_index_write(&mut __c_ind_common_new),
                                                      (__new_row.0.clone(),),
                                                      (__new_row.1.clone(),),
                                                   );
                                                   ::ascent::internal::RelIndexWrite::index_insert(
                                                      &mut c_indices_none_new
                                                         .to_rel_index_write(&mut __c_ind_common_new),
                                                      (),
                                                      (__new_row.0.clone(), __new_row.1.clone()),
                                                   );
                                                   __changed = true;
                                                }
                                             }
                                          });
                                       }
                                    });
                                 });
                              }
                           },
                        );
                     } else {
                        b_indices_1_total.to_rel_index(&__b_ind_common_total).iter_all().for_each(
                           |(__cl1_joined_columns, __cl1_tuple_indices)| {
                              let __cl1_joined_columns = __cl1_joined_columns.tuple_of_borrowed();
                              let z = __cl1_joined_columns.0;
                              if let Some(__matching) =
                                 c_indices_0_delta.to_rel_index(&__c_ind_common_delta).index_get(&(z.clone(),))
                              {
                                 __cl1_tuple_indices.for_each(|cl1_val| {
                                    let cl1_val = cl1_val.tuple_of_borrowed();
                                    let y: &i32 = cl1_val.0;
                                    __matching.clone().for_each(|__val| {
                                       let __val = __val.tuple_of_borrowed();
                                       let w: &i32 = __val.0;
                                       if let Some(__matching) =
                                          a_indices_1_total.to_rel_index(&__a_ind_common_total).index_get(&(y.clone(),))
                                       {
                                          __matching.for_each(|__val| {
                                             let __val = __val.tuple_of_borrowed();
                                             let x: &i32 = __val.0;
                                             let __new_row: (i32, i32) = (
                                                ascent::internal::Convert::convert(y),
                                                ascent::internal::Convert::convert(z),
                                             );
                                             if !::ascent::internal::RelFullIndexRead::contains_key(
                                                &a_indices_0_1_total.to_rel_index(&__a_ind_common_total),
                                                &__new_row,
                                             ) && !::ascent::internal::RelFullIndexRead::contains_key(
                                                &a_indices_0_1_delta.to_rel_index(&__a_ind_common_delta),
                                                &__new_row,
                                             ) {
                                                if ::ascent::internal::RelFullIndexWrite::insert_if_not_present(
                                                   &mut a_indices_0_1_new.to_rel_index_write(&mut __a_ind_common_new),
                                                   &__new_row,
                                                   (),
                                                ) {
                                                   let __new_row_ind = _self.a.len();
                                                   _self.a.push((__new_row.0.clone(), __new_row.1.clone()));
                                                   ::ascent::internal::RelIndexWrite::index_insert(
                                                      &mut a_indices_1_new.to_rel_index_write(&mut __a_ind_common_new),
                                                      (__new_row.1.clone(),),
                                                      (__new_row.0.clone(),),
                                                   );
                                                   __changed = true;
                                                }
                                             }
                                             let __new_row: (i32, i32) = (
                                                ascent::internal::Convert::convert(z),
                                                ascent::internal::Convert::convert(w),
                                             );
                                             if !::ascent::internal::RelFullIndexRead::contains_key(
                                                &b_indices_0_1_total.to_rel_index(&__b_ind_common_total),
                                                &__new_row,
                                             ) && !::ascent::internal::RelFullIndexRead::contains_key(
                                                &b_indices_0_1_delta.to_rel_index(&__b_ind_common_delta),
                                                &__new_row,
                                             ) {
                                                if ::ascent::internal::RelFullIndexWrite::insert_if_not_present(
                                                   &mut b_indices_0_1_new.to_rel_index_write(&mut __b_ind_common_new),
                                                   &__new_row,
                                                   (),
                                                ) {
                                                   let __new_row_ind = _self.b.len();
                                                   _self.b.push((__new_row.0.clone(), __new_row.1.clone()));
                                                   ::ascent::internal::RelIndexWrite::index_insert(
                                                      &mut b_indices_0_new.to_rel_index_write(&mut __b_ind_common_new),
                                                      (__new_row.0.clone(),),
                                                      (__new_row.1.clone(),),
                                                   );
                                                   ::ascent::internal::RelIndexWrite::index_insert(
                                                      &mut b_indices_1_new.to_rel_index_write(&mut __b_ind_common_new),
                                                      (__new_row.1.clone(),),
                                                      (__new_row.0.clone(),),
                                                   );
                                                   ::ascent::internal::RelIndexWrite::index_insert(
                                                      &mut b_indices_none_new
                                                         .to_rel_index_write(&mut __b_ind_common_new),
                                                      (),
                                                      (__new_row.0.clone(), __new_row.1.clone()),
                                                   );
                                                   __changed = true;
                                                }
                                             }
                                             let __new_row: (i32, i32) = (
                                                ascent::internal::Convert::convert(x),
                                                ascent::internal::Convert::convert(y),
                                             );
                                             if !::ascent::internal::RelFullIndexRead::contains_key(
                                                &c_indices_0_1_total.to_rel_index(&__c_ind_common_total),
                                                &__new_row,
                                             ) && !::ascent::internal::RelFullIndexRead::contains_key(
                                                &c_indices_0_1_delta.to_rel_index(&__c_ind_common_delta),
                                                &__new_row,
                                             ) {
                                                if ::ascent::internal::RelFullIndexWrite::insert_if_not_present(
                                                   &mut c_indices_0_1_new.to_rel_index_write(&mut __c_ind_common_new),
                                                   &__new_row,
                                                   (),
                                                ) {
                                                   let __new_row_ind = _self.c.len();
                                                   _self.c.push((__new_row.0.clone(), __new_row.1.clone()));
                                                   ::ascent::internal::RelIndexWrite::index_insert(
                                                      &mut c_indices_0_new.to_rel_index_write(&mut __c_ind_common_new),
                                                      (__new_row.0.clone(),),
                                                      (__new_row.1.clone(),),
                                                   );
                                                   ::ascent::internal::RelIndexWrite::index_insert(
                                                      &mut c_indices_none_new
                                                         .to_rel_index_write(&mut __c_ind_common_new),
                                                      (),
                                                      (__new_row.0.clone(), __new_row.1.clone()),
                                                   );
                                                   __changed = true;
                                                }
                                             }
                                          });
                                       }
                                    });
                                 });
                              }
                           },
                        );
                     }
                  }
               }
               ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
                  &mut __a_ind_common_new, &mut __a_ind_common_delta, &mut __a_ind_common_total,
               );
               ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
                  &mut a_indices_0_1_new.to_rel_index_write(&mut __a_ind_common_new),
                  &mut a_indices_0_1_delta.to_rel_index_write(&mut __a_ind_common_delta),
                  &mut a_indices_0_1_total.to_rel_index_write(&mut __a_ind_common_total),
               );
               ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
                  &mut a_indices_1_new.to_rel_index_write(&mut __a_ind_common_new),
                  &mut a_indices_1_delta.to_rel_index_write(&mut __a_ind_common_delta),
                  &mut a_indices_1_total.to_rel_index_write(&mut __a_ind_common_total),
               );
               ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
                  &mut __b_ind_common_new, &mut __b_ind_common_delta, &mut __b_ind_common_total,
               );
               ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
                  &mut b_indices_0_new.to_rel_index_write(&mut __b_ind_common_new),
                  &mut b_indices_0_delta.to_rel_index_write(&mut __b_ind_common_delta),
                  &mut b_indices_0_total.to_rel_index_write(&mut __b_ind_common_total),
               );
               ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
                  &mut b_indices_0_1_new.to_rel_index_write(&mut __b_ind_common_new),
                  &mut b_indices_0_1_delta.to_rel_index_write(&mut __b_ind_common_delta),
                  &mut b_indices_0_1_total.to_rel_index_write(&mut __b_ind_common_total),
               );
               ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
                  &mut b_indices_1_new.to_rel_index_write(&mut __b_ind_common_new),
                  &mut b_indices_1_delta.to_rel_index_write(&mut __b_ind_common_delta),
                  &mut b_indices_1_total.to_rel_index_write(&mut __b_ind_common_total),
               );
               ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
                  &mut b_indices_none_new.to_rel_index_write(&mut __b_ind_common_new),
                  &mut b_indices_none_delta.to_rel_index_write(&mut __b_ind_common_delta),
                  &mut b_indices_none_total.to_rel_index_write(&mut __b_ind_common_total),
               );
               ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
                  &mut __c_ind_common_new, &mut __c_ind_common_delta, &mut __c_ind_common_total,
               );
               ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
                  &mut c_indices_0_new.to_rel_index_write(&mut __c_ind_common_new),
                  &mut c_indices_0_delta.to_rel_index_write(&mut __c_ind_common_delta),
                  &mut c_indices_0_total.to_rel_index_write(&mut __c_ind_common_total),
               );
               ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
                  &mut c_indices_0_1_new.to_rel_index_write(&mut __c_ind_common_new),
                  &mut c_indices_0_1_delta.to_rel_index_write(&mut __c_ind_common_delta),
                  &mut c_indices_0_1_total.to_rel_index_write(&mut __c_ind_common_total),
               );
               ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
                  &mut c_indices_none_new.to_rel_index_write(&mut __c_ind_common_new),
                  &mut c_indices_none_delta.to_rel_index_write(&mut __c_ind_common_delta),
                  &mut c_indices_none_total.to_rel_index_write(&mut __c_ind_common_total),
               );
               _self.scc_iters[0usize] += 1;
               if !__changed {
                  break;
               }
               __check_return_conditions!();
            }
            _self.__a_ind_common = __a_ind_common_total;
            _self.a_indices_0_1 = a_indices_0_1_total;
            _self.a_indices_1 = a_indices_1_total;
            _self.__b_ind_common = __b_ind_common_total;
            _self.b_indices_0 = b_indices_0_total;
            _self.b_indices_0_1 = b_indices_0_1_total;
            _self.b_indices_1 = b_indices_1_total;
            _self.b_indices_none = b_indices_none_total;
            _self.__c_ind_common = __c_ind_common_total;
            _self.c_indices_0 = c_indices_0_total;
            _self.c_indices_0_1 = c_indices_0_1_total;
            _self.c_indices_none = c_indices_none_total;
            _self.scc_times[0usize] += _scc_start_time.elapsed();
         }
      }
      #[allow(noop_method_call, suspicious_double_ref_op)]
      fn update_indices_priv(&mut self) {
         #![allow(clippy::all)]
         let before = ::ascent::internal::Instant::now();
         use ascent::internal::{RelIndexWrite, ToRelIndex0};
         for (_i, tuple) in self.a.iter().enumerate() {
            let selection_tuple = (tuple.0.clone(), tuple.1.clone());
            let rel_ind = &mut self.a_indices_0_1;
            ascent::internal::RelIndexWrite::index_insert(
               &mut rel_ind.to_rel_index_write(&mut self.__a_ind_common),
               selection_tuple,
               (),
            );
            let selection_tuple = (tuple.1.clone(),);
            let rel_ind = &mut self.a_indices_1;
            ascent::internal::RelIndexWrite::index_insert(
               &mut rel_ind.to_rel_index_write(&mut self.__a_ind_common),
               selection_tuple,
               (tuple.0.clone(),),
            );
         }
         for (_i, tuple) in self.b.iter().enumerate() {
            let selection_tuple = (tuple.0.clone(),);
            let rel_ind = &mut self.b_indices_0;
            ascent::internal::RelIndexWrite::index_insert(
               &mut rel_ind.to_rel_index_write(&mut self.__b_ind_common),
               selection_tuple,
               (tuple.1.clone(),),
            );
            let selection_tuple = (tuple.0.clone(), tuple.1.clone());
            let rel_ind = &mut self.b_indices_0_1;
            ascent::internal::RelIndexWrite::index_insert(
               &mut rel_ind.to_rel_index_write(&mut self.__b_ind_common),
               selection_tuple,
               (),
            );
            let selection_tuple = (tuple.1.clone(),);
            let rel_ind = &mut self.b_indices_1;
            ascent::internal::RelIndexWrite::index_insert(
               &mut rel_ind.to_rel_index_write(&mut self.__b_ind_common),
               selection_tuple,
               (tuple.0.clone(),),
            );
            let selection_tuple = ();
            let rel_ind = &mut self.b_indices_none;
            ascent::internal::RelIndexWrite::index_insert(
               &mut rel_ind.to_rel_index_write(&mut self.__b_ind_common),
               selection_tuple,
               (tuple.0.clone(), tuple.1.clone()),
            );
         }
         for (_i, tuple) in self.c.iter().enumerate() {
            let selection_tuple = (tuple.0.clone(),);
            let rel_ind = &mut self.c_indices_0;
            ascent::internal::RelIndexWrite::index_insert(
               &mut rel_ind.to_rel_index_write(&mut self.__c_ind_common),
               selection_tuple,
               (tuple.1.clone(),),
            );
            let selection_tuple = (tuple.0.clone(), tuple.1.clone());
            let rel_ind = &mut self.c_indices_0_1;
            ascent::internal::RelIndexWrite::index_insert(
               &mut rel_ind.to_rel_index_write(&mut self.__c_ind_common),
               selection_tuple,
               (),
            );
            let selection_tuple = ();
            let rel_ind = &mut self.c_indices_none;
            ascent::internal::RelIndexWrite::index_insert(
               &mut rel_ind.to_rel_index_write(&mut self.__c_ind_common),
               selection_tuple,
               (tuple.0.clone(), tuple.1.clone()),
            );
         }
         self.update_indices_duration += before.elapsed();
      }
      #[deprecated = "Explicit call to update_indices not required anymore."]
      pub fn update_indices(&mut self) { self.update_indices_priv(); }
      fn type_constraints() {
         #![allow(clippy::all)]
         let _type_constraints: ascent::internal::TypeConstraints<i32>;
      }
      pub fn summary() -> &'static str {
         "scc 0, is_looping: true:\n  a, b, c <-- a_indices_1_delta, b_indices_0_total+delta, c_indices_0_total+delta [SIMPLE JOIN]\n  a, b, c <-- b_indices_1_delta, c_indices_0_total+delta, a_indices_1_total [SIMPLE JOIN]\n  a, b, c <-- c_indices_0_delta, b_indices_1_total, a_indices_1_total [SIMPLE JOIN]\n  dynamic relations: a, b, c\n"
      }
      pub fn relation_sizes_summary(&self) -> String {
         #![allow(clippy::all)]
         use std::fmt::Write;
         let mut res = String::new();
         writeln!(&mut res, "{} size: {}", "a", self.a.len()).unwrap();
         writeln!(&mut res, "{} size: {}", "b", self.b.len()).unwrap();
         writeln!(&mut res, "{} size: {}", "c", self.c.len()).unwrap();
         res
      }
      pub fn scc_times_summary(&self) -> String {
         #![allow(clippy::all)]
         use std::fmt::Write;
         let mut res = String::new();
         writeln!(&mut res, "update_indices time: {:?}", self.update_indices_duration).unwrap();
         writeln!(&mut res, "scc {}: iterations: {}, time: {:?}", "0", self.scc_iters[0usize], self.scc_times[0usize])
            .unwrap();
         res
      }
   }
   impl Default for AscentProgram {
      fn default() -> Self {
         let mut _self = AscentProgram {
            a: Default::default(),
            __a_ind_common: Default::default(),
            a_indices_0_1: Default::default(),
            a_indices_1: Default::default(),
            b: Default::default(),
            __b_ind_common: Default::default(),
            b_indices_0: Default::default(),
            b_indices_0_1: Default::default(),
            b_indices_1: Default::default(),
            b_indices_none: Default::default(),
            c: Default::default(),
            __c_ind_common: Default::default(),
            c_indices_0: Default::default(),
            c_indices_0_1: Default::default(),
            c_indices_none: Default::default(),
            scc_times: [std::time::Duration::ZERO; 1usize],
            scc_iters: [0; 1usize],
            update_time_nanos: Default::default(),
            update_indices_duration: std::time::Duration::default(),
         };
         _self
      }
   };
}
