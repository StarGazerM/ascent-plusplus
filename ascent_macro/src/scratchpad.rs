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
   ::ascent::rel::rel_codegen! { ExtTest_edge , (i32 , i32) , [[0 , 1]] , ser , () }
   ::ascent::rel::rel_codegen! { ExtTest_path , (i32 , i32) , [[] , [0 , 1]] , ser , () }
   struct ExtTest {
      #[doc = "\nlogical indices: edge_indices_0_1"]
      pub edge: ::ascent::rel::rel!(ExtTest_edge, (i32, i32), [[0, 1]], ser, ()),
      pub __edge_ind_common: ::ascent::rel::rel_ind_common!(ExtTest_edge, (i32, i32), [[0, 1]], ser, ()),
      pub edge_indices_0_1: ::ascent::rel::rel_full_ind!(ExtTest_edge, (i32, i32), [[0, 1]], ser, (), (i32, i32), ()),
      scc_times: [std::time::Duration; 1usize],
      scc_iters: [usize; 1usize],
      pub update_time_nanos: std::sync::atomic::AtomicU64,
      pub update_indices_duration: std::time::Duration,
      pub runtime_total: ExtTestRuntime,
      pub runtime_new: ExtTestRuntime,
      pub runtime_delta: ExtTestRuntime,
      pub tc: std::rc::Rc<TC>,
   }
   struct ExtTestRuntime {
      #[doc = "\nlogical indices: edge_indices_0_1"]
      pub edge: ::ascent::rel::rel!(ExtTest_edge, (i32, i32), [[0, 1]], ser, ()),
      pub __edge_ind_common: ::ascent::rel::rel_ind_common!(ExtTest_edge, (i32, i32), [[0, 1]], ser, ()),
      pub edge_indices_0_1: ::ascent::rel::rel_full_ind!(ExtTest_edge, (i32, i32), [[0, 1]], ser, (), (i32, i32), ()),
   }
   impl ExtTest {
      #[allow(unused_imports, noop_method_call, suspicious_double_ref_op)]
      #[doc = "Runs the Ascent program to a fixed point."]
      pub fn run(&mut self) { self.run_with_init_flag(true); }
      pub fn run_with_init_flag(&mut self, init_flag: bool) {
         macro_rules! __check_return_conditions {
            () => {};
         }
         use ascent::internal::RelIndexWrite;
         use ascent::internal::{RelIndexRead, RelIndexReadAll, ToRelIndex0, TupleOfBorrowed};
         use core::cmp::PartialEq;
         if init_flag {
            self.update_indices_priv()
         };
         let _self = self;
         ascent::internal::comment("scc 0");
         {
            let _scc_start_time = ::ascent::internal::Instant::now();
            _self.runtime_delta.__edge_ind_common = ::std::mem::take(&mut _self.__edge_ind_common);
            _self.runtime_total.__edge_ind_common = Default::default();
            _self.runtime_new.__edge_ind_common = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut _self.runtime_new.__edge_ind_common,
               &mut _self.runtime_delta.__edge_ind_common,
               &mut _self.runtime_total.__edge_ind_common,
            );
            _self.runtime_delta.edge_indices_0_1 = ::std::mem::take(&mut _self.edge_indices_0_1);
            _self.runtime_total.edge_indices_0_1 = Default::default();
            _self.runtime_new.edge_indices_0_1 = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut _self.runtime_new.edge_indices_0_1.to_rel_index_write(&mut _self.runtime_new.__edge_ind_common),
               &mut _self.runtime_delta.edge_indices_0_1.to_rel_index_write(&mut _self.runtime_delta.__edge_ind_common),
               &mut _self.runtime_total.edge_indices_0_1.to_rel_index_write(&mut _self.runtime_total.__edge_ind_common),
            );
            #[allow(unused_assignments, unused_variables)]
            {
               let mut __changed = false;
               let mut __default_id = 0;
               ascent::internal::comment("edge <-- path_indices_none_total");
               if true {
                  if let Some(__matching) = _self
                     .tc
                     .runtime_total
                     .path_indices_none
                     .to_rel_index(&_self.tc.runtime_total.__path_ind_common)
                     .index_get(&())
                  {
                     __matching.for_each(|__val| {
                        let mut __dep_changed = false;
                        let mut __default_id = 0;
                        let __val = __val.tuple_of_borrowed();
                        let x: &i32 = __val.0;
                        let y: &i32 = __val.1;
                        let __new_row: (i32, i32) =
                           (ascent::internal::Convert::convert(x), ascent::internal::Convert::convert(y));
                        let mut __new_edge = 0;
                        if !::ascent::internal::RelFullIndexRead::contains_key(
                           &_self.runtime_total.edge_indices_0_1.to_rel_index(&_self.runtime_total.__edge_ind_common),
                           &__new_row,
                        ) && !::ascent::internal::RelFullIndexRead::contains_key(
                           &_self.runtime_delta.edge_indices_0_1.to_rel_index(&_self.runtime_delta.__edge_ind_common),
                           &__new_row,
                        ) {
                           if ::ascent::internal::RelFullIndexWrite::insert_if_not_present(
                              &mut _self
                                 .runtime_new
                                 .edge_indices_0_1
                                 .to_rel_index_write(&mut _self.runtime_new.__edge_ind_common),
                              &__new_row,
                              (),
                           ) {
                              __new_edge = _self.edge.len();
                              _self.edge.push((__new_row.0, __new_row.1));
                              __default_id = __new_edge;
                              __changed = true;
                           } else {
                           }
                        } else {
                        }
                     });
                  }
               }
               ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
                  &mut _self.runtime_new.__edge_ind_common,
                  &mut _self.runtime_delta.__edge_ind_common,
                  &mut _self.runtime_total.__edge_ind_common,
               );
               ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
                  &mut _self.runtime_new.edge_indices_0_1.to_rel_index_write(&mut _self.runtime_new.__edge_ind_common),
                  &mut _self
                     .runtime_delta
                     .edge_indices_0_1
                     .to_rel_index_write(&mut _self.runtime_delta.__edge_ind_common),
                  &mut _self
                     .runtime_total
                     .edge_indices_0_1
                     .to_rel_index_write(&mut _self.runtime_total.__edge_ind_common),
               );
               ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
                  &mut _self.runtime_new.__edge_ind_common,
                  &mut _self.runtime_delta.__edge_ind_common,
                  &mut _self.runtime_total.__edge_ind_common,
               );
               ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
                  &mut _self.runtime_new.edge_indices_0_1.to_rel_index_write(&mut _self.runtime_new.__edge_ind_common),
                  &mut _self
                     .runtime_delta
                     .edge_indices_0_1
                     .to_rel_index_write(&mut _self.runtime_delta.__edge_ind_common),
                  &mut _self
                     .runtime_total
                     .edge_indices_0_1
                     .to_rel_index_write(&mut _self.runtime_total.__edge_ind_common),
               );
               _self.scc_iters[0usize] += 1;
               __check_return_conditions!();
            }
            _self.__edge_ind_common = std::mem::take(&mut _self.runtime_total.__edge_ind_common);
            _self.edge_indices_0_1 = std::mem::take(&mut _self.runtime_total.edge_indices_0_1);
            _self.scc_times[0usize] += _scc_start_time.elapsed();
         }
      }
      #[allow(noop_method_call, suspicious_double_ref_op)]
      pub fn update_indices_priv(&mut self) {
         let before = ::ascent::internal::Instant::now();
         self.update_indices_edge();
         self.update_indices_duration += before.elapsed();
      }
      #[allow(noop_method_call, suspicious_double_ref_op)]
      pub fn update_indices_edge(&mut self) {
         use ascent::internal::RelIndexWrite;
         use ascent::internal::ToRelIndex0;
         for (_i, tuple) in self.edge.iter().enumerate() {
            let selection_tuple = (tuple.0.clone(), tuple.1.clone());
            let rel_ind = &mut self.edge_indices_0_1;
            ascent::internal::RelIndexWrite::index_insert(
               &mut rel_ind.to_rel_index_write(&mut self.__edge_ind_common),
               selection_tuple,
               (),
            );
         }
      }
      #[deprecated = "Explicit call to update_indices not required anymore."]
      pub fn update_indices(&mut self) { self.update_indices_priv(); }
      fn type_constraints() { let _type_constraints: ascent::internal::TypeConstraints<i32>; }
      pub fn summary() -> &'static str {
         "scc 0, is_looping: false:\n  edge <-- path_indices_none_total\n  dynamic relations: edge\n"
      }
      pub fn relation_sizes_summary(&self) -> String {
         use std::fmt::Write;
         let mut res = String::new();
         writeln!(&mut res, "{} size: {}", "edge", self.edge.len()).unwrap();
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
   impl Default for ExtTestRuntime {
      fn default() -> Self {
         let mut _self = ExtTestRuntime {
            edge: Default::default(),
            __edge_ind_common: Default::default(),
            edge_indices_0_1: Default::default(),
         };
         _self
      }
   }
   impl Default for ExtTest {
      fn default() -> Self {
         let mut _self = ExtTest {
            edge: Default::default(),
            __edge_ind_common: Default::default(),
            edge_indices_0_1: Default::default(),
            scc_times: [std::time::Duration::ZERO; 1usize],
            scc_iters: [0; 1usize],
            update_time_nanos: Default::default(),
            update_indices_duration: std::time::Duration::default(),
            runtime_total: Default::default(),
            runtime_new: Default::default(),
            runtime_delta: Default::default(),
            tc: std::rc::Rc::new(Default::default()),
         };
         _self
      }
   }
   ::ascent::rel::rel_codegen! { TC_edge , (i32 , i32) , [[0 , 1]] , ser , () }
   ::ascent::rel::rel_codegen! { TC_path , (i32 , i32) , [[] , [0 , 1]] , ser , () }
   struct TC {
      #[doc = "\nlogical indices: edge_indices_0_1"]
      pub edge: ::ascent::rel::rel!(TC_edge, (i32, i32), [[0, 1]], ser, ()),
      pub __edge_ind_common: ::ascent::rel::rel_ind_common!(TC_edge, (i32, i32), [[0, 1]], ser, ()),
      pub edge_indices_0_1: ::ascent::rel::rel_full_ind!(TC_edge, (i32, i32), [[0, 1]], ser, (), (i32, i32), ()),
      #[doc = "\nlogical indices: path_indices_0_1; path_indices_none"]
      pub path: ::ascent::rel::rel!(TC_path, (i32, i32), [[], [0, 1]], ser, ()),
      pub __path_ind_common: ::ascent::rel::rel_ind_common!(TC_path, (i32, i32), [[], [0, 1]], ser, ()),
      pub path_indices_0_1: ::ascent::rel::rel_full_ind!(TC_path, (i32, i32), [[], [0, 1]], ser, (), (i32, i32), ()),
      pub path_indices_none: ::ascent::rel::rel_ind!(TC_path, (i32, i32), [[], [0, 1]], ser, (), [], (), (i32, i32)),
      scc_times: [std::time::Duration; 0usize],
      scc_iters: [usize; 0usize],
      pub update_time_nanos: std::sync::atomic::AtomicU64,
      pub update_indices_duration: std::time::Duration,
      pub runtime_total: TCRuntime,
      pub runtime_new: TCRuntime,
      pub runtime_delta: TCRuntime,
   }
   struct TCRuntime {
      #[doc = "\nlogical indices: edge_indices_0_1"]
      pub edge: ::ascent::rel::rel!(TC_edge, (i32, i32), [[0, 1]], ser, ()),
      pub __edge_ind_common: ::ascent::rel::rel_ind_common!(TC_edge, (i32, i32), [[0, 1]], ser, ()),
      pub edge_indices_0_1: ::ascent::rel::rel_full_ind!(TC_edge, (i32, i32), [[0, 1]], ser, (), (i32, i32), ()),
      #[doc = "\nlogical indices: path_indices_0_1; path_indices_none"]
      pub path: ::ascent::rel::rel!(TC_path, (i32, i32), [[], [0, 1]], ser, ()),
      pub __path_ind_common: ::ascent::rel::rel_ind_common!(TC_path, (i32, i32), [[], [0, 1]], ser, ()),
      pub path_indices_0_1: ::ascent::rel::rel_full_ind!(TC_path, (i32, i32), [[], [0, 1]], ser, (), (i32, i32), ()),
      pub path_indices_none: ::ascent::rel::rel_ind!(TC_path, (i32, i32), [[], [0, 1]], ser, (), [], (), (i32, i32)),
   }
   impl TC {
      #[allow(unused_imports, noop_method_call, suspicious_double_ref_op)]
      #[doc = "Runs the Ascent program to a fixed point."]
      pub fn run(&mut self) { self.run_with_init_flag(true); }
      pub fn run_with_init_flag(&mut self, init_flag: bool) {
         macro_rules! __check_return_conditions {
            () => {};
         }
         use ascent::internal::RelIndexWrite;
         use ascent::internal::{RelIndexRead, RelIndexReadAll, ToRelIndex0, TupleOfBorrowed};
         use core::cmp::PartialEq;
         if init_flag {
            self.update_indices_priv()
         };
         let _self = self;
      }
      #[allow(noop_method_call, suspicious_double_ref_op)]
      pub fn update_indices_priv(&mut self) {
         let before = ::ascent::internal::Instant::now();
         self.update_indices_edge();
         self.update_indices_path();
         self.update_indices_duration += before.elapsed();
      }
      #[allow(noop_method_call, suspicious_double_ref_op)]
      pub fn update_indices_edge(&mut self) {
         use ascent::internal::RelIndexWrite;
         use ascent::internal::ToRelIndex0;
         for (_i, tuple) in self.edge.iter().enumerate() {
            let selection_tuple = (tuple.0.clone(), tuple.1.clone());
            let rel_ind = &mut self.edge_indices_0_1;
            ascent::internal::RelIndexWrite::index_insert(
               &mut rel_ind.to_rel_index_write(&mut self.__edge_ind_common),
               selection_tuple,
               (),
            );
         }
      }
      #[allow(noop_method_call, suspicious_double_ref_op)]
      pub fn update_indices_path(&mut self) {
         use ascent::internal::RelIndexWrite;
         use ascent::internal::ToRelIndex0;
         for (_i, tuple) in self.path.iter().enumerate() {
            let selection_tuple = (tuple.0.clone(), tuple.1.clone());
            let rel_ind = &mut self.path_indices_0_1;
            ascent::internal::RelIndexWrite::index_insert(
               &mut rel_ind.to_rel_index_write(&mut self.__path_ind_common),
               selection_tuple,
               (),
            );
            let selection_tuple = ();
            let rel_ind = &mut self.path_indices_none;
            ascent::internal::RelIndexWrite::index_insert(
               &mut rel_ind.to_rel_index_write(&mut self.__path_ind_common),
               selection_tuple,
               (tuple.0.clone(), tuple.1.clone()),
            );
         }
      }
      #[deprecated = "Explicit call to update_indices not required anymore."]
      pub fn update_indices(&mut self) { self.update_indices_priv(); }
      fn type_constraints() { let _type_constraints: ascent::internal::TypeConstraints<i32>; }
      pub fn summary() -> &'static str { "" }
      pub fn relation_sizes_summary(&self) -> String {
         use std::fmt::Write;
         let mut res = String::new();
         writeln!(&mut res, "{} size: {}", "edge", self.edge.len()).unwrap();
         writeln!(&mut res, "{} size: {}", "path", self.path.len()).unwrap();
         res
      }
      pub fn scc_times_summary(&self) -> String {
         use std::fmt::Write;
         let mut res = String::new();
         writeln!(&mut res, "update_indices time: {:?}", self.update_indices_duration).unwrap();
         res
      }
   }
   impl Default for TCRuntime {
      fn default() -> Self {
         let mut _self = TCRuntime {
            edge: Default::default(),
            __edge_ind_common: Default::default(),
            edge_indices_0_1: Default::default(),
            path: Default::default(),
            __path_ind_common: Default::default(),
            path_indices_0_1: Default::default(),
            path_indices_none: Default::default(),
         };
         _self
      }
   }
   impl Default for TC {
      fn default() -> Self {
         let mut _self = TC {
            edge: Default::default(),
            __edge_ind_common: Default::default(),
            edge_indices_0_1: Default::default(),
            path: Default::default(),
            __path_ind_common: Default::default(),
            path_indices_0_1: Default::default(),
            path_indices_none: Default::default(),
            scc_times: [std::time::Duration::ZERO; 0usize],
            scc_iters: [0; 0usize],
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
