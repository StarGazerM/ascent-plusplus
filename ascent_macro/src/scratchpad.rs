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
   ::ascent::rel::rel_codegen! { EquivTest_foo , (usize ,) , [[] , [0]] , ser , () }
   ::ascent::rel::rel_codegen! { EquivTest_bar , (usize ,) , [[] , [0]] , ser , () }
   struct EquivTest {
      pub bar: ::ascent::rel::rel!(EquivTest_bar, (usize,), [[], [0]], ser, ()),
      pub foo: ::ascent::rel::rel!(EquivTest_foo, (usize,), [[], [0]], ser, ()),
      scc_times: [std::time::Duration; 3usize],
      scc_iters: [usize; 3usize],
      pub update_time_nanos: std::sync::atomic::AtomicU64,
      pub update_indices_duration: std::time::Duration,
      pub runtime_total: EquivTestRuntime,
      pub runtime_new: EquivTestRuntime,
      pub runtime_delta: EquivTestRuntime,
      pub equiv_ids_: ascent::union_find::EqRel<usize>,
   }
   struct EquivTestRuntime {
      #[doc = "\nlogical indices: bar_indices_0; bar_indices_none"]
      pub bar: ::ascent::rel::rel!(EquivTest_bar, (usize,), [[], [0]], ser, ()),
      pub __bar_ind_common: ::ascent::rel::rel_ind_common!(EquivTest_bar, (usize,), [[], [0]], ser, ()),
      pub bar_indices_0: ::ascent::rel::rel_full_ind!(EquivTest_bar, (usize,), [[], [0]], ser, (), (usize,), ()),
      pub bar_indices_none: ::ascent::rel::rel_ind!(EquivTest_bar, (usize,), [[], [0]], ser, (), [], (), (usize,)),
      #[doc = "\nlogical indices: foo_indices_0; foo_indices_none"]
      pub foo: ::ascent::rel::rel!(EquivTest_foo, (usize,), [[], [0]], ser, ()),
      pub __foo_ind_common: ::ascent::rel::rel_ind_common!(EquivTest_foo, (usize,), [[], [0]], ser, ()),
      pub foo_indices_0: ::ascent::rel::rel_full_ind!(EquivTest_foo, (usize,), [[], [0]], ser, (), (usize,), ()),
      pub foo_indices_none: ::ascent::rel::rel_ind!(EquivTest_foo, (usize,), [[], [0]], ser, (), [], (), (usize,)),
   }
   impl EquivTest {
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
            let __new_row: (usize,) = (1,);
            let mut __new_foo = 0;
            if !::ascent::internal::RelFullIndexRead::contains_key(
               &_self.runtime_total.foo_indices_0.to_rel_index(&_self.runtime_total.__foo_ind_common),
               &__new_row,
            ) && !::ascent::internal::RelFullIndexRead::contains_key(
               &_self.runtime_delta.foo_indices_0.to_rel_index(&_self.runtime_delta.__foo_ind_common),
               &__new_row,
            ) {
               __new_foo = {
                  use std::hash::{Hash, Hasher};
                  let mut hasher = ::std::hash::DefaultHasher::new();
                  (__new_row.clone(), "foo").hash(&mut hasher);
                  hasher.finish() as usize
               };
               if ::ascent::internal::RelFullIndexWrite::insert_if_not_present(
                  &mut _self.runtime_new.foo_indices_0.to_rel_index_write(&mut _self.runtime_new.__foo_ind_common),
                  &__new_row,
                  (),
               ) {
                  let __new_row_to_be_pushed = (__new_row.0.clone(),);
                  __new_foo = _self.foo.len();
                  _self.foo.push(__new_row_to_be_pushed);
                  __default_id = __new_foo;
                  ::ascent::internal::RelIndexWrite::index_insert(
                     &mut _self
                        .runtime_new
                        .foo_indices_none
                        .to_rel_index_write(&mut _self.runtime_new.__foo_ind_common),
                     (),
                     (__new_row.0.clone(),),
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
            &mut _self.runtime_new.foo_indices_0.to_rel_index_write(&mut _self.runtime_new.__foo_ind_common),
            &mut _self.runtime_delta.foo_indices_0.to_rel_index_write(&mut _self.runtime_delta.__foo_ind_common),
            &mut _self.runtime_total.foo_indices_0.to_rel_index_write(&mut _self.runtime_total.__foo_ind_common),
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
            &mut _self.runtime_new.foo_indices_0.to_rel_index_write(&mut _self.runtime_new.__foo_ind_common),
            &mut _self.runtime_delta.foo_indices_0.to_rel_index_write(&mut _self.runtime_delta.__foo_ind_common),
            &mut _self.runtime_total.foo_indices_0.to_rel_index_write(&mut _self.runtime_total.__foo_ind_common),
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
            _self.runtime_delta.__foo_ind_common = ::std::mem::take(&mut _self.runtime_total.__foo_ind_common);
            _self.runtime_total.__foo_ind_common = Default::default();
            _self.runtime_new.__foo_ind_common = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut _self.runtime_new.__foo_ind_common,
               &mut _self.runtime_delta.__foo_ind_common,
               &mut _self.runtime_total.__foo_ind_common,
            );
            _self.runtime_delta.foo_indices_0 = ::std::mem::take(&mut _self.runtime_total.foo_indices_0);
            _self.runtime_total.foo_indices_0 = Default::default();
            _self.runtime_new.foo_indices_0 = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut _self.runtime_new.foo_indices_0.to_rel_index_write(&mut _self.runtime_new.__foo_ind_common),
               &mut _self.runtime_delta.foo_indices_0.to_rel_index_write(&mut _self.runtime_delta.__foo_ind_common),
               &mut _self.runtime_total.foo_indices_0.to_rel_index_write(&mut _self.runtime_total.__foo_ind_common),
            );
            _self.runtime_delta.foo_indices_none = ::std::mem::take(&mut _self.runtime_total.foo_indices_none);
            _self.runtime_total.foo_indices_none = Default::default();
            _self.runtime_new.foo_indices_none = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut _self.runtime_new.foo_indices_none.to_rel_index_write(&mut _self.runtime_new.__foo_ind_common),
               &mut _self.runtime_delta.foo_indices_none.to_rel_index_write(&mut _self.runtime_delta.__foo_ind_common),
               &mut _self.runtime_total.foo_indices_none.to_rel_index_write(&mut _self.runtime_total.__foo_ind_common),
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
         ascent::internal::comment("bar <-- ");
         if true {
            let __new_row: (usize,) = (2,);
            let mut __new_bar = 0;
            if !::ascent::internal::RelFullIndexRead::contains_key(
               &_self.runtime_total.bar_indices_0.to_rel_index(&_self.runtime_total.__bar_ind_common),
               &__new_row,
            ) && !::ascent::internal::RelFullIndexRead::contains_key(
               &_self.runtime_delta.bar_indices_0.to_rel_index(&_self.runtime_delta.__bar_ind_common),
               &__new_row,
            ) {
               __new_bar = {
                  use std::hash::{Hash, Hasher};
                  let mut hasher = ::std::hash::DefaultHasher::new();
                  (__new_row.clone(), "bar").hash(&mut hasher);
                  hasher.finish() as usize
               };
               if ::ascent::internal::RelFullIndexWrite::insert_if_not_present(
                  &mut _self.runtime_new.bar_indices_0.to_rel_index_write(&mut _self.runtime_new.__bar_ind_common),
                  &__new_row,
                  (),
               ) {
                  let __new_row_to_be_pushed = (__new_row.0.clone(),);
                  __new_bar = _self.bar.len();
                  _self.bar.push(__new_row_to_be_pushed);
                  __default_id = __new_bar;
                  ::ascent::internal::RelIndexWrite::index_insert(
                     &mut _self
                        .runtime_new
                        .bar_indices_none
                        .to_rel_index_write(&mut _self.runtime_new.__bar_ind_common),
                     (),
                     (__new_row.0.clone(),),
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
            &mut _self.runtime_new.bar_indices_none.to_rel_index_write(&mut _self.runtime_new.__bar_ind_common),
            &mut _self.runtime_delta.bar_indices_none.to_rel_index_write(&mut _self.runtime_delta.__bar_ind_common),
            &mut _self.runtime_total.bar_indices_none.to_rel_index_write(&mut _self.runtime_total.__bar_ind_common),
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
            &mut _self.runtime_new.bar_indices_none.to_rel_index_write(&mut _self.runtime_new.__bar_ind_common),
            &mut _self.runtime_delta.bar_indices_none.to_rel_index_write(&mut _self.runtime_delta.__bar_ind_common),
            &mut _self.runtime_total.bar_indices_none.to_rel_index_write(&mut _self.runtime_total.__bar_ind_common),
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
            _self.runtime_delta.bar_indices_0 = ::std::mem::take(&mut _self.runtime_total.bar_indices_0);
            _self.runtime_total.bar_indices_0 = Default::default();
            _self.runtime_new.bar_indices_0 = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut _self.runtime_new.bar_indices_0.to_rel_index_write(&mut _self.runtime_new.__bar_ind_common),
               &mut _self.runtime_delta.bar_indices_0.to_rel_index_write(&mut _self.runtime_delta.__bar_ind_common),
               &mut _self.runtime_total.bar_indices_0.to_rel_index_write(&mut _self.runtime_total.__bar_ind_common),
            );
            _self.runtime_delta.bar_indices_none = ::std::mem::take(&mut _self.runtime_total.bar_indices_none);
            _self.runtime_total.bar_indices_none = Default::default();
            _self.runtime_new.bar_indices_none = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut _self.runtime_new.bar_indices_none.to_rel_index_write(&mut _self.runtime_new.__bar_ind_common),
               &mut _self.runtime_delta.bar_indices_none.to_rel_index_write(&mut _self.runtime_delta.__bar_ind_common),
               &mut _self.runtime_total.bar_indices_none.to_rel_index_write(&mut _self.runtime_total.__bar_ind_common),
            );
            _self.scc_1_exec();
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
         ascent::internal::comment(
            "inflated_a <=> rep_b <-- foo_indices_none_total, a <=> inflated_a, bar_indices_none_total, b <=> rep_b",
         );
         if true {
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
                  let a: &usize = __val.0;
                  let __iter_items_41441: Vec<_> =
                     if let Some(set) = _self.equiv_ids_.set_of(a) { set.cloned().collect() } else { vec![a.clone()] };
                  for inflated_a in __iter_items_41441 {
                     if let Some(__matching) = _self
                        .runtime_total
                        .bar_indices_none
                        .to_rel_index(&_self.runtime_total.__bar_ind_common)
                        .index_get(&())
                     {
                        __matching.for_each(|__val| {
                           let mut __dep_changed = false;
                           let mut __default_id = 0;
                           let __val = __val.tuple_of_borrowed();
                           let b: &usize = __val.0;
                           let rep_b = _self.equiv_ids_.get_dominant_elem(b).unwrap_or(b);
                           let __src_var_repr_inflated_a = _self.equiv_ids_.elem_set(&inflated_a);
                           let __dst_var_repr_rep_b = _self.equiv_ids_.elem_set(&rep_b);
                           if let Some(__src_var_repr_inflated_a) = __src_var_repr_inflated_a {
                              if let Some(__dst_var_repr_rep_b) = __dst_var_repr_rep_b {
                                 _self.equiv_ids_.add(__src_var_repr_inflated_a, __dst_var_repr_rep_b);
                              } else {
                                 _self.equiv_ids_.add(__src_var_repr_inflated_a, rep_b.clone());
                              }
                           } else {
                              _self.equiv_ids_.add(inflated_a.clone(), rep_b.clone());
                           }
                           __changed = true;
                        });
                     }
                  }
               });
            }
         }
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
            _self.scc_2_exec();
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
         let res = _self.scc_2();
         if !res {
            return false;
         }
         true
      }
      #[allow(noop_method_call, suspicious_double_ref_op)]
      pub fn update_indices_priv(&mut self) {
         let before = ::ascent::internal::Instant::now();
         self.update_indices_bar();
         self.update_indices_foo();
         self.update_indices_duration += before.elapsed();
      }
      #[allow(noop_method_call, suspicious_double_ref_op)]
      pub fn update_indices_bar(&mut self) {
         use ascent::internal::RelIndexWrite;
         use ascent::internal::ToRelIndex0;
         for (_i, tuple) in self.bar.iter().enumerate() {
            let selection_tuple = (tuple.0.clone(),);
            let rel_ind = &mut self.runtime_total.bar_indices_0;
            ascent::internal::RelIndexWrite::index_insert(
               &mut rel_ind.to_rel_index_write(&mut self.runtime_total.__bar_ind_common),
               selection_tuple,
               (),
            );
            let selection_tuple = ();
            let rel_ind = &mut self.runtime_total.bar_indices_none;
            ascent::internal::RelIndexWrite::index_insert(
               &mut rel_ind.to_rel_index_write(&mut self.runtime_total.__bar_ind_common),
               selection_tuple,
               (tuple.0.clone(),),
            );
         }
      }
      #[allow(noop_method_call, suspicious_double_ref_op)]
      pub fn update_indices_foo(&mut self) {
         use ascent::internal::RelIndexWrite;
         use ascent::internal::ToRelIndex0;
         for (_i, tuple) in self.foo.iter().enumerate() {
            let selection_tuple = (tuple.0.clone(),);
            let rel_ind = &mut self.runtime_total.foo_indices_0;
            ascent::internal::RelIndexWrite::index_insert(
               &mut rel_ind.to_rel_index_write(&mut self.runtime_total.__foo_ind_common),
               selection_tuple,
               (),
            );
            let selection_tuple = ();
            let rel_ind = &mut self.runtime_total.foo_indices_none;
            ascent::internal::RelIndexWrite::index_insert(
               &mut rel_ind.to_rel_index_write(&mut self.runtime_total.__foo_ind_common),
               selection_tuple,
               (tuple.0.clone(),),
            );
         }
      }
      #[deprecated = "Explicit call to update_indices not required anymore."]
      pub fn update_indices(&mut self) { self.update_indices_priv(); }
      fn type_constraints() { let _type_constraints: ascent::internal::TypeConstraints<usize>; }
      pub fn summary() -> &'static str {
         "scc 0, is_looping: false:\n  foo <-- \n  dynamic relations: foo\nscc 1, is_looping: false:\n  bar <-- \n  dynamic relations: bar\nscc 2, is_looping: false:\n  inflated_a <=> rep_b <-- foo_indices_none_total, a <=> inflated_a, bar_indices_none_total, b <=> rep_b\n  dynamic relations: \n"
      }
      pub fn relation_sizes_summary(&self) -> String {
         use std::fmt::Write;
         let mut res = String::new();
         writeln!(&mut res, "{} size: {}", "bar", self.bar.len()).unwrap();
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
   impl Default for EquivTestRuntime {
      fn default() -> Self {
         let mut _self = EquivTestRuntime {
            bar: Default::default(),
            __bar_ind_common: Default::default(),
            bar_indices_0: Default::default(),
            bar_indices_none: Default::default(),
            foo: Default::default(),
            __foo_ind_common: Default::default(),
            foo_indices_0: Default::default(),
            foo_indices_none: Default::default(),
         };
         _self
      }
   }
   impl Default for EquivTest {
      fn default() -> Self {
         let mut _self = EquivTest {
            bar: Default::default(),
            foo: Default::default(),
            scc_times: [std::time::Duration::ZERO; 3usize],
            scc_iters: [0; 3usize],
            update_time_nanos: Default::default(),
            update_indices_duration: std::time::Duration::default(),
            runtime_total: Default::default(),
            runtime_new: Default::default(),
            runtime_delta: Default::default(),
            equiv_ids_: Default::default(),
         };
         _self
      }
   };
}
