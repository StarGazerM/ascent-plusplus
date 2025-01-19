#![allow(unused_imports)]
use std::fmt::Debug;
use std::hash::Hash;
use std::ops::Deref;
use std::{clone, cmp::max, rc::Rc};

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
struct Tag(&'static str, usize);

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
   ::ascent::rel::rel_codegen! { AscentProgram_bar , (i32 , i32) , [[0] , [0 , 1]] , ser , () }
   ::ascent::rel::rel_codegen! { AscentProgram_baz , (i32 , i32) , [[] , [0 , 1]] , ser , () }
   ::ascent::rel::rel_codegen! { AscentProgram_foo , (i32 , i32) , [[] , [0 , 1] , [1]] , ser , () }
   pub struct AscentProgram {
      #[doc = "\nlogical indices: bar_indices_0; bar_indices_0_1"]
      pub bar: ::ascent::rel::rel!(AscentProgram_bar, (i32, i32), [[0], [0, 1]], ser, ()),
      pub __bar_ind_common: ::ascent::rel::rel_ind_common!(AscentProgram_bar, (i32, i32), [[0], [0, 1]], ser, ()),
      pub bar_indices_0:
         ::ascent::rel::rel_ind!(AscentProgram_bar, (i32, i32), [[0], [0, 1]], ser, (), [0], (i32,), (i32,)),
      pub bar_indices_0_1:
         ::ascent::rel::rel_full_ind!(AscentProgram_bar, (i32, i32), [[0], [0, 1]], ser, (), (i32, i32), ()),
      #[doc = "\nlogical indices: baz_indices_0_1; baz_indices_none"]
      pub baz: ::ascent::rel::rel!(AscentProgram_baz, (i32, i32), [[], [0, 1]], ser, ()),
      pub __baz_ind_common: ::ascent::rel::rel_ind_common!(AscentProgram_baz, (i32, i32), [[], [0, 1]], ser, ()),
      pub baz_indices_0_1:
         ::ascent::rel::rel_full_ind!(AscentProgram_baz, (i32, i32), [[], [0, 1]], ser, (), (i32, i32), ()),
      pub baz_indices_none:
         ::ascent::rel::rel_ind!(AscentProgram_baz, (i32, i32), [[], [0, 1]], ser, (), [], (), (i32, i32)),
      #[doc = "\nlogical indices: foo_indices_0_1; foo_indices_1; foo_indices_none"]
      pub foo: ::ascent::rel::rel!(AscentProgram_foo, (i32, i32), [[], [0, 1], [1]], ser, ()),
      pub __foo_ind_common: ::ascent::rel::rel_ind_common!(AscentProgram_foo, (i32, i32), [[], [0, 1], [1]], ser, ()),
      pub foo_indices_0_1:
         ::ascent::rel::rel_full_ind!(AscentProgram_foo, (i32, i32), [[], [0, 1], [1]], ser, (), (i32, i32), ()),
      pub foo_indices_1:
         ::ascent::rel::rel_ind!(AscentProgram_foo, (i32, i32), [[], [0, 1], [1]], ser, (), [1], (i32,), (i32,)),
      pub foo_indices_none:
         ::ascent::rel::rel_ind!(AscentProgram_foo, (i32, i32), [[], [0, 1], [1]], ser, (), [], (), (i32, i32)),
      scc_times: [std::time::Duration; 5usize],
      scc_iters: [usize; 5usize],
      pub update_time_nanos: std::sync::atomic::AtomicU64,
      pub update_indices_duration: std::time::Duration,
      pub runtime_total: AscentProgramRuntime,
      pub runtime_new: AscentProgramRuntime,
      pub runtime_delta: AscentProgramRuntime,
   }
   pub struct AscentProgramRuntime {
      #[doc = "\nlogical indices: bar_indices_0; bar_indices_0_1"]
      pub bar: ::ascent::rel::rel!(AscentProgram_bar, (i32, i32), [[0], [0, 1]], ser, ()),
      pub __bar_ind_common: ::ascent::rel::rel_ind_common!(AscentProgram_bar, (i32, i32), [[0], [0, 1]], ser, ()),
      pub bar_indices_0:
         ::ascent::rel::rel_ind!(AscentProgram_bar, (i32, i32), [[0], [0, 1]], ser, (), [0], (i32,), (i32,)),
      pub bar_indices_0_1:
         ::ascent::rel::rel_full_ind!(AscentProgram_bar, (i32, i32), [[0], [0, 1]], ser, (), (i32, i32), ()),
      #[doc = "\nlogical indices: baz_indices_0_1; baz_indices_none"]
      pub baz: ::ascent::rel::rel!(AscentProgram_baz, (i32, i32), [[], [0, 1]], ser, ()),
      pub __baz_ind_common: ::ascent::rel::rel_ind_common!(AscentProgram_baz, (i32, i32), [[], [0, 1]], ser, ()),
      pub baz_indices_0_1:
         ::ascent::rel::rel_full_ind!(AscentProgram_baz, (i32, i32), [[], [0, 1]], ser, (), (i32, i32), ()),
      pub baz_indices_none:
         ::ascent::rel::rel_ind!(AscentProgram_baz, (i32, i32), [[], [0, 1]], ser, (), [], (), (i32, i32)),
      #[doc = "\nlogical indices: foo_indices_0_1; foo_indices_1; foo_indices_none"]
      pub foo: ::ascent::rel::rel!(AscentProgram_foo, (i32, i32), [[], [0, 1], [1]], ser, ()),
      pub __foo_ind_common: ::ascent::rel::rel_ind_common!(AscentProgram_foo, (i32, i32), [[], [0, 1], [1]], ser, ()),
      pub foo_indices_0_1:
         ::ascent::rel::rel_full_ind!(AscentProgram_foo, (i32, i32), [[], [0, 1], [1]], ser, (), (i32, i32), ()),
      pub foo_indices_1:
         ::ascent::rel::rel_ind!(AscentProgram_foo, (i32, i32), [[], [0, 1], [1]], ser, (), [1], (i32,), (i32,)),
      pub foo_indices_none:
         ::ascent::rel::rel_ind!(AscentProgram_foo, (i32, i32), [[], [0, 1], [1]], ser, (), [], (), (i32, i32)),
   }
   impl AscentProgram {
      #[allow(unused_assignments, unused_variables, dead_code)]
      pub fn scc_0_exec(&mut self) -> bool {
         use ascent::internal::RelIndexWrite;
         use ascent::internal::{RelIndexRead, RelIndexReadAll, ToRelIndex0, TupleOfBorrowed};
         use core::cmp::PartialEq;
         let _self = self;
         let _scc_start_time = ::ascent::internal::Instant::now();
         let mut __changed = false;
         let mut __default_id = 0;
         ascent::internal::comment("foo <-- ");
         if true {
            let __new_row: (i32, i32) = (1, 2);
            let mut __new_foo = 0;
            if !::ascent::internal::RelFullIndexRead::contains_key(
               &_self.runtime_total.foo_indices_0_1.to_rel_index(&_self.runtime_total.__foo_ind_common),
               &__new_row,
            ) && !::ascent::internal::RelFullIndexRead::contains_key(
               &_self.runtime_delta.foo_indices_0_1.to_rel_index(&_self.runtime_delta.__foo_ind_common),
               &__new_row,
            ) {
               if ::ascent::internal::RelFullIndexWrite::insert_if_not_present(
                  &mut _self.runtime_new.foo_indices_0_1.to_rel_index_write(&mut _self.runtime_new.__foo_ind_common),
                  &__new_row,
                  (),
               ) {
                  __new_foo = _self.foo.len();
                  _self.foo.push((__new_row.0.clone(), __new_row.1.clone()));
                  __default_id = __new_foo;
                  ::ascent::internal::RelIndexWrite::index_insert(
                     &mut _self.runtime_new.foo_indices_1.to_rel_index_write(&mut _self.runtime_new.__foo_ind_common),
                     (__new_row.1.clone(),),
                     (__new_row.0.clone(),),
                  );
                  ::ascent::internal::RelIndexWrite::index_insert(
                     &mut _self
                        .runtime_new
                        .foo_indices_none
                        .to_rel_index_write(&mut _self.runtime_new.__foo_ind_common),
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
            &mut _self.runtime_new.__foo_ind_common,
            &mut _self.runtime_delta.__foo_ind_common,
            &mut _self.runtime_total.__foo_ind_common,
         );
         ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
            &mut _self.runtime_new.foo_indices_0_1.to_rel_index_write(&mut _self.runtime_new.__foo_ind_common),
            &mut _self.runtime_delta.foo_indices_0_1.to_rel_index_write(&mut _self.runtime_delta.__foo_ind_common),
            &mut _self.runtime_total.foo_indices_0_1.to_rel_index_write(&mut _self.runtime_total.__foo_ind_common),
         );
         ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
            &mut _self.runtime_new.foo_indices_1.to_rel_index_write(&mut _self.runtime_new.__foo_ind_common),
            &mut _self.runtime_delta.foo_indices_1.to_rel_index_write(&mut _self.runtime_delta.__foo_ind_common),
            &mut _self.runtime_total.foo_indices_1.to_rel_index_write(&mut _self.runtime_total.__foo_ind_common),
         );
         ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
            &mut _self.runtime_new.foo_indices_none.to_rel_index_write(&mut _self.runtime_new.__foo_ind_common),
            &mut _self.runtime_delta.foo_indices_none.to_rel_index_write(&mut _self.runtime_delta.__foo_ind_common),
            &mut _self.runtime_total.foo_indices_none.to_rel_index_write(&mut _self.runtime_total.__foo_ind_common),
         );
         ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
            &mut _self.runtime_new.__foo_ind_common,
            &mut _self.runtime_delta.__foo_ind_common,
            &mut _self.runtime_total.__foo_ind_common,
         );
         ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
            &mut _self.runtime_new.foo_indices_0_1.to_rel_index_write(&mut _self.runtime_new.__foo_ind_common),
            &mut _self.runtime_delta.foo_indices_0_1.to_rel_index_write(&mut _self.runtime_delta.__foo_ind_common),
            &mut _self.runtime_total.foo_indices_0_1.to_rel_index_write(&mut _self.runtime_total.__foo_ind_common),
         );
         ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
            &mut _self.runtime_new.foo_indices_1.to_rel_index_write(&mut _self.runtime_new.__foo_ind_common),
            &mut _self.runtime_delta.foo_indices_1.to_rel_index_write(&mut _self.runtime_delta.__foo_ind_common),
            &mut _self.runtime_total.foo_indices_1.to_rel_index_write(&mut _self.runtime_total.__foo_ind_common),
         );
         ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
            &mut _self.runtime_new.foo_indices_none.to_rel_index_write(&mut _self.runtime_new.__foo_ind_common),
            &mut _self.runtime_delta.foo_indices_none.to_rel_index_write(&mut _self.runtime_delta.__foo_ind_common),
            &mut _self.runtime_total.foo_indices_none.to_rel_index_write(&mut _self.runtime_total.__foo_ind_common),
         );
         _self.scc_iters[0usize] += 1;
         let need_break = true;
         _self.scc_times[0usize] += _scc_start_time.elapsed();
         need_break
      }
      #[allow(unused_assignments, unused_variables, dead_code)]
      pub fn scc_0(&mut self) -> bool {
         ascent::internal::comment("scc 0");
         {
            let _self = self;
            use ascent::internal::RelIndexWrite;
            use ascent::internal::{RelIndexRead, RelIndexReadAll, ToRelIndex0, TupleOfBorrowed};
            use core::cmp::PartialEq;
            _self.runtime_delta.__foo_ind_common = ::std::mem::take(&mut _self.__foo_ind_common);
            _self.runtime_total.__foo_ind_common = Default::default();
            _self.runtime_new.__foo_ind_common = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut _self.runtime_new.__foo_ind_common,
               &mut _self.runtime_delta.__foo_ind_common,
               &mut _self.runtime_total.__foo_ind_common,
            );
            _self.runtime_delta.foo_indices_0_1 = ::std::mem::take(&mut _self.foo_indices_0_1);
            _self.runtime_total.foo_indices_0_1 = Default::default();
            _self.runtime_new.foo_indices_0_1 = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut _self.runtime_new.foo_indices_0_1.to_rel_index_write(&mut _self.runtime_new.__foo_ind_common),
               &mut _self.runtime_delta.foo_indices_0_1.to_rel_index_write(&mut _self.runtime_delta.__foo_ind_common),
               &mut _self.runtime_total.foo_indices_0_1.to_rel_index_write(&mut _self.runtime_total.__foo_ind_common),
            );
            _self.runtime_delta.foo_indices_1 = ::std::mem::take(&mut _self.foo_indices_1);
            _self.runtime_total.foo_indices_1 = Default::default();
            _self.runtime_new.foo_indices_1 = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut _self.runtime_new.foo_indices_1.to_rel_index_write(&mut _self.runtime_new.__foo_ind_common),
               &mut _self.runtime_delta.foo_indices_1.to_rel_index_write(&mut _self.runtime_delta.__foo_ind_common),
               &mut _self.runtime_total.foo_indices_1.to_rel_index_write(&mut _self.runtime_total.__foo_ind_common),
            );
            _self.runtime_delta.foo_indices_none = ::std::mem::take(&mut _self.foo_indices_none);
            _self.runtime_total.foo_indices_none = Default::default();
            _self.runtime_new.foo_indices_none = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut _self.runtime_new.foo_indices_none.to_rel_index_write(&mut _self.runtime_new.__foo_ind_common),
               &mut _self.runtime_delta.foo_indices_none.to_rel_index_write(&mut _self.runtime_delta.__foo_ind_common),
               &mut _self.runtime_total.foo_indices_none.to_rel_index_write(&mut _self.runtime_total.__foo_ind_common),
            );
            _self.scc_0_exec();
            _self.__foo_ind_common = std::mem::take(&mut _self.runtime_total.__foo_ind_common);
            _self.foo_indices_0_1 = std::mem::take(&mut _self.runtime_total.foo_indices_0_1);
            _self.foo_indices_1 = std::mem::take(&mut _self.runtime_total.foo_indices_1);
            _self.foo_indices_none = std::mem::take(&mut _self.runtime_total.foo_indices_none);
         }
         true
      }
      #[allow(unused_assignments, unused_variables, dead_code)]
      pub fn scc_1_exec(&mut self) -> bool {
         use ascent::internal::RelIndexWrite;
         use ascent::internal::{RelIndexRead, RelIndexReadAll, ToRelIndex0, TupleOfBorrowed};
         use core::cmp::PartialEq;
         let _self = self;
         let _scc_start_time = ::ascent::internal::Instant::now();
         let mut __changed = false;
         let mut __default_id = 0;
         ascent::internal::comment("foo <-- ");
         if true {
            let __new_row: (i32, i32) = (10, 2);
            let mut __new_foo = 0;
            if !::ascent::internal::RelFullIndexRead::contains_key(
               &_self.runtime_total.foo_indices_0_1.to_rel_index(&_self.runtime_total.__foo_ind_common),
               &__new_row,
            ) && !::ascent::internal::RelFullIndexRead::contains_key(
               &_self.runtime_delta.foo_indices_0_1.to_rel_index(&_self.runtime_delta.__foo_ind_common),
               &__new_row,
            ) {
               if ::ascent::internal::RelFullIndexWrite::insert_if_not_present(
                  &mut _self.runtime_new.foo_indices_0_1.to_rel_index_write(&mut _self.runtime_new.__foo_ind_common),
                  &__new_row,
                  (),
               ) {
                  __new_foo = _self.foo.len();
                  _self.foo.push((__new_row.0.clone(), __new_row.1.clone()));
                  __default_id = __new_foo;
                  ::ascent::internal::RelIndexWrite::index_insert(
                     &mut _self.runtime_new.foo_indices_1.to_rel_index_write(&mut _self.runtime_new.__foo_ind_common),
                     (__new_row.1.clone(),),
                     (__new_row.0.clone(),),
                  );
                  ::ascent::internal::RelIndexWrite::index_insert(
                     &mut _self
                        .runtime_new
                        .foo_indices_none
                        .to_rel_index_write(&mut _self.runtime_new.__foo_ind_common),
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
            &mut _self.runtime_new.__foo_ind_common,
            &mut _self.runtime_delta.__foo_ind_common,
            &mut _self.runtime_total.__foo_ind_common,
         );
         ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
            &mut _self.runtime_new.foo_indices_0_1.to_rel_index_write(&mut _self.runtime_new.__foo_ind_common),
            &mut _self.runtime_delta.foo_indices_0_1.to_rel_index_write(&mut _self.runtime_delta.__foo_ind_common),
            &mut _self.runtime_total.foo_indices_0_1.to_rel_index_write(&mut _self.runtime_total.__foo_ind_common),
         );
         ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
            &mut _self.runtime_new.foo_indices_1.to_rel_index_write(&mut _self.runtime_new.__foo_ind_common),
            &mut _self.runtime_delta.foo_indices_1.to_rel_index_write(&mut _self.runtime_delta.__foo_ind_common),
            &mut _self.runtime_total.foo_indices_1.to_rel_index_write(&mut _self.runtime_total.__foo_ind_common),
         );
         ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
            &mut _self.runtime_new.foo_indices_none.to_rel_index_write(&mut _self.runtime_new.__foo_ind_common),
            &mut _self.runtime_delta.foo_indices_none.to_rel_index_write(&mut _self.runtime_delta.__foo_ind_common),
            &mut _self.runtime_total.foo_indices_none.to_rel_index_write(&mut _self.runtime_total.__foo_ind_common),
         );
         ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
            &mut _self.runtime_new.__foo_ind_common,
            &mut _self.runtime_delta.__foo_ind_common,
            &mut _self.runtime_total.__foo_ind_common,
         );
         ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
            &mut _self.runtime_new.foo_indices_0_1.to_rel_index_write(&mut _self.runtime_new.__foo_ind_common),
            &mut _self.runtime_delta.foo_indices_0_1.to_rel_index_write(&mut _self.runtime_delta.__foo_ind_common),
            &mut _self.runtime_total.foo_indices_0_1.to_rel_index_write(&mut _self.runtime_total.__foo_ind_common),
         );
         ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
            &mut _self.runtime_new.foo_indices_1.to_rel_index_write(&mut _self.runtime_new.__foo_ind_common),
            &mut _self.runtime_delta.foo_indices_1.to_rel_index_write(&mut _self.runtime_delta.__foo_ind_common),
            &mut _self.runtime_total.foo_indices_1.to_rel_index_write(&mut _self.runtime_total.__foo_ind_common),
         );
         ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
            &mut _self.runtime_new.foo_indices_none.to_rel_index_write(&mut _self.runtime_new.__foo_ind_common),
            &mut _self.runtime_delta.foo_indices_none.to_rel_index_write(&mut _self.runtime_delta.__foo_ind_common),
            &mut _self.runtime_total.foo_indices_none.to_rel_index_write(&mut _self.runtime_total.__foo_ind_common),
         );
         _self.scc_iters[1usize] += 1;
         let need_break = true;
         _self.scc_times[1usize] += _scc_start_time.elapsed();
         need_break
      }
      #[allow(unused_assignments, unused_variables, dead_code)]
      pub fn scc_1(&mut self) -> bool {
         ascent::internal::comment("scc 1");
         {
            let _self = self;
            use ascent::internal::RelIndexWrite;
            use ascent::internal::{RelIndexRead, RelIndexReadAll, ToRelIndex0, TupleOfBorrowed};
            use core::cmp::PartialEq;
            _self.runtime_delta.__foo_ind_common = ::std::mem::take(&mut _self.__foo_ind_common);
            _self.runtime_total.__foo_ind_common = Default::default();
            _self.runtime_new.__foo_ind_common = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut _self.runtime_new.__foo_ind_common,
               &mut _self.runtime_delta.__foo_ind_common,
               &mut _self.runtime_total.__foo_ind_common,
            );
            _self.runtime_delta.foo_indices_0_1 = ::std::mem::take(&mut _self.foo_indices_0_1);
            _self.runtime_total.foo_indices_0_1 = Default::default();
            _self.runtime_new.foo_indices_0_1 = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut _self.runtime_new.foo_indices_0_1.to_rel_index_write(&mut _self.runtime_new.__foo_ind_common),
               &mut _self.runtime_delta.foo_indices_0_1.to_rel_index_write(&mut _self.runtime_delta.__foo_ind_common),
               &mut _self.runtime_total.foo_indices_0_1.to_rel_index_write(&mut _self.runtime_total.__foo_ind_common),
            );
            _self.runtime_delta.foo_indices_1 = ::std::mem::take(&mut _self.foo_indices_1);
            _self.runtime_total.foo_indices_1 = Default::default();
            _self.runtime_new.foo_indices_1 = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut _self.runtime_new.foo_indices_1.to_rel_index_write(&mut _self.runtime_new.__foo_ind_common),
               &mut _self.runtime_delta.foo_indices_1.to_rel_index_write(&mut _self.runtime_delta.__foo_ind_common),
               &mut _self.runtime_total.foo_indices_1.to_rel_index_write(&mut _self.runtime_total.__foo_ind_common),
            );
            _self.runtime_delta.foo_indices_none = ::std::mem::take(&mut _self.foo_indices_none);
            _self.runtime_total.foo_indices_none = Default::default();
            _self.runtime_new.foo_indices_none = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut _self.runtime_new.foo_indices_none.to_rel_index_write(&mut _self.runtime_new.__foo_ind_common),
               &mut _self.runtime_delta.foo_indices_none.to_rel_index_write(&mut _self.runtime_delta.__foo_ind_common),
               &mut _self.runtime_total.foo_indices_none.to_rel_index_write(&mut _self.runtime_total.__foo_ind_common),
            );
            _self.scc_1_exec();
            _self.__foo_ind_common = std::mem::take(&mut _self.runtime_total.__foo_ind_common);
            _self.foo_indices_0_1 = std::mem::take(&mut _self.runtime_total.foo_indices_0_1);
            _self.foo_indices_1 = std::mem::take(&mut _self.runtime_total.foo_indices_1);
            _self.foo_indices_none = std::mem::take(&mut _self.runtime_total.foo_indices_none);
         }
         true
      }
      #[allow(unused_assignments, unused_variables, dead_code)]
      pub fn scc_2_exec(&mut self) -> bool {
         use ascent::internal::RelIndexWrite;
         use ascent::internal::{RelIndexRead, RelIndexReadAll, ToRelIndex0, TupleOfBorrowed};
         use core::cmp::PartialEq;
         let _self = self;
         let _scc_start_time = ::ascent::internal::Instant::now();
         let mut __changed = false;
         let mut __default_id = 0;
         ascent::internal::comment("bar <-- ");
         if true {
            let __new_row: (i32, i32) = (2, 3);
            let mut __new_bar = 0;
            if !::ascent::internal::RelFullIndexRead::contains_key(
               &_self.runtime_total.bar_indices_0_1.to_rel_index(&_self.runtime_total.__bar_ind_common),
               &__new_row,
            ) && !::ascent::internal::RelFullIndexRead::contains_key(
               &_self.runtime_delta.bar_indices_0_1.to_rel_index(&_self.runtime_delta.__bar_ind_common),
               &__new_row,
            ) {
               if ::ascent::internal::RelFullIndexWrite::insert_if_not_present(
                  &mut _self.runtime_new.bar_indices_0_1.to_rel_index_write(&mut _self.runtime_new.__bar_ind_common),
                  &__new_row,
                  (),
               ) {
                  __new_bar = _self.bar.len();
                  _self.bar.push((__new_row.0.clone(), __new_row.1.clone()));
                  __default_id = __new_bar;
                  ::ascent::internal::RelIndexWrite::index_insert(
                     &mut _self.runtime_new.bar_indices_0.to_rel_index_write(&mut _self.runtime_new.__bar_ind_common),
                     (__new_row.0.clone(),),
                     (__new_row.1.clone(),),
                  );
                  __changed = true;
               } else {
               }
            } else {
            }
         }
         ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
            &mut _self.runtime_new.__bar_ind_common,
            &mut _self.runtime_delta.__bar_ind_common,
            &mut _self.runtime_total.__bar_ind_common,
         );
         ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
            &mut _self.runtime_new.bar_indices_0.to_rel_index_write(&mut _self.runtime_new.__bar_ind_common),
            &mut _self.runtime_delta.bar_indices_0.to_rel_index_write(&mut _self.runtime_delta.__bar_ind_common),
            &mut _self.runtime_total.bar_indices_0.to_rel_index_write(&mut _self.runtime_total.__bar_ind_common),
         );
         ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
            &mut _self.runtime_new.bar_indices_0_1.to_rel_index_write(&mut _self.runtime_new.__bar_ind_common),
            &mut _self.runtime_delta.bar_indices_0_1.to_rel_index_write(&mut _self.runtime_delta.__bar_ind_common),
            &mut _self.runtime_total.bar_indices_0_1.to_rel_index_write(&mut _self.runtime_total.__bar_ind_common),
         );
         ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
            &mut _self.runtime_new.__bar_ind_common,
            &mut _self.runtime_delta.__bar_ind_common,
            &mut _self.runtime_total.__bar_ind_common,
         );
         ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
            &mut _self.runtime_new.bar_indices_0.to_rel_index_write(&mut _self.runtime_new.__bar_ind_common),
            &mut _self.runtime_delta.bar_indices_0.to_rel_index_write(&mut _self.runtime_delta.__bar_ind_common),
            &mut _self.runtime_total.bar_indices_0.to_rel_index_write(&mut _self.runtime_total.__bar_ind_common),
         );
         ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
            &mut _self.runtime_new.bar_indices_0_1.to_rel_index_write(&mut _self.runtime_new.__bar_ind_common),
            &mut _self.runtime_delta.bar_indices_0_1.to_rel_index_write(&mut _self.runtime_delta.__bar_ind_common),
            &mut _self.runtime_total.bar_indices_0_1.to_rel_index_write(&mut _self.runtime_total.__bar_ind_common),
         );
         _self.scc_iters[2usize] += 1;
         let need_break = true;
         _self.scc_times[2usize] += _scc_start_time.elapsed();
         need_break
      }
      #[allow(unused_assignments, unused_variables, dead_code)]
      pub fn scc_2(&mut self) -> bool {
         ascent::internal::comment("scc 2");
         {
            let _self = self;
            use ascent::internal::RelIndexWrite;
            use ascent::internal::{RelIndexRead, RelIndexReadAll, ToRelIndex0, TupleOfBorrowed};
            use core::cmp::PartialEq;
            _self.runtime_delta.__bar_ind_common = ::std::mem::take(&mut _self.__bar_ind_common);
            _self.runtime_total.__bar_ind_common = Default::default();
            _self.runtime_new.__bar_ind_common = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut _self.runtime_new.__bar_ind_common,
               &mut _self.runtime_delta.__bar_ind_common,
               &mut _self.runtime_total.__bar_ind_common,
            );
            _self.runtime_delta.bar_indices_0 = ::std::mem::take(&mut _self.bar_indices_0);
            _self.runtime_total.bar_indices_0 = Default::default();
            _self.runtime_new.bar_indices_0 = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut _self.runtime_new.bar_indices_0.to_rel_index_write(&mut _self.runtime_new.__bar_ind_common),
               &mut _self.runtime_delta.bar_indices_0.to_rel_index_write(&mut _self.runtime_delta.__bar_ind_common),
               &mut _self.runtime_total.bar_indices_0.to_rel_index_write(&mut _self.runtime_total.__bar_ind_common),
            );
            _self.runtime_delta.bar_indices_0_1 = ::std::mem::take(&mut _self.bar_indices_0_1);
            _self.runtime_total.bar_indices_0_1 = Default::default();
            _self.runtime_new.bar_indices_0_1 = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut _self.runtime_new.bar_indices_0_1.to_rel_index_write(&mut _self.runtime_new.__bar_ind_common),
               &mut _self.runtime_delta.bar_indices_0_1.to_rel_index_write(&mut _self.runtime_delta.__bar_ind_common),
               &mut _self.runtime_total.bar_indices_0_1.to_rel_index_write(&mut _self.runtime_total.__bar_ind_common),
            );
            _self.scc_2_exec();
            _self.__bar_ind_common = std::mem::take(&mut _self.runtime_total.__bar_ind_common);
            _self.bar_indices_0 = std::mem::take(&mut _self.runtime_total.bar_indices_0);
            _self.bar_indices_0_1 = std::mem::take(&mut _self.runtime_total.bar_indices_0_1);
         }
         true
      }
      #[allow(unused_assignments, unused_variables, dead_code)]
      pub fn scc_3_exec(&mut self) -> bool {
         use ascent::internal::RelIndexWrite;
         use ascent::internal::{RelIndexRead, RelIndexReadAll, ToRelIndex0, TupleOfBorrowed};
         use core::cmp::PartialEq;
         let _self = self;
         let _scc_start_time = ::ascent::internal::Instant::now();
         let mut __changed = false;
         let mut __default_id = 0;
         ascent::internal::comment("bar <-- ");
         if true {
            let __new_row: (i32, i32) = (2, 1);
            let mut __new_bar = 0;
            if !::ascent::internal::RelFullIndexRead::contains_key(
               &_self.runtime_total.bar_indices_0_1.to_rel_index(&_self.runtime_total.__bar_ind_common),
               &__new_row,
            ) && !::ascent::internal::RelFullIndexRead::contains_key(
               &_self.runtime_delta.bar_indices_0_1.to_rel_index(&_self.runtime_delta.__bar_ind_common),
               &__new_row,
            ) {
               if ::ascent::internal::RelFullIndexWrite::insert_if_not_present(
                  &mut _self.runtime_new.bar_indices_0_1.to_rel_index_write(&mut _self.runtime_new.__bar_ind_common),
                  &__new_row,
                  (),
               ) {
                  __new_bar = _self.bar.len();
                  _self.bar.push((__new_row.0.clone(), __new_row.1.clone()));
                  __default_id = __new_bar;
                  ::ascent::internal::RelIndexWrite::index_insert(
                     &mut _self.runtime_new.bar_indices_0.to_rel_index_write(&mut _self.runtime_new.__bar_ind_common),
                     (__new_row.0.clone(),),
                     (__new_row.1.clone(),),
                  );
                  __changed = true;
               } else {
               }
            } else {
            }
         }
         ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
            &mut _self.runtime_new.__bar_ind_common,
            &mut _self.runtime_delta.__bar_ind_common,
            &mut _self.runtime_total.__bar_ind_common,
         );
         ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
            &mut _self.runtime_new.bar_indices_0.to_rel_index_write(&mut _self.runtime_new.__bar_ind_common),
            &mut _self.runtime_delta.bar_indices_0.to_rel_index_write(&mut _self.runtime_delta.__bar_ind_common),
            &mut _self.runtime_total.bar_indices_0.to_rel_index_write(&mut _self.runtime_total.__bar_ind_common),
         );
         ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
            &mut _self.runtime_new.bar_indices_0_1.to_rel_index_write(&mut _self.runtime_new.__bar_ind_common),
            &mut _self.runtime_delta.bar_indices_0_1.to_rel_index_write(&mut _self.runtime_delta.__bar_ind_common),
            &mut _self.runtime_total.bar_indices_0_1.to_rel_index_write(&mut _self.runtime_total.__bar_ind_common),
         );
         ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
            &mut _self.runtime_new.__bar_ind_common,
            &mut _self.runtime_delta.__bar_ind_common,
            &mut _self.runtime_total.__bar_ind_common,
         );
         ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
            &mut _self.runtime_new.bar_indices_0.to_rel_index_write(&mut _self.runtime_new.__bar_ind_common),
            &mut _self.runtime_delta.bar_indices_0.to_rel_index_write(&mut _self.runtime_delta.__bar_ind_common),
            &mut _self.runtime_total.bar_indices_0.to_rel_index_write(&mut _self.runtime_total.__bar_ind_common),
         );
         ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
            &mut _self.runtime_new.bar_indices_0_1.to_rel_index_write(&mut _self.runtime_new.__bar_ind_common),
            &mut _self.runtime_delta.bar_indices_0_1.to_rel_index_write(&mut _self.runtime_delta.__bar_ind_common),
            &mut _self.runtime_total.bar_indices_0_1.to_rel_index_write(&mut _self.runtime_total.__bar_ind_common),
         );
         _self.scc_iters[3usize] += 1;
         let need_break = true;
         _self.scc_times[3usize] += _scc_start_time.elapsed();
         need_break
      }
      #[allow(unused_assignments, unused_variables, dead_code)]
      pub fn scc_3(&mut self) -> bool {
         ascent::internal::comment("scc 3");
         {
            let _self = self;
            use ascent::internal::RelIndexWrite;
            use ascent::internal::{RelIndexRead, RelIndexReadAll, ToRelIndex0, TupleOfBorrowed};
            use core::cmp::PartialEq;
            _self.runtime_delta.__bar_ind_common = ::std::mem::take(&mut _self.__bar_ind_common);
            _self.runtime_total.__bar_ind_common = Default::default();
            _self.runtime_new.__bar_ind_common = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut _self.runtime_new.__bar_ind_common,
               &mut _self.runtime_delta.__bar_ind_common,
               &mut _self.runtime_total.__bar_ind_common,
            );
            _self.runtime_delta.bar_indices_0 = ::std::mem::take(&mut _self.bar_indices_0);
            _self.runtime_total.bar_indices_0 = Default::default();
            _self.runtime_new.bar_indices_0 = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut _self.runtime_new.bar_indices_0.to_rel_index_write(&mut _self.runtime_new.__bar_ind_common),
               &mut _self.runtime_delta.bar_indices_0.to_rel_index_write(&mut _self.runtime_delta.__bar_ind_common),
               &mut _self.runtime_total.bar_indices_0.to_rel_index_write(&mut _self.runtime_total.__bar_ind_common),
            );
            _self.runtime_delta.bar_indices_0_1 = ::std::mem::take(&mut _self.bar_indices_0_1);
            _self.runtime_total.bar_indices_0_1 = Default::default();
            _self.runtime_new.bar_indices_0_1 = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut _self.runtime_new.bar_indices_0_1.to_rel_index_write(&mut _self.runtime_new.__bar_ind_common),
               &mut _self.runtime_delta.bar_indices_0_1.to_rel_index_write(&mut _self.runtime_delta.__bar_ind_common),
               &mut _self.runtime_total.bar_indices_0_1.to_rel_index_write(&mut _self.runtime_total.__bar_ind_common),
            );
            _self.scc_3_exec();
            _self.__bar_ind_common = std::mem::take(&mut _self.runtime_total.__bar_ind_common);
            _self.bar_indices_0 = std::mem::take(&mut _self.runtime_total.bar_indices_0);
            _self.bar_indices_0_1 = std::mem::take(&mut _self.runtime_total.bar_indices_0_1);
         }
         true
      }
      #[allow(unused_assignments, unused_variables, dead_code)]
      pub fn scc_4_exec(&mut self) -> bool {
         use ascent::internal::RelIndexWrite;
         use ascent::internal::{RelIndexRead, RelIndexReadAll, ToRelIndex0, TupleOfBorrowed};
         use core::cmp::PartialEq;
         let _self = self;
         let _scc_start_time = ::ascent::internal::Instant::now();
         let mut __changed = false;
         ascent::internal::comment("baz <-- foo_indices_1_delta, bar_indices_0_total+delta, if ⋯ [SIMPLE JOIN]");
         if _self.runtime_delta.foo_indices_1.to_rel_index(&_self.runtime_delta.__foo_ind_common).len() > 0 {
            if _self.runtime_delta.foo_indices_1.to_rel_index(&_self.runtime_delta.__foo_ind_common).len()
               <= ascent::internal::RelIndexCombined::new(
                  &_self.runtime_total.bar_indices_0.to_rel_index(&_self.runtime_total.__bar_ind_common),
                  &_self.runtime_delta.bar_indices_0.to_rel_index(&_self.runtime_delta.__bar_ind_common),
               )
               .len()
            {
               _self
                  .runtime_delta
                  .foo_indices_1
                  .to_rel_index(&_self.runtime_delta.__foo_ind_common)
                  .iter_all()
                  .for_each(|(__cl1_joined_columns, __cl1_tuple_indices)| {
                     let __cl1_joined_columns = __cl1_joined_columns.tuple_of_borrowed();
                     let y = __cl1_joined_columns.0;
                     if let Some(__matching) = ascent::internal::RelIndexCombined::new(
                        &_self.runtime_total.bar_indices_0.to_rel_index(&_self.runtime_total.__bar_ind_common),
                        &_self.runtime_delta.bar_indices_0.to_rel_index(&_self.runtime_delta.__bar_ind_common),
                     )
                     .index_get(&(y.clone(),))
                     {
                        __cl1_tuple_indices.for_each(|cl1_val| {
                           let cl1_val = cl1_val.tuple_of_borrowed();
                           let x: &i32 = cl1_val.0;
                           if *x != 10 {
                              __matching.clone().for_each(|__val| {
                                 let mut __dep_changed = false;
                                 let mut __default_id = 0;
                                 let __val = __val.tuple_of_borrowed();
                                 let __arg_pattern_: &i32 = __val.0;
                                 if let z = __arg_pattern_ {
                                    if *z < 4 {
                                       if x != z {
                                          let __new_row: (i32, i32) = (*x, *z);
                                          let mut __new_baz = 0;
                                          if !::ascent::internal::RelFullIndexRead::contains_key(
                                             &_self
                                                .runtime_total
                                                .baz_indices_0_1
                                                .to_rel_index(&_self.runtime_total.__baz_ind_common),
                                             &__new_row,
                                          ) && !::ascent::internal::RelFullIndexRead::contains_key(
                                             &_self
                                                .runtime_delta
                                                .baz_indices_0_1
                                                .to_rel_index(&_self.runtime_delta.__baz_ind_common),
                                             &__new_row,
                                          ) {
                                             if ::ascent::internal::RelFullIndexWrite::insert_if_not_present(
                                                &mut _self
                                                   .runtime_new
                                                   .baz_indices_0_1
                                                   .to_rel_index_write(&mut _self.runtime_new.__baz_ind_common),
                                                &__new_row,
                                                (),
                                             ) {
                                                __new_baz = _self.baz.len();
                                                _self.baz.push((__new_row.0.clone(), __new_row.1.clone()));
                                                __default_id = __new_baz;
                                                ::ascent::internal::RelIndexWrite::index_insert(
                                                   &mut _self
                                                      .runtime_new
                                                      .baz_indices_none
                                                      .to_rel_index_write(&mut _self.runtime_new.__baz_ind_common),
                                                   (),
                                                   (__new_row.0.clone(), __new_row.1.clone()),
                                                );
                                                __changed = true;
                                             } else {
                                             }
                                          } else {
                                          }
                                       }
                                    }
                                 }
                              });
                           }
                        });
                     }
                  });
            } else {
               ascent::internal::RelIndexCombined::new(
                  &_self.runtime_total.bar_indices_0.to_rel_index(&_self.runtime_total.__bar_ind_common),
                  &_self.runtime_delta.bar_indices_0.to_rel_index(&_self.runtime_delta.__bar_ind_common),
               )
               .iter_all()
               .for_each(|(__cl1_joined_columns, __cl1_tuple_indices)| {
                  let __cl1_joined_columns = __cl1_joined_columns.tuple_of_borrowed();
                  let y = __cl1_joined_columns.0;
                  if let Some(__matching) = _self
                     .runtime_delta
                     .foo_indices_1
                     .to_rel_index(&_self.runtime_delta.__foo_ind_common)
                     .index_get(&(y.clone(),))
                  {
                     __cl1_tuple_indices.for_each(|cl1_val| {
                        let cl1_val = cl1_val.tuple_of_borrowed();
                        let __arg_pattern_: &i32 = cl1_val.0;
                        if let z = __arg_pattern_ {
                           if *z < 4 {
                              __matching.clone().for_each(|__val| {
                                 let mut __dep_changed = false;
                                 let mut __default_id = 0;
                                 let __val = __val.tuple_of_borrowed();
                                 let x: &i32 = __val.0;
                                 if *x != 10 {
                                    if x != z {
                                       let __new_row: (i32, i32) = (*x, *z);
                                       let mut __new_baz = 0;
                                       if !::ascent::internal::RelFullIndexRead::contains_key(
                                          &_self
                                             .runtime_total
                                             .baz_indices_0_1
                                             .to_rel_index(&_self.runtime_total.__baz_ind_common),
                                          &__new_row,
                                       ) && !::ascent::internal::RelFullIndexRead::contains_key(
                                          &_self
                                             .runtime_delta
                                             .baz_indices_0_1
                                             .to_rel_index(&_self.runtime_delta.__baz_ind_common),
                                          &__new_row,
                                       ) {
                                          if ::ascent::internal::RelFullIndexWrite::insert_if_not_present(
                                             &mut _self
                                                .runtime_new
                                                .baz_indices_0_1
                                                .to_rel_index_write(&mut _self.runtime_new.__baz_ind_common),
                                             &__new_row,
                                             (),
                                          ) {
                                             __new_baz = _self.baz.len();
                                             _self.baz.push((__new_row.0.clone(), __new_row.1.clone()));
                                             __default_id = __new_baz;
                                             ::ascent::internal::RelIndexWrite::index_insert(
                                                &mut _self
                                                   .runtime_new
                                                   .baz_indices_none
                                                   .to_rel_index_write(&mut _self.runtime_new.__baz_ind_common),
                                                (),
                                                (__new_row.0.clone(), __new_row.1.clone()),
                                             );
                                             __changed = true;
                                          } else {
                                          }
                                       } else {
                                       }
                                    }
                                 }
                              });
                           }
                        }
                     });
                  }
               });
            }
         }
         ascent::internal::comment("baz <-- foo_indices_1_total, bar_indices_0_delta, if ⋯ [SIMPLE JOIN]");
         if _self.runtime_delta.bar_indices_0.to_rel_index(&_self.runtime_delta.__bar_ind_common).len() > 0 {
            if _self.runtime_total.foo_indices_1.to_rel_index(&_self.runtime_total.__foo_ind_common).len()
               <= _self.runtime_delta.bar_indices_0.to_rel_index(&_self.runtime_delta.__bar_ind_common).len()
            {
               _self
                  .runtime_total
                  .foo_indices_1
                  .to_rel_index(&_self.runtime_total.__foo_ind_common)
                  .iter_all()
                  .for_each(|(__cl1_joined_columns, __cl1_tuple_indices)| {
                     let __cl1_joined_columns = __cl1_joined_columns.tuple_of_borrowed();
                     let y = __cl1_joined_columns.0;
                     if let Some(__matching) = _self
                        .runtime_delta
                        .bar_indices_0
                        .to_rel_index(&_self.runtime_delta.__bar_ind_common)
                        .index_get(&(y.clone(),))
                     {
                        __cl1_tuple_indices.for_each(|cl1_val| {
                           let cl1_val = cl1_val.tuple_of_borrowed();
                           let x: &i32 = cl1_val.0;
                           if *x != 10 {
                              __matching.clone().for_each(|__val| {
                                 let mut __dep_changed = false;
                                 let mut __default_id = 0;
                                 let __val = __val.tuple_of_borrowed();
                                 let __arg_pattern_: &i32 = __val.0;
                                 if let z = __arg_pattern_ {
                                    if *z < 4 {
                                       if x != z {
                                          let __new_row: (i32, i32) = (*x, *z);
                                          let mut __new_baz = 0;
                                          if !::ascent::internal::RelFullIndexRead::contains_key(
                                             &_self
                                                .runtime_total
                                                .baz_indices_0_1
                                                .to_rel_index(&_self.runtime_total.__baz_ind_common),
                                             &__new_row,
                                          ) && !::ascent::internal::RelFullIndexRead::contains_key(
                                             &_self
                                                .runtime_delta
                                                .baz_indices_0_1
                                                .to_rel_index(&_self.runtime_delta.__baz_ind_common),
                                             &__new_row,
                                          ) {
                                             if ::ascent::internal::RelFullIndexWrite::insert_if_not_present(
                                                &mut _self
                                                   .runtime_new
                                                   .baz_indices_0_1
                                                   .to_rel_index_write(&mut _self.runtime_new.__baz_ind_common),
                                                &__new_row,
                                                (),
                                             ) {
                                                __new_baz = _self.baz.len();
                                                _self.baz.push((__new_row.0.clone(), __new_row.1.clone()));
                                                __default_id = __new_baz;
                                                ::ascent::internal::RelIndexWrite::index_insert(
                                                   &mut _self
                                                      .runtime_new
                                                      .baz_indices_none
                                                      .to_rel_index_write(&mut _self.runtime_new.__baz_ind_common),
                                                   (),
                                                   (__new_row.0.clone(), __new_row.1.clone()),
                                                );
                                                __changed = true;
                                             } else {
                                             }
                                          } else {
                                          }
                                       }
                                    }
                                 }
                              });
                           }
                        });
                     }
                  });
            } else {
               _self
                  .runtime_delta
                  .bar_indices_0
                  .to_rel_index(&_self.runtime_delta.__bar_ind_common)
                  .iter_all()
                  .for_each(|(__cl1_joined_columns, __cl1_tuple_indices)| {
                     let __cl1_joined_columns = __cl1_joined_columns.tuple_of_borrowed();
                     let y = __cl1_joined_columns.0;
                     if let Some(__matching) = _self
                        .runtime_total
                        .foo_indices_1
                        .to_rel_index(&_self.runtime_total.__foo_ind_common)
                        .index_get(&(y.clone(),))
                     {
                        __cl1_tuple_indices.for_each(|cl1_val| {
                           let cl1_val = cl1_val.tuple_of_borrowed();
                           let __arg_pattern_: &i32 = cl1_val.0;
                           if let z = __arg_pattern_ {
                              if *z < 4 {
                                 __matching.clone().for_each(|__val| {
                                    let mut __dep_changed = false;
                                    let mut __default_id = 0;
                                    let __val = __val.tuple_of_borrowed();
                                    let x: &i32 = __val.0;
                                    if *x != 10 {
                                       if x != z {
                                          let __new_row: (i32, i32) = (*x, *z);
                                          let mut __new_baz = 0;
                                          if !::ascent::internal::RelFullIndexRead::contains_key(
                                             &_self
                                                .runtime_total
                                                .baz_indices_0_1
                                                .to_rel_index(&_self.runtime_total.__baz_ind_common),
                                             &__new_row,
                                          ) && !::ascent::internal::RelFullIndexRead::contains_key(
                                             &_self
                                                .runtime_delta
                                                .baz_indices_0_1
                                                .to_rel_index(&_self.runtime_delta.__baz_ind_common),
                                             &__new_row,
                                          ) {
                                             if ::ascent::internal::RelFullIndexWrite::insert_if_not_present(
                                                &mut _self
                                                   .runtime_new
                                                   .baz_indices_0_1
                                                   .to_rel_index_write(&mut _self.runtime_new.__baz_ind_common),
                                                &__new_row,
                                                (),
                                             ) {
                                                __new_baz = _self.baz.len();
                                                _self.baz.push((__new_row.0.clone(), __new_row.1.clone()));
                                                __default_id = __new_baz;
                                                ::ascent::internal::RelIndexWrite::index_insert(
                                                   &mut _self
                                                      .runtime_new
                                                      .baz_indices_none
                                                      .to_rel_index_write(&mut _self.runtime_new.__baz_ind_common),
                                                   (),
                                                   (__new_row.0.clone(), __new_row.1.clone()),
                                                );
                                                __changed = true;
                                             } else {
                                             }
                                          } else {
                                          }
                                       }
                                    }
                                 });
                              }
                           }
                        });
                     }
                  });
            }
         }
         ascent::internal::comment("foo, bar <-- baz_indices_none_delta");
         if _self.runtime_delta.baz_indices_none.to_rel_index(&_self.runtime_delta.__baz_ind_common).len() > 0 {
            if let Some(__matching) = _self
               .runtime_delta
               .baz_indices_none
               .to_rel_index(&_self.runtime_delta.__baz_ind_common)
               .index_get(&())
            {
               __matching.for_each(|__val| {
                  let mut __dep_changed = false;
                  let mut __default_id = 0;
                  let __val = __val.tuple_of_borrowed();
                  let x: &i32 = __val.0;
                  let y: &i32 = __val.1;
                  let __new_row: (i32, i32) = (*x, *y);
                  let mut __new_foo = 0;
                  if !::ascent::internal::RelFullIndexRead::contains_key(
                     &_self.runtime_total.foo_indices_0_1.to_rel_index(&_self.runtime_total.__foo_ind_common),
                     &__new_row,
                  ) && !::ascent::internal::RelFullIndexRead::contains_key(
                     &_self.runtime_delta.foo_indices_0_1.to_rel_index(&_self.runtime_delta.__foo_ind_common),
                     &__new_row,
                  ) {
                     if ::ascent::internal::RelFullIndexWrite::insert_if_not_present(
                        &mut _self
                           .runtime_new
                           .foo_indices_0_1
                           .to_rel_index_write(&mut _self.runtime_new.__foo_ind_common),
                        &__new_row,
                        (),
                     ) {
                        __new_foo = _self.foo.len();
                        _self.foo.push((__new_row.0.clone(), __new_row.1.clone()));
                        __default_id = __new_foo;
                        ::ascent::internal::RelIndexWrite::index_insert(
                           &mut _self
                              .runtime_new
                              .foo_indices_1
                              .to_rel_index_write(&mut _self.runtime_new.__foo_ind_common),
                           (__new_row.1.clone(),),
                           (__new_row.0.clone(),),
                        );
                        ::ascent::internal::RelIndexWrite::index_insert(
                           &mut _self
                              .runtime_new
                              .foo_indices_none
                              .to_rel_index_write(&mut _self.runtime_new.__foo_ind_common),
                           (),
                           (__new_row.0.clone(), __new_row.1.clone()),
                        );
                        __changed = true;
                     } else {
                     }
                  } else {
                  }
                  let __new_row: (i32, i32) = (*x, *y);
                  let mut __new_bar = 0;
                  if !::ascent::internal::RelFullIndexRead::contains_key(
                     &_self.runtime_total.bar_indices_0_1.to_rel_index(&_self.runtime_total.__bar_ind_common),
                     &__new_row,
                  ) && !::ascent::internal::RelFullIndexRead::contains_key(
                     &_self.runtime_delta.bar_indices_0_1.to_rel_index(&_self.runtime_delta.__bar_ind_common),
                     &__new_row,
                  ) {
                     if ::ascent::internal::RelFullIndexWrite::insert_if_not_present(
                        &mut _self
                           .runtime_new
                           .bar_indices_0_1
                           .to_rel_index_write(&mut _self.runtime_new.__bar_ind_common),
                        &__new_row,
                        (),
                     ) {
                        __new_bar = _self.bar.len();
                        _self.bar.push((__new_row.0.clone(), __new_row.1.clone()));
                        __default_id = __new_bar;
                        ::ascent::internal::RelIndexWrite::index_insert(
                           &mut _self
                              .runtime_new
                              .bar_indices_0
                              .to_rel_index_write(&mut _self.runtime_new.__bar_ind_common),
                           (__new_row.0.clone(),),
                           (__new_row.1.clone(),),
                        );
                        __changed = true;
                     } else {
                     }
                  } else {
                  }
               });
            }
         }
         ascent::internal::comment("baz <-- foo_indices_none_delta, bar_indices_0_total+delta, if ⋯");
         if _self.runtime_delta.foo_indices_none.to_rel_index(&_self.runtime_delta.__foo_ind_common).len() > 0 {
            if let Some(__matching) = _self
               .runtime_delta
               .foo_indices_none
               .to_rel_index(&_self.runtime_delta.__foo_ind_common)
               .index_get(&())
            {
               __matching.for_each(|__val| {
                  let mut __dep_changed = false;
                  let mut __default_id = 0;
                  let __val = __val.tuple_of_borrowed();
                  let x: &i32 = __val.0;
                  let y: &i32 = __val.1;
                  if *x != 10 {
                     if let Some(__matching) = ascent::internal::RelIndexCombined::new(
                        &_self.runtime_total.bar_indices_0.to_rel_index(&_self.runtime_total.__bar_ind_common),
                        &_self.runtime_delta.bar_indices_0.to_rel_index(&_self.runtime_delta.__bar_ind_common),
                     )
                     .index_get(&(y.clone(),))
                     {
                        __matching.for_each(|__val| {
                           let mut __dep_changed = false;
                           let mut __default_id = 0;
                           let __val = __val.tuple_of_borrowed();
                           let z: &i32 = __val.0;
                           if *z * x != 444 {
                              if x != z {
                                 let __new_row: (i32, i32) = (*x, *z);
                                 let mut __new_baz = 0;
                                 if !::ascent::internal::RelFullIndexRead::contains_key(
                                    &_self
                                       .runtime_total
                                       .baz_indices_0_1
                                       .to_rel_index(&_self.runtime_total.__baz_ind_common),
                                    &__new_row,
                                 ) && !::ascent::internal::RelFullIndexRead::contains_key(
                                    &_self
                                       .runtime_delta
                                       .baz_indices_0_1
                                       .to_rel_index(&_self.runtime_delta.__baz_ind_common),
                                    &__new_row,
                                 ) {
                                    if ::ascent::internal::RelFullIndexWrite::insert_if_not_present(
                                       &mut _self
                                          .runtime_new
                                          .baz_indices_0_1
                                          .to_rel_index_write(&mut _self.runtime_new.__baz_ind_common),
                                       &__new_row,
                                       (),
                                    ) {
                                       __new_baz = _self.baz.len();
                                       _self.baz.push((__new_row.0.clone(), __new_row.1.clone()));
                                       __default_id = __new_baz;
                                       ::ascent::internal::RelIndexWrite::index_insert(
                                          &mut _self
                                             .runtime_new
                                             .baz_indices_none
                                             .to_rel_index_write(&mut _self.runtime_new.__baz_ind_common),
                                          (),
                                          (__new_row.0.clone(), __new_row.1.clone()),
                                       );
                                       __changed = true;
                                    } else {
                                    }
                                 } else {
                                 }
                              }
                           }
                        });
                     }
                  }
               });
            }
         }
         ascent::internal::comment("baz <-- foo_indices_none_total, bar_indices_0_delta, if ⋯");
         if _self.runtime_delta.bar_indices_0.to_rel_index(&_self.runtime_delta.__bar_ind_common).len() > 0 {
            if let Some(__matching) = _self
               .runtime_total
               .foo_indices_none
               .to_rel_index(&_self.runtime_total.__foo_ind_common)
               .index_get(&())
            {
               __matching.for_each(|__val| {
                  let mut __dep_changed = false;
                  let mut __default_id = 0;
                  let __val = __val.tuple_of_borrowed();
                  let x: &i32 = __val.0;
                  let y: &i32 = __val.1;
                  if *x != 10 {
                     if let Some(__matching) = _self
                        .runtime_delta
                        .bar_indices_0
                        .to_rel_index(&_self.runtime_delta.__bar_ind_common)
                        .index_get(&(y.clone(),))
                     {
                        __matching.for_each(|__val| {
                           let mut __dep_changed = false;
                           let mut __default_id = 0;
                           let __val = __val.tuple_of_borrowed();
                           let z: &i32 = __val.0;
                           if *z * x != 444 {
                              if x != z {
                                 let __new_row: (i32, i32) = (*x, *z);
                                 let mut __new_baz = 0;
                                 if !::ascent::internal::RelFullIndexRead::contains_key(
                                    &_self
                                       .runtime_total
                                       .baz_indices_0_1
                                       .to_rel_index(&_self.runtime_total.__baz_ind_common),
                                    &__new_row,
                                 ) && !::ascent::internal::RelFullIndexRead::contains_key(
                                    &_self
                                       .runtime_delta
                                       .baz_indices_0_1
                                       .to_rel_index(&_self.runtime_delta.__baz_ind_common),
                                    &__new_row,
                                 ) {
                                    if ::ascent::internal::RelFullIndexWrite::insert_if_not_present(
                                       &mut _self
                                          .runtime_new
                                          .baz_indices_0_1
                                          .to_rel_index_write(&mut _self.runtime_new.__baz_ind_common),
                                       &__new_row,
                                       (),
                                    ) {
                                       __new_baz = _self.baz.len();
                                       _self.baz.push((__new_row.0.clone(), __new_row.1.clone()));
                                       __default_id = __new_baz;
                                       ::ascent::internal::RelIndexWrite::index_insert(
                                          &mut _self
                                             .runtime_new
                                             .baz_indices_none
                                             .to_rel_index_write(&mut _self.runtime_new.__baz_ind_common),
                                          (),
                                          (__new_row.0.clone(), __new_row.1.clone()),
                                       );
                                       __changed = true;
                                    } else {
                                    }
                                 } else {
                                 }
                              }
                           }
                        });
                     }
                  }
               });
            }
         }
         ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
            &mut _self.runtime_new.__bar_ind_common,
            &mut _self.runtime_delta.__bar_ind_common,
            &mut _self.runtime_total.__bar_ind_common,
         );
         ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
            &mut _self.runtime_new.bar_indices_0.to_rel_index_write(&mut _self.runtime_new.__bar_ind_common),
            &mut _self.runtime_delta.bar_indices_0.to_rel_index_write(&mut _self.runtime_delta.__bar_ind_common),
            &mut _self.runtime_total.bar_indices_0.to_rel_index_write(&mut _self.runtime_total.__bar_ind_common),
         );
         ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
            &mut _self.runtime_new.bar_indices_0_1.to_rel_index_write(&mut _self.runtime_new.__bar_ind_common),
            &mut _self.runtime_delta.bar_indices_0_1.to_rel_index_write(&mut _self.runtime_delta.__bar_ind_common),
            &mut _self.runtime_total.bar_indices_0_1.to_rel_index_write(&mut _self.runtime_total.__bar_ind_common),
         );
         ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
            &mut _self.runtime_new.__baz_ind_common,
            &mut _self.runtime_delta.__baz_ind_common,
            &mut _self.runtime_total.__baz_ind_common,
         );
         ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
            &mut _self.runtime_new.baz_indices_0_1.to_rel_index_write(&mut _self.runtime_new.__baz_ind_common),
            &mut _self.runtime_delta.baz_indices_0_1.to_rel_index_write(&mut _self.runtime_delta.__baz_ind_common),
            &mut _self.runtime_total.baz_indices_0_1.to_rel_index_write(&mut _self.runtime_total.__baz_ind_common),
         );
         ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
            &mut _self.runtime_new.baz_indices_none.to_rel_index_write(&mut _self.runtime_new.__baz_ind_common),
            &mut _self.runtime_delta.baz_indices_none.to_rel_index_write(&mut _self.runtime_delta.__baz_ind_common),
            &mut _self.runtime_total.baz_indices_none.to_rel_index_write(&mut _self.runtime_total.__baz_ind_common),
         );
         ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
            &mut _self.runtime_new.__foo_ind_common,
            &mut _self.runtime_delta.__foo_ind_common,
            &mut _self.runtime_total.__foo_ind_common,
         );
         ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
            &mut _self.runtime_new.foo_indices_0_1.to_rel_index_write(&mut _self.runtime_new.__foo_ind_common),
            &mut _self.runtime_delta.foo_indices_0_1.to_rel_index_write(&mut _self.runtime_delta.__foo_ind_common),
            &mut _self.runtime_total.foo_indices_0_1.to_rel_index_write(&mut _self.runtime_total.__foo_ind_common),
         );
         ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
            &mut _self.runtime_new.foo_indices_1.to_rel_index_write(&mut _self.runtime_new.__foo_ind_common),
            &mut _self.runtime_delta.foo_indices_1.to_rel_index_write(&mut _self.runtime_delta.__foo_ind_common),
            &mut _self.runtime_total.foo_indices_1.to_rel_index_write(&mut _self.runtime_total.__foo_ind_common),
         );
         ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
            &mut _self.runtime_new.foo_indices_none.to_rel_index_write(&mut _self.runtime_new.__foo_ind_common),
            &mut _self.runtime_delta.foo_indices_none.to_rel_index_write(&mut _self.runtime_delta.__foo_ind_common),
            &mut _self.runtime_total.foo_indices_none.to_rel_index_write(&mut _self.runtime_total.__foo_ind_common),
         );
         _self.scc_iters[4usize] += 1;
         let need_break = !__changed;
         _self.scc_times[4usize] += _scc_start_time.elapsed();
         need_break
      }
      #[allow(unused_assignments, unused_variables, dead_code)]
      pub fn scc_4(&mut self) -> bool {
         ascent::internal::comment("scc 4");
         {
            macro_rules! __check_return_conditions {
               () => {};
            }
            let _self = self;
            use ascent::internal::RelIndexWrite;
            use ascent::internal::{RelIndexRead, RelIndexReadAll, ToRelIndex0, TupleOfBorrowed};
            use core::cmp::PartialEq;
            _self.runtime_delta.__bar_ind_common = ::std::mem::take(&mut _self.__bar_ind_common);
            _self.runtime_total.__bar_ind_common = Default::default();
            _self.runtime_new.__bar_ind_common = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut _self.runtime_new.__bar_ind_common,
               &mut _self.runtime_delta.__bar_ind_common,
               &mut _self.runtime_total.__bar_ind_common,
            );
            _self.runtime_delta.bar_indices_0 = ::std::mem::take(&mut _self.bar_indices_0);
            _self.runtime_total.bar_indices_0 = Default::default();
            _self.runtime_new.bar_indices_0 = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut _self.runtime_new.bar_indices_0.to_rel_index_write(&mut _self.runtime_new.__bar_ind_common),
               &mut _self.runtime_delta.bar_indices_0.to_rel_index_write(&mut _self.runtime_delta.__bar_ind_common),
               &mut _self.runtime_total.bar_indices_0.to_rel_index_write(&mut _self.runtime_total.__bar_ind_common),
            );
            _self.runtime_delta.bar_indices_0_1 = ::std::mem::take(&mut _self.bar_indices_0_1);
            _self.runtime_total.bar_indices_0_1 = Default::default();
            _self.runtime_new.bar_indices_0_1 = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut _self.runtime_new.bar_indices_0_1.to_rel_index_write(&mut _self.runtime_new.__bar_ind_common),
               &mut _self.runtime_delta.bar_indices_0_1.to_rel_index_write(&mut _self.runtime_delta.__bar_ind_common),
               &mut _self.runtime_total.bar_indices_0_1.to_rel_index_write(&mut _self.runtime_total.__bar_ind_common),
            );
            _self.runtime_delta.__baz_ind_common = ::std::mem::take(&mut _self.__baz_ind_common);
            _self.runtime_total.__baz_ind_common = Default::default();
            _self.runtime_new.__baz_ind_common = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut _self.runtime_new.__baz_ind_common,
               &mut _self.runtime_delta.__baz_ind_common,
               &mut _self.runtime_total.__baz_ind_common,
            );
            _self.runtime_delta.baz_indices_0_1 = ::std::mem::take(&mut _self.baz_indices_0_1);
            _self.runtime_total.baz_indices_0_1 = Default::default();
            _self.runtime_new.baz_indices_0_1 = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut _self.runtime_new.baz_indices_0_1.to_rel_index_write(&mut _self.runtime_new.__baz_ind_common),
               &mut _self.runtime_delta.baz_indices_0_1.to_rel_index_write(&mut _self.runtime_delta.__baz_ind_common),
               &mut _self.runtime_total.baz_indices_0_1.to_rel_index_write(&mut _self.runtime_total.__baz_ind_common),
            );
            _self.runtime_delta.baz_indices_none = ::std::mem::take(&mut _self.baz_indices_none);
            _self.runtime_total.baz_indices_none = Default::default();
            _self.runtime_new.baz_indices_none = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut _self.runtime_new.baz_indices_none.to_rel_index_write(&mut _self.runtime_new.__baz_ind_common),
               &mut _self.runtime_delta.baz_indices_none.to_rel_index_write(&mut _self.runtime_delta.__baz_ind_common),
               &mut _self.runtime_total.baz_indices_none.to_rel_index_write(&mut _self.runtime_total.__baz_ind_common),
            );
            _self.runtime_delta.__foo_ind_common = ::std::mem::take(&mut _self.__foo_ind_common);
            _self.runtime_total.__foo_ind_common = Default::default();
            _self.runtime_new.__foo_ind_common = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut _self.runtime_new.__foo_ind_common,
               &mut _self.runtime_delta.__foo_ind_common,
               &mut _self.runtime_total.__foo_ind_common,
            );
            _self.runtime_delta.foo_indices_0_1 = ::std::mem::take(&mut _self.foo_indices_0_1);
            _self.runtime_total.foo_indices_0_1 = Default::default();
            _self.runtime_new.foo_indices_0_1 = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut _self.runtime_new.foo_indices_0_1.to_rel_index_write(&mut _self.runtime_new.__foo_ind_common),
               &mut _self.runtime_delta.foo_indices_0_1.to_rel_index_write(&mut _self.runtime_delta.__foo_ind_common),
               &mut _self.runtime_total.foo_indices_0_1.to_rel_index_write(&mut _self.runtime_total.__foo_ind_common),
            );
            _self.runtime_delta.foo_indices_1 = ::std::mem::take(&mut _self.foo_indices_1);
            _self.runtime_total.foo_indices_1 = Default::default();
            _self.runtime_new.foo_indices_1 = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut _self.runtime_new.foo_indices_1.to_rel_index_write(&mut _self.runtime_new.__foo_ind_common),
               &mut _self.runtime_delta.foo_indices_1.to_rel_index_write(&mut _self.runtime_delta.__foo_ind_common),
               &mut _self.runtime_total.foo_indices_1.to_rel_index_write(&mut _self.runtime_total.__foo_ind_common),
            );
            _self.runtime_delta.foo_indices_none = ::std::mem::take(&mut _self.foo_indices_none);
            _self.runtime_total.foo_indices_none = Default::default();
            _self.runtime_new.foo_indices_none = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut _self.runtime_new.foo_indices_none.to_rel_index_write(&mut _self.runtime_new.__foo_ind_common),
               &mut _self.runtime_delta.foo_indices_none.to_rel_index_write(&mut _self.runtime_delta.__foo_ind_common),
               &mut _self.runtime_total.foo_indices_none.to_rel_index_write(&mut _self.runtime_total.__foo_ind_common),
            );
            loop {
               let need_brack = _self.scc_4_exec();
               if need_brack {
                  break;
               }
               __check_return_conditions!();
            }
            _self.__bar_ind_common = std::mem::take(&mut _self.runtime_total.__bar_ind_common);
            _self.bar_indices_0 = std::mem::take(&mut _self.runtime_total.bar_indices_0);
            _self.bar_indices_0_1 = std::mem::take(&mut _self.runtime_total.bar_indices_0_1);
            _self.__baz_ind_common = std::mem::take(&mut _self.runtime_total.__baz_ind_common);
            _self.baz_indices_0_1 = std::mem::take(&mut _self.runtime_total.baz_indices_0_1);
            _self.baz_indices_none = std::mem::take(&mut _self.runtime_total.baz_indices_none);
            _self.__foo_ind_common = std::mem::take(&mut _self.runtime_total.__foo_ind_common);
            _self.foo_indices_0_1 = std::mem::take(&mut _self.runtime_total.foo_indices_0_1);
            _self.foo_indices_1 = std::mem::take(&mut _self.runtime_total.foo_indices_1);
            _self.foo_indices_none = std::mem::take(&mut _self.runtime_total.foo_indices_none);
         }
         true
      }
      #[allow(unused_imports, noop_method_call, suspicious_double_ref_op)]
      #[doc = "Runs the Ascent program to a fixed point."]
      pub fn run(&mut self) -> bool { self.run_with_init_flag(true) }
      pub fn run_with_init_flag(&mut self, init_flag: bool) -> bool {
         if init_flag {
            self.update_indices_priv()
         };
         let _self = self;
         let res = _self.scc_0();
         if !res {
            return false;
         }
         let res = _self.scc_1();
         if !res {
            return false;
         }
         let res = _self.scc_2();
         if !res {
            return false;
         }
         let res = _self.scc_3();
         if !res {
            return false;
         }
         let res = _self.scc_4();
         if !res {
            return false;
         }
         true
      }
      #[allow(noop_method_call, suspicious_double_ref_op)]
      pub fn update_indices_priv(&mut self) {
         let before = ::ascent::internal::Instant::now();
         self.update_indices_bar();
         self.update_indices_baz();
         self.update_indices_foo();
         self.update_indices_duration += before.elapsed();
      }
      #[allow(noop_method_call, suspicious_double_ref_op)]
      pub fn update_indices_bar(&mut self) {
         use ascent::internal::RelIndexWrite;
         use ascent::internal::ToRelIndex0;
         for (_i, tuple) in self.bar.iter().enumerate() {
            let selection_tuple = (tuple.0.clone(),);
            let rel_ind = &mut self.bar_indices_0;
            ascent::internal::RelIndexWrite::index_insert(
               &mut rel_ind.to_rel_index_write(&mut self.__bar_ind_common),
               selection_tuple,
               (tuple.1.clone(),),
            );
            let selection_tuple = (tuple.0.clone(), tuple.1.clone());
            let rel_ind = &mut self.bar_indices_0_1;
            ascent::internal::RelIndexWrite::index_insert(
               &mut rel_ind.to_rel_index_write(&mut self.__bar_ind_common),
               selection_tuple,
               (),
            );
         }
      }
      #[allow(noop_method_call, suspicious_double_ref_op)]
      pub fn update_indices_baz(&mut self) {
         use ascent::internal::RelIndexWrite;
         use ascent::internal::ToRelIndex0;
         for (_i, tuple) in self.baz.iter().enumerate() {
            let selection_tuple = (tuple.0.clone(), tuple.1.clone());
            let rel_ind = &mut self.baz_indices_0_1;
            ascent::internal::RelIndexWrite::index_insert(
               &mut rel_ind.to_rel_index_write(&mut self.__baz_ind_common),
               selection_tuple,
               (),
            );
            let selection_tuple = ();
            let rel_ind = &mut self.baz_indices_none;
            ascent::internal::RelIndexWrite::index_insert(
               &mut rel_ind.to_rel_index_write(&mut self.__baz_ind_common),
               selection_tuple,
               (tuple.0.clone(), tuple.1.clone()),
            );
         }
      }
      #[allow(noop_method_call, suspicious_double_ref_op)]
      pub fn update_indices_foo(&mut self) {
         use ascent::internal::RelIndexWrite;
         use ascent::internal::ToRelIndex0;
         for (_i, tuple) in self.foo.iter().enumerate() {
            let selection_tuple = (tuple.0.clone(), tuple.1.clone());
            let rel_ind = &mut self.foo_indices_0_1;
            ascent::internal::RelIndexWrite::index_insert(
               &mut rel_ind.to_rel_index_write(&mut self.__foo_ind_common),
               selection_tuple,
               (),
            );
            let selection_tuple = (tuple.1.clone(),);
            let rel_ind = &mut self.foo_indices_1;
            ascent::internal::RelIndexWrite::index_insert(
               &mut rel_ind.to_rel_index_write(&mut self.__foo_ind_common),
               selection_tuple,
               (tuple.0.clone(),),
            );
            let selection_tuple = ();
            let rel_ind = &mut self.foo_indices_none;
            ascent::internal::RelIndexWrite::index_insert(
               &mut rel_ind.to_rel_index_write(&mut self.__foo_ind_common),
               selection_tuple,
               (tuple.0.clone(), tuple.1.clone()),
            );
         }
      }
      #[deprecated = "Explicit call to update_indices not required anymore."]
      pub fn update_indices(&mut self) { self.update_indices_priv(); }
      fn type_constraints() { let _type_constraints: ascent::internal::TypeConstraints<i32>; }
      pub fn summary() -> &'static str {
         "scc 0, is_looping: false:\n  foo <-- \n  dynamic relations: foo\nscc 1, is_looping: false:\n  foo <-- \n  dynamic relations: foo\nscc 2, is_looping: false:\n  bar <-- \n  dynamic relations: bar\nscc 3, is_looping: false:\n  bar <-- \n  dynamic relations: bar\nscc 4, is_looping: true:\n  baz <-- foo_indices_1_delta, bar_indices_0_total+delta, if ⋯ [SIMPLE JOIN]\n  baz <-- foo_indices_1_total, bar_indices_0_delta, if ⋯ [SIMPLE JOIN]\n  foo, bar <-- baz_indices_none_delta\n  baz <-- foo_indices_none_delta, bar_indices_0_total+delta, if ⋯\n  baz <-- foo_indices_none_total, bar_indices_0_delta, if ⋯\n  dynamic relations: bar, baz, foo\n"
      }
      pub fn relation_sizes_summary(&self) -> String {
         use std::fmt::Write;
         let mut res = String::new();
         writeln!(&mut res, "{} size: {}", "bar", self.bar.len()).unwrap();
         writeln!(&mut res, "{} size: {}", "baz", self.baz.len()).unwrap();
         writeln!(&mut res, "{} size: {}", "foo", self.foo.len()).unwrap();
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
         writeln!(&mut res, "scc {}: iterations: {}, time: {:?}", "2", self.scc_iters[2usize], self.scc_times[2usize])
            .unwrap();
         writeln!(&mut res, "scc {}: iterations: {}, time: {:?}", "3", self.scc_iters[3usize], self.scc_times[3usize])
            .unwrap();
         writeln!(&mut res, "scc {}: iterations: {}, time: {:?}", "4", self.scc_iters[4usize], self.scc_times[4usize])
            .unwrap();
         res
      }
   }
   impl Default for AscentProgramRuntime {
      fn default() -> Self {
         let mut _self = AscentProgramRuntime {
            bar: Default::default(),
            __bar_ind_common: Default::default(),
            bar_indices_0: Default::default(),
            bar_indices_0_1: Default::default(),
            baz: Default::default(),
            __baz_ind_common: Default::default(),
            baz_indices_0_1: Default::default(),
            baz_indices_none: Default::default(),
            foo: Default::default(),
            __foo_ind_common: Default::default(),
            foo_indices_0_1: Default::default(),
            foo_indices_1: Default::default(),
            foo_indices_none: Default::default(),
         };
         _self
      }
   }
   impl Default for AscentProgram {
      fn default() -> Self {
         let mut _self = AscentProgram {
            bar: Default::default(),
            __bar_ind_common: Default::default(),
            bar_indices_0: Default::default(),
            bar_indices_0_1: Default::default(),
            baz: Default::default(),
            __baz_ind_common: Default::default(),
            baz_indices_0_1: Default::default(),
            baz_indices_none: Default::default(),
            foo: Default::default(),
            __foo_ind_common: Default::default(),
            foo_indices_0_1: Default::default(),
            foo_indices_1: Default::default(),
            foo_indices_none: Default::default(),
            scc_times: [std::time::Duration::ZERO; 5usize],
            scc_iters: [0; 5usize],
            update_time_nanos: Default::default(),
            update_indices_duration: std::time::Duration::default(),
            runtime_total: Default::default(),
            runtime_new: Default::default(),
            runtime_delta: Default::default(),
         };
         _self
      }
   };
}
