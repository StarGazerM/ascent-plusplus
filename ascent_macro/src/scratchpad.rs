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
   ::ascent::rel::rel_codegen! { AscentProgram_input , (i32 , i32) , [[] , [0 , 1]] , ser , () }
   ::ascent::rel::rel_codegen! { AscentProgram_output , (i32 , i32) , [[0 , 1]] , ser , () }
   pub struct AscentProgram {
      pub input: ::ascent::rel::rel!(AscentProgram_input, (i32, i32), [[], [0, 1]], ser, ()),
      pub output: ::ascent::rel::rel!(AscentProgram_output, (i32, i32), [[0, 1]], ser, ()),
      scc_times: [std::time::Duration; 2usize],
      scc_iters: [usize; 2usize],
      pub update_time_nanos: std::sync::atomic::AtomicU64,
      pub update_indices_duration: std::time::Duration,
      pub runtime_total: AscentProgramRuntime,
      pub runtime_new: AscentProgramRuntime,
      pub runtime_delta: AscentProgramRuntime,
   }
   pub struct AscentProgramRuntime {
      #[doc = "\nlogical indices: input_indices_0_1; input_indices_none"]
      pub input: ::ascent::rel::rel!(AscentProgram_input, (i32, i32), [[], [0, 1]], ser, ()),
      pub __input_ind_common: ::ascent::rel::rel_ind_common!(AscentProgram_input, (i32, i32), [[], [0, 1]], ser, ()),
      pub input_indices_0_1:
         ::ascent::rel::rel_full_ind!(AscentProgram_input, (i32, i32), [[], [0, 1]], ser, (), (i32, i32), ()),
      pub input_indices_none:
         ::ascent::rel::rel_ind!(AscentProgram_input, (i32, i32), [[], [0, 1]], ser, (), [], (), (i32, i32)),
      #[doc = "\nlogical indices: output_indices_0_1"]
      pub output: ::ascent::rel::rel!(AscentProgram_output, (i32, i32), [[0, 1]], ser, ()),
      pub __output_ind_common: ::ascent::rel::rel_ind_common!(AscentProgram_output, (i32, i32), [[0, 1]], ser, ()),
      pub output_indices_0_1:
         ::ascent::rel::rel_full_ind!(AscentProgram_output, (i32, i32), [[0, 1]], ser, (), (i32, i32), ()),
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
         ascent::internal::comment("input <-- ");
         if true {
            let __new_row: (i32, i32) = (1, 2);
            let mut __new_input = 0;
            if !::ascent::internal::RelFullIndexRead::contains_key(
               &_self.runtime_total.input_indices_0_1.to_rel_index(&_self.runtime_total.__input_ind_common),
               &__new_row,
            ) && !::ascent::internal::RelFullIndexRead::contains_key(
               &_self.runtime_delta.input_indices_0_1.to_rel_index(&_self.runtime_delta.__input_ind_common),
               &__new_row,
            ) {
               if ::ascent::internal::RelFullIndexWrite::insert_if_not_present(
                  &mut _self
                     .runtime_new
                     .input_indices_0_1
                     .to_rel_index_write(&mut _self.runtime_new.__input_ind_common),
                  &__new_row,
                  (),
               ) {
                  __new_input = _self.input.len();
                  _self.input.push((__new_row.0.clone(), __new_row.1.clone()));
                  __default_id = __new_input;
                  ::ascent::internal::RelIndexWrite::index_insert(
                     &mut _self
                        .runtime_new
                        .input_indices_none
                        .to_rel_index_write(&mut _self.runtime_new.__input_ind_common),
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
            &mut _self.runtime_new.__input_ind_common,
            &mut _self.runtime_delta.__input_ind_common,
            &mut _self.runtime_total.__input_ind_common,
         );
         ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
            &mut _self.runtime_new.input_indices_0_1.to_rel_index_write(&mut _self.runtime_new.__input_ind_common),
            &mut _self
               .runtime_delta
               .input_indices_0_1
               .to_rel_index_write(&mut _self.runtime_delta.__input_ind_common),
            &mut _self
               .runtime_total
               .input_indices_0_1
               .to_rel_index_write(&mut _self.runtime_total.__input_ind_common),
         );
         ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
            &mut _self.runtime_new.input_indices_none.to_rel_index_write(&mut _self.runtime_new.__input_ind_common),
            &mut _self
               .runtime_delta
               .input_indices_none
               .to_rel_index_write(&mut _self.runtime_delta.__input_ind_common),
            &mut _self
               .runtime_total
               .input_indices_none
               .to_rel_index_write(&mut _self.runtime_total.__input_ind_common),
         );
         ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
            &mut _self.runtime_new.__input_ind_common,
            &mut _self.runtime_delta.__input_ind_common,
            &mut _self.runtime_total.__input_ind_common,
         );
         ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
            &mut _self.runtime_new.input_indices_0_1.to_rel_index_write(&mut _self.runtime_new.__input_ind_common),
            &mut _self
               .runtime_delta
               .input_indices_0_1
               .to_rel_index_write(&mut _self.runtime_delta.__input_ind_common),
            &mut _self
               .runtime_total
               .input_indices_0_1
               .to_rel_index_write(&mut _self.runtime_total.__input_ind_common),
         );
         ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
            &mut _self.runtime_new.input_indices_none.to_rel_index_write(&mut _self.runtime_new.__input_ind_common),
            &mut _self
               .runtime_delta
               .input_indices_none
               .to_rel_index_write(&mut _self.runtime_delta.__input_ind_common),
            &mut _self
               .runtime_total
               .input_indices_none
               .to_rel_index_write(&mut _self.runtime_total.__input_ind_common),
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
            _self.runtime_delta.__input_ind_common = ::std::mem::take(&mut _self.runtime_total.__input_ind_common);
            _self.runtime_total.__input_ind_common = Default::default();
            _self.runtime_new.__input_ind_common = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut _self.runtime_new.__input_ind_common,
               &mut _self.runtime_delta.__input_ind_common,
               &mut _self.runtime_total.__input_ind_common,
            );
            _self.runtime_delta.input_indices_0_1 = ::std::mem::take(&mut _self.runtime_total.input_indices_0_1);
            _self.runtime_total.input_indices_0_1 = Default::default();
            _self.runtime_new.input_indices_0_1 = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut _self.runtime_new.input_indices_0_1.to_rel_index_write(&mut _self.runtime_new.__input_ind_common),
               &mut _self
                  .runtime_delta
                  .input_indices_0_1
                  .to_rel_index_write(&mut _self.runtime_delta.__input_ind_common),
               &mut _self
                  .runtime_total
                  .input_indices_0_1
                  .to_rel_index_write(&mut _self.runtime_total.__input_ind_common),
            );
            _self.runtime_delta.input_indices_none = ::std::mem::take(&mut _self.runtime_total.input_indices_none);
            _self.runtime_total.input_indices_none = Default::default();
            _self.runtime_new.input_indices_none = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut _self.runtime_new.input_indices_none.to_rel_index_write(&mut _self.runtime_new.__input_ind_common),
               &mut _self
                  .runtime_delta
                  .input_indices_none
                  .to_rel_index_write(&mut _self.runtime_delta.__input_ind_common),
               &mut _self
                  .runtime_total
                  .input_indices_none
                  .to_rel_index_write(&mut _self.runtime_total.__input_ind_common),
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
         ascent::internal::comment("output <-- input_indices_none_total");
         if true {
            if let Some(__matching) = _self
               .runtime_total
               .input_indices_none
               .to_rel_index(&_self.runtime_total.__input_ind_common)
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
                  let mut __new_output = 0;
                  if !::ascent::internal::RelFullIndexRead::contains_key(
                     &_self.runtime_total.output_indices_0_1.to_rel_index(&_self.runtime_total.__output_ind_common),
                     &__new_row,
                  ) && !::ascent::internal::RelFullIndexRead::contains_key(
                     &_self.runtime_delta.output_indices_0_1.to_rel_index(&_self.runtime_delta.__output_ind_common),
                     &__new_row,
                  ) {
                     if ::ascent::internal::RelFullIndexWrite::insert_if_not_present(
                        &mut _self
                           .runtime_new
                           .output_indices_0_1
                           .to_rel_index_write(&mut _self.runtime_new.__output_ind_common),
                        &__new_row,
                        (),
                     ) {
                        __new_output = _self.output.len();
                        _self.output.push((__new_row.0, __new_row.1));
                        __default_id = __new_output;
                        __changed = true;
                     } else {
                     }
                  } else {
                  }
               });
            }
         }
         ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
            &mut _self.runtime_new.__output_ind_common,
            &mut _self.runtime_delta.__output_ind_common,
            &mut _self.runtime_total.__output_ind_common,
         );
         ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
            &mut _self.runtime_new.output_indices_0_1.to_rel_index_write(&mut _self.runtime_new.__output_ind_common),
            &mut _self
               .runtime_delta
               .output_indices_0_1
               .to_rel_index_write(&mut _self.runtime_delta.__output_ind_common),
            &mut _self
               .runtime_total
               .output_indices_0_1
               .to_rel_index_write(&mut _self.runtime_total.__output_ind_common),
         );
         ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
            &mut _self.runtime_new.__output_ind_common,
            &mut _self.runtime_delta.__output_ind_common,
            &mut _self.runtime_total.__output_ind_common,
         );
         ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
            &mut _self.runtime_new.output_indices_0_1.to_rel_index_write(&mut _self.runtime_new.__output_ind_common),
            &mut _self
               .runtime_delta
               .output_indices_0_1
               .to_rel_index_write(&mut _self.runtime_delta.__output_ind_common),
            &mut _self
               .runtime_total
               .output_indices_0_1
               .to_rel_index_write(&mut _self.runtime_total.__output_ind_common),
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
            _self.runtime_delta.__output_ind_common = ::std::mem::take(&mut _self.runtime_total.__output_ind_common);
            _self.runtime_total.__output_ind_common = Default::default();
            _self.runtime_new.__output_ind_common = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut _self.runtime_new.__output_ind_common,
               &mut _self.runtime_delta.__output_ind_common,
               &mut _self.runtime_total.__output_ind_common,
            );
            _self.runtime_delta.output_indices_0_1 = ::std::mem::take(&mut _self.runtime_total.output_indices_0_1);
            _self.runtime_total.output_indices_0_1 = Default::default();
            _self.runtime_new.output_indices_0_1 = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut _self.runtime_new.output_indices_0_1.to_rel_index_write(&mut _self.runtime_new.__output_ind_common),
               &mut _self
                  .runtime_delta
                  .output_indices_0_1
                  .to_rel_index_write(&mut _self.runtime_delta.__output_ind_common),
               &mut _self
                  .runtime_total
                  .output_indices_0_1
                  .to_rel_index_write(&mut _self.runtime_total.__output_ind_common),
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
         _self.output.clear();
         _self.runtime_total.output_indices_0_1.clear();
         _self.runtime_new.output_indices_0_1.clear();
         _self.runtime_delta.output_indices_0_1.clear();
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
         _self.input.clear();
         _self.runtime_total.input_indices_0_1.clear();
         _self.runtime_new.input_indices_0_1.clear();
         _self.runtime_delta.input_indices_0_1.clear();
         _self.input.clear();
         _self.runtime_total.input_indices_none.0.clear();
         _self.runtime_new.input_indices_none.0.clear();
         _self.runtime_delta.input_indices_none.0.clear();
         true
      }
      #[allow(noop_method_call, suspicious_double_ref_op)]
      pub fn update_indices_priv(&mut self) {
         let before = ::ascent::internal::Instant::now();
         self.update_indices_input();
         self.update_indices_output();
         self.update_indices_duration += before.elapsed();
      }
      #[allow(noop_method_call, suspicious_double_ref_op)]
      pub fn update_indices_input(&mut self) {
         use ascent::internal::RelIndexWrite;
         use ascent::internal::ToRelIndex0;
         for (_i, tuple) in self.input.iter().enumerate() {
            let selection_tuple = (tuple.0.clone(), tuple.1.clone());
            let rel_ind = &mut self.runtime_total.input_indices_0_1;
            ascent::internal::RelIndexWrite::index_insert(
               &mut rel_ind.to_rel_index_write(&mut self.runtime_total.__input_ind_common),
               selection_tuple,
               (),
            );
            let selection_tuple = ();
            let rel_ind = &mut self.runtime_total.input_indices_none;
            ascent::internal::RelIndexWrite::index_insert(
               &mut rel_ind.to_rel_index_write(&mut self.runtime_total.__input_ind_common),
               selection_tuple,
               (tuple.0.clone(), tuple.1.clone()),
            );
         }
      }
      #[allow(noop_method_call, suspicious_double_ref_op)]
      pub fn update_indices_output(&mut self) {
         use ascent::internal::RelIndexWrite;
         use ascent::internal::ToRelIndex0;
         for (_i, tuple) in self.output.iter().enumerate() {
            let selection_tuple = (tuple.0.clone(), tuple.1.clone());
            let rel_ind = &mut self.runtime_total.output_indices_0_1;
            ascent::internal::RelIndexWrite::index_insert(
               &mut rel_ind.to_rel_index_write(&mut self.runtime_total.__output_ind_common),
               selection_tuple,
               (),
            );
         }
      }
      #[deprecated = "Explicit call to update_indices not required anymore."]
      pub fn update_indices(&mut self) { self.update_indices_priv(); }
      fn type_constraints() { let _type_constraints: ascent::internal::TypeConstraints<i32>; }
      pub fn summary() -> &'static str {
         "scc 0, is_looping: false:\n  input <-- \n  dynamic relations: input\nscc 1, is_looping: false:\n  output <-- input_indices_none_total\n  dynamic relations: output\n"
      }
      pub fn relation_sizes_summary(&self) -> String {
         use std::fmt::Write;
         let mut res = String::new();
         writeln!(&mut res, "{} size: {}", "input", self.input.len()).unwrap();
         writeln!(&mut res, "{} size: {}", "output", self.output.len()).unwrap();
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
            input: Default::default(),
            __input_ind_common: Default::default(),
            input_indices_0_1: Default::default(),
            input_indices_none: Default::default(),
            output: Default::default(),
            __output_ind_common: Default::default(),
            output_indices_0_1: Default::default(),
         };
         _self
      }
   }
   impl Default for AscentProgram {
      fn default() -> Self {
         let mut _self = AscentProgram {
            input: Default::default(),
            output: Default::default(),
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
