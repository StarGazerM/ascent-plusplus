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
   ::ascent::rel::rel_codegen! { AscentProgram_foo , (i32 , i32 , usize) , [[0 , 1 , 2] , [1]] , ser , () }
   ::ascent::rel::rel_codegen! { AscentProgram_bar , (i32 , i32 , usize) , [[0] , [0 , 1 , 2]] , ser , () }
   pub struct AscentProgram {
      pub bar: ::ascent::rel::rel!(AscentProgram_bar, (i32, i32, usize), [[0], [0, 1, 2]], ser, ()),
      pub foo: ::ascent::rel::rel!(AscentProgram_foo, (i32, i32, usize), [[0, 1, 2], [1]], ser, ()),
      scc_times: [std::time::Duration; 1usize],
      scc_iters: [usize; 1usize],
      pub update_time_nanos: std::sync::atomic::AtomicU64,
      pub update_indices_duration: std::time::Duration,
      pub runtime_total: AscentProgramRuntime,
      pub runtime_new: AscentProgramRuntime,
      pub runtime_delta: AscentProgramRuntime,
      pub equiv_ids_: ascent::union_find::EqRel<usize>,
   }
   pub struct AscentProgramRuntime {
      #[doc = "\nlogical indices: bar_indices_0; bar_indices_0_1_2"]
      pub bar: ::ascent::rel::rel!(AscentProgram_bar, (i32, i32, usize), [[0], [0, 1, 2]], ser, ()),
      pub __bar_ind_common:
         ::ascent::rel::rel_ind_common!(AscentProgram_bar, (i32, i32, usize), [[0], [0, 1, 2]], ser, ()),
      pub bar_indices_0: ::ascent::rel::rel_ind!(
         AscentProgram_bar,
         (i32, i32, usize),
         [[0], [0, 1, 2]],
         ser,
         (),
         [0],
         (i32,),
         (i32, usize)
      ),
      pub bar_indices_0_1_2: ::ascent::rel::rel_full_ind!(
         AscentProgram_bar,
         (i32, i32, usize),
         [[0], [0, 1, 2]],
         ser,
         (),
         (i32, i32, usize),
         ()
      ),
      #[doc = "\nlogical indices: foo_indices_0_1_2; foo_indices_1"]
      pub foo: ::ascent::rel::rel!(AscentProgram_foo, (i32, i32, usize), [[0, 1, 2], [1]], ser, ()),
      pub __foo_ind_common:
         ::ascent::rel::rel_ind_common!(AscentProgram_foo, (i32, i32, usize), [[0, 1, 2], [1]], ser, ()),
      pub foo_indices_0_1_2: ::ascent::rel::rel_full_ind!(
         AscentProgram_foo,
         (i32, i32, usize),
         [[0, 1, 2], [1]],
         ser,
         (),
         (i32, i32, usize),
         ()
      ),
      pub foo_indices_1: ::ascent::rel::rel_ind!(
         AscentProgram_foo,
         (i32, i32, usize),
         [[0, 1, 2], [1]],
         ser,
         (),
         [1],
         (i32,),
         (i32, usize)
      ),
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
         ascent::internal::comment("i1 <=> i2 <-- foo_indices_1_total, bar_indices_0_total [SIMPLE JOIN]");
         if true {
            if _self.runtime_total.foo_indices_1.to_rel_index(&_self.runtime_total.__foo_ind_common).len()
               <= _self.runtime_total.bar_indices_0.to_rel_index(&_self.runtime_total.__bar_ind_common).len()
            {
               _self
                  .runtime_total
                  .foo_indices_1
                  .to_rel_index(&_self.runtime_total.__foo_ind_common)
                  .iter_all()
                  .for_each(|(__cl1_joined_columns, __cl1_tuple_indices)| {
                     let mut __gen_bang = false;
                     let __cl1_joined_columns = __cl1_joined_columns.tuple_of_borrowed();
                     let b = __cl1_joined_columns.0;
                     if let Some(__matching) = _self
                        .runtime_total
                        .bar_indices_0
                        .to_rel_index(&_self.runtime_total.__bar_ind_common)
                        .index_get(&(b.clone(),))
                     {
                        __cl1_tuple_indices.for_each(|cl1_val| {
                           let cl1_val = cl1_val.tuple_of_borrowed();
                           let a: &i32 = cl1_val.0;
                           let i1: &usize = cl1_val.1;
                           __matching.clone().for_each(|__val| {
                              let mut __dep_changed = false;
                              let mut __default_id = 0;
                              let __val = __val.tuple_of_borrowed();
                              let c: &i32 = __val.0;
                              let i2: &usize = __val.1;
                              _self.equiv_ids_.add(*i1, *i2);
                              __changed = true;
                           });
                        });
                     }
                  });
            } else {
               _self
                  .runtime_total
                  .bar_indices_0
                  .to_rel_index(&_self.runtime_total.__bar_ind_common)
                  .iter_all()
                  .for_each(|(__cl1_joined_columns, __cl1_tuple_indices)| {
                     let mut __gen_bang = false;
                     let __cl1_joined_columns = __cl1_joined_columns.tuple_of_borrowed();
                     let b = __cl1_joined_columns.0;
                     if let Some(__matching) = _self
                        .runtime_total
                        .foo_indices_1
                        .to_rel_index(&_self.runtime_total.__foo_ind_common)
                        .index_get(&(b.clone(),))
                     {
                        __cl1_tuple_indices.for_each(|cl1_val| {
                           let cl1_val = cl1_val.tuple_of_borrowed();
                           let c: &i32 = cl1_val.0;
                           let i2: &usize = cl1_val.1;
                           __matching.clone().for_each(|__val| {
                              let mut __dep_changed = false;
                              let mut __default_id = 0;
                              let __val = __val.tuple_of_borrowed();
                              let a: &i32 = __val.0;
                              let i1: &usize = __val.1;
                              _self.equiv_ids_.add(*i1, *i2);
                              __changed = true;
                           });
                        });
                     }
                  });
            }
         }
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
            _self.scc_0_exec();
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
               (tuple.1.clone(), tuple.2.clone()),
            );
            let selection_tuple = (tuple.0.clone(), tuple.1.clone(), tuple.2.clone());
            let rel_ind = &mut self.runtime_total.bar_indices_0_1_2;
            ascent::internal::RelIndexWrite::index_insert(
               &mut rel_ind.to_rel_index_write(&mut self.runtime_total.__bar_ind_common),
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
            let selection_tuple = (tuple.0.clone(), tuple.1.clone(), tuple.2.clone());
            let rel_ind = &mut self.runtime_total.foo_indices_0_1_2;
            ascent::internal::RelIndexWrite::index_insert(
               &mut rel_ind.to_rel_index_write(&mut self.runtime_total.__foo_ind_common),
               selection_tuple,
               (),
            );
            let selection_tuple = (tuple.1.clone(),);
            let rel_ind = &mut self.runtime_total.foo_indices_1;
            ascent::internal::RelIndexWrite::index_insert(
               &mut rel_ind.to_rel_index_write(&mut self.runtime_total.__foo_ind_common),
               selection_tuple,
               (tuple.0.clone(), tuple.2.clone()),
            );
         }
      }
      #[deprecated = "Explicit call to update_indices not required anymore."]
      pub fn update_indices(&mut self) { self.update_indices_priv(); }
      fn type_constraints() {
         let _type_constraints: ascent::internal::TypeConstraints<i32>;
         let _type_constraints: ascent::internal::TypeConstraints<usize>;
      }
      pub fn summary() -> &'static str {
         "scc 0, is_looping: false:\n  i1 <=> i2 <-- foo_indices_1_total, bar_indices_0_total [SIMPLE JOIN]\n  dynamic relations: \n"
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
         res
      }
   }
   impl Default for AscentProgramRuntime {
      fn default() -> Self {
         let mut _self = AscentProgramRuntime {
            bar: Default::default(),
            __bar_ind_common: Default::default(),
            bar_indices_0: Default::default(),
            bar_indices_0_1_2: Default::default(),
            foo: Default::default(),
            __foo_ind_common: Default::default(),
            foo_indices_0_1_2: Default::default(),
            foo_indices_1: Default::default(),
         };
         _self
      }
   }
   impl Default for AscentProgram {
      fn default() -> Self {
         let mut _self = AscentProgram {
            bar: Default::default(),
            foo: Default::default(),
            scc_times: [std::time::Duration::ZERO; 1usize],
            scc_iters: [0; 1usize],
            update_time_nanos: Default::default(),
            update_indices_duration: std::time::Duration::default(),
            runtime_total: Default::default(),
            runtime_new: Default::default(),
            runtime_delta: Default::default(),
            equiv_ids_: ascent::union_find::EqRel::default(),
         };
         _self
      }
   };
}
