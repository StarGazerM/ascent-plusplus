#![allow(unused_imports)]
use std::fmt::Debug;
use std::hash::Hash;
use std::ops::Deref;
use std::{clone, cmp::max, rc::Rc};

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

#[warn(warnings)]
#[allow(unused_imports)]
#[allow(dead_code)]
#[allow(redundant_semicolons)]
#[cfg(test)]
fn _test<T: FactTypes>() {
   use ascent::aggregators::*;
   use ascent::lattice::set::Set;
   use ascent::Dual;

   use ascent::rel as custom_ds;
   ::ascent::rel::rel_codegen! { AscentProgram_foobar , (i32 , usize) , [[0 , 1]] , ser , () }
   ::ascent::rel::rel_codegen! { AscentProgram_foo , (i32 , i32) , [[] , [0 , 1]] , ser , () }
   ::ascent::rel::rel_codegen! { AscentProgram_bar , (i32 , i32) , [[0 , 1]] , ser , () }
   pub struct AscentProgram {
      #[doc = "\nlogical indices: bar_indices_0_1"]
      pub bar: ::ascent::rel::rel!(AscentProgram_bar, (i32, i32), [[0, 1]], ser, ()),
      pub __bar_ind_common: ::ascent::rel::rel_ind_common!(AscentProgram_bar, (i32, i32), [[0, 1]], ser, ()),
      pub bar_indices_0_1:
         ::ascent::rel::rel_full_ind!(AscentProgram_bar, (i32, i32), [[0, 1]], ser, (), (i32, i32), ()),
      #[doc = "\nlogical indices: foo_indices_0_1; foo_indices_none"]
      pub foo: ::ascent::rel::rel!(AscentProgram_foo, (i32, i32), [[], [0, 1]], ser, ()),
      pub __foo_ind_common: ::ascent::rel::rel_ind_common!(AscentProgram_foo, (i32, i32), [[], [0, 1]], ser, ()),
      pub foo_indices_0_1:
         ::ascent::rel::rel_full_ind!(AscentProgram_foo, (i32, i32), [[], [0, 1]], ser, (), (i32, i32), ()),
      pub foo_indices_none:
         ::ascent::rel::rel_ind!(AscentProgram_foo, (i32, i32), [[], [0, 1]], ser, (), [], (), (i32, i32)),
      #[doc = "\nlogical indices: foobar_indices_0_1"]
      pub foobar: ::ascent::rel::rel!(AscentProgram_foobar, (i32, usize), [[0, 1]], ser, ()),
      pub __foobar_ind_common: ::ascent::rel::rel_ind_common!(AscentProgram_foobar, (i32, usize), [[0, 1]], ser, ()),
      pub foobar_indices_0_1:
         ::ascent::rel::rel_full_ind!(AscentProgram_foobar, (i32, usize), [[0, 1]], ser, (), (i32, usize), ()),
      scc_times: [std::time::Duration; 2usize],
      scc_iters: [usize; 2usize],
      pub update_time_nanos: std::sync::atomic::AtomicU64,
      pub update_indices_duration: std::time::Duration,
   }
   impl AscentProgram {
      #[allow(unused_imports, noop_method_call, suspicious_double_ref_op)]
      #[doc = "Runs the Ascent program to a fixed point."]
      pub fn run(&mut self) {
         macro_rules! __check_return_conditions {
            () => {};
         }
         use ascent::internal::RelIndexWrite;
         use ascent::internal::{RelIndexRead, RelIndexReadAll, ToRelIndex0, TupleOfBorrowed};
         use core::cmp::PartialEq;
         self.update_indices_priv();
         let _self = self;
         ascent::internal::comment("scc 0");
         {
            let _scc_start_time = ::ascent::internal::Instant::now();
            let mut __foo_ind_common_delta: ::ascent::rel::rel_ind_common!(
               AscentProgram_foo,
               (i32, i32),
               [[], [0, 1]],
               ser,
               ()
            ) = ::std::mem::take(&mut _self.__foo_ind_common);
            let mut __foo_ind_common_total: ::ascent::rel::rel_ind_common!(
               AscentProgram_foo,
               (i32, i32),
               [[], [0, 1]],
               ser,
               ()
            ) = Default::default();
            let mut __foo_ind_common_new: ::ascent::rel::rel_ind_common!(
               AscentProgram_foo,
               (i32, i32),
               [[], [0, 1]],
               ser,
               ()
            ) = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut __foo_ind_common_new,
               &mut __foo_ind_common_delta,
               &mut __foo_ind_common_total,
            );
            let mut foo_indices_0_1_delta: ::ascent::rel::rel_full_ind!(
               AscentProgram_foo,
               (i32, i32),
               [[], [0, 1]],
               ser,
               (),
               (i32, i32),
               ()
            ) = ::std::mem::take(&mut _self.foo_indices_0_1);
            let mut foo_indices_0_1_total: ::ascent::rel::rel_full_ind!(
               AscentProgram_foo,
               (i32, i32),
               [[], [0, 1]],
               ser,
               (),
               (i32, i32),
               ()
            ) = Default::default();
            let mut foo_indices_0_1_new: ::ascent::rel::rel_full_ind!(
               AscentProgram_foo,
               (i32, i32),
               [[], [0, 1]],
               ser,
               (),
               (i32, i32),
               ()
            ) = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut foo_indices_0_1_new.to_rel_index_write(&mut __foo_ind_common_new),
               &mut foo_indices_0_1_delta.to_rel_index_write(&mut __foo_ind_common_delta),
               &mut foo_indices_0_1_total.to_rel_index_write(&mut __foo_ind_common_total),
            );
            let mut foo_indices_none_delta: ::ascent::rel::rel_ind!(
               AscentProgram_foo,
               (i32, i32),
               [[], [0, 1]],
               ser,
               (),
               [],
               (),
               (i32, i32)
            ) = ::std::mem::take(&mut _self.foo_indices_none);
            let mut foo_indices_none_total: ::ascent::rel::rel_ind!(
               AscentProgram_foo,
               (i32, i32),
               [[], [0, 1]],
               ser,
               (),
               [],
               (),
               (i32, i32)
            ) = Default::default();
            let mut foo_indices_none_new: ::ascent::rel::rel_ind!(
               AscentProgram_foo,
               (i32, i32),
               [[], [0, 1]],
               ser,
               (),
               [],
               (),
               (i32, i32)
            ) = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut foo_indices_none_new.to_rel_index_write(&mut __foo_ind_common_new),
               &mut foo_indices_none_delta.to_rel_index_write(&mut __foo_ind_common_delta),
               &mut foo_indices_none_total.to_rel_index_write(&mut __foo_ind_common_total),
            );
            #[allow(unused_assignments, unused_variables)]
            {
               let mut __changed = false;
               let mut __default_id = 0;
               ascent::internal::comment("foo <-- ");
               {
                  let __new_row: (i32, i32) = (1, 2);
                  let mut __new_foo = 0;
                  if !::ascent::internal::RelFullIndexRead::contains_key(
                     &foo_indices_0_1_total.to_rel_index(&__foo_ind_common_total),
                     &__new_row,
                  ) && !::ascent::internal::RelFullIndexRead::contains_key(
                     &foo_indices_0_1_delta.to_rel_index(&__foo_ind_common_delta),
                     &__new_row,
                  ) {
                     if ::ascent::internal::RelFullIndexWrite::insert_if_not_present(
                        &mut foo_indices_0_1_new.to_rel_index_write(&mut __foo_ind_common_new),
                        &__new_row,
                        (),
                     ) {
                        __new_foo = _self.foo.len();
                        _self.foo.push((__new_row.0.clone(), __new_row.1.clone()));
                        __default_id = __new_foo;
                        ::ascent::internal::RelIndexWrite::index_insert(
                           &mut foo_indices_none_new.to_rel_index_write(&mut __foo_ind_common_new),
                           (),
                           (__new_row.0.clone(), __new_row.1.clone()),
                        );
                        __changed = true;
                     } else {
                     }
                  } else {
                  }
               }
               ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
                  &mut __foo_ind_common_new,
                  &mut __foo_ind_common_delta,
                  &mut __foo_ind_common_total,
               );
               ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
                  &mut foo_indices_0_1_new.to_rel_index_write(&mut __foo_ind_common_new),
                  &mut foo_indices_0_1_delta.to_rel_index_write(&mut __foo_ind_common_delta),
                  &mut foo_indices_0_1_total.to_rel_index_write(&mut __foo_ind_common_total),
               );
               ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
                  &mut foo_indices_none_new.to_rel_index_write(&mut __foo_ind_common_new),
                  &mut foo_indices_none_delta.to_rel_index_write(&mut __foo_ind_common_delta),
                  &mut foo_indices_none_total.to_rel_index_write(&mut __foo_ind_common_total),
               );
               ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
                  &mut __foo_ind_common_new,
                  &mut __foo_ind_common_delta,
                  &mut __foo_ind_common_total,
               );
               ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
                  &mut foo_indices_0_1_new.to_rel_index_write(&mut __foo_ind_common_new),
                  &mut foo_indices_0_1_delta.to_rel_index_write(&mut __foo_ind_common_delta),
                  &mut foo_indices_0_1_total.to_rel_index_write(&mut __foo_ind_common_total),
               );
               ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
                  &mut foo_indices_none_new.to_rel_index_write(&mut __foo_ind_common_new),
                  &mut foo_indices_none_delta.to_rel_index_write(&mut __foo_ind_common_delta),
                  &mut foo_indices_none_total.to_rel_index_write(&mut __foo_ind_common_total),
               );
               _self.scc_iters[0usize] += 1;
               __check_return_conditions!();
            }
            _self.__foo_ind_common = __foo_ind_common_total;
            _self.foo_indices_0_1 = foo_indices_0_1_total;
            _self.foo_indices_none = foo_indices_none_total;
            _self.scc_times[0usize] += _scc_start_time.elapsed();
         }
         ascent::internal::comment("scc 1");
         {
            let _scc_start_time = ::ascent::internal::Instant::now();
            let mut __bar_ind_common_delta: ::ascent::rel::rel_ind_common!(
               AscentProgram_bar,
               (i32, i32),
               [[0, 1]],
               ser,
               ()
            ) = ::std::mem::take(&mut _self.__bar_ind_common);
            let mut __bar_ind_common_total: ::ascent::rel::rel_ind_common!(
               AscentProgram_bar,
               (i32, i32),
               [[0, 1]],
               ser,
               ()
            ) = Default::default();
            let mut __bar_ind_common_new: ::ascent::rel::rel_ind_common!(
               AscentProgram_bar,
               (i32, i32),
               [[0, 1]],
               ser,
               ()
            ) = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut __bar_ind_common_new,
               &mut __bar_ind_common_delta,
               &mut __bar_ind_common_total,
            );
            let mut bar_indices_0_1_delta: ::ascent::rel::rel_full_ind!(
               AscentProgram_bar,
               (i32, i32),
               [[0, 1]],
               ser,
               (),
               (i32, i32),
               ()
            ) = ::std::mem::take(&mut _self.bar_indices_0_1);
            let mut bar_indices_0_1_total: ::ascent::rel::rel_full_ind!(
               AscentProgram_bar,
               (i32, i32),
               [[0, 1]],
               ser,
               (),
               (i32, i32),
               ()
            ) = Default::default();
            let mut bar_indices_0_1_new: ::ascent::rel::rel_full_ind!(
               AscentProgram_bar,
               (i32, i32),
               [[0, 1]],
               ser,
               (),
               (i32, i32),
               ()
            ) = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut bar_indices_0_1_new.to_rel_index_write(&mut __bar_ind_common_new),
               &mut bar_indices_0_1_delta.to_rel_index_write(&mut __bar_ind_common_delta),
               &mut bar_indices_0_1_total.to_rel_index_write(&mut __bar_ind_common_total),
            );
            let mut __foobar_ind_common_delta: ::ascent::rel::rel_ind_common!(
               AscentProgram_foobar,
               (i32, usize),
               [[0, 1]],
               ser,
               ()
            ) = ::std::mem::take(&mut _self.__foobar_ind_common);
            let mut __foobar_ind_common_total: ::ascent::rel::rel_ind_common!(
               AscentProgram_foobar,
               (i32, usize),
               [[0, 1]],
               ser,
               ()
            ) = Default::default();
            let mut __foobar_ind_common_new: ::ascent::rel::rel_ind_common!(
               AscentProgram_foobar,
               (i32, usize),
               [[0, 1]],
               ser,
               ()
            ) = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut __foobar_ind_common_new,
               &mut __foobar_ind_common_delta,
               &mut __foobar_ind_common_total,
            );
            let mut foobar_indices_0_1_delta: ::ascent::rel::rel_full_ind!(
               AscentProgram_foobar,
               (i32, usize),
               [[0, 1]],
               ser,
               (),
               (i32, usize),
               ()
            ) = ::std::mem::take(&mut _self.foobar_indices_0_1);
            let mut foobar_indices_0_1_total: ::ascent::rel::rel_full_ind!(
               AscentProgram_foobar,
               (i32, usize),
               [[0, 1]],
               ser,
               (),
               (i32, usize),
               ()
            ) = Default::default();
            let mut foobar_indices_0_1_new: ::ascent::rel::rel_full_ind!(
               AscentProgram_foobar,
               (i32, usize),
               [[0, 1]],
               ser,
               (),
               (i32, usize),
               ()
            ) = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut foobar_indices_0_1_new.to_rel_index_write(&mut __foobar_ind_common_new),
               &mut foobar_indices_0_1_delta.to_rel_index_write(&mut __foobar_ind_common_delta),
               &mut foobar_indices_0_1_total.to_rel_index_write(&mut __foobar_ind_common_total),
            );
            let __foo_ind_common_total: ::ascent::rel::rel_ind_common!(
               AscentProgram_foo,
               (i32, i32),
               [[], [0, 1]],
               ser,
               ()
            ) = std::mem::take(&mut _self.__foo_ind_common);
            let foo_indices_none_total: ::ascent::rel::rel_ind!(
               AscentProgram_foo,
               (i32, i32),
               [[], [0, 1]],
               ser,
               (),
               [],
               (),
               (i32, i32)
            ) = std::mem::take(&mut _self.foo_indices_none);
            #[allow(unused_assignments, unused_variables)]
            {
               let mut __changed = false;
               let mut __default_id = 0;
               ascent::internal::comment("bar, foobar <-- foo_indices_none_total");
               {
                  if let Some(__matching) = foo_indices_none_total.to_rel_index(&__foo_ind_common_total).index_get(&())
                  {
                     __matching.for_each(|__val| {
                        let mut __dep_changed = false;
                        let mut __default_id = 0;
                        let __val = __val.tuple_of_borrowed();
                        let x: &i32 = __val.0;
                        let y: &i32 = __val.1;
                        let __new_row: (i32, i32) =
                           (ascent::internal::Convert::convert(x), ascent::internal::Convert::convert(y));
                        let mut new_bar = 0;
                        if !::ascent::internal::RelFullIndexRead::contains_key(
                           &bar_indices_0_1_total.to_rel_index(&__bar_ind_common_total),
                           &__new_row,
                        ) && !::ascent::internal::RelFullIndexRead::contains_key(
                           &bar_indices_0_1_delta.to_rel_index(&__bar_ind_common_delta),
                           &__new_row,
                        ) {
                           if ::ascent::internal::RelFullIndexWrite::insert_if_not_present(
                              &mut bar_indices_0_1_new.to_rel_index_write(&mut __bar_ind_common_new),
                              &__new_row,
                              (),
                           ) {
                              new_bar = _self.bar.len();
                              _self.bar.push((__new_row.0, __new_row.1));
                              __default_id = new_bar;
                              __changed = true;
                           } else {
                              return;
                           }
                        } else {
                           return;
                        }
                        let __new_row: (i32, usize) =
                           (ascent::internal::Convert::convert(x), ascent::internal::Convert::convert(new_bar));
                        let mut __new_foobar = 0;
                        if !::ascent::internal::RelFullIndexRead::contains_key(
                           &foobar_indices_0_1_total.to_rel_index(&__foobar_ind_common_total),
                           &__new_row,
                        ) && !::ascent::internal::RelFullIndexRead::contains_key(
                           &foobar_indices_0_1_delta.to_rel_index(&__foobar_ind_common_delta),
                           &__new_row,
                        ) {
                           if ::ascent::internal::RelFullIndexWrite::insert_if_not_present(
                              &mut foobar_indices_0_1_new.to_rel_index_write(&mut __foobar_ind_common_new),
                              &__new_row,
                              (),
                           ) {
                              __new_foobar = _self.foobar.len();
                              _self.foobar.push((__new_row.0, __new_row.1));
                              __default_id = __new_foobar;
                              __changed = true;
                           } else {
                           }
                        } else {
                        }
                     });
                  }
               }
               ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
                  &mut __bar_ind_common_new,
                  &mut __bar_ind_common_delta,
                  &mut __bar_ind_common_total,
               );
               ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
                  &mut bar_indices_0_1_new.to_rel_index_write(&mut __bar_ind_common_new),
                  &mut bar_indices_0_1_delta.to_rel_index_write(&mut __bar_ind_common_delta),
                  &mut bar_indices_0_1_total.to_rel_index_write(&mut __bar_ind_common_total),
               );
               ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
                  &mut __foobar_ind_common_new,
                  &mut __foobar_ind_common_delta,
                  &mut __foobar_ind_common_total,
               );
               ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
                  &mut foobar_indices_0_1_new.to_rel_index_write(&mut __foobar_ind_common_new),
                  &mut foobar_indices_0_1_delta.to_rel_index_write(&mut __foobar_ind_common_delta),
                  &mut foobar_indices_0_1_total.to_rel_index_write(&mut __foobar_ind_common_total),
               );
               ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
                  &mut __bar_ind_common_new,
                  &mut __bar_ind_common_delta,
                  &mut __bar_ind_common_total,
               );
               ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
                  &mut bar_indices_0_1_new.to_rel_index_write(&mut __bar_ind_common_new),
                  &mut bar_indices_0_1_delta.to_rel_index_write(&mut __bar_ind_common_delta),
                  &mut bar_indices_0_1_total.to_rel_index_write(&mut __bar_ind_common_total),
               );
               ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
                  &mut __foobar_ind_common_new,
                  &mut __foobar_ind_common_delta,
                  &mut __foobar_ind_common_total,
               );
               ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
                  &mut foobar_indices_0_1_new.to_rel_index_write(&mut __foobar_ind_common_new),
                  &mut foobar_indices_0_1_delta.to_rel_index_write(&mut __foobar_ind_common_delta),
                  &mut foobar_indices_0_1_total.to_rel_index_write(&mut __foobar_ind_common_total),
               );
               _self.scc_iters[1usize] += 1;
               __check_return_conditions!();
            }
            _self.__bar_ind_common = __bar_ind_common_total;
            _self.bar_indices_0_1 = bar_indices_0_1_total;
            _self.__foobar_ind_common = __foobar_ind_common_total;
            _self.foobar_indices_0_1 = foobar_indices_0_1_total;
            _self.__foo_ind_common = __foo_ind_common_total;
            _self.foo_indices_none = foo_indices_none_total;
            _self.scc_times[1usize] += _scc_start_time.elapsed();
         }
      }
      #[allow(noop_method_call, suspicious_double_ref_op)]
      fn update_indices_priv(&mut self) {
         let before = ::ascent::internal::Instant::now();
         use ascent::internal::RelIndexWrite;
         use ascent::internal::ToRelIndex0;
         for (_i, tuple) in self.bar.iter().enumerate() {
            let selection_tuple = (tuple.0.clone(), tuple.1.clone());
            let rel_ind = &mut self.bar_indices_0_1;
            ascent::internal::RelIndexWrite::index_insert(
               &mut rel_ind.to_rel_index_write(&mut self.__bar_ind_common),
               selection_tuple,
               (),
            );
         }
         for (_i, tuple) in self.foo.iter().enumerate() {
            let selection_tuple = (tuple.0.clone(), tuple.1.clone());
            let rel_ind = &mut self.foo_indices_0_1;
            ascent::internal::RelIndexWrite::index_insert(
               &mut rel_ind.to_rel_index_write(&mut self.__foo_ind_common),
               selection_tuple,
               (),
            );
            let selection_tuple = ();
            let rel_ind = &mut self.foo_indices_none;
            ascent::internal::RelIndexWrite::index_insert(
               &mut rel_ind.to_rel_index_write(&mut self.__foo_ind_common),
               selection_tuple,
               (tuple.0.clone(), tuple.1.clone()),
            );
         }
         for (_i, tuple) in self.foobar.iter().enumerate() {
            let selection_tuple = (tuple.0.clone(), tuple.1.clone());
            let rel_ind = &mut self.foobar_indices_0_1;
            ascent::internal::RelIndexWrite::index_insert(
               &mut rel_ind.to_rel_index_write(&mut self.__foobar_ind_common),
               selection_tuple,
               (),
            );
         }
         self.update_indices_duration += before.elapsed();
      }
      #[deprecated = "Explicit call to update_indices not required anymore."]
      pub fn update_indices(&mut self) {
         self.update_indices_priv();
      }
      fn type_constraints() {
         let _type_constraints: ascent::internal::TypeConstraints<i32>;
         let _type_constraints: ascent::internal::TypeConstraints<usize>;
      }
      pub fn summary() -> &'static str {
         "scc 0, is_looping: false:\n  foo <-- \n  dynamic relations: foo\nscc 1, is_looping: false:\n  bar, foobar <-- foo_indices_none_total\n  dynamic relations: bar, foobar\n"
      }
      pub fn relation_sizes_summary(&self) -> String {
         use std::fmt::Write;
         let mut res = String::new();
         writeln!(&mut res, "{} size: {}", "bar", self.bar.len()).unwrap();
         writeln!(&mut res, "{} size: {}", "foo", self.foo.len()).unwrap();
         writeln!(&mut res, "{} size: {}", "foobar", self.foobar.len()).unwrap();
         res
      }
      pub fn scc_times_summary(&self) -> String {
         use std::fmt::Write;
         let mut res = String::new();
         writeln!(&mut res, "update_indices time: {:?}", self.update_indices_duration).unwrap();
         writeln!(&mut res, "scc {}: iterations: {}, time: {:?}", "0", self.scc_iters[0usize], self.scc_times[0usize])
            .unwrap();
         writeln!(&mut res, "scc {}: iterations: {}, time: {:?}", "1", self.scc_iters[1usize], self.scc_times[1usize])
            .unwrap();
         res
      }
   }
   impl Default for AscentProgram {
      fn default() -> Self {
         let mut _self = AscentProgram {
            bar: Default::default(),
            __bar_ind_common: Default::default(),
            bar_indices_0_1: Default::default(),
            foo: Default::default(),
            __foo_ind_common: Default::default(),
            foo_indices_0_1: Default::default(),
            foo_indices_none: Default::default(),
            foobar: Default::default(),
            __foobar_ind_common: Default::default(),
            foobar_indices_0_1: Default::default(),
            scc_times: [std::time::Duration::ZERO; 2usize],
            scc_iters: [0; 2usize],
            update_time_nanos: Default::default(),
            update_indices_duration: std::time::Duration::default(),
         };
         _self
      }
   };
}
