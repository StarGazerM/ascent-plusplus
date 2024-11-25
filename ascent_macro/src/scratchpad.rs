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
   ::ascent::rel::rel_codegen! { AscentProgram_baz , (i32 , i32) , [[] , [0 , 1]] , par , () }
   ::ascent::rel::rel_codegen! { AscentProgram_foo , (i32 ,) , [[0]] , par , () }
   ::ascent::rel::rel_codegen! { AscentProgram_bar , (i32 ,) , [[0]] , par , () }
   pub struct AscentProgram {
      #[doc = "\nlogical indices: bar_indices_0"]
      pub bar: ::ascent::rel::rel!(AscentProgram_bar, (i32,), [[0]], par, ()),
      pub __bar_delete: ::ascent::rel::rel!(AscentProgram_bar, (i32,), [[0]], par, ()),
      pub __bar_ind_common: ::ascent::rel::rel_ind_common!(AscentProgram_bar, (i32,), [[0]], par, ()),
      pub bar_indices_0: ::ascent::rel::rel_full_ind!(
         AscentProgram_bar,
         (i32,),
         [[0]],
         par,
         (),
         (i32,),
         ascent::internal::FullRelCounter
      ),
      #[doc = "\nlogical indices: baz_indices_0_1; baz_indices_none"]
      pub baz: ::ascent::rel::rel!(AscentProgram_baz, (i32, i32), [[], [0, 1]], par, ()),
      pub __baz_delete: ::ascent::rel::rel!(AscentProgram_baz, (i32, i32), [[], [0, 1]], par, ()),
      pub __baz_ind_common: ::ascent::rel::rel_ind_common!(AscentProgram_baz, (i32, i32), [[], [0, 1]], par, ()),
      pub baz_indices_0_1: ::ascent::rel::rel_full_ind!(
         AscentProgram_baz,
         (i32, i32),
         [[], [0, 1]],
         par,
         (),
         (i32, i32),
         ascent::internal::FullRelCounter
      ),
      pub baz_indices_none:
         ::ascent::rel::rel_ind!(AscentProgram_baz, (i32, i32), [[], [0, 1]], par, (), [], (), (i32, i32)),
      #[doc = "\nlogical indices: foo_indices_0"]
      pub foo: ::ascent::rel::rel!(AscentProgram_foo, (i32,), [[0]], par, ()),
      pub __foo_delete: ::ascent::rel::rel!(AscentProgram_foo, (i32,), [[0]], par, ()),
      pub __foo_ind_common: ::ascent::rel::rel_ind_common!(AscentProgram_foo, (i32,), [[0]], par, ()),
      pub foo_indices_0: ::ascent::rel::rel_full_ind!(
         AscentProgram_foo,
         (i32,),
         [[0]],
         par,
         (),
         (i32,),
         ascent::internal::FullRelCounter
      ),
      scc_times: [std::time::Duration; 3usize],
      scc_iters: [usize; 3usize],
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
         use ascent::internal::CRelIndexRead;
         use ascent::internal::CRelIndexReadAll;
         use ascent::internal::CRelIndexWrite;
         use ascent::internal::Freezable;
         use ascent::internal::{RelIndexRead, RelIndexReadAll, ToRelIndex0, TupleOfBorrowed};
         use ascent::rayon::iter::ParallelBridge;
         use ascent::rayon::iter::ParallelIterator;
         use core::cmp::PartialEq;
         self.update_indices_priv();
         let _self = self;
         ascent::internal::comment("scc 0");
         {
            let _scc_start_time = ::ascent::internal::Instant::now();
            let mut __foo_ind_common_delta: ::ascent::rel::rel_ind_common!(AscentProgram_foo, (i32,), [[0]], par, ()) =
               ::std::mem::take(&mut _self.__foo_ind_common);
            let mut __foo_ind_common_total: ::ascent::rel::rel_ind_common!(AscentProgram_foo, (i32,), [[0]], par, ()) =
               Default::default();
            let mut __foo_ind_common_new: ::ascent::rel::rel_ind_common!(AscentProgram_foo, (i32,), [[0]], par, ()) =
               Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut __foo_ind_common_new,
               &mut __foo_ind_common_delta,
               &mut __foo_ind_common_total,
            );
            let mut foo_indices_0_delta: ::ascent::rel::rel_full_ind!(
               AscentProgram_foo,
               (i32,),
               [[0]],
               par,
               (),
               (i32,),
               ascent::internal::FullRelCounter
            ) = ::std::mem::take(&mut _self.foo_indices_0);
            let mut foo_indices_0_total: ::ascent::rel::rel_full_ind!(
               AscentProgram_foo,
               (i32,),
               [[0]],
               par,
               (),
               (i32,),
               ascent::internal::FullRelCounter
            ) = Default::default();
            let mut foo_indices_0_new: ::ascent::rel::rel_full_ind!(
               AscentProgram_foo,
               (i32,),
               [[0]],
               par,
               (),
               (i32,),
               ascent::internal::FullRelCounter
            ) = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut foo_indices_0_new.to_rel_index_write(&mut __foo_ind_common_new),
               &mut foo_indices_0_delta.to_rel_index_write(&mut __foo_ind_common_delta),
               &mut foo_indices_0_total.to_rel_index_write(&mut __foo_ind_common_total),
            );
            #[allow(unused_assignments, unused_variables)]
            {
               let __changed = std::sync::atomic::AtomicBool::new(false);
               let mut __default_id = 0;
               __foo_ind_common_total.freeze();
               __foo_ind_common_delta.freeze();
               foo_indices_0_total.freeze();
               foo_indices_0_delta.freeze();
               ascent::internal::comment("foo <-- ");
               {
                  let __new_row: (i32,) = (1,);
                  let mut __new_foo = 0;
                  if !::ascent::internal::RelFullIndexRead::contains_key(
                     &foo_indices_0_total.to_rel_index(&__foo_ind_common_total),
                     &__new_row,
                  ) && !::ascent::internal::RelFullIndexRead::contains_key(
                     &foo_indices_0_delta.to_rel_index(&__foo_ind_common_delta),
                     &__new_row,
                  ) {
                     if ::ascent::internal::CRelFullIndexWrite::insert_if_not_present(
                        &foo_indices_0_new.to_c_rel_index_write(&__foo_ind_common_new),
                        &__new_row,
                        (1 as usize).into(),
                     ) {
                        __new_foo = _self.foo.push((__new_row.0,));
                        __default_id = __new_foo;
                        __changed.store(true, std::sync::atomic::Ordering::Relaxed);
                     } else {
                     }
                  } else {
                  }
               }
               __foo_ind_common_total.unfreeze();
               __foo_ind_common_delta.unfreeze();
               foo_indices_0_total.unfreeze();
               foo_indices_0_delta.unfreeze();
               ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
                  &mut __foo_ind_common_new,
                  &mut __foo_ind_common_delta,
                  &mut __foo_ind_common_total,
               );
               ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
                  &mut foo_indices_0_new.to_rel_index_write(&mut __foo_ind_common_new),
                  &mut foo_indices_0_delta.to_rel_index_write(&mut __foo_ind_common_delta),
                  &mut foo_indices_0_total.to_rel_index_write(&mut __foo_ind_common_total),
               );
               ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
                  &mut __foo_ind_common_new,
                  &mut __foo_ind_common_delta,
                  &mut __foo_ind_common_total,
               );
               ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
                  &mut foo_indices_0_new.to_rel_index_write(&mut __foo_ind_common_new),
                  &mut foo_indices_0_delta.to_rel_index_write(&mut __foo_ind_common_delta),
                  &mut foo_indices_0_total.to_rel_index_write(&mut __foo_ind_common_total),
               );
               _self.scc_iters[0usize] += 1;
               __check_return_conditions!();
            }
            _self.__foo_ind_common = __foo_ind_common_total;
            _self.foo_indices_0 = foo_indices_0_total;
            _self.scc_times[0usize] += _scc_start_time.elapsed();
         }
         ascent::internal::comment("scc 1");
         {
            let _scc_start_time = ::ascent::internal::Instant::now();
            let mut __bar_ind_common_delta: ::ascent::rel::rel_ind_common!(AscentProgram_bar, (i32,), [[0]], par, ()) =
               ::std::mem::take(&mut _self.__bar_ind_common);
            let mut __bar_ind_common_total: ::ascent::rel::rel_ind_common!(AscentProgram_bar, (i32,), [[0]], par, ()) =
               Default::default();
            let mut __bar_ind_common_new: ::ascent::rel::rel_ind_common!(AscentProgram_bar, (i32,), [[0]], par, ()) =
               Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut __bar_ind_common_new,
               &mut __bar_ind_common_delta,
               &mut __bar_ind_common_total,
            );
            let mut bar_indices_0_delta: ::ascent::rel::rel_full_ind!(
               AscentProgram_bar,
               (i32,),
               [[0]],
               par,
               (),
               (i32,),
               ascent::internal::FullRelCounter
            ) = ::std::mem::take(&mut _self.bar_indices_0);
            let mut bar_indices_0_total: ::ascent::rel::rel_full_ind!(
               AscentProgram_bar,
               (i32,),
               [[0]],
               par,
               (),
               (i32,),
               ascent::internal::FullRelCounter
            ) = Default::default();
            let mut bar_indices_0_new: ::ascent::rel::rel_full_ind!(
               AscentProgram_bar,
               (i32,),
               [[0]],
               par,
               (),
               (i32,),
               ascent::internal::FullRelCounter
            ) = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut bar_indices_0_new.to_rel_index_write(&mut __bar_ind_common_new),
               &mut bar_indices_0_delta.to_rel_index_write(&mut __bar_ind_common_delta),
               &mut bar_indices_0_total.to_rel_index_write(&mut __bar_ind_common_total),
            );
            #[allow(unused_assignments, unused_variables)]
            {
               let __changed = std::sync::atomic::AtomicBool::new(false);
               let mut __default_id = 0;
               __bar_ind_common_total.freeze();
               __bar_ind_common_delta.freeze();
               bar_indices_0_total.freeze();
               bar_indices_0_delta.freeze();
               ascent::internal::comment("bar <-- ");
               {
                  let __new_row: (i32,) = (3,);
                  let mut __new_bar = 0;
                  if !::ascent::internal::RelFullIndexRead::contains_key(
                     &bar_indices_0_total.to_rel_index(&__bar_ind_common_total),
                     &__new_row,
                  ) && !::ascent::internal::RelFullIndexRead::contains_key(
                     &bar_indices_0_delta.to_rel_index(&__bar_ind_common_delta),
                     &__new_row,
                  ) {
                     if ::ascent::internal::CRelFullIndexWrite::insert_if_not_present(
                        &bar_indices_0_new.to_c_rel_index_write(&__bar_ind_common_new),
                        &__new_row,
                        (1 as usize).into(),
                     ) {
                        __new_bar = _self.bar.push((__new_row.0,));
                        __default_id = __new_bar;
                        __changed.store(true, std::sync::atomic::Ordering::Relaxed);
                     } else {
                     }
                  } else {
                  }
               }
               __bar_ind_common_total.unfreeze();
               __bar_ind_common_delta.unfreeze();
               bar_indices_0_total.unfreeze();
               bar_indices_0_delta.unfreeze();
               ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
                  &mut __bar_ind_common_new,
                  &mut __bar_ind_common_delta,
                  &mut __bar_ind_common_total,
               );
               ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
                  &mut bar_indices_0_new.to_rel_index_write(&mut __bar_ind_common_new),
                  &mut bar_indices_0_delta.to_rel_index_write(&mut __bar_ind_common_delta),
                  &mut bar_indices_0_total.to_rel_index_write(&mut __bar_ind_common_total),
               );
               ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
                  &mut __bar_ind_common_new,
                  &mut __bar_ind_common_delta,
                  &mut __bar_ind_common_total,
               );
               ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
                  &mut bar_indices_0_new.to_rel_index_write(&mut __bar_ind_common_new),
                  &mut bar_indices_0_delta.to_rel_index_write(&mut __bar_ind_common_delta),
                  &mut bar_indices_0_total.to_rel_index_write(&mut __bar_ind_common_total),
               );
               _self.scc_iters[1usize] += 1;
               __check_return_conditions!();
            }
            _self.__bar_ind_common = __bar_ind_common_total;
            _self.bar_indices_0 = bar_indices_0_total;
            _self.scc_times[1usize] += _scc_start_time.elapsed();
         }
         ascent::internal::comment("scc 2");
         {
            let _scc_start_time = ::ascent::internal::Instant::now();
            let mut __bar_ind_common_delta: ::ascent::rel::rel_ind_common!(AscentProgram_bar, (i32,), [[0]], par, ()) =
               ::std::mem::take(&mut _self.__bar_ind_common);
            let mut __bar_ind_common_total: ::ascent::rel::rel_ind_common!(AscentProgram_bar, (i32,), [[0]], par, ()) =
               Default::default();
            let mut __bar_ind_common_new: ::ascent::rel::rel_ind_common!(AscentProgram_bar, (i32,), [[0]], par, ()) =
               Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut __bar_ind_common_new,
               &mut __bar_ind_common_delta,
               &mut __bar_ind_common_total,
            );
            let mut bar_indices_0_delta: ::ascent::rel::rel_full_ind!(
               AscentProgram_bar,
               (i32,),
               [[0]],
               par,
               (),
               (i32,),
               ascent::internal::FullRelCounter
            ) = ::std::mem::take(&mut _self.bar_indices_0);
            let mut bar_indices_0_total: ::ascent::rel::rel_full_ind!(
               AscentProgram_bar,
               (i32,),
               [[0]],
               par,
               (),
               (i32,),
               ascent::internal::FullRelCounter
            ) = Default::default();
            let mut bar_indices_0_new: ::ascent::rel::rel_full_ind!(
               AscentProgram_bar,
               (i32,),
               [[0]],
               par,
               (),
               (i32,),
               ascent::internal::FullRelCounter
            ) = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut bar_indices_0_new.to_rel_index_write(&mut __bar_ind_common_new),
               &mut bar_indices_0_delta.to_rel_index_write(&mut __bar_ind_common_delta),
               &mut bar_indices_0_total.to_rel_index_write(&mut __bar_ind_common_total),
            );
            let mut __baz_ind_common_delta: ::ascent::rel::rel_ind_common!(
               AscentProgram_baz,
               (i32, i32),
               [[], [0, 1]],
               par,
               ()
            ) = ::std::mem::take(&mut _self.__baz_ind_common);
            let mut __baz_ind_common_total: ::ascent::rel::rel_ind_common!(
               AscentProgram_baz,
               (i32, i32),
               [[], [0, 1]],
               par,
               ()
            ) = Default::default();
            let mut __baz_ind_common_new: ::ascent::rel::rel_ind_common!(
               AscentProgram_baz,
               (i32, i32),
               [[], [0, 1]],
               par,
               ()
            ) = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut __baz_ind_common_new,
               &mut __baz_ind_common_delta,
               &mut __baz_ind_common_total,
            );
            let mut baz_indices_0_1_delta: ::ascent::rel::rel_full_ind!(
               AscentProgram_baz,
               (i32, i32),
               [[], [0, 1]],
               par,
               (),
               (i32, i32),
               ascent::internal::FullRelCounter
            ) = ::std::mem::take(&mut _self.baz_indices_0_1);
            let mut baz_indices_0_1_total: ::ascent::rel::rel_full_ind!(
               AscentProgram_baz,
               (i32, i32),
               [[], [0, 1]],
               par,
               (),
               (i32, i32),
               ascent::internal::FullRelCounter
            ) = Default::default();
            let mut baz_indices_0_1_new: ::ascent::rel::rel_full_ind!(
               AscentProgram_baz,
               (i32, i32),
               [[], [0, 1]],
               par,
               (),
               (i32, i32),
               ascent::internal::FullRelCounter
            ) = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut baz_indices_0_1_new.to_rel_index_write(&mut __baz_ind_common_new),
               &mut baz_indices_0_1_delta.to_rel_index_write(&mut __baz_ind_common_delta),
               &mut baz_indices_0_1_total.to_rel_index_write(&mut __baz_ind_common_total),
            );
            let mut baz_indices_none_delta: ::ascent::rel::rel_ind!(
               AscentProgram_baz,
               (i32, i32),
               [[], [0, 1]],
               par,
               (),
               [],
               (),
               (i32, i32)
            ) = ::std::mem::take(&mut _self.baz_indices_none);
            let mut baz_indices_none_total: ::ascent::rel::rel_ind!(
               AscentProgram_baz,
               (i32, i32),
               [[], [0, 1]],
               par,
               (),
               [],
               (),
               (i32, i32)
            ) = Default::default();
            let mut baz_indices_none_new: ::ascent::rel::rel_ind!(
               AscentProgram_baz,
               (i32, i32),
               [[], [0, 1]],
               par,
               (),
               [],
               (),
               (i32, i32)
            ) = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut baz_indices_none_new.to_rel_index_write(&mut __baz_ind_common_new),
               &mut baz_indices_none_delta.to_rel_index_write(&mut __baz_ind_common_delta),
               &mut baz_indices_none_total.to_rel_index_write(&mut __baz_ind_common_total),
            );
            let mut __foo_ind_common_delta: ::ascent::rel::rel_ind_common!(AscentProgram_foo, (i32,), [[0]], par, ()) =
               ::std::mem::take(&mut _self.__foo_ind_common);
            let mut __foo_ind_common_total: ::ascent::rel::rel_ind_common!(AscentProgram_foo, (i32,), [[0]], par, ()) =
               Default::default();
            let mut __foo_ind_common_new: ::ascent::rel::rel_ind_common!(AscentProgram_foo, (i32,), [[0]], par, ()) =
               Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut __foo_ind_common_new,
               &mut __foo_ind_common_delta,
               &mut __foo_ind_common_total,
            );
            let mut foo_indices_0_delta: ::ascent::rel::rel_full_ind!(
               AscentProgram_foo,
               (i32,),
               [[0]],
               par,
               (),
               (i32,),
               ascent::internal::FullRelCounter
            ) = ::std::mem::take(&mut _self.foo_indices_0);
            let mut foo_indices_0_total: ::ascent::rel::rel_full_ind!(
               AscentProgram_foo,
               (i32,),
               [[0]],
               par,
               (),
               (i32,),
               ascent::internal::FullRelCounter
            ) = Default::default();
            let mut foo_indices_0_new: ::ascent::rel::rel_full_ind!(
               AscentProgram_foo,
               (i32,),
               [[0]],
               par,
               (),
               (i32,),
               ascent::internal::FullRelCounter
            ) = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut foo_indices_0_new.to_rel_index_write(&mut __foo_ind_common_new),
               &mut foo_indices_0_delta.to_rel_index_write(&mut __foo_ind_common_delta),
               &mut foo_indices_0_total.to_rel_index_write(&mut __foo_ind_common_total),
            );
            #[allow(unused_assignments, unused_variables)]
            loop {
               let __changed = std::sync::atomic::AtomicBool::new(false);
               __bar_ind_common_total.freeze();
               __bar_ind_common_delta.freeze();
               bar_indices_0_total.freeze();
               bar_indices_0_delta.freeze();
               __baz_ind_common_total.freeze();
               __baz_ind_common_delta.freeze();
               baz_indices_0_1_total.freeze();
               baz_indices_0_1_delta.freeze();
               baz_indices_none_total.freeze();
               baz_indices_none_delta.freeze();
               __foo_ind_common_total.freeze();
               __foo_ind_common_delta.freeze();
               foo_indices_0_total.freeze();
               foo_indices_0_delta.freeze();
               ascent::internal::comment("baz <-- foo_indices_0_delta, bar_indices_0_total+delta [SIMPLE JOIN]");
               {
                  if foo_indices_0_delta.to_rel_index(&__foo_ind_common_delta).len()
                     <= ascent::internal::RelIndexCombined::new(
                        &bar_indices_0_total.to_rel_index(&__bar_ind_common_total),
                        &bar_indices_0_delta.to_rel_index(&__bar_ind_common_delta),
                     )
                     .len()
                  {
                     foo_indices_0_delta.to_rel_index(&__foo_ind_common_delta).c_iter_all().for_each(
                        |(__cl1_joined_columns, __cl1_tuple_indices)| {
                           let __cl1_joined_columns = __cl1_joined_columns.tuple_of_borrowed();
                           let x = __cl1_joined_columns.0;
                           if let Some(__matching) = ascent::internal::RelIndexCombined::new(
                              &bar_indices_0_total.to_rel_index(&__bar_ind_common_total),
                              &bar_indices_0_delta.to_rel_index(&__bar_ind_common_delta),
                           )
                           .c_index_get(&(x.clone(),))
                           {
                              __cl1_tuple_indices.for_each(|cl1_val| {
                                 __matching.clone().for_each(|__val| {
                                    let mut __dep_changed = false;
                                    let mut __default_id = 0;
                                    let __new_row: (i32, i32) = (ascent::internal::Convert::convert(x), x + 1);
                                    let mut __new_baz = 0;
                                    if !::ascent::internal::RelFullIndexRead::contains_key(
                                       &baz_indices_0_1_total.to_rel_index(&__baz_ind_common_total),
                                       &__new_row,
                                    ) && !::ascent::internal::RelFullIndexRead::contains_key(
                                       &baz_indices_0_1_delta.to_rel_index(&__baz_ind_common_delta),
                                       &__new_row,
                                    ) {
                                       if ::ascent::internal::CRelFullIndexWrite::insert_if_not_present(
                                          &baz_indices_0_1_new.to_c_rel_index_write(&__baz_ind_common_new),
                                          &__new_row,
                                          (1 as usize).into(),
                                       ) {
                                          __new_baz = _self.baz.push((__new_row.0.clone(), __new_row.1.clone()));
                                          __default_id = __new_baz;
                                          ::ascent::internal::CRelIndexWrite::index_insert(
                                             &baz_indices_none_new.to_c_rel_index_write(&__baz_ind_common_new),
                                             (),
                                             (__new_row.0.clone(), __new_row.1.clone()),
                                          );
                                          __changed.store(true, std::sync::atomic::Ordering::Relaxed);
                                       } else {
                                       }
                                    } else {
                                    }
                                 });
                              });
                           }
                        },
                     );
                  } else {
                     ascent::internal::RelIndexCombined::new(
                        &bar_indices_0_total.to_rel_index(&__bar_ind_common_total),
                        &bar_indices_0_delta.to_rel_index(&__bar_ind_common_delta),
                     )
                     .c_iter_all()
                     .for_each(|(__cl1_joined_columns, __cl1_tuple_indices)| {
                        let __cl1_joined_columns = __cl1_joined_columns.tuple_of_borrowed();
                        let x = __cl1_joined_columns.0;
                        if let Some(__matching) =
                           foo_indices_0_delta.to_rel_index(&__foo_ind_common_delta).c_index_get(&(x.clone(),))
                        {
                           __cl1_tuple_indices.for_each(|cl1_val| {
                              __matching.clone().for_each(|__val| {
                                 let mut __dep_changed = false;
                                 let mut __default_id = 0;
                                 let __new_row: (i32, i32) = (ascent::internal::Convert::convert(x), x + 1);
                                 let mut __new_baz = 0;
                                 if !::ascent::internal::RelFullIndexRead::contains_key(
                                    &baz_indices_0_1_total.to_rel_index(&__baz_ind_common_total),
                                    &__new_row,
                                 ) && !::ascent::internal::RelFullIndexRead::contains_key(
                                    &baz_indices_0_1_delta.to_rel_index(&__baz_ind_common_delta),
                                    &__new_row,
                                 ) {
                                    if ::ascent::internal::CRelFullIndexWrite::insert_if_not_present(
                                       &baz_indices_0_1_new.to_c_rel_index_write(&__baz_ind_common_new),
                                       &__new_row,
                                       (1 as usize).into(),
                                    ) {
                                       __new_baz = _self.baz.push((__new_row.0.clone(), __new_row.1.clone()));
                                       __default_id = __new_baz;
                                       ::ascent::internal::CRelIndexWrite::index_insert(
                                          &baz_indices_none_new.to_c_rel_index_write(&__baz_ind_common_new),
                                          (),
                                          (__new_row.0.clone(), __new_row.1.clone()),
                                       );
                                       __changed.store(true, std::sync::atomic::Ordering::Relaxed);
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
               ascent::internal::comment("baz <-- foo_indices_0_total, bar_indices_0_delta [SIMPLE JOIN]");
               {
                  if foo_indices_0_total.to_rel_index(&__foo_ind_common_total).len()
                     <= bar_indices_0_delta.to_rel_index(&__bar_ind_common_delta).len()
                  {
                     foo_indices_0_total.to_rel_index(&__foo_ind_common_total).c_iter_all().for_each(
                        |(__cl1_joined_columns, __cl1_tuple_indices)| {
                           let __cl1_joined_columns = __cl1_joined_columns.tuple_of_borrowed();
                           let x = __cl1_joined_columns.0;
                           if let Some(__matching) =
                              bar_indices_0_delta.to_rel_index(&__bar_ind_common_delta).c_index_get(&(x.clone(),))
                           {
                              __cl1_tuple_indices.for_each(|cl1_val| {
                                 __matching.clone().for_each(|__val| {
                                    let mut __dep_changed = false;
                                    let mut __default_id = 0;
                                    let __new_row: (i32, i32) = (ascent::internal::Convert::convert(x), x + 1);
                                    let mut __new_baz = 0;
                                    if !::ascent::internal::RelFullIndexRead::contains_key(
                                       &baz_indices_0_1_total.to_rel_index(&__baz_ind_common_total),
                                       &__new_row,
                                    ) && !::ascent::internal::RelFullIndexRead::contains_key(
                                       &baz_indices_0_1_delta.to_rel_index(&__baz_ind_common_delta),
                                       &__new_row,
                                    ) {
                                       if ::ascent::internal::CRelFullIndexWrite::insert_if_not_present(
                                          &baz_indices_0_1_new.to_c_rel_index_write(&__baz_ind_common_new),
                                          &__new_row,
                                          (1 as usize).into(),
                                       ) {
                                          __new_baz = _self.baz.push((__new_row.0.clone(), __new_row.1.clone()));
                                          __default_id = __new_baz;
                                          ::ascent::internal::CRelIndexWrite::index_insert(
                                             &baz_indices_none_new.to_c_rel_index_write(&__baz_ind_common_new),
                                             (),
                                             (__new_row.0.clone(), __new_row.1.clone()),
                                          );
                                          __changed.store(true, std::sync::atomic::Ordering::Relaxed);
                                       } else {
                                       }
                                    } else {
                                    }
                                 });
                              });
                           }
                        },
                     );
                  } else {
                     bar_indices_0_delta.to_rel_index(&__bar_ind_common_delta).c_iter_all().for_each(
                        |(__cl1_joined_columns, __cl1_tuple_indices)| {
                           let __cl1_joined_columns = __cl1_joined_columns.tuple_of_borrowed();
                           let x = __cl1_joined_columns.0;
                           if let Some(__matching) =
                              foo_indices_0_total.to_rel_index(&__foo_ind_common_total).c_index_get(&(x.clone(),))
                           {
                              __cl1_tuple_indices.for_each(|cl1_val| {
                                 __matching.clone().for_each(|__val| {
                                    let mut __dep_changed = false;
                                    let mut __default_id = 0;
                                    let __new_row: (i32, i32) = (ascent::internal::Convert::convert(x), x + 1);
                                    let mut __new_baz = 0;
                                    if !::ascent::internal::RelFullIndexRead::contains_key(
                                       &baz_indices_0_1_total.to_rel_index(&__baz_ind_common_total),
                                       &__new_row,
                                    ) && !::ascent::internal::RelFullIndexRead::contains_key(
                                       &baz_indices_0_1_delta.to_rel_index(&__baz_ind_common_delta),
                                       &__new_row,
                                    ) {
                                       if ::ascent::internal::CRelFullIndexWrite::insert_if_not_present(
                                          &baz_indices_0_1_new.to_c_rel_index_write(&__baz_ind_common_new),
                                          &__new_row,
                                          (1 as usize).into(),
                                       ) {
                                          __new_baz = _self.baz.push((__new_row.0.clone(), __new_row.1.clone()));
                                          __default_id = __new_baz;
                                          ::ascent::internal::CRelIndexWrite::index_insert(
                                             &baz_indices_none_new.to_c_rel_index_write(&__baz_ind_common_new),
                                             (),
                                             (__new_row.0.clone(), __new_row.1.clone()),
                                          );
                                          __changed.store(true, std::sync::atomic::Ordering::Relaxed);
                                       } else {
                                       }
                                    } else {
                                    }
                                 });
                              });
                           }
                        },
                     );
                  }
               }
               ascent::internal::comment("foo, bar <-- baz_indices_none_delta");
               {
                  if let Some(__matching) =
                     baz_indices_none_delta.to_rel_index(&__baz_ind_common_delta).c_index_get(&())
                  {
                     __matching.for_each(|__val| {
                        let mut __dep_changed = false;
                        let mut __default_id = 0;
                        let __val = __val.tuple_of_borrowed();
                        let x: &i32 = __val.0;
                        let y: &i32 = __val.1;
                        let __new_row: (i32,) = (ascent::internal::Convert::convert(x),);
                        let mut __new_foo = 0;
                        if !::ascent::internal::RelFullIndexRead::contains_key(
                           &foo_indices_0_total.to_rel_index(&__foo_ind_common_total),
                           &__new_row,
                        ) && !::ascent::internal::RelFullIndexRead::contains_key(
                           &foo_indices_0_delta.to_rel_index(&__foo_ind_common_delta),
                           &__new_row,
                        ) {
                           if ::ascent::internal::CRelFullIndexWrite::insert_if_not_present(
                              &foo_indices_0_new.to_c_rel_index_write(&__foo_ind_common_new),
                              &__new_row,
                              (1 as usize).into(),
                           ) {
                              __new_foo = _self.foo.push((__new_row.0,));
                              __default_id = __new_foo;
                              __changed.store(true, std::sync::atomic::Ordering::Relaxed);
                           } else {
                           }
                        } else {
                        }
                        let __new_row: (i32,) = (ascent::internal::Convert::convert(y),);
                        let mut __new_bar = 0;
                        if !::ascent::internal::RelFullIndexRead::contains_key(
                           &bar_indices_0_total.to_rel_index(&__bar_ind_common_total),
                           &__new_row,
                        ) && !::ascent::internal::RelFullIndexRead::contains_key(
                           &bar_indices_0_delta.to_rel_index(&__bar_ind_common_delta),
                           &__new_row,
                        ) {
                           if ::ascent::internal::CRelFullIndexWrite::insert_if_not_present(
                              &bar_indices_0_new.to_c_rel_index_write(&__bar_ind_common_new),
                              &__new_row,
                              (1 as usize).into(),
                           ) {
                              __new_bar = _self.bar.push((__new_row.0,));
                              __default_id = __new_bar;
                              __changed.store(true, std::sync::atomic::Ordering::Relaxed);
                           } else {
                           }
                        } else {
                        }
                     });
                  }
               }
               __bar_ind_common_total.unfreeze();
               __bar_ind_common_delta.unfreeze();
               bar_indices_0_total.unfreeze();
               bar_indices_0_delta.unfreeze();
               __baz_ind_common_total.unfreeze();
               __baz_ind_common_delta.unfreeze();
               baz_indices_0_1_total.unfreeze();
               baz_indices_0_1_delta.unfreeze();
               baz_indices_none_total.unfreeze();
               baz_indices_none_delta.unfreeze();
               __foo_ind_common_total.unfreeze();
               __foo_ind_common_delta.unfreeze();
               foo_indices_0_total.unfreeze();
               foo_indices_0_delta.unfreeze();
               ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
                  &mut __bar_ind_common_new,
                  &mut __bar_ind_common_delta,
                  &mut __bar_ind_common_total,
               );
               ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
                  &mut bar_indices_0_new.to_rel_index_write(&mut __bar_ind_common_new),
                  &mut bar_indices_0_delta.to_rel_index_write(&mut __bar_ind_common_delta),
                  &mut bar_indices_0_total.to_rel_index_write(&mut __bar_ind_common_total),
               );
               ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
                  &mut __baz_ind_common_new,
                  &mut __baz_ind_common_delta,
                  &mut __baz_ind_common_total,
               );
               ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
                  &mut baz_indices_0_1_new.to_rel_index_write(&mut __baz_ind_common_new),
                  &mut baz_indices_0_1_delta.to_rel_index_write(&mut __baz_ind_common_delta),
                  &mut baz_indices_0_1_total.to_rel_index_write(&mut __baz_ind_common_total),
               );
               ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
                  &mut baz_indices_none_new.to_rel_index_write(&mut __baz_ind_common_new),
                  &mut baz_indices_none_delta.to_rel_index_write(&mut __baz_ind_common_delta),
                  &mut baz_indices_none_total.to_rel_index_write(&mut __baz_ind_common_total),
               );
               ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
                  &mut __foo_ind_common_new,
                  &mut __foo_ind_common_delta,
                  &mut __foo_ind_common_total,
               );
               ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
                  &mut foo_indices_0_new.to_rel_index_write(&mut __foo_ind_common_new),
                  &mut foo_indices_0_delta.to_rel_index_write(&mut __foo_ind_common_delta),
                  &mut foo_indices_0_total.to_rel_index_write(&mut __foo_ind_common_total),
               );
               _self.scc_iters[2usize] += 1;
               if !__changed.load(std::sync::atomic::Ordering::Relaxed) {
                  break;
               }
               __check_return_conditions!();
            }
            _self.__bar_ind_common = __bar_ind_common_total;
            _self.bar_indices_0 = bar_indices_0_total;
            _self.__baz_ind_common = __baz_ind_common_total;
            _self.baz_indices_0_1 = baz_indices_0_1_total;
            _self.baz_indices_none = baz_indices_none_total;
            _self.__foo_ind_common = __foo_ind_common_total;
            _self.foo_indices_0 = foo_indices_0_total;
            _self.scc_times[2usize] += _scc_start_time.elapsed();
         }
      }
      #[allow(noop_method_call, suspicious_double_ref_op)]
      fn update_indices_priv(&mut self) {
         let before = ::ascent::internal::Instant::now();
         use ascent::internal::CRelIndexWrite;
         use ascent::internal::ToRelIndex0;
         use ascent::rayon::iter::{IntoParallelIterator, ParallelIterator};
         (0..self.bar.len()).into_par_iter().for_each(|_i| {
            let tuple = &self.bar[_i];
            let selection_tuple = (tuple.0.clone(),);
            let rel_ind = &self.bar_indices_0;
            ascent::internal::CRelIndexWrite::index_insert(
               &rel_ind.to_c_rel_index_write(&self.__bar_ind_common),
               selection_tuple,
               (1 as usize).into(),
            );
         });
         (0..self.baz.len()).into_par_iter().for_each(|_i| {
            let tuple = &self.baz[_i];
            let selection_tuple = (tuple.0.clone(), tuple.1.clone());
            let rel_ind = &self.baz_indices_0_1;
            ascent::internal::CRelIndexWrite::index_insert(
               &rel_ind.to_c_rel_index_write(&self.__baz_ind_common),
               selection_tuple,
               (1 as usize).into(),
            );
            let selection_tuple = ();
            let rel_ind = &self.baz_indices_none;
            ascent::internal::CRelIndexWrite::index_insert(
               &rel_ind.to_c_rel_index_write(&self.__baz_ind_common),
               selection_tuple,
               (tuple.0.clone(), tuple.1.clone()),
            );
         });
         (0..self.foo.len()).into_par_iter().for_each(|_i| {
            let tuple = &self.foo[_i];
            let selection_tuple = (tuple.0.clone(),);
            let rel_ind = &self.foo_indices_0;
            ascent::internal::CRelIndexWrite::index_insert(
               &rel_ind.to_c_rel_index_write(&self.__foo_ind_common),
               selection_tuple,
               (1 as usize).into(),
            );
         });
         self.update_indices_duration += before.elapsed();
      }
      #[deprecated = "Explicit call to update_indices not required anymore."]
      pub fn update_indices(&mut self) { self.update_indices_priv(); }
      fn type_constraints() {
         let _type_constraints: ascent::internal::TypeConstraints<i32>;
         let _par_constraints: ascent::internal::ParTypeConstraints<i32>;
      }
      pub fn summary() -> &'static str {
         "scc 0, is_looping: false:\n  foo <-- \n  dynamic relations: foo\nscc 1, is_looping: false:\n  bar <-- \n  dynamic relations: bar\nscc 2, is_looping: true:\n  baz <-- foo_indices_0_delta, bar_indices_0_total+delta [SIMPLE JOIN]\n  baz <-- foo_indices_0_total, bar_indices_0_delta [SIMPLE JOIN]\n  foo, bar <-- baz_indices_none_delta\n  dynamic relations: bar, baz, foo\n"
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
         res
      }
   }
   impl Default for AscentProgram {
      fn default() -> Self {
         let mut _self = AscentProgram {
            bar: Default::default(),
            __bar_ind_common: Default::default(),
            __bar_delete: Default::default(),
            bar_indices_0: Default::default(),
            baz: Default::default(),
            __baz_ind_common: Default::default(),
            __baz_delete: Default::default(),
            baz_indices_0_1: Default::default(),
            baz_indices_none: Default::default(),
            foo: Default::default(),
            __foo_ind_common: Default::default(),
            __foo_delete: Default::default(),
            foo_indices_0: Default::default(),
            scc_times: [std::time::Duration::ZERO; 3usize],
            scc_iters: [0; 3usize],
            update_time_nanos: Default::default(),
            update_indices_duration: std::time::Duration::default(),
         };
         _self
      }
   };
}
