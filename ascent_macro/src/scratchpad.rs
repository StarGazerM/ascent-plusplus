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
   ::ascent::rel::rel_codegen! { AscentProgram_b , (i32 , i32) , [[0] , [0 , 1]] , ser , () }
   ::ascent::rel::rel_codegen! { AscentProgram_a , (i32 , i32) , [[0 , 1] , [1]] , ser , () }
   ::ascent::rel::rel_codegen! { AscentProgram_c , (i32 , i32) , [[0] , [0 , 1]] , ser , () }
   pub struct AscentProgram {
      pub a: ::ascent::rel::rel!(AscentProgram_a, (i32, i32), [[0, 1], [1]], ser, ()),
      pub b: ::ascent::rel::rel!(AscentProgram_b, (i32, i32), [[0], [0, 1]], ser, ()),
      pub c: ::ascent::rel::rel!(AscentProgram_c, (i32, i32), [[0], [0, 1]], ser, ()),
      scc_times: [std::time::Duration; 1usize],
      scc_iters: [usize; 1usize],
      pub update_time_nanos: std::sync::atomic::AtomicU64,
      pub update_indices_duration: std::time::Duration,
      pub runtime_total: AscentProgramRuntime,
      pub runtime_new: AscentProgramRuntime,
      pub runtime_delta: AscentProgramRuntime,
   }
   pub struct AscentProgramRuntime {
      #[doc = "\nlogical indices: a_indices_0_1; a_indices_1"]
      pub a: ::ascent::rel::rel!(AscentProgram_a, (i32, i32), [[0, 1], [1]], ser, ()),
      pub __a_ind_common: ::ascent::rel::rel_ind_common!(AscentProgram_a, (i32, i32), [[0, 1], [1]], ser, ()),
      pub a_indices_0_1:
         ::ascent::rel::rel_full_ind!(AscentProgram_a, (i32, i32), [[0, 1], [1]], ser, (), (i32, i32), ()),
      pub a_indices_1:
         ::ascent::rel::rel_ind!(AscentProgram_a, (i32, i32), [[0, 1], [1]], ser, (), [1], (i32,), (i32,)),
      #[doc = "\nlogical indices: b_indices_0; b_indices_0_1"]
      pub b: ::ascent::rel::rel!(AscentProgram_b, (i32, i32), [[0], [0, 1]], ser, ()),
      pub __b_ind_common: ::ascent::rel::rel_ind_common!(AscentProgram_b, (i32, i32), [[0], [0, 1]], ser, ()),
      pub b_indices_0:
         ::ascent::rel::rel_ind!(AscentProgram_b, (i32, i32), [[0], [0, 1]], ser, (), [0], (i32,), (i32,)),
      pub b_indices_0_1:
         ::ascent::rel::rel_full_ind!(AscentProgram_b, (i32, i32), [[0], [0, 1]], ser, (), (i32, i32), ()),
      #[doc = "\nlogical indices: c_indices_0; c_indices_0_1"]
      pub c: ::ascent::rel::rel!(AscentProgram_c, (i32, i32), [[0], [0, 1]], ser, ()),
      pub __c_ind_common: ::ascent::rel::rel_ind_common!(AscentProgram_c, (i32, i32), [[0], [0, 1]], ser, ()),
      pub c_indices_0:
         ::ascent::rel::rel_ind!(AscentProgram_c, (i32, i32), [[0], [0, 1]], ser, (), [0], (i32,), (i32,)),
      pub c_indices_0_1:
         ::ascent::rel::rel_full_ind!(AscentProgram_c, (i32, i32), [[0], [0, 1]], ser, (), (i32, i32), ()),
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
         ascent::internal::comment(
            "a, b, c <-- a_indices_1_delta, b_indices_0_total+delta, c_indices_0_total+delta [SIMPLE JOIN]",
         );
         if _self.runtime_delta.a_indices_1.to_rel_index(&_self.runtime_delta.__a_ind_common).len() > 0 {
            if _self.runtime_delta.a_indices_1.to_rel_index(&_self.runtime_delta.__a_ind_common).len()
               <= ascent::internal::RelIndexCombined::new(
                  &_self.runtime_total.b_indices_0.to_rel_index(&_self.runtime_total.__b_ind_common),
                  &_self.runtime_delta.b_indices_0.to_rel_index(&_self.runtime_delta.__b_ind_common),
               )
               .len()
            {
               _self
                  .runtime_delta
                  .a_indices_1
                  .to_rel_index(&_self.runtime_delta.__a_ind_common)
                  .iter_all()
                  .for_each(|(__cl1_joined_columns, __cl1_tuple_indices)| {
                     let __cl1_joined_columns = __cl1_joined_columns.tuple_of_borrowed();
                     let y = __cl1_joined_columns.0;
                     if let Some(__matching) = ascent::internal::RelIndexCombined::new(
                        &_self.runtime_total.b_indices_0.to_rel_index(&_self.runtime_total.__b_ind_common),
                        &_self.runtime_delta.b_indices_0.to_rel_index(&_self.runtime_delta.__b_ind_common),
                     )
                     .index_get(&(y.clone(),))
                     {
                        __cl1_tuple_indices.for_each(|cl1_val| {
                           let cl1_val = cl1_val.tuple_of_borrowed();
                           let x: &i32 = cl1_val.0;
                           __matching.clone().for_each(|__val| {
                              let mut __dep_changed = false;
                              let mut __default_id = 0;
                              let __val = __val.tuple_of_borrowed();
                              let z: &i32 = __val.0;
                              if let Some(__matching) = ascent::internal::RelIndexCombined::new(
                                 &_self.runtime_total.c_indices_0.to_rel_index(&_self.runtime_total.__c_ind_common),
                                 &_self.runtime_delta.c_indices_0.to_rel_index(&_self.runtime_delta.__c_ind_common),
                              )
                              .index_get(&(z.clone(),))
                              {
                                 __matching.for_each(|__val| {
                                    let mut __dep_changed = false;
                                    let mut __default_id = 0;
                                    let __val = __val.tuple_of_borrowed();
                                    let w: &i32 = __val.0;
                                    let __new_row: (i32, i32) =
                                       (ascent::internal::Convert::convert(y), ascent::internal::Convert::convert(z));
                                    let mut __new_a = 0;
                                    if !::ascent::internal::RelFullIndexRead::contains_key(
                                       &_self
                                          .runtime_total
                                          .a_indices_0_1
                                          .to_rel_index(&_self.runtime_total.__a_ind_common),
                                       &__new_row,
                                    ) && !::ascent::internal::RelFullIndexRead::contains_key(
                                       &_self
                                          .runtime_delta
                                          .a_indices_0_1
                                          .to_rel_index(&_self.runtime_delta.__a_ind_common),
                                       &__new_row,
                                    ) {
                                       if ::ascent::internal::RelFullIndexWrite::insert_if_not_present(
                                          &mut _self
                                             .runtime_new
                                             .a_indices_0_1
                                             .to_rel_index_write(&mut _self.runtime_new.__a_ind_common),
                                          &__new_row,
                                          (),
                                       ) {
                                          __new_a = _self.a.len();
                                          _self.a.push((__new_row.0.clone(), __new_row.1.clone()));
                                          __default_id = __new_a;
                                          ::ascent::internal::RelIndexWrite::index_insert(
                                             &mut _self
                                                .runtime_new
                                                .a_indices_1
                                                .to_rel_index_write(&mut _self.runtime_new.__a_ind_common),
                                             (__new_row.1.clone(),),
                                             (__new_row.0.clone(),),
                                          );
                                          __changed = true;
                                       } else {
                                       }
                                    } else {
                                    }
                                    let __new_row: (i32, i32) =
                                       (ascent::internal::Convert::convert(z), ascent::internal::Convert::convert(w));
                                    let mut __new_b = 0;
                                    if !::ascent::internal::RelFullIndexRead::contains_key(
                                       &_self
                                          .runtime_total
                                          .b_indices_0_1
                                          .to_rel_index(&_self.runtime_total.__b_ind_common),
                                       &__new_row,
                                    ) && !::ascent::internal::RelFullIndexRead::contains_key(
                                       &_self
                                          .runtime_delta
                                          .b_indices_0_1
                                          .to_rel_index(&_self.runtime_delta.__b_ind_common),
                                       &__new_row,
                                    ) {
                                       if ::ascent::internal::RelFullIndexWrite::insert_if_not_present(
                                          &mut _self
                                             .runtime_new
                                             .b_indices_0_1
                                             .to_rel_index_write(&mut _self.runtime_new.__b_ind_common),
                                          &__new_row,
                                          (),
                                       ) {
                                          __new_b = _self.b.len();
                                          _self.b.push((__new_row.0.clone(), __new_row.1.clone()));
                                          __default_id = __new_b;
                                          ::ascent::internal::RelIndexWrite::index_insert(
                                             &mut _self
                                                .runtime_new
                                                .b_indices_0
                                                .to_rel_index_write(&mut _self.runtime_new.__b_ind_common),
                                             (__new_row.0.clone(),),
                                             (__new_row.1.clone(),),
                                          );
                                          __changed = true;
                                       } else {
                                       }
                                    } else {
                                    }
                                    let __new_row: (i32, i32) =
                                       (ascent::internal::Convert::convert(x), ascent::internal::Convert::convert(y));
                                    let mut __new_c = 0;
                                    if !::ascent::internal::RelFullIndexRead::contains_key(
                                       &_self
                                          .runtime_total
                                          .c_indices_0_1
                                          .to_rel_index(&_self.runtime_total.__c_ind_common),
                                       &__new_row,
                                    ) && !::ascent::internal::RelFullIndexRead::contains_key(
                                       &_self
                                          .runtime_delta
                                          .c_indices_0_1
                                          .to_rel_index(&_self.runtime_delta.__c_ind_common),
                                       &__new_row,
                                    ) {
                                       if ::ascent::internal::RelFullIndexWrite::insert_if_not_present(
                                          &mut _self
                                             .runtime_new
                                             .c_indices_0_1
                                             .to_rel_index_write(&mut _self.runtime_new.__c_ind_common),
                                          &__new_row,
                                          (),
                                       ) {
                                          __new_c = _self.c.len();
                                          _self.c.push((__new_row.0.clone(), __new_row.1.clone()));
                                          __default_id = __new_c;
                                          ::ascent::internal::RelIndexWrite::index_insert(
                                             &mut _self
                                                .runtime_new
                                                .c_indices_0
                                                .to_rel_index_write(&mut _self.runtime_new.__c_ind_common),
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
                           });
                        });
                     }
                  });
            } else {
               ascent::internal::RelIndexCombined::new(
                  &_self.runtime_total.b_indices_0.to_rel_index(&_self.runtime_total.__b_ind_common),
                  &_self.runtime_delta.b_indices_0.to_rel_index(&_self.runtime_delta.__b_ind_common),
               )
               .iter_all()
               .for_each(|(__cl1_joined_columns, __cl1_tuple_indices)| {
                  let __cl1_joined_columns = __cl1_joined_columns.tuple_of_borrowed();
                  let y = __cl1_joined_columns.0;
                  if let Some(__matching) = _self
                     .runtime_delta
                     .a_indices_1
                     .to_rel_index(&_self.runtime_delta.__a_ind_common)
                     .index_get(&(y.clone(),))
                  {
                     __cl1_tuple_indices.for_each(|cl1_val| {
                        let cl1_val = cl1_val.tuple_of_borrowed();
                        let z: &i32 = cl1_val.0;
                        __matching.clone().for_each(|__val| {
                           let mut __dep_changed = false;
                           let mut __default_id = 0;
                           let __val = __val.tuple_of_borrowed();
                           let x: &i32 = __val.0;
                           if let Some(__matching) = ascent::internal::RelIndexCombined::new(
                              &_self.runtime_total.c_indices_0.to_rel_index(&_self.runtime_total.__c_ind_common),
                              &_self.runtime_delta.c_indices_0.to_rel_index(&_self.runtime_delta.__c_ind_common),
                           )
                           .index_get(&(z.clone(),))
                           {
                              __matching.for_each(|__val| {
                                 let mut __dep_changed = false;
                                 let mut __default_id = 0;
                                 let __val = __val.tuple_of_borrowed();
                                 let w: &i32 = __val.0;
                                 let __new_row: (i32, i32) =
                                    (ascent::internal::Convert::convert(y), ascent::internal::Convert::convert(z));
                                 let mut __new_a = 0;
                                 if !::ascent::internal::RelFullIndexRead::contains_key(
                                    &_self
                                       .runtime_total
                                       .a_indices_0_1
                                       .to_rel_index(&_self.runtime_total.__a_ind_common),
                                    &__new_row,
                                 ) && !::ascent::internal::RelFullIndexRead::contains_key(
                                    &_self
                                       .runtime_delta
                                       .a_indices_0_1
                                       .to_rel_index(&_self.runtime_delta.__a_ind_common),
                                    &__new_row,
                                 ) {
                                    if ::ascent::internal::RelFullIndexWrite::insert_if_not_present(
                                       &mut _self
                                          .runtime_new
                                          .a_indices_0_1
                                          .to_rel_index_write(&mut _self.runtime_new.__a_ind_common),
                                       &__new_row,
                                       (),
                                    ) {
                                       __new_a = _self.a.len();
                                       _self.a.push((__new_row.0.clone(), __new_row.1.clone()));
                                       __default_id = __new_a;
                                       ::ascent::internal::RelIndexWrite::index_insert(
                                          &mut _self
                                             .runtime_new
                                             .a_indices_1
                                             .to_rel_index_write(&mut _self.runtime_new.__a_ind_common),
                                          (__new_row.1.clone(),),
                                          (__new_row.0.clone(),),
                                       );
                                       __changed = true;
                                    } else {
                                    }
                                 } else {
                                 }
                                 let __new_row: (i32, i32) =
                                    (ascent::internal::Convert::convert(z), ascent::internal::Convert::convert(w));
                                 let mut __new_b = 0;
                                 if !::ascent::internal::RelFullIndexRead::contains_key(
                                    &_self
                                       .runtime_total
                                       .b_indices_0_1
                                       .to_rel_index(&_self.runtime_total.__b_ind_common),
                                    &__new_row,
                                 ) && !::ascent::internal::RelFullIndexRead::contains_key(
                                    &_self
                                       .runtime_delta
                                       .b_indices_0_1
                                       .to_rel_index(&_self.runtime_delta.__b_ind_common),
                                    &__new_row,
                                 ) {
                                    if ::ascent::internal::RelFullIndexWrite::insert_if_not_present(
                                       &mut _self
                                          .runtime_new
                                          .b_indices_0_1
                                          .to_rel_index_write(&mut _self.runtime_new.__b_ind_common),
                                       &__new_row,
                                       (),
                                    ) {
                                       __new_b = _self.b.len();
                                       _self.b.push((__new_row.0.clone(), __new_row.1.clone()));
                                       __default_id = __new_b;
                                       ::ascent::internal::RelIndexWrite::index_insert(
                                          &mut _self
                                             .runtime_new
                                             .b_indices_0
                                             .to_rel_index_write(&mut _self.runtime_new.__b_ind_common),
                                          (__new_row.0.clone(),),
                                          (__new_row.1.clone(),),
                                       );
                                       __changed = true;
                                    } else {
                                    }
                                 } else {
                                 }
                                 let __new_row: (i32, i32) =
                                    (ascent::internal::Convert::convert(x), ascent::internal::Convert::convert(y));
                                 let mut __new_c = 0;
                                 if !::ascent::internal::RelFullIndexRead::contains_key(
                                    &_self
                                       .runtime_total
                                       .c_indices_0_1
                                       .to_rel_index(&_self.runtime_total.__c_ind_common),
                                    &__new_row,
                                 ) && !::ascent::internal::RelFullIndexRead::contains_key(
                                    &_self
                                       .runtime_delta
                                       .c_indices_0_1
                                       .to_rel_index(&_self.runtime_delta.__c_ind_common),
                                    &__new_row,
                                 ) {
                                    if ::ascent::internal::RelFullIndexWrite::insert_if_not_present(
                                       &mut _self
                                          .runtime_new
                                          .c_indices_0_1
                                          .to_rel_index_write(&mut _self.runtime_new.__c_ind_common),
                                       &__new_row,
                                       (),
                                    ) {
                                       __new_c = _self.c.len();
                                       _self.c.push((__new_row.0.clone(), __new_row.1.clone()));
                                       __default_id = __new_c;
                                       ::ascent::internal::RelIndexWrite::index_insert(
                                          &mut _self
                                             .runtime_new
                                             .c_indices_0
                                             .to_rel_index_write(&mut _self.runtime_new.__c_ind_common),
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
                        });
                     });
                  }
               });
            }
         }
         ascent::internal::comment(
            "a, b, c <-- a_indices_1_total, b_indices_0_delta, c_indices_0_total+delta [SIMPLE JOIN]",
         );
         if _self.runtime_delta.b_indices_0.to_rel_index(&_self.runtime_delta.__b_ind_common).len() > 0 {
            if _self.runtime_total.a_indices_1.to_rel_index(&_self.runtime_total.__a_ind_common).len()
               <= _self.runtime_delta.b_indices_0.to_rel_index(&_self.runtime_delta.__b_ind_common).len()
            {
               _self
                  .runtime_total
                  .a_indices_1
                  .to_rel_index(&_self.runtime_total.__a_ind_common)
                  .iter_all()
                  .for_each(|(__cl1_joined_columns, __cl1_tuple_indices)| {
                     let __cl1_joined_columns = __cl1_joined_columns.tuple_of_borrowed();
                     let y = __cl1_joined_columns.0;
                     if let Some(__matching) = _self
                        .runtime_delta
                        .b_indices_0
                        .to_rel_index(&_self.runtime_delta.__b_ind_common)
                        .index_get(&(y.clone(),))
                     {
                        __cl1_tuple_indices.for_each(|cl1_val| {
                           let cl1_val = cl1_val.tuple_of_borrowed();
                           let x: &i32 = cl1_val.0;
                           __matching.clone().for_each(|__val| {
                              let mut __dep_changed = false;
                              let mut __default_id = 0;
                              let __val = __val.tuple_of_borrowed();
                              let z: &i32 = __val.0;
                              if let Some(__matching) = ascent::internal::RelIndexCombined::new(
                                 &_self.runtime_total.c_indices_0.to_rel_index(&_self.runtime_total.__c_ind_common),
                                 &_self.runtime_delta.c_indices_0.to_rel_index(&_self.runtime_delta.__c_ind_common),
                              )
                              .index_get(&(z.clone(),))
                              {
                                 __matching.for_each(|__val| {
                                    let mut __dep_changed = false;
                                    let mut __default_id = 0;
                                    let __val = __val.tuple_of_borrowed();
                                    let w: &i32 = __val.0;
                                    let __new_row: (i32, i32) =
                                       (ascent::internal::Convert::convert(y), ascent::internal::Convert::convert(z));
                                    let mut __new_a = 0;
                                    if !::ascent::internal::RelFullIndexRead::contains_key(
                                       &_self
                                          .runtime_total
                                          .a_indices_0_1
                                          .to_rel_index(&_self.runtime_total.__a_ind_common),
                                       &__new_row,
                                    ) && !::ascent::internal::RelFullIndexRead::contains_key(
                                       &_self
                                          .runtime_delta
                                          .a_indices_0_1
                                          .to_rel_index(&_self.runtime_delta.__a_ind_common),
                                       &__new_row,
                                    ) {
                                       if ::ascent::internal::RelFullIndexWrite::insert_if_not_present(
                                          &mut _self
                                             .runtime_new
                                             .a_indices_0_1
                                             .to_rel_index_write(&mut _self.runtime_new.__a_ind_common),
                                          &__new_row,
                                          (),
                                       ) {
                                          __new_a = _self.a.len();
                                          _self.a.push((__new_row.0.clone(), __new_row.1.clone()));
                                          __default_id = __new_a;
                                          ::ascent::internal::RelIndexWrite::index_insert(
                                             &mut _self
                                                .runtime_new
                                                .a_indices_1
                                                .to_rel_index_write(&mut _self.runtime_new.__a_ind_common),
                                             (__new_row.1.clone(),),
                                             (__new_row.0.clone(),),
                                          );
                                          __changed = true;
                                       } else {
                                       }
                                    } else {
                                    }
                                    let __new_row: (i32, i32) =
                                       (ascent::internal::Convert::convert(z), ascent::internal::Convert::convert(w));
                                    let mut __new_b = 0;
                                    if !::ascent::internal::RelFullIndexRead::contains_key(
                                       &_self
                                          .runtime_total
                                          .b_indices_0_1
                                          .to_rel_index(&_self.runtime_total.__b_ind_common),
                                       &__new_row,
                                    ) && !::ascent::internal::RelFullIndexRead::contains_key(
                                       &_self
                                          .runtime_delta
                                          .b_indices_0_1
                                          .to_rel_index(&_self.runtime_delta.__b_ind_common),
                                       &__new_row,
                                    ) {
                                       if ::ascent::internal::RelFullIndexWrite::insert_if_not_present(
                                          &mut _self
                                             .runtime_new
                                             .b_indices_0_1
                                             .to_rel_index_write(&mut _self.runtime_new.__b_ind_common),
                                          &__new_row,
                                          (),
                                       ) {
                                          __new_b = _self.b.len();
                                          _self.b.push((__new_row.0.clone(), __new_row.1.clone()));
                                          __default_id = __new_b;
                                          ::ascent::internal::RelIndexWrite::index_insert(
                                             &mut _self
                                                .runtime_new
                                                .b_indices_0
                                                .to_rel_index_write(&mut _self.runtime_new.__b_ind_common),
                                             (__new_row.0.clone(),),
                                             (__new_row.1.clone(),),
                                          );
                                          __changed = true;
                                       } else {
                                       }
                                    } else {
                                    }
                                    let __new_row: (i32, i32) =
                                       (ascent::internal::Convert::convert(x), ascent::internal::Convert::convert(y));
                                    let mut __new_c = 0;
                                    if !::ascent::internal::RelFullIndexRead::contains_key(
                                       &_self
                                          .runtime_total
                                          .c_indices_0_1
                                          .to_rel_index(&_self.runtime_total.__c_ind_common),
                                       &__new_row,
                                    ) && !::ascent::internal::RelFullIndexRead::contains_key(
                                       &_self
                                          .runtime_delta
                                          .c_indices_0_1
                                          .to_rel_index(&_self.runtime_delta.__c_ind_common),
                                       &__new_row,
                                    ) {
                                       if ::ascent::internal::RelFullIndexWrite::insert_if_not_present(
                                          &mut _self
                                             .runtime_new
                                             .c_indices_0_1
                                             .to_rel_index_write(&mut _self.runtime_new.__c_ind_common),
                                          &__new_row,
                                          (),
                                       ) {
                                          __new_c = _self.c.len();
                                          _self.c.push((__new_row.0.clone(), __new_row.1.clone()));
                                          __default_id = __new_c;
                                          ::ascent::internal::RelIndexWrite::index_insert(
                                             &mut _self
                                                .runtime_new
                                                .c_indices_0
                                                .to_rel_index_write(&mut _self.runtime_new.__c_ind_common),
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
                           });
                        });
                     }
                  });
            } else {
               _self
                  .runtime_delta
                  .b_indices_0
                  .to_rel_index(&_self.runtime_delta.__b_ind_common)
                  .iter_all()
                  .for_each(|(__cl1_joined_columns, __cl1_tuple_indices)| {
                     let __cl1_joined_columns = __cl1_joined_columns.tuple_of_borrowed();
                     let y = __cl1_joined_columns.0;
                     if let Some(__matching) = _self
                        .runtime_total
                        .a_indices_1
                        .to_rel_index(&_self.runtime_total.__a_ind_common)
                        .index_get(&(y.clone(),))
                     {
                        __cl1_tuple_indices.for_each(|cl1_val| {
                           let cl1_val = cl1_val.tuple_of_borrowed();
                           let z: &i32 = cl1_val.0;
                           __matching.clone().for_each(|__val| {
                              let mut __dep_changed = false;
                              let mut __default_id = 0;
                              let __val = __val.tuple_of_borrowed();
                              let x: &i32 = __val.0;
                              if let Some(__matching) = ascent::internal::RelIndexCombined::new(
                                 &_self.runtime_total.c_indices_0.to_rel_index(&_self.runtime_total.__c_ind_common),
                                 &_self.runtime_delta.c_indices_0.to_rel_index(&_self.runtime_delta.__c_ind_common),
                              )
                              .index_get(&(z.clone(),))
                              {
                                 __matching.for_each(|__val| {
                                    let mut __dep_changed = false;
                                    let mut __default_id = 0;
                                    let __val = __val.tuple_of_borrowed();
                                    let w: &i32 = __val.0;
                                    let __new_row: (i32, i32) =
                                       (ascent::internal::Convert::convert(y), ascent::internal::Convert::convert(z));
                                    let mut __new_a = 0;
                                    if !::ascent::internal::RelFullIndexRead::contains_key(
                                       &_self
                                          .runtime_total
                                          .a_indices_0_1
                                          .to_rel_index(&_self.runtime_total.__a_ind_common),
                                       &__new_row,
                                    ) && !::ascent::internal::RelFullIndexRead::contains_key(
                                       &_self
                                          .runtime_delta
                                          .a_indices_0_1
                                          .to_rel_index(&_self.runtime_delta.__a_ind_common),
                                       &__new_row,
                                    ) {
                                       if ::ascent::internal::RelFullIndexWrite::insert_if_not_present(
                                          &mut _self
                                             .runtime_new
                                             .a_indices_0_1
                                             .to_rel_index_write(&mut _self.runtime_new.__a_ind_common),
                                          &__new_row,
                                          (),
                                       ) {
                                          __new_a = _self.a.len();
                                          _self.a.push((__new_row.0.clone(), __new_row.1.clone()));
                                          __default_id = __new_a;
                                          ::ascent::internal::RelIndexWrite::index_insert(
                                             &mut _self
                                                .runtime_new
                                                .a_indices_1
                                                .to_rel_index_write(&mut _self.runtime_new.__a_ind_common),
                                             (__new_row.1.clone(),),
                                             (__new_row.0.clone(),),
                                          );
                                          __changed = true;
                                       } else {
                                       }
                                    } else {
                                    }
                                    let __new_row: (i32, i32) =
                                       (ascent::internal::Convert::convert(z), ascent::internal::Convert::convert(w));
                                    let mut __new_b = 0;
                                    if !::ascent::internal::RelFullIndexRead::contains_key(
                                       &_self
                                          .runtime_total
                                          .b_indices_0_1
                                          .to_rel_index(&_self.runtime_total.__b_ind_common),
                                       &__new_row,
                                    ) && !::ascent::internal::RelFullIndexRead::contains_key(
                                       &_self
                                          .runtime_delta
                                          .b_indices_0_1
                                          .to_rel_index(&_self.runtime_delta.__b_ind_common),
                                       &__new_row,
                                    ) {
                                       if ::ascent::internal::RelFullIndexWrite::insert_if_not_present(
                                          &mut _self
                                             .runtime_new
                                             .b_indices_0_1
                                             .to_rel_index_write(&mut _self.runtime_new.__b_ind_common),
                                          &__new_row,
                                          (),
                                       ) {
                                          __new_b = _self.b.len();
                                          _self.b.push((__new_row.0.clone(), __new_row.1.clone()));
                                          __default_id = __new_b;
                                          ::ascent::internal::RelIndexWrite::index_insert(
                                             &mut _self
                                                .runtime_new
                                                .b_indices_0
                                                .to_rel_index_write(&mut _self.runtime_new.__b_ind_common),
                                             (__new_row.0.clone(),),
                                             (__new_row.1.clone(),),
                                          );
                                          __changed = true;
                                       } else {
                                       }
                                    } else {
                                    }
                                    let __new_row: (i32, i32) =
                                       (ascent::internal::Convert::convert(x), ascent::internal::Convert::convert(y));
                                    let mut __new_c = 0;
                                    if !::ascent::internal::RelFullIndexRead::contains_key(
                                       &_self
                                          .runtime_total
                                          .c_indices_0_1
                                          .to_rel_index(&_self.runtime_total.__c_ind_common),
                                       &__new_row,
                                    ) && !::ascent::internal::RelFullIndexRead::contains_key(
                                       &_self
                                          .runtime_delta
                                          .c_indices_0_1
                                          .to_rel_index(&_self.runtime_delta.__c_ind_common),
                                       &__new_row,
                                    ) {
                                       if ::ascent::internal::RelFullIndexWrite::insert_if_not_present(
                                          &mut _self
                                             .runtime_new
                                             .c_indices_0_1
                                             .to_rel_index_write(&mut _self.runtime_new.__c_ind_common),
                                          &__new_row,
                                          (),
                                       ) {
                                          __new_c = _self.c.len();
                                          _self.c.push((__new_row.0.clone(), __new_row.1.clone()));
                                          __default_id = __new_c;
                                          ::ascent::internal::RelIndexWrite::index_insert(
                                             &mut _self
                                                .runtime_new
                                                .c_indices_0
                                                .to_rel_index_write(&mut _self.runtime_new.__c_ind_common),
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
                           });
                        });
                     }
                  });
            }
         }
         ascent::internal::comment("a, b, c <-- a_indices_1_total, b_indices_0_total, c_indices_0_delta [SIMPLE JOIN]");
         if _self.runtime_delta.c_indices_0.to_rel_index(&_self.runtime_delta.__c_ind_common).len() > 0 {
            if _self.runtime_total.a_indices_1.to_rel_index(&_self.runtime_total.__a_ind_common).len()
               <= _self.runtime_total.b_indices_0.to_rel_index(&_self.runtime_total.__b_ind_common).len()
            {
               _self
                  .runtime_total
                  .a_indices_1
                  .to_rel_index(&_self.runtime_total.__a_ind_common)
                  .iter_all()
                  .for_each(|(__cl1_joined_columns, __cl1_tuple_indices)| {
                     let __cl1_joined_columns = __cl1_joined_columns.tuple_of_borrowed();
                     let y = __cl1_joined_columns.0;
                     if let Some(__matching) = _self
                        .runtime_total
                        .b_indices_0
                        .to_rel_index(&_self.runtime_total.__b_ind_common)
                        .index_get(&(y.clone(),))
                     {
                        __cl1_tuple_indices.for_each(|cl1_val| {
                           let cl1_val = cl1_val.tuple_of_borrowed();
                           let x: &i32 = cl1_val.0;
                           __matching.clone().for_each(|__val| {
                              let mut __dep_changed = false;
                              let mut __default_id = 0;
                              let __val = __val.tuple_of_borrowed();
                              let z: &i32 = __val.0;
                              if let Some(__matching) = _self
                                 .runtime_delta
                                 .c_indices_0
                                 .to_rel_index(&_self.runtime_delta.__c_ind_common)
                                 .index_get(&(z.clone(),))
                              {
                                 __matching.for_each(|__val| {
                                    let mut __dep_changed = false;
                                    let mut __default_id = 0;
                                    let __val = __val.tuple_of_borrowed();
                                    let w: &i32 = __val.0;
                                    let __new_row: (i32, i32) =
                                       (ascent::internal::Convert::convert(y), ascent::internal::Convert::convert(z));
                                    let mut __new_a = 0;
                                    if !::ascent::internal::RelFullIndexRead::contains_key(
                                       &_self
                                          .runtime_total
                                          .a_indices_0_1
                                          .to_rel_index(&_self.runtime_total.__a_ind_common),
                                       &__new_row,
                                    ) && !::ascent::internal::RelFullIndexRead::contains_key(
                                       &_self
                                          .runtime_delta
                                          .a_indices_0_1
                                          .to_rel_index(&_self.runtime_delta.__a_ind_common),
                                       &__new_row,
                                    ) {
                                       if ::ascent::internal::RelFullIndexWrite::insert_if_not_present(
                                          &mut _self
                                             .runtime_new
                                             .a_indices_0_1
                                             .to_rel_index_write(&mut _self.runtime_new.__a_ind_common),
                                          &__new_row,
                                          (),
                                       ) {
                                          __new_a = _self.a.len();
                                          _self.a.push((__new_row.0.clone(), __new_row.1.clone()));
                                          __default_id = __new_a;
                                          ::ascent::internal::RelIndexWrite::index_insert(
                                             &mut _self
                                                .runtime_new
                                                .a_indices_1
                                                .to_rel_index_write(&mut _self.runtime_new.__a_ind_common),
                                             (__new_row.1.clone(),),
                                             (__new_row.0.clone(),),
                                          );
                                          __changed = true;
                                       } else {
                                       }
                                    } else {
                                    }
                                    let __new_row: (i32, i32) =
                                       (ascent::internal::Convert::convert(z), ascent::internal::Convert::convert(w));
                                    let mut __new_b = 0;
                                    if !::ascent::internal::RelFullIndexRead::contains_key(
                                       &_self
                                          .runtime_total
                                          .b_indices_0_1
                                          .to_rel_index(&_self.runtime_total.__b_ind_common),
                                       &__new_row,
                                    ) && !::ascent::internal::RelFullIndexRead::contains_key(
                                       &_self
                                          .runtime_delta
                                          .b_indices_0_1
                                          .to_rel_index(&_self.runtime_delta.__b_ind_common),
                                       &__new_row,
                                    ) {
                                       if ::ascent::internal::RelFullIndexWrite::insert_if_not_present(
                                          &mut _self
                                             .runtime_new
                                             .b_indices_0_1
                                             .to_rel_index_write(&mut _self.runtime_new.__b_ind_common),
                                          &__new_row,
                                          (),
                                       ) {
                                          __new_b = _self.b.len();
                                          _self.b.push((__new_row.0.clone(), __new_row.1.clone()));
                                          __default_id = __new_b;
                                          ::ascent::internal::RelIndexWrite::index_insert(
                                             &mut _self
                                                .runtime_new
                                                .b_indices_0
                                                .to_rel_index_write(&mut _self.runtime_new.__b_ind_common),
                                             (__new_row.0.clone(),),
                                             (__new_row.1.clone(),),
                                          );
                                          __changed = true;
                                       } else {
                                       }
                                    } else {
                                    }
                                    let __new_row: (i32, i32) =
                                       (ascent::internal::Convert::convert(x), ascent::internal::Convert::convert(y));
                                    let mut __new_c = 0;
                                    if !::ascent::internal::RelFullIndexRead::contains_key(
                                       &_self
                                          .runtime_total
                                          .c_indices_0_1
                                          .to_rel_index(&_self.runtime_total.__c_ind_common),
                                       &__new_row,
                                    ) && !::ascent::internal::RelFullIndexRead::contains_key(
                                       &_self
                                          .runtime_delta
                                          .c_indices_0_1
                                          .to_rel_index(&_self.runtime_delta.__c_ind_common),
                                       &__new_row,
                                    ) {
                                       if ::ascent::internal::RelFullIndexWrite::insert_if_not_present(
                                          &mut _self
                                             .runtime_new
                                             .c_indices_0_1
                                             .to_rel_index_write(&mut _self.runtime_new.__c_ind_common),
                                          &__new_row,
                                          (),
                                       ) {
                                          __new_c = _self.c.len();
                                          _self.c.push((__new_row.0.clone(), __new_row.1.clone()));
                                          __default_id = __new_c;
                                          ::ascent::internal::RelIndexWrite::index_insert(
                                             &mut _self
                                                .runtime_new
                                                .c_indices_0
                                                .to_rel_index_write(&mut _self.runtime_new.__c_ind_common),
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
                           });
                        });
                     }
                  });
            } else {
               _self
                  .runtime_total
                  .b_indices_0
                  .to_rel_index(&_self.runtime_total.__b_ind_common)
                  .iter_all()
                  .for_each(|(__cl1_joined_columns, __cl1_tuple_indices)| {
                     let __cl1_joined_columns = __cl1_joined_columns.tuple_of_borrowed();
                     let y = __cl1_joined_columns.0;
                     if let Some(__matching) = _self
                        .runtime_total
                        .a_indices_1
                        .to_rel_index(&_self.runtime_total.__a_ind_common)
                        .index_get(&(y.clone(),))
                     {
                        __cl1_tuple_indices.for_each(|cl1_val| {
                           let cl1_val = cl1_val.tuple_of_borrowed();
                           let z: &i32 = cl1_val.0;
                           __matching.clone().for_each(|__val| {
                              let mut __dep_changed = false;
                              let mut __default_id = 0;
                              let __val = __val.tuple_of_borrowed();
                              let x: &i32 = __val.0;
                              if let Some(__matching) = _self
                                 .runtime_delta
                                 .c_indices_0
                                 .to_rel_index(&_self.runtime_delta.__c_ind_common)
                                 .index_get(&(z.clone(),))
                              {
                                 __matching.for_each(|__val| {
                                    let mut __dep_changed = false;
                                    let mut __default_id = 0;
                                    let __val = __val.tuple_of_borrowed();
                                    let w: &i32 = __val.0;
                                    let __new_row: (i32, i32) =
                                       (ascent::internal::Convert::convert(y), ascent::internal::Convert::convert(z));
                                    let mut __new_a = 0;
                                    if !::ascent::internal::RelFullIndexRead::contains_key(
                                       &_self
                                          .runtime_total
                                          .a_indices_0_1
                                          .to_rel_index(&_self.runtime_total.__a_ind_common),
                                       &__new_row,
                                    ) && !::ascent::internal::RelFullIndexRead::contains_key(
                                       &_self
                                          .runtime_delta
                                          .a_indices_0_1
                                          .to_rel_index(&_self.runtime_delta.__a_ind_common),
                                       &__new_row,
                                    ) {
                                       if ::ascent::internal::RelFullIndexWrite::insert_if_not_present(
                                          &mut _self
                                             .runtime_new
                                             .a_indices_0_1
                                             .to_rel_index_write(&mut _self.runtime_new.__a_ind_common),
                                          &__new_row,
                                          (),
                                       ) {
                                          __new_a = _self.a.len();
                                          _self.a.push((__new_row.0.clone(), __new_row.1.clone()));
                                          __default_id = __new_a;
                                          ::ascent::internal::RelIndexWrite::index_insert(
                                             &mut _self
                                                .runtime_new
                                                .a_indices_1
                                                .to_rel_index_write(&mut _self.runtime_new.__a_ind_common),
                                             (__new_row.1.clone(),),
                                             (__new_row.0.clone(),),
                                          );
                                          __changed = true;
                                       } else {
                                       }
                                    } else {
                                    }
                                    let __new_row: (i32, i32) =
                                       (ascent::internal::Convert::convert(z), ascent::internal::Convert::convert(w));
                                    let mut __new_b = 0;
                                    if !::ascent::internal::RelFullIndexRead::contains_key(
                                       &_self
                                          .runtime_total
                                          .b_indices_0_1
                                          .to_rel_index(&_self.runtime_total.__b_ind_common),
                                       &__new_row,
                                    ) && !::ascent::internal::RelFullIndexRead::contains_key(
                                       &_self
                                          .runtime_delta
                                          .b_indices_0_1
                                          .to_rel_index(&_self.runtime_delta.__b_ind_common),
                                       &__new_row,
                                    ) {
                                       if ::ascent::internal::RelFullIndexWrite::insert_if_not_present(
                                          &mut _self
                                             .runtime_new
                                             .b_indices_0_1
                                             .to_rel_index_write(&mut _self.runtime_new.__b_ind_common),
                                          &__new_row,
                                          (),
                                       ) {
                                          __new_b = _self.b.len();
                                          _self.b.push((__new_row.0.clone(), __new_row.1.clone()));
                                          __default_id = __new_b;
                                          ::ascent::internal::RelIndexWrite::index_insert(
                                             &mut _self
                                                .runtime_new
                                                .b_indices_0
                                                .to_rel_index_write(&mut _self.runtime_new.__b_ind_common),
                                             (__new_row.0.clone(),),
                                             (__new_row.1.clone(),),
                                          );
                                          __changed = true;
                                       } else {
                                       }
                                    } else {
                                    }
                                    let __new_row: (i32, i32) =
                                       (ascent::internal::Convert::convert(x), ascent::internal::Convert::convert(y));
                                    let mut __new_c = 0;
                                    if !::ascent::internal::RelFullIndexRead::contains_key(
                                       &_self
                                          .runtime_total
                                          .c_indices_0_1
                                          .to_rel_index(&_self.runtime_total.__c_ind_common),
                                       &__new_row,
                                    ) && !::ascent::internal::RelFullIndexRead::contains_key(
                                       &_self
                                          .runtime_delta
                                          .c_indices_0_1
                                          .to_rel_index(&_self.runtime_delta.__c_ind_common),
                                       &__new_row,
                                    ) {
                                       if ::ascent::internal::RelFullIndexWrite::insert_if_not_present(
                                          &mut _self
                                             .runtime_new
                                             .c_indices_0_1
                                             .to_rel_index_write(&mut _self.runtime_new.__c_ind_common),
                                          &__new_row,
                                          (),
                                       ) {
                                          __new_c = _self.c.len();
                                          _self.c.push((__new_row.0.clone(), __new_row.1.clone()));
                                          __default_id = __new_c;
                                          ::ascent::internal::RelIndexWrite::index_insert(
                                             &mut _self
                                                .runtime_new
                                                .c_indices_0
                                                .to_rel_index_write(&mut _self.runtime_new.__c_ind_common),
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
                           });
                        });
                     }
                  });
            }
         }
         ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
            &mut _self.runtime_new.__a_ind_common,
            &mut _self.runtime_delta.__a_ind_common,
            &mut _self.runtime_total.__a_ind_common,
         );
         ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
            &mut _self.runtime_new.a_indices_0_1.to_rel_index_write(&mut _self.runtime_new.__a_ind_common),
            &mut _self.runtime_delta.a_indices_0_1.to_rel_index_write(&mut _self.runtime_delta.__a_ind_common),
            &mut _self.runtime_total.a_indices_0_1.to_rel_index_write(&mut _self.runtime_total.__a_ind_common),
         );
         ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
            &mut _self.runtime_new.a_indices_1.to_rel_index_write(&mut _self.runtime_new.__a_ind_common),
            &mut _self.runtime_delta.a_indices_1.to_rel_index_write(&mut _self.runtime_delta.__a_ind_common),
            &mut _self.runtime_total.a_indices_1.to_rel_index_write(&mut _self.runtime_total.__a_ind_common),
         );
         ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
            &mut _self.runtime_new.__b_ind_common,
            &mut _self.runtime_delta.__b_ind_common,
            &mut _self.runtime_total.__b_ind_common,
         );
         ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
            &mut _self.runtime_new.b_indices_0.to_rel_index_write(&mut _self.runtime_new.__b_ind_common),
            &mut _self.runtime_delta.b_indices_0.to_rel_index_write(&mut _self.runtime_delta.__b_ind_common),
            &mut _self.runtime_total.b_indices_0.to_rel_index_write(&mut _self.runtime_total.__b_ind_common),
         );
         ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
            &mut _self.runtime_new.b_indices_0_1.to_rel_index_write(&mut _self.runtime_new.__b_ind_common),
            &mut _self.runtime_delta.b_indices_0_1.to_rel_index_write(&mut _self.runtime_delta.__b_ind_common),
            &mut _self.runtime_total.b_indices_0_1.to_rel_index_write(&mut _self.runtime_total.__b_ind_common),
         );
         ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
            &mut _self.runtime_new.__c_ind_common,
            &mut _self.runtime_delta.__c_ind_common,
            &mut _self.runtime_total.__c_ind_common,
         );
         ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
            &mut _self.runtime_new.c_indices_0.to_rel_index_write(&mut _self.runtime_new.__c_ind_common),
            &mut _self.runtime_delta.c_indices_0.to_rel_index_write(&mut _self.runtime_delta.__c_ind_common),
            &mut _self.runtime_total.c_indices_0.to_rel_index_write(&mut _self.runtime_total.__c_ind_common),
         );
         ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
            &mut _self.runtime_new.c_indices_0_1.to_rel_index_write(&mut _self.runtime_new.__c_ind_common),
            &mut _self.runtime_delta.c_indices_0_1.to_rel_index_write(&mut _self.runtime_delta.__c_ind_common),
            &mut _self.runtime_total.c_indices_0_1.to_rel_index_write(&mut _self.runtime_total.__c_ind_common),
         );
         _self.scc_iters[0usize] += 1;
         let need_break = !__changed;
         _self.scc_times[0usize] += _scc_start_time.elapsed();
         need_break
      }
      #[allow(unused_assignments, unused_variables, dead_code)]
      pub fn scc_0(&mut self) -> bool {
         ascent::internal::comment("scc 0");
         {
            macro_rules! __check_return_conditions {
               () => {};
            }
            let _self = self;
            use ascent::internal::RelIndexWrite;
            use ascent::internal::{RelIndexRead, RelIndexReadAll, ToRelIndex0, TupleOfBorrowed};
            use core::cmp::PartialEq;
            _self.runtime_delta.__a_ind_common = ::std::mem::take(&mut _self.runtime_total.__a_ind_common);
            _self.runtime_total.__a_ind_common = Default::default();
            _self.runtime_new.__a_ind_common = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut _self.runtime_new.__a_ind_common,
               &mut _self.runtime_delta.__a_ind_common,
               &mut _self.runtime_total.__a_ind_common,
            );
            _self.runtime_delta.a_indices_0_1 = ::std::mem::take(&mut _self.runtime_total.a_indices_0_1);
            _self.runtime_total.a_indices_0_1 = Default::default();
            _self.runtime_new.a_indices_0_1 = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut _self.runtime_new.a_indices_0_1.to_rel_index_write(&mut _self.runtime_new.__a_ind_common),
               &mut _self.runtime_delta.a_indices_0_1.to_rel_index_write(&mut _self.runtime_delta.__a_ind_common),
               &mut _self.runtime_total.a_indices_0_1.to_rel_index_write(&mut _self.runtime_total.__a_ind_common),
            );
            _self.runtime_delta.a_indices_1 = ::std::mem::take(&mut _self.runtime_total.a_indices_1);
            _self.runtime_total.a_indices_1 = Default::default();
            _self.runtime_new.a_indices_1 = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut _self.runtime_new.a_indices_1.to_rel_index_write(&mut _self.runtime_new.__a_ind_common),
               &mut _self.runtime_delta.a_indices_1.to_rel_index_write(&mut _self.runtime_delta.__a_ind_common),
               &mut _self.runtime_total.a_indices_1.to_rel_index_write(&mut _self.runtime_total.__a_ind_common),
            );
            _self.runtime_delta.__b_ind_common = ::std::mem::take(&mut _self.runtime_total.__b_ind_common);
            _self.runtime_total.__b_ind_common = Default::default();
            _self.runtime_new.__b_ind_common = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut _self.runtime_new.__b_ind_common,
               &mut _self.runtime_delta.__b_ind_common,
               &mut _self.runtime_total.__b_ind_common,
            );
            _self.runtime_delta.b_indices_0 = ::std::mem::take(&mut _self.runtime_total.b_indices_0);
            _self.runtime_total.b_indices_0 = Default::default();
            _self.runtime_new.b_indices_0 = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut _self.runtime_new.b_indices_0.to_rel_index_write(&mut _self.runtime_new.__b_ind_common),
               &mut _self.runtime_delta.b_indices_0.to_rel_index_write(&mut _self.runtime_delta.__b_ind_common),
               &mut _self.runtime_total.b_indices_0.to_rel_index_write(&mut _self.runtime_total.__b_ind_common),
            );
            _self.runtime_delta.b_indices_0_1 = ::std::mem::take(&mut _self.runtime_total.b_indices_0_1);
            _self.runtime_total.b_indices_0_1 = Default::default();
            _self.runtime_new.b_indices_0_1 = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut _self.runtime_new.b_indices_0_1.to_rel_index_write(&mut _self.runtime_new.__b_ind_common),
               &mut _self.runtime_delta.b_indices_0_1.to_rel_index_write(&mut _self.runtime_delta.__b_ind_common),
               &mut _self.runtime_total.b_indices_0_1.to_rel_index_write(&mut _self.runtime_total.__b_ind_common),
            );
            _self.runtime_delta.__c_ind_common = ::std::mem::take(&mut _self.runtime_total.__c_ind_common);
            _self.runtime_total.__c_ind_common = Default::default();
            _self.runtime_new.__c_ind_common = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut _self.runtime_new.__c_ind_common,
               &mut _self.runtime_delta.__c_ind_common,
               &mut _self.runtime_total.__c_ind_common,
            );
            _self.runtime_delta.c_indices_0 = ::std::mem::take(&mut _self.runtime_total.c_indices_0);
            _self.runtime_total.c_indices_0 = Default::default();
            _self.runtime_new.c_indices_0 = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut _self.runtime_new.c_indices_0.to_rel_index_write(&mut _self.runtime_new.__c_ind_common),
               &mut _self.runtime_delta.c_indices_0.to_rel_index_write(&mut _self.runtime_delta.__c_ind_common),
               &mut _self.runtime_total.c_indices_0.to_rel_index_write(&mut _self.runtime_total.__c_ind_common),
            );
            _self.runtime_delta.c_indices_0_1 = ::std::mem::take(&mut _self.runtime_total.c_indices_0_1);
            _self.runtime_total.c_indices_0_1 = Default::default();
            _self.runtime_new.c_indices_0_1 = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut _self.runtime_new.c_indices_0_1.to_rel_index_write(&mut _self.runtime_new.__c_ind_common),
               &mut _self.runtime_delta.c_indices_0_1.to_rel_index_write(&mut _self.runtime_delta.__c_ind_common),
               &mut _self.runtime_total.c_indices_0_1.to_rel_index_write(&mut _self.runtime_total.__c_ind_common),
            );
            loop {
               let need_break = _self.scc_0_exec();
               if need_break {
                  break;
               }
               __check_return_conditions!();
            }
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
         true
      }
      #[allow(noop_method_call, suspicious_double_ref_op)]
      pub fn update_indices_priv(&mut self) {
         let before = ::ascent::internal::Instant::now();
         self.update_indices_a();
         self.update_indices_b();
         self.update_indices_c();
         self.update_indices_duration += before.elapsed();
      }
      #[allow(noop_method_call, suspicious_double_ref_op)]
      pub fn update_indices_a(&mut self) {
         use ascent::internal::RelIndexWrite;
         use ascent::internal::ToRelIndex0;
         for (_i, tuple) in self.a.iter().enumerate() {
            let selection_tuple = (tuple.0.clone(), tuple.1.clone());
            let rel_ind = &mut self.runtime_total.a_indices_0_1;
            ascent::internal::RelIndexWrite::index_insert(
               &mut rel_ind.to_rel_index_write(&mut self.runtime_total.__a_ind_common),
               selection_tuple,
               (),
            );
            let selection_tuple = (tuple.1.clone(),);
            let rel_ind = &mut self.runtime_total.a_indices_1;
            ascent::internal::RelIndexWrite::index_insert(
               &mut rel_ind.to_rel_index_write(&mut self.runtime_total.__a_ind_common),
               selection_tuple,
               (tuple.0.clone(),),
            );
         }
      }
      #[allow(noop_method_call, suspicious_double_ref_op)]
      pub fn update_indices_b(&mut self) {
         use ascent::internal::RelIndexWrite;
         use ascent::internal::ToRelIndex0;
         for (_i, tuple) in self.b.iter().enumerate() {
            let selection_tuple = (tuple.0.clone(),);
            let rel_ind = &mut self.runtime_total.b_indices_0;
            ascent::internal::RelIndexWrite::index_insert(
               &mut rel_ind.to_rel_index_write(&mut self.runtime_total.__b_ind_common),
               selection_tuple,
               (tuple.1.clone(),),
            );
            let selection_tuple = (tuple.0.clone(), tuple.1.clone());
            let rel_ind = &mut self.runtime_total.b_indices_0_1;
            ascent::internal::RelIndexWrite::index_insert(
               &mut rel_ind.to_rel_index_write(&mut self.runtime_total.__b_ind_common),
               selection_tuple,
               (),
            );
         }
      }
      #[allow(noop_method_call, suspicious_double_ref_op)]
      pub fn update_indices_c(&mut self) {
         use ascent::internal::RelIndexWrite;
         use ascent::internal::ToRelIndex0;
         for (_i, tuple) in self.c.iter().enumerate() {
            let selection_tuple = (tuple.0.clone(),);
            let rel_ind = &mut self.runtime_total.c_indices_0;
            ascent::internal::RelIndexWrite::index_insert(
               &mut rel_ind.to_rel_index_write(&mut self.runtime_total.__c_ind_common),
               selection_tuple,
               (tuple.1.clone(),),
            );
            let selection_tuple = (tuple.0.clone(), tuple.1.clone());
            let rel_ind = &mut self.runtime_total.c_indices_0_1;
            ascent::internal::RelIndexWrite::index_insert(
               &mut rel_ind.to_rel_index_write(&mut self.runtime_total.__c_ind_common),
               selection_tuple,
               (),
            );
         }
      }
      #[deprecated = "Explicit call to update_indices not required anymore."]
      pub fn update_indices(&mut self) { self.update_indices_priv(); }
      fn type_constraints() { let _type_constraints: ascent::internal::TypeConstraints<i32>; }
      pub fn summary() -> &'static str {
         "scc 0, is_looping: true:\n  a, b, c <-- a_indices_1_delta, b_indices_0_total+delta, c_indices_0_total+delta [SIMPLE JOIN]\n  a, b, c <-- a_indices_1_total, b_indices_0_delta, c_indices_0_total+delta [SIMPLE JOIN]\n  a, b, c <-- a_indices_1_total, b_indices_0_total, c_indices_0_delta [SIMPLE JOIN]\n  dynamic relations: a, b, c\n"
      }
      pub fn relation_sizes_summary(&self) -> String {
         use std::fmt::Write;
         let mut res = String::new();
         writeln!(&mut res, "{} size: {}", "a", self.a.len()).unwrap();
         writeln!(&mut res, "{} size: {}", "b", self.b.len()).unwrap();
         writeln!(&mut res, "{} size: {}", "c", self.c.len()).unwrap();
         res
      }
      pub fn scc_times_summary(&self) -> String {
         use std::fmt::Write;
         let mut res = String::new();
         writeln!(&mut res, "update_indices time: {:?}", self.update_indices_duration).unwrap();
         writeln!(&mut res, "scc {}: iterations: {}, time: {:?}", "0", self.scc_iters[0usize], self.scc_times[0usize])
            .unwrap();
         res
      }
   }
   impl Default for AscentProgramRuntime {
      fn default() -> Self {
         let mut _self = AscentProgramRuntime {
            a: Default::default(),
            __a_ind_common: Default::default(),
            a_indices_0_1: Default::default(),
            a_indices_1: Default::default(),
            b: Default::default(),
            __b_ind_common: Default::default(),
            b_indices_0: Default::default(),
            b_indices_0_1: Default::default(),
            c: Default::default(),
            __c_ind_common: Default::default(),
            c_indices_0: Default::default(),
            c_indices_0_1: Default::default(),
         };
         _self
      }
   }
   impl Default for AscentProgram {
      fn default() -> Self {
         let mut _self = AscentProgram {
            a: Default::default(),
            b: Default::default(),
            c: Default::default(),
            scc_times: [std::time::Duration::ZERO; 1usize],
            scc_iters: [0; 1usize],
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
