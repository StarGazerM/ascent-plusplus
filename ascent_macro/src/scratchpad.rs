#![allow(unused_imports)]
use std::fmt::Debug;
use std::hash::Hash;
use std::ops::Deref;
use std::{cell::RefCell, clone, cmp::max, rc::Rc};

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
   ::ascent::rel::rel_codegen! { AscentProgram_foo , (i32 , i32) , [[0 , 1]] , ser , () }
   ::ascent::rel::rel_codegen! { AscentProgram_bar_id , (i32 , i32 , usize) , [[0 , 1] , [0 , 1 , 2]] , ser , () }
   ::ascent::rel::rel_codegen! { AscentProgram_foobar , (i32 , usize) , [[0 , 1]] , ser , () }
   ::ascent::rel::rel_codegen! { AscentProgram_bar , (i32 , i32) , [[0 , 1]] , ser , () }
   pub struct AscentProgram {
      pub bar: ::ascent::rel::rel!(AscentProgram_bar, (i32, i32), [[0, 1]], ser, ()),
      pub bar_id: ::ascent::rel::rel!(AscentProgram_bar_id, (i32, i32, usize), [[0, 1], [0, 1, 2]], ser, ()),
      pub foo: ::ascent::rel::rel!(AscentProgram_foo, (i32, i32), [[0, 1]], ser, ()),
      pub foobar: ::ascent::rel::rel!(AscentProgram_foobar, (i32, usize), [[0, 1]], ser, ()),
      scc_times: [std::time::Duration; 2usize],
      scc_iters: [usize; 2usize],
      pub update_time_nanos: std::sync::atomic::AtomicU64,
      pub update_indices_duration: std::time::Duration,
      pub runtime_total: AscentProgramRuntime,
      pub runtime_new: AscentProgramRuntime,
      pub runtime_delta: AscentProgramRuntime,
   }
   pub struct AscentProgramRuntime {
      #[doc = "\nlogical indices: bar_indices_0_1"]
      pub bar: ::ascent::rel::rel!(AscentProgram_bar, (i32, i32), [[0, 1]], ser, ()),
      pub __bar_ind_common: ::ascent::rel::rel_ind_common!(AscentProgram_bar, (i32, i32), [[0, 1]], ser, ()),
      pub bar_indices_0_1:
         ::ascent::rel::rel_full_ind!(AscentProgram_bar, (i32, i32), [[0, 1]], ser, (), (i32, i32), ()),
      #[doc = "\nlogical indices: bar_id_indices_0_1; bar_id_indices_0_1_2"]
      pub bar_id: ::ascent::rel::rel!(AscentProgram_bar_id, (i32, i32, usize), [[0, 1], [0, 1, 2]], ser, ()),
      pub __bar_id_ind_common:
         ::ascent::rel::rel_ind_common!(AscentProgram_bar_id, (i32, i32, usize), [[0, 1], [0, 1, 2]], ser, ()),
      pub bar_id_indices_0_1: ::ascent::rel::rel_ind!(
         AscentProgram_bar_id,
         (i32, i32, usize),
         [[0, 1], [0, 1, 2]],
         ser,
         (),
         [0, 1],
         (i32, i32),
         (usize,)
      ),
      pub bar_id_indices_0_1_2: ::ascent::rel::rel_full_ind!(
         AscentProgram_bar_id,
         (i32, i32, usize),
         [[0, 1], [0, 1, 2]],
         ser,
         (),
         (i32, i32, usize),
         ()
      ),
      #[doc = "\nlogical indices: foo_indices_0_1"]
      pub foo: ::ascent::rel::rel!(AscentProgram_foo, (i32, i32), [[0, 1]], ser, ()),
      pub __foo_ind_common: ::ascent::rel::rel_ind_common!(AscentProgram_foo, (i32, i32), [[0, 1]], ser, ()),
      pub foo_indices_0_1:
         ::ascent::rel::rel_full_ind!(AscentProgram_foo, (i32, i32), [[0, 1]], ser, (), (i32, i32), ()),
      #[doc = "\nlogical indices: foobar_indices_0_1"]
      pub foobar: ::ascent::rel::rel!(AscentProgram_foobar, (i32, usize), [[0, 1]], ser, ()),
      pub __foobar_ind_common: ::ascent::rel::rel_ind_common!(AscentProgram_foobar, (i32, usize), [[0, 1]], ser, ()),
      pub foobar_indices_0_1:
         ::ascent::rel::rel_full_ind!(AscentProgram_foobar, (i32, usize), [[0, 1]], ser, (), (i32, usize), ()),
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
                  let __new_row_to_be_pushed = (__new_row.0, __new_row.1);
                  __new_foo = _self.foo.len();
                  _self.foo.push(__new_row_to_be_pushed);
                  __default_id = __new_foo;
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
            &mut _self.runtime_new.__foo_ind_common,
            &mut _self.runtime_delta.__foo_ind_common,
            &mut _self.runtime_total.__foo_ind_common,
         );
         ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
            &mut _self.runtime_new.foo_indices_0_1.to_rel_index_write(&mut _self.runtime_new.__foo_ind_common),
            &mut _self.runtime_delta.foo_indices_0_1.to_rel_index_write(&mut _self.runtime_delta.__foo_ind_common),
            &mut _self.runtime_total.foo_indices_0_1.to_rel_index_write(&mut _self.runtime_total.__foo_ind_common),
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
            _self.runtime_delta.__foo_ind_common = ::std::mem::take(&mut _self.runtime_total.__foo_ind_common);
            _self.runtime_total.__foo_ind_common = Default::default();
            _self.runtime_new.__foo_ind_common = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut _self.runtime_new.__foo_ind_common,
               &mut _self.runtime_delta.__foo_ind_common,
               &mut _self.runtime_total.__foo_ind_common,
            );
            _self.runtime_delta.foo_indices_0_1 = ::std::mem::take(&mut _self.runtime_total.foo_indices_0_1);
            _self.runtime_total.foo_indices_0_1 = Default::default();
            _self.runtime_new.foo_indices_0_1 = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut _self.runtime_new.foo_indices_0_1.to_rel_index_write(&mut _self.runtime_new.__foo_ind_common),
               &mut _self.runtime_delta.foo_indices_0_1.to_rel_index_write(&mut _self.runtime_delta.__foo_ind_common),
               &mut _self.runtime_total.foo_indices_0_1.to_rel_index_write(&mut _self.runtime_total.__foo_ind_common),
            );
            _self.scc_0_exec();
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
         ascent::internal::comment("bar, foobar <-- foo_indices_0_1_total, foo_indices_0_1_total [SIMPLE JOIN]");
         if true {
            if _self.runtime_total.foo_indices_0_1.to_rel_index(&_self.runtime_total.__foo_ind_common).len()
               <= _self.runtime_total.foo_indices_0_1.to_rel_index(&_self.runtime_total.__foo_ind_common).len()
            {
               _self
                  .runtime_total
                  .foo_indices_0_1
                  .to_rel_index(&_self.runtime_total.__foo_ind_common)
                  .iter_all()
                  .for_each(|(__cl1_joined_columns, __cl1_tuple_indices)| {
                     let mut __gen_bang = false;
                     let __cl1_joined_columns = __cl1_joined_columns.tuple_of_borrowed();
                     let x = __cl1_joined_columns.0;
                     let y = __cl1_joined_columns.1;
                     if let Some(__matching) = _self
                        .runtime_total
                        .foo_indices_0_1
                        .to_rel_index(&_self.runtime_total.__foo_ind_common)
                        .index_get(&(y.clone(), x.clone()))
                     {
                        __cl1_tuple_indices.for_each(|cl1_val| {
                           __matching.clone().for_each(|__val| {
                              let mut __dep_changed = false;
                              let mut __default_id = 0;
                              let __new_row: (i32, i32) =
                                 (ascent::internal::Convert::convert(x), ascent::internal::Convert::convert(y));
                              let mut new_bar = 0;
                              if !::ascent::internal::RelFullIndexRead::contains_key(
                                 &_self
                                    .runtime_total
                                    .bar_indices_0_1
                                    .to_rel_index(&_self.runtime_total.__bar_ind_common),
                                 &__new_row,
                              ) && !::ascent::internal::RelFullIndexRead::contains_key(
                                 &_self
                                    .runtime_delta
                                    .bar_indices_0_1
                                    .to_rel_index(&_self.runtime_delta.__bar_ind_common),
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
                                    let __new_row_to_be_pushed = (__new_row.0, __new_row.1);
                                    _self.bar.push(__new_row_to_be_pushed);
                                    new_bar = {
                                       use std::hash::Hasher;
                                       let mut hasher = ::std::hash::DefaultHasher::new();
                                       (__new_row.0, __new_row.1).hash(&mut hasher);
                                       hasher.finish() as usize
                                    };
                                    __default_id = new_bar;
                                    __changed = true;
                                    let __new_row_id = (__new_row.0.clone(), __new_row.1.clone(), new_bar.clone());
                                    ::ascent::internal::RelFullIndexWrite::insert_if_not_present(
                                       &mut _self
                                          .runtime_new
                                          .bar_id_indices_0_1_2
                                          .to_rel_index_write(&mut _self.runtime_new.__bar_id_ind_common),
                                       &__new_row_id,
                                       (),
                                    );
                                 } else {
                                 }
                              } else {
                              }
                              let __new_row: (i32, usize) =
                                 (ascent::internal::Convert::convert(x), ascent::internal::Convert::convert(new_bar));
                              let mut __new_foobar = 0;
                              if !::ascent::internal::RelFullIndexRead::contains_key(
                                 &_self
                                    .runtime_total
                                    .foobar_indices_0_1
                                    .to_rel_index(&_self.runtime_total.__foobar_ind_common),
                                 &__new_row,
                              ) && !::ascent::internal::RelFullIndexRead::contains_key(
                                 &_self
                                    .runtime_delta
                                    .foobar_indices_0_1
                                    .to_rel_index(&_self.runtime_delta.__foobar_ind_common),
                                 &__new_row,
                              ) {
                                 if ::ascent::internal::RelFullIndexWrite::insert_if_not_present(
                                    &mut _self
                                       .runtime_new
                                       .foobar_indices_0_1
                                       .to_rel_index_write(&mut _self.runtime_new.__foobar_ind_common),
                                    &__new_row,
                                    (),
                                 ) {
                                    let __new_row_to_be_pushed = (__new_row.0, __new_row.1);
                                    __new_foobar = _self.foobar.len();
                                    _self.foobar.push(__new_row_to_be_pushed);
                                    __default_id = __new_foobar;
                                    __changed = true;
                                 } else {
                                 }
                              } else {
                              }
                           });
                        });
                     }
                  });
            } else {
               _self
                  .runtime_total
                  .foo_indices_0_1
                  .to_rel_index(&_self.runtime_total.__foo_ind_common)
                  .iter_all()
                  .for_each(|(__cl1_joined_columns, __cl1_tuple_indices)| {
                     let mut __gen_bang = false;
                     let __cl1_joined_columns = __cl1_joined_columns.tuple_of_borrowed();
                     let y = __cl1_joined_columns.0;
                     let x = __cl1_joined_columns.1;
                     if let Some(__matching) = _self
                        .runtime_total
                        .foo_indices_0_1
                        .to_rel_index(&_self.runtime_total.__foo_ind_common)
                        .index_get(&(x.clone(), y.clone()))
                     {
                        __cl1_tuple_indices.for_each(|cl1_val| {
                           __matching.clone().for_each(|__val| {
                              let mut __dep_changed = false;
                              let mut __default_id = 0;
                              let __new_row: (i32, i32) =
                                 (ascent::internal::Convert::convert(x), ascent::internal::Convert::convert(y));
                              let mut new_bar = 0;
                              if !::ascent::internal::RelFullIndexRead::contains_key(
                                 &_self
                                    .runtime_total
                                    .bar_indices_0_1
                                    .to_rel_index(&_self.runtime_total.__bar_ind_common),
                                 &__new_row,
                              ) && !::ascent::internal::RelFullIndexRead::contains_key(
                                 &_self
                                    .runtime_delta
                                    .bar_indices_0_1
                                    .to_rel_index(&_self.runtime_delta.__bar_ind_common),
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
                                    let __new_row_to_be_pushed = (__new_row.0, __new_row.1);
                                    _self.bar.push(__new_row_to_be_pushed);
                                    new_bar = {
                                       use std::hash::Hasher;
                                       let mut hasher = ::std::hash::DefaultHasher::new();
                                       (__new_row.0, __new_row.1).hash(&mut hasher);
                                       hasher.finish() as usize
                                    };
                                    __default_id = new_bar;
                                    __changed = true;
                                    let __new_row_id = (__new_row.0.clone(), __new_row.1.clone(), new_bar.clone());
                                    ::ascent::internal::RelFullIndexWrite::insert_if_not_present(
                                       &mut _self
                                          .runtime_new
                                          .bar_id_indices_0_1_2
                                          .to_rel_index_write(&mut _self.runtime_new.__bar_id_ind_common),
                                       &__new_row_id,
                                       (),
                                    );
                                 } else {
                                 }
                              } else {
                              }
                              let __new_row: (i32, usize) =
                                 (ascent::internal::Convert::convert(x), ascent::internal::Convert::convert(new_bar));
                              let mut __new_foobar = 0;
                              if !::ascent::internal::RelFullIndexRead::contains_key(
                                 &_self
                                    .runtime_total
                                    .foobar_indices_0_1
                                    .to_rel_index(&_self.runtime_total.__foobar_ind_common),
                                 &__new_row,
                              ) && !::ascent::internal::RelFullIndexRead::contains_key(
                                 &_self
                                    .runtime_delta
                                    .foobar_indices_0_1
                                    .to_rel_index(&_self.runtime_delta.__foobar_ind_common),
                                 &__new_row,
                              ) {
                                 if ::ascent::internal::RelFullIndexWrite::insert_if_not_present(
                                    &mut _self
                                       .runtime_new
                                       .foobar_indices_0_1
                                       .to_rel_index_write(&mut _self.runtime_new.__foobar_ind_common),
                                    &__new_row,
                                    (),
                                 ) {
                                    let __new_row_to_be_pushed = (__new_row.0, __new_row.1);
                                    __new_foobar = _self.foobar.len();
                                    _self.foobar.push(__new_row_to_be_pushed);
                                    __default_id = __new_foobar;
                                    __changed = true;
                                 } else {
                                 }
                              } else {
                              }
                           });
                        });
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
            &mut _self.runtime_new.bar_indices_0_1.to_rel_index_write(&mut _self.runtime_new.__bar_ind_common),
            &mut _self.runtime_delta.bar_indices_0_1.to_rel_index_write(&mut _self.runtime_delta.__bar_ind_common),
            &mut _self.runtime_total.bar_indices_0_1.to_rel_index_write(&mut _self.runtime_total.__bar_ind_common),
         );
         ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
            &mut _self.runtime_new.__foobar_ind_common,
            &mut _self.runtime_delta.__foobar_ind_common,
            &mut _self.runtime_total.__foobar_ind_common,
         );
         ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
            &mut _self.runtime_new.foobar_indices_0_1.to_rel_index_write(&mut _self.runtime_new.__foobar_ind_common),
            &mut _self
               .runtime_delta
               .foobar_indices_0_1
               .to_rel_index_write(&mut _self.runtime_delta.__foobar_ind_common),
            &mut _self
               .runtime_total
               .foobar_indices_0_1
               .to_rel_index_write(&mut _self.runtime_total.__foobar_ind_common),
         );
         ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
            &mut _self.runtime_new.__bar_ind_common,
            &mut _self.runtime_delta.__bar_ind_common,
            &mut _self.runtime_total.__bar_ind_common,
         );
         ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
            &mut _self.runtime_new.bar_indices_0_1.to_rel_index_write(&mut _self.runtime_new.__bar_ind_common),
            &mut _self.runtime_delta.bar_indices_0_1.to_rel_index_write(&mut _self.runtime_delta.__bar_ind_common),
            &mut _self.runtime_total.bar_indices_0_1.to_rel_index_write(&mut _self.runtime_total.__bar_ind_common),
         );
         ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
            &mut _self.runtime_new.__foobar_ind_common,
            &mut _self.runtime_delta.__foobar_ind_common,
            &mut _self.runtime_total.__foobar_ind_common,
         );
         ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
            &mut _self.runtime_new.foobar_indices_0_1.to_rel_index_write(&mut _self.runtime_new.__foobar_ind_common),
            &mut _self
               .runtime_delta
               .foobar_indices_0_1
               .to_rel_index_write(&mut _self.runtime_delta.__foobar_ind_common),
            &mut _self
               .runtime_total
               .foobar_indices_0_1
               .to_rel_index_write(&mut _self.runtime_total.__foobar_ind_common),
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
            _self.runtime_delta.__bar_ind_common = ::std::mem::take(&mut _self.runtime_total.__bar_ind_common);
            _self.runtime_total.__bar_ind_common = Default::default();
            _self.runtime_new.__bar_ind_common = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut _self.runtime_new.__bar_ind_common,
               &mut _self.runtime_delta.__bar_ind_common,
               &mut _self.runtime_total.__bar_ind_common,
            );
            _self.runtime_delta.bar_indices_0_1 = ::std::mem::take(&mut _self.runtime_total.bar_indices_0_1);
            _self.runtime_total.bar_indices_0_1 = Default::default();
            _self.runtime_new.bar_indices_0_1 = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut _self.runtime_new.bar_indices_0_1.to_rel_index_write(&mut _self.runtime_new.__bar_ind_common),
               &mut _self.runtime_delta.bar_indices_0_1.to_rel_index_write(&mut _self.runtime_delta.__bar_ind_common),
               &mut _self.runtime_total.bar_indices_0_1.to_rel_index_write(&mut _self.runtime_total.__bar_ind_common),
            );
            _self.runtime_delta.__foobar_ind_common = ::std::mem::take(&mut _self.runtime_total.__foobar_ind_common);
            _self.runtime_total.__foobar_ind_common = Default::default();
            _self.runtime_new.__foobar_ind_common = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut _self.runtime_new.__foobar_ind_common,
               &mut _self.runtime_delta.__foobar_ind_common,
               &mut _self.runtime_total.__foobar_ind_common,
            );
            _self.runtime_delta.foobar_indices_0_1 = ::std::mem::take(&mut _self.runtime_total.foobar_indices_0_1);
            _self.runtime_total.foobar_indices_0_1 = Default::default();
            _self.runtime_new.foobar_indices_0_1 = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut _self.runtime_new.foobar_indices_0_1.to_rel_index_write(&mut _self.runtime_new.__foobar_ind_common),
               &mut _self
                  .runtime_delta
                  .foobar_indices_0_1
                  .to_rel_index_write(&mut _self.runtime_delta.__foobar_ind_common),
               &mut _self
                  .runtime_total
                  .foobar_indices_0_1
                  .to_rel_index_write(&mut _self.runtime_total.__foobar_ind_common),
            );
            _self.scc_1_exec();
         }
         true
      }
      #[allow(unused_imports, noop_method_call, suspicious_double_ref_op)]
      #[doc = "Runs the Ascent program to a fixed point."]
      pub fn run(&mut self) -> bool { self.run_with_init_flag(true) }
      pub fn run_with_init_flag(&mut self, init_flag: bool) -> bool {
         let _self = self;
         if init_flag {
            _self.update_indices_priv()
         };
         let res = _self.scc_0();
         if !res {
            return false;
         }
         let res = _self.scc_1();
         if !res {
            return false;
         }
         true
      }
      #[allow(noop_method_call, suspicious_double_ref_op)]
      pub fn update_indices_priv(&mut self) {
         let before = ::ascent::internal::Instant::now();
         self.update_indices_bar();
         self.update_indices_bar_id();
         self.update_indices_foo();
         self.update_indices_foobar();
         self.update_indices_duration += before.elapsed();
      }
      #[allow(noop_method_call, suspicious_double_ref_op)]
      pub fn update_indices_bar(&mut self) {
         use ascent::internal::RelIndexWrite;
         use ascent::internal::ToRelIndex0;
         for (_i, tuple) in self.bar.iter().enumerate() {
            let selection_tuple = (tuple.0.clone(), tuple.1.clone());
            let rel_ind = &mut self.runtime_total.bar_indices_0_1;
            ascent::internal::RelIndexWrite::index_insert(
               &mut rel_ind.to_rel_index_write(&mut self.runtime_total.__bar_ind_common),
               selection_tuple,
               (),
            );
         }
      }
      #[allow(noop_method_call, suspicious_double_ref_op)]
      pub fn update_indices_bar_id(&mut self) {
         use ascent::internal::RelIndexWrite;
         use ascent::internal::ToRelIndex0;
         for (_i, tuple) in self.bar_id.iter().enumerate() {
            let selection_tuple = (tuple.0.clone(), tuple.1.clone());
            let rel_ind = &mut self.runtime_total.bar_id_indices_0_1;
            ascent::internal::RelIndexWrite::index_insert(
               &mut rel_ind.to_rel_index_write(&mut self.runtime_total.__bar_id_ind_common),
               selection_tuple,
               (tuple.2.clone(),),
            );
            let selection_tuple = (tuple.0.clone(), tuple.1.clone(), tuple.2.clone());
            let rel_ind = &mut self.runtime_total.bar_id_indices_0_1_2;
            ascent::internal::RelIndexWrite::index_insert(
               &mut rel_ind.to_rel_index_write(&mut self.runtime_total.__bar_id_ind_common),
               selection_tuple,
               (),
            );
         }
      }
      #[allow(noop_method_call, suspicious_double_ref_op)]
      pub fn update_indices_foo(&mut self) {
         use ascent::internal::RelIndexWrite;
         use ascent::internal::ToRelIndex0;
         for (_i, tuple) in self.foo.iter().enumerate() {
            let selection_tuple = (tuple.0.clone(), tuple.1.clone());
            let rel_ind = &mut self.runtime_total.foo_indices_0_1;
            ascent::internal::RelIndexWrite::index_insert(
               &mut rel_ind.to_rel_index_write(&mut self.runtime_total.__foo_ind_common),
               selection_tuple,
               (),
            );
         }
      }
      #[allow(noop_method_call, suspicious_double_ref_op)]
      pub fn update_indices_foobar(&mut self) {
         use ascent::internal::RelIndexWrite;
         use ascent::internal::ToRelIndex0;
         for (_i, tuple) in self.foobar.iter().enumerate() {
            let selection_tuple = (tuple.0.clone(), tuple.1.clone());
            let rel_ind = &mut self.runtime_total.foobar_indices_0_1;
            ascent::internal::RelIndexWrite::index_insert(
               &mut rel_ind.to_rel_index_write(&mut self.runtime_total.__foobar_ind_common),
               selection_tuple,
               (),
            );
         }
      }
      #[deprecated = "Explicit call to update_indices not required anymore."]
      pub fn update_indices(&mut self) { self.update_indices_priv(); }
      fn type_constraints() {
         let _type_constraints: ascent::internal::TypeConstraints<i32>;
         let _type_constraints: ascent::internal::TypeConstraints<usize>;
         let _type_constraints: ascent::internal::TypeConstraints<usize>;
      }
      pub fn summary() -> &'static str {
         "scc 0, is_looping: false:\n  foo <-- \n  dynamic relations: foo\nscc 1, is_looping: false:\n  bar, foobar <-- foo_indices_0_1_total, foo_indices_0_1_total [SIMPLE JOIN]\n  dynamic relations: bar, foobar\n"
      }
      pub fn relation_sizes_summary(&self) -> String {
         use std::fmt::Write;
         let mut res = String::new();
         writeln!(&mut res, "{} size: {}", "bar", self.bar.len()).unwrap();
         writeln!(&mut res, "{} size: {}", "bar_id", self.bar_id.len()).unwrap();
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
   impl Default for AscentProgramRuntime {
      fn default() -> Self {
         let mut _self = AscentProgramRuntime {
            bar: Default::default(),
            __bar_ind_common: Default::default(),
            bar_indices_0_1: Default::default(),
            bar_id: Default::default(),
            __bar_id_ind_common: Default::default(),
            bar_id_indices_0_1: Default::default(),
            bar_id_indices_0_1_2: Default::default(),
            foo: Default::default(),
            __foo_ind_common: Default::default(),
            foo_indices_0_1: Default::default(),
            foobar: Default::default(),
            __foobar_ind_common: Default::default(),
            foobar_indices_0_1: Default::default(),
         };
         _self
      }
   }
   impl Default for AscentProgram {
      fn default() -> Self {
         let mut _self = AscentProgram {
            bar: Default::default(),
            bar_id: Default::default(),
            foo: Default::default(),
            foobar: Default::default(),
            scc_times: [std::time::Duration::ZERO; 2usize],
            scc_iters: [0; 2usize],
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
