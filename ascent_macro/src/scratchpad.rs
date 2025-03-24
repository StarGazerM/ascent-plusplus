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
   ::ascent::rel::rel_codegen! { TCWhere_edge_raw , (i32 , i32) , [[] , [0 , 1]] , par , () }
   ::ascent::rel::rel_codegen! { TCWhere_provenance , (Tag , Tag) , [[0 , 1]] , par , () }
   ::ascent::rel::rel_codegen! { TCWhere_path , (i32 , i32) , [[0 , 1]] , par , () }
   ::ascent::rel::rel_codegen! { TCWhere_edge , (i32 , i32) , [[0 , 1]] , par , () }
   ::ascent::rel::rel_codegen! { TCWhere_edge_id , (i32 , i32 , usize) , [[] , [0 , 1 , 2] , [1]] , par , () }
   ::ascent::rel::rel_codegen! { TCWhere_path_id , (i32 , i32 , usize) , [[0] , [0 , 1 , 2]] , par , () }
   struct TCWhere {
      pub edge: ::ascent::rel::rel!(TCWhere_edge, (i32, i32), [[0, 1]], par, ()),
      pub edge_id: ::ascent::rel::rel!(TCWhere_edge_id, (i32, i32, usize), [[], [0, 1, 2], [1]], par, ()),
      pub edge_raw: ::ascent::rel::rel!(TCWhere_edge_raw, (i32, i32), [[], [0, 1]], par, ()),
      pub path: ::ascent::rel::rel!(TCWhere_path, (i32, i32), [[0, 1]], par, ()),
      pub path_id: ::ascent::rel::rel!(TCWhere_path_id, (i32, i32, usize), [[0], [0, 1, 2]], par, ()),
      pub provenance: ::ascent::rel::rel!(TCWhere_provenance, (Tag, Tag), [[0, 1]], par, ()),
      scc_times: [std::time::Duration; 3usize],
      scc_iters: [usize; 3usize],
      pub update_time_nanos: std::sync::atomic::AtomicU64,
      pub update_indices_duration: std::time::Duration,
      pub runtime_total: TCWhereRuntime,
      pub runtime_new: TCWhereRuntime,
      pub runtime_delta: TCWhereRuntime,
   }
   struct TCWhereRuntime {
      #[doc = "\nlogical indices: edge_indices_0_1"]
      pub edge: ::ascent::rel::rel!(TCWhere_edge, (i32, i32), [[0, 1]], par, ()),
      pub __edge_ind_common: ::ascent::rel::rel_ind_common!(TCWhere_edge, (i32, i32), [[0, 1]], par, ()),
      pub edge_indices_0_1: ::ascent::rel::rel_full_ind!(TCWhere_edge, (i32, i32), [[0, 1]], par, (), (i32, i32), ()),
      #[doc = "\nlogical indices: edge_id_indices_0_1_2; edge_id_indices_1; edge_id_indices_none"]
      pub edge_id: ::ascent::rel::rel!(TCWhere_edge_id, (i32, i32, usize), [[], [0, 1, 2], [1]], par, ()),
      pub __edge_id_ind_common:
         ::ascent::rel::rel_ind_common!(TCWhere_edge_id, (i32, i32, usize), [[], [0, 1, 2], [1]], par, ()),
      pub edge_id_indices_0_1_2: ::ascent::rel::rel_full_ind!(
         TCWhere_edge_id,
         (i32, i32, usize),
         [[], [0, 1, 2], [1]],
         par,
         (),
         (i32, i32, usize),
         ()
      ),
      pub edge_id_indices_1: ::ascent::rel::rel_ind!(
         TCWhere_edge_id,
         (i32, i32, usize),
         [[], [0, 1, 2], [1]],
         par,
         (),
         [1],
         (i32,),
         (i32, usize)
      ),
      pub edge_id_indices_none: ::ascent::rel::rel_ind!(
         TCWhere_edge_id,
         (i32, i32, usize),
         [[], [0, 1, 2], [1]],
         par,
         (),
         [],
         (),
         (i32, i32, usize)
      ),
      #[doc = "\nlogical indices: edge_raw_indices_0_1; edge_raw_indices_none"]
      pub edge_raw: ::ascent::rel::rel!(TCWhere_edge_raw, (i32, i32), [[], [0, 1]], par, ()),
      pub __edge_raw_ind_common: ::ascent::rel::rel_ind_common!(TCWhere_edge_raw, (i32, i32), [[], [0, 1]], par, ()),
      pub edge_raw_indices_0_1:
         ::ascent::rel::rel_full_ind!(TCWhere_edge_raw, (i32, i32), [[], [0, 1]], par, (), (i32, i32), ()),
      pub edge_raw_indices_none:
         ::ascent::rel::rel_ind!(TCWhere_edge_raw, (i32, i32), [[], [0, 1]], par, (), [], (), (i32, i32)),
      #[doc = "\nlogical indices: path_indices_0_1"]
      pub path: ::ascent::rel::rel!(TCWhere_path, (i32, i32), [[0, 1]], par, ()),
      pub __path_ind_common: ::ascent::rel::rel_ind_common!(TCWhere_path, (i32, i32), [[0, 1]], par, ()),
      pub path_indices_0_1: ::ascent::rel::rel_full_ind!(TCWhere_path, (i32, i32), [[0, 1]], par, (), (i32, i32), ()),
      #[doc = "\nlogical indices: path_id_indices_0; path_id_indices_0_1_2"]
      pub path_id: ::ascent::rel::rel!(TCWhere_path_id, (i32, i32, usize), [[0], [0, 1, 2]], par, ()),
      pub __path_id_ind_common:
         ::ascent::rel::rel_ind_common!(TCWhere_path_id, (i32, i32, usize), [[0], [0, 1, 2]], par, ()),
      pub path_id_indices_0: ::ascent::rel::rel_ind!(
         TCWhere_path_id,
         (i32, i32, usize),
         [[0], [0, 1, 2]],
         par,
         (),
         [0],
         (i32,),
         (i32, usize)
      ),
      pub path_id_indices_0_1_2: ::ascent::rel::rel_full_ind!(
         TCWhere_path_id,
         (i32, i32, usize),
         [[0], [0, 1, 2]],
         par,
         (),
         (i32, i32, usize),
         ()
      ),
      #[doc = "\nlogical indices: provenance_indices_0_1"]
      pub provenance: ::ascent::rel::rel!(TCWhere_provenance, (Tag, Tag), [[0, 1]], par, ()),
      pub __provenance_ind_common: ::ascent::rel::rel_ind_common!(TCWhere_provenance, (Tag, Tag), [[0, 1]], par, ()),
      pub provenance_indices_0_1:
         ::ascent::rel::rel_full_ind!(TCWhere_provenance, (Tag, Tag), [[0, 1]], par, (), (Tag, Tag), ()),
   }
   impl TCWhere {
      #[allow(unused_assignments, unused_variables, dead_code)]
      pub fn scc_0_exec(&mut self) -> bool {
         use ascent::internal::CRelIndexRead;
         use ascent::internal::CRelIndexReadAll;
         use ascent::internal::CRelIndexWrite;
         use ascent::internal::Freezable;
         use ascent::internal::{RelIndexRead, RelIndexReadAll, ToRelIndex0, TupleOfBorrowed};
         use ascent::rayon::iter::ParallelBridge;
         use ascent::rayon::iter::ParallelIterator;
         use core::cmp::PartialEq;
         let _self = self;
         let _scc_start_time = ::ascent::internal::Instant::now();
         let __changed = std::sync::atomic::AtomicBool::new(false);
         let mut __default_id = 0;
         _self.runtime_total.__edge_ind_common.freeze();
         _self.runtime_delta.__edge_ind_common.freeze();
         _self.runtime_total.edge_indices_0_1.freeze();
         _self.runtime_delta.edge_indices_0_1.freeze();
         _self.runtime_total.__edge_id_ind_common.freeze();
         _self.runtime_delta.__edge_id_ind_common.freeze();
         _self.runtime_total.edge_id_indices_0_1_2.freeze();
         _self.runtime_delta.edge_id_indices_0_1_2.freeze();
         _self.runtime_total.edge_id_indices_1.freeze();
         _self.runtime_delta.edge_id_indices_1.freeze();
         _self.runtime_total.edge_id_indices_none.freeze();
         _self.runtime_delta.edge_id_indices_none.freeze();
         ascent::internal::comment("edge, edge_id <-- edge_raw_indices_none_total");
         if true {
            if let Some(__matching) = _self
               .runtime_total
               .edge_raw_indices_none
               .to_rel_index(&_self.runtime_total.__edge_raw_ind_common)
               .c_index_get(&())
            {
               __matching.for_each(|__val| {
                  let mut __dep_changed = false;
                  let mut __default_id = 0;
                  let __val = __val.tuple_of_borrowed();
                  let x: &i32 = __val.0;
                  let y: &i32 = __val.1;
                  let __new_row: (i32, i32) =
                     (ascent::internal::Convert::convert(x), ascent::internal::Convert::convert(y));
                  let mut eid = 0;
                  if !::ascent::internal::RelFullIndexRead::contains_key(
                     &_self.runtime_total.edge_indices_0_1.to_rel_index(&_self.runtime_total.__edge_ind_common),
                     &__new_row,
                  ) && !::ascent::internal::RelFullIndexRead::contains_key(
                     &_self.runtime_delta.edge_indices_0_1.to_rel_index(&_self.runtime_delta.__edge_ind_common),
                     &__new_row,
                  ) {
                     eid = {
                        use std::hash::{Hash, Hasher};
                        let mut hasher = ::std::hash::DefaultHasher::new();
                        __new_row.hash(&mut hasher);
                        hasher.finish() as usize
                     };
                     if ::ascent::internal::CRelFullIndexWrite::insert_if_not_present(
                        &_self.runtime_new.edge_indices_0_1.to_c_rel_index_write(&_self.runtime_new.__edge_ind_common),
                        &__new_row,
                        (),
                     ) {
                        let __new_row_to_be_pushed = (__new_row.0, __new_row.1);
                        _self.edge.push(__new_row_to_be_pushed);
                        __default_id = eid;
                        __changed.store(true, std::sync::atomic::Ordering::Relaxed);
                     } else {
                        return;
                     }
                  } else {
                     return;
                  }
                  let __new_row: (i32, i32, usize) = (
                     ascent::internal::Convert::convert(x),
                     ascent::internal::Convert::convert(y),
                     ascent::internal::Convert::convert(eid),
                  );
                  let mut __new_edge_id = 0;
                  if !::ascent::internal::RelFullIndexRead::contains_key(
                     &_self.runtime_total.edge_id_indices_0_1_2.to_rel_index(&_self.runtime_total.__edge_id_ind_common),
                     &__new_row,
                  ) && !::ascent::internal::RelFullIndexRead::contains_key(
                     &_self.runtime_delta.edge_id_indices_0_1_2.to_rel_index(&_self.runtime_delta.__edge_id_ind_common),
                     &__new_row,
                  ) {
                     __new_edge_id = {
                        use std::hash::{Hash, Hasher};
                        let mut hasher = ::std::hash::DefaultHasher::new();
                        __new_row.hash(&mut hasher);
                        hasher.finish() as usize
                     };
                     if ::ascent::internal::CRelFullIndexWrite::insert_if_not_present(
                        &_self
                           .runtime_new
                           .edge_id_indices_0_1_2
                           .to_c_rel_index_write(&_self.runtime_new.__edge_id_ind_common),
                        &__new_row,
                        (),
                     ) {
                        let __new_row_to_be_pushed = (__new_row.0.clone(), __new_row.1.clone(), __new_row.2.clone());
                        __new_edge_id = _self.edge_id.push(__new_row_to_be_pushed);
                        __default_id = __new_edge_id;
                        ::ascent::internal::CRelIndexWrite::index_insert(
                           &_self
                              .runtime_new
                              .edge_id_indices_1
                              .to_c_rel_index_write(&_self.runtime_new.__edge_id_ind_common),
                           (__new_row.1.clone(),),
                           (__new_row.0.clone(), __new_row.2.clone()),
                        );
                        ::ascent::internal::CRelIndexWrite::index_insert(
                           &_self
                              .runtime_new
                              .edge_id_indices_none
                              .to_c_rel_index_write(&_self.runtime_new.__edge_id_ind_common),
                           (),
                           (__new_row.0.clone(), __new_row.1.clone(), __new_row.2.clone()),
                        );
                        __changed.store(true, std::sync::atomic::Ordering::Relaxed);
                     } else {
                     }
                  } else {
                  }
               });
            }
         }
         _self.runtime_total.__edge_ind_common.unfreeze();
         _self.runtime_delta.__edge_ind_common.unfreeze();
         _self.runtime_total.edge_indices_0_1.unfreeze();
         _self.runtime_delta.edge_indices_0_1.unfreeze();
         _self.runtime_total.__edge_id_ind_common.unfreeze();
         _self.runtime_delta.__edge_id_ind_common.unfreeze();
         _self.runtime_total.edge_id_indices_0_1_2.unfreeze();
         _self.runtime_delta.edge_id_indices_0_1_2.unfreeze();
         _self.runtime_total.edge_id_indices_1.unfreeze();
         _self.runtime_delta.edge_id_indices_1.unfreeze();
         _self.runtime_total.edge_id_indices_none.unfreeze();
         _self.runtime_delta.edge_id_indices_none.unfreeze();
         ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
            &mut _self.runtime_new.__edge_ind_common,
            &mut _self.runtime_delta.__edge_ind_common,
            &mut _self.runtime_total.__edge_ind_common,
         );
         ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
            &mut _self.runtime_new.edge_indices_0_1.to_rel_index_write(&mut _self.runtime_new.__edge_ind_common),
            &mut _self.runtime_delta.edge_indices_0_1.to_rel_index_write(&mut _self.runtime_delta.__edge_ind_common),
            &mut _self.runtime_total.edge_indices_0_1.to_rel_index_write(&mut _self.runtime_total.__edge_ind_common),
         );
         ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
            &mut _self.runtime_new.__edge_id_ind_common,
            &mut _self.runtime_delta.__edge_id_ind_common,
            &mut _self.runtime_total.__edge_id_ind_common,
         );
         ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
            &mut _self
               .runtime_new
               .edge_id_indices_0_1_2
               .to_rel_index_write(&mut _self.runtime_new.__edge_id_ind_common),
            &mut _self
               .runtime_delta
               .edge_id_indices_0_1_2
               .to_rel_index_write(&mut _self.runtime_delta.__edge_id_ind_common),
            &mut _self
               .runtime_total
               .edge_id_indices_0_1_2
               .to_rel_index_write(&mut _self.runtime_total.__edge_id_ind_common),
         );
         ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
            &mut _self.runtime_new.edge_id_indices_1.to_rel_index_write(&mut _self.runtime_new.__edge_id_ind_common),
            &mut _self
               .runtime_delta
               .edge_id_indices_1
               .to_rel_index_write(&mut _self.runtime_delta.__edge_id_ind_common),
            &mut _self
               .runtime_total
               .edge_id_indices_1
               .to_rel_index_write(&mut _self.runtime_total.__edge_id_ind_common),
         );
         ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
            &mut _self
               .runtime_new
               .edge_id_indices_none
               .to_rel_index_write(&mut _self.runtime_new.__edge_id_ind_common),
            &mut _self
               .runtime_delta
               .edge_id_indices_none
               .to_rel_index_write(&mut _self.runtime_delta.__edge_id_ind_common),
            &mut _self
               .runtime_total
               .edge_id_indices_none
               .to_rel_index_write(&mut _self.runtime_total.__edge_id_ind_common),
         );
         ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
            &mut _self.runtime_new.__edge_ind_common,
            &mut _self.runtime_delta.__edge_ind_common,
            &mut _self.runtime_total.__edge_ind_common,
         );
         ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
            &mut _self.runtime_new.edge_indices_0_1.to_rel_index_write(&mut _self.runtime_new.__edge_ind_common),
            &mut _self.runtime_delta.edge_indices_0_1.to_rel_index_write(&mut _self.runtime_delta.__edge_ind_common),
            &mut _self.runtime_total.edge_indices_0_1.to_rel_index_write(&mut _self.runtime_total.__edge_ind_common),
         );
         ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
            &mut _self.runtime_new.__edge_id_ind_common,
            &mut _self.runtime_delta.__edge_id_ind_common,
            &mut _self.runtime_total.__edge_id_ind_common,
         );
         ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
            &mut _self
               .runtime_new
               .edge_id_indices_0_1_2
               .to_rel_index_write(&mut _self.runtime_new.__edge_id_ind_common),
            &mut _self
               .runtime_delta
               .edge_id_indices_0_1_2
               .to_rel_index_write(&mut _self.runtime_delta.__edge_id_ind_common),
            &mut _self
               .runtime_total
               .edge_id_indices_0_1_2
               .to_rel_index_write(&mut _self.runtime_total.__edge_id_ind_common),
         );
         ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
            &mut _self.runtime_new.edge_id_indices_1.to_rel_index_write(&mut _self.runtime_new.__edge_id_ind_common),
            &mut _self
               .runtime_delta
               .edge_id_indices_1
               .to_rel_index_write(&mut _self.runtime_delta.__edge_id_ind_common),
            &mut _self
               .runtime_total
               .edge_id_indices_1
               .to_rel_index_write(&mut _self.runtime_total.__edge_id_ind_common),
         );
         ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
            &mut _self
               .runtime_new
               .edge_id_indices_none
               .to_rel_index_write(&mut _self.runtime_new.__edge_id_ind_common),
            &mut _self
               .runtime_delta
               .edge_id_indices_none
               .to_rel_index_write(&mut _self.runtime_delta.__edge_id_ind_common),
            &mut _self
               .runtime_total
               .edge_id_indices_none
               .to_rel_index_write(&mut _self.runtime_total.__edge_id_ind_common),
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
            use ascent::internal::CRelIndexRead;
            use ascent::internal::CRelIndexReadAll;
            use ascent::internal::CRelIndexWrite;
            use ascent::internal::Freezable;
            use ascent::internal::{RelIndexRead, RelIndexReadAll, ToRelIndex0, TupleOfBorrowed};
            use ascent::rayon::iter::ParallelBridge;
            use ascent::rayon::iter::ParallelIterator;
            use core::cmp::PartialEq;
            _self.runtime_delta.__edge_ind_common = ::std::mem::take(&mut _self.runtime_total.__edge_ind_common);
            _self.runtime_total.__edge_ind_common = Default::default();
            _self.runtime_new.__edge_ind_common = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut _self.runtime_new.__edge_ind_common,
               &mut _self.runtime_delta.__edge_ind_common,
               &mut _self.runtime_total.__edge_ind_common,
            );
            _self.runtime_delta.edge_indices_0_1 = ::std::mem::take(&mut _self.runtime_total.edge_indices_0_1);
            _self.runtime_total.edge_indices_0_1 = Default::default();
            _self.runtime_new.edge_indices_0_1 = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut _self.runtime_new.edge_indices_0_1.to_rel_index_write(&mut _self.runtime_new.__edge_ind_common),
               &mut _self.runtime_delta.edge_indices_0_1.to_rel_index_write(&mut _self.runtime_delta.__edge_ind_common),
               &mut _self.runtime_total.edge_indices_0_1.to_rel_index_write(&mut _self.runtime_total.__edge_ind_common),
            );
            _self.runtime_delta.__edge_id_ind_common = ::std::mem::take(&mut _self.runtime_total.__edge_id_ind_common);
            _self.runtime_total.__edge_id_ind_common = Default::default();
            _self.runtime_new.__edge_id_ind_common = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut _self.runtime_new.__edge_id_ind_common,
               &mut _self.runtime_delta.__edge_id_ind_common,
               &mut _self.runtime_total.__edge_id_ind_common,
            );
            _self.runtime_delta.edge_id_indices_0_1_2 =
               ::std::mem::take(&mut _self.runtime_total.edge_id_indices_0_1_2);
            _self.runtime_total.edge_id_indices_0_1_2 = Default::default();
            _self.runtime_new.edge_id_indices_0_1_2 = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut _self
                  .runtime_new
                  .edge_id_indices_0_1_2
                  .to_rel_index_write(&mut _self.runtime_new.__edge_id_ind_common),
               &mut _self
                  .runtime_delta
                  .edge_id_indices_0_1_2
                  .to_rel_index_write(&mut _self.runtime_delta.__edge_id_ind_common),
               &mut _self
                  .runtime_total
                  .edge_id_indices_0_1_2
                  .to_rel_index_write(&mut _self.runtime_total.__edge_id_ind_common),
            );
            _self.runtime_delta.edge_id_indices_1 = ::std::mem::take(&mut _self.runtime_total.edge_id_indices_1);
            _self.runtime_total.edge_id_indices_1 = Default::default();
            _self.runtime_new.edge_id_indices_1 = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut _self.runtime_new.edge_id_indices_1.to_rel_index_write(&mut _self.runtime_new.__edge_id_ind_common),
               &mut _self
                  .runtime_delta
                  .edge_id_indices_1
                  .to_rel_index_write(&mut _self.runtime_delta.__edge_id_ind_common),
               &mut _self
                  .runtime_total
                  .edge_id_indices_1
                  .to_rel_index_write(&mut _self.runtime_total.__edge_id_ind_common),
            );
            _self.runtime_delta.edge_id_indices_none = ::std::mem::take(&mut _self.runtime_total.edge_id_indices_none);
            _self.runtime_total.edge_id_indices_none = Default::default();
            _self.runtime_new.edge_id_indices_none = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut _self
                  .runtime_new
                  .edge_id_indices_none
                  .to_rel_index_write(&mut _self.runtime_new.__edge_id_ind_common),
               &mut _self
                  .runtime_delta
                  .edge_id_indices_none
                  .to_rel_index_write(&mut _self.runtime_delta.__edge_id_ind_common),
               &mut _self
                  .runtime_total
                  .edge_id_indices_none
                  .to_rel_index_write(&mut _self.runtime_total.__edge_id_ind_common),
            );
            _self.runtime_total.__edge_raw_ind_common.freeze();
            _self.runtime_total.edge_raw_indices_none.freeze();
            _self.scc_0_exec();
         }
         true
      }
      #[allow(unused_assignments, unused_variables, dead_code)]
      pub fn scc_1_exec(&mut self) -> bool {
         use ascent::internal::CRelIndexRead;
         use ascent::internal::CRelIndexReadAll;
         use ascent::internal::CRelIndexWrite;
         use ascent::internal::Freezable;
         use ascent::internal::{RelIndexRead, RelIndexReadAll, ToRelIndex0, TupleOfBorrowed};
         use ascent::rayon::iter::ParallelBridge;
         use ascent::rayon::iter::ParallelIterator;
         use core::cmp::PartialEq;
         let _self = self;
         let _scc_start_time = ::ascent::internal::Instant::now();
         let __changed = std::sync::atomic::AtomicBool::new(false);
         let mut __default_id = 0;
         _self.runtime_total.__path_ind_common.freeze();
         _self.runtime_delta.__path_ind_common.freeze();
         _self.runtime_total.path_indices_0_1.freeze();
         _self.runtime_delta.path_indices_0_1.freeze();
         _self.runtime_total.__path_id_ind_common.freeze();
         _self.runtime_delta.__path_id_ind_common.freeze();
         _self.runtime_total.path_id_indices_0.freeze();
         _self.runtime_delta.path_id_indices_0.freeze();
         _self.runtime_total.path_id_indices_0_1_2.freeze();
         _self.runtime_delta.path_id_indices_0_1_2.freeze();
         _self.runtime_total.__provenance_ind_common.freeze();
         _self.runtime_delta.__provenance_ind_common.freeze();
         _self.runtime_total.provenance_indices_0_1.freeze();
         _self.runtime_delta.provenance_indices_0_1.freeze();
         ascent::internal::comment("path, path_id, provenance <-- edge_id_indices_none_total");
         if true {
            if let Some(__matching) = _self
               .runtime_total
               .edge_id_indices_none
               .to_rel_index(&_self.runtime_total.__edge_id_ind_common)
               .c_index_get(&())
            {
               __matching.for_each(|__val| {
                  let mut __dep_changed = false;
                  let mut __default_id = 0;
                  let __val = __val.tuple_of_borrowed();
                  let x: &i32 = __val.0;
                  let y: &i32 = __val.1;
                  let eid: &usize = __val.2;
                  let __new_row: (i32, i32) =
                     (ascent::internal::Convert::convert(x), ascent::internal::Convert::convert(y));
                  let mut new_id = 0;
                  if !::ascent::internal::RelFullIndexRead::contains_key(
                     &_self.runtime_total.path_indices_0_1.to_rel_index(&_self.runtime_total.__path_ind_common),
                     &__new_row,
                  ) && !::ascent::internal::RelFullIndexRead::contains_key(
                     &_self.runtime_delta.path_indices_0_1.to_rel_index(&_self.runtime_delta.__path_ind_common),
                     &__new_row,
                  ) {
                     new_id = {
                        use std::hash::{Hash, Hasher};
                        let mut hasher = ::std::hash::DefaultHasher::new();
                        __new_row.hash(&mut hasher);
                        hasher.finish() as usize
                     };
                     if ::ascent::internal::CRelFullIndexWrite::insert_if_not_present(
                        &_self.runtime_new.path_indices_0_1.to_c_rel_index_write(&_self.runtime_new.__path_ind_common),
                        &__new_row,
                        (),
                     ) {
                        let __new_row_to_be_pushed = (__new_row.0, __new_row.1);
                        _self.path.push(__new_row_to_be_pushed);
                        __default_id = new_id;
                        __changed.store(true, std::sync::atomic::Ordering::Relaxed);
                     } else {
                        return;
                     }
                  } else {
                     return;
                  }
                  let __new_row: (i32, i32, usize) = (
                     ascent::internal::Convert::convert(x),
                     ascent::internal::Convert::convert(y),
                     ascent::internal::Convert::convert(new_id),
                  );
                  let mut __new_path_id = 0;
                  if !::ascent::internal::RelFullIndexRead::contains_key(
                     &_self.runtime_total.path_id_indices_0_1_2.to_rel_index(&_self.runtime_total.__path_id_ind_common),
                     &__new_row,
                  ) && !::ascent::internal::RelFullIndexRead::contains_key(
                     &_self.runtime_delta.path_id_indices_0_1_2.to_rel_index(&_self.runtime_delta.__path_id_ind_common),
                     &__new_row,
                  ) {
                     __new_path_id = {
                        use std::hash::{Hash, Hasher};
                        let mut hasher = ::std::hash::DefaultHasher::new();
                        __new_row.hash(&mut hasher);
                        hasher.finish() as usize
                     };
                     if ::ascent::internal::CRelFullIndexWrite::insert_if_not_present(
                        &_self
                           .runtime_new
                           .path_id_indices_0_1_2
                           .to_c_rel_index_write(&_self.runtime_new.__path_id_ind_common),
                        &__new_row,
                        (),
                     ) {
                        let __new_row_to_be_pushed = (__new_row.0.clone(), __new_row.1.clone(), __new_row.2.clone());
                        __new_path_id = _self.path_id.push(__new_row_to_be_pushed);
                        __default_id = __new_path_id;
                        ::ascent::internal::CRelIndexWrite::index_insert(
                           &_self
                              .runtime_new
                              .path_id_indices_0
                              .to_c_rel_index_write(&_self.runtime_new.__path_id_ind_common),
                           (__new_row.0.clone(),),
                           (__new_row.1.clone(), __new_row.2.clone()),
                        );
                        __changed.store(true, std::sync::atomic::Ordering::Relaxed);
                     } else {
                     }
                  } else {
                  }
                  let __new_row: (Tag, Tag) = (Tag("path", new_id), Tag("edge", *eid));
                  let mut __new_provenance = 0;
                  if !::ascent::internal::RelFullIndexRead::contains_key(
                     &_self
                        .runtime_total
                        .provenance_indices_0_1
                        .to_rel_index(&_self.runtime_total.__provenance_ind_common),
                     &__new_row,
                  ) && !::ascent::internal::RelFullIndexRead::contains_key(
                     &_self
                        .runtime_delta
                        .provenance_indices_0_1
                        .to_rel_index(&_self.runtime_delta.__provenance_ind_common),
                     &__new_row,
                  ) {
                     __new_provenance = {
                        use std::hash::{Hash, Hasher};
                        let mut hasher = ::std::hash::DefaultHasher::new();
                        __new_row.hash(&mut hasher);
                        hasher.finish() as usize
                     };
                     if ::ascent::internal::CRelFullIndexWrite::insert_if_not_present(
                        &_self
                           .runtime_new
                           .provenance_indices_0_1
                           .to_c_rel_index_write(&_self.runtime_new.__provenance_ind_common),
                        &__new_row,
                        (),
                     ) {
                        let __new_row_to_be_pushed = (__new_row.0, __new_row.1);
                        __new_provenance = _self.provenance.push(__new_row_to_be_pushed);
                        __default_id = __new_provenance;
                        __changed.store(true, std::sync::atomic::Ordering::Relaxed);
                     } else {
                     }
                  } else {
                  }
               });
            }
         }
         _self.runtime_total.__path_ind_common.unfreeze();
         _self.runtime_delta.__path_ind_common.unfreeze();
         _self.runtime_total.path_indices_0_1.unfreeze();
         _self.runtime_delta.path_indices_0_1.unfreeze();
         _self.runtime_total.__path_id_ind_common.unfreeze();
         _self.runtime_delta.__path_id_ind_common.unfreeze();
         _self.runtime_total.path_id_indices_0.unfreeze();
         _self.runtime_delta.path_id_indices_0.unfreeze();
         _self.runtime_total.path_id_indices_0_1_2.unfreeze();
         _self.runtime_delta.path_id_indices_0_1_2.unfreeze();
         _self.runtime_total.__provenance_ind_common.unfreeze();
         _self.runtime_delta.__provenance_ind_common.unfreeze();
         _self.runtime_total.provenance_indices_0_1.unfreeze();
         _self.runtime_delta.provenance_indices_0_1.unfreeze();
         ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
            &mut _self.runtime_new.__path_ind_common,
            &mut _self.runtime_delta.__path_ind_common,
            &mut _self.runtime_total.__path_ind_common,
         );
         ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
            &mut _self.runtime_new.path_indices_0_1.to_rel_index_write(&mut _self.runtime_new.__path_ind_common),
            &mut _self.runtime_delta.path_indices_0_1.to_rel_index_write(&mut _self.runtime_delta.__path_ind_common),
            &mut _self.runtime_total.path_indices_0_1.to_rel_index_write(&mut _self.runtime_total.__path_ind_common),
         );
         ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
            &mut _self.runtime_new.__path_id_ind_common,
            &mut _self.runtime_delta.__path_id_ind_common,
            &mut _self.runtime_total.__path_id_ind_common,
         );
         ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
            &mut _self.runtime_new.path_id_indices_0.to_rel_index_write(&mut _self.runtime_new.__path_id_ind_common),
            &mut _self
               .runtime_delta
               .path_id_indices_0
               .to_rel_index_write(&mut _self.runtime_delta.__path_id_ind_common),
            &mut _self
               .runtime_total
               .path_id_indices_0
               .to_rel_index_write(&mut _self.runtime_total.__path_id_ind_common),
         );
         ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
            &mut _self
               .runtime_new
               .path_id_indices_0_1_2
               .to_rel_index_write(&mut _self.runtime_new.__path_id_ind_common),
            &mut _self
               .runtime_delta
               .path_id_indices_0_1_2
               .to_rel_index_write(&mut _self.runtime_delta.__path_id_ind_common),
            &mut _self
               .runtime_total
               .path_id_indices_0_1_2
               .to_rel_index_write(&mut _self.runtime_total.__path_id_ind_common),
         );
         ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
            &mut _self.runtime_new.__provenance_ind_common,
            &mut _self.runtime_delta.__provenance_ind_common,
            &mut _self.runtime_total.__provenance_ind_common,
         );
         ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
            &mut _self
               .runtime_new
               .provenance_indices_0_1
               .to_rel_index_write(&mut _self.runtime_new.__provenance_ind_common),
            &mut _self
               .runtime_delta
               .provenance_indices_0_1
               .to_rel_index_write(&mut _self.runtime_delta.__provenance_ind_common),
            &mut _self
               .runtime_total
               .provenance_indices_0_1
               .to_rel_index_write(&mut _self.runtime_total.__provenance_ind_common),
         );
         ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
            &mut _self.runtime_new.__path_ind_common,
            &mut _self.runtime_delta.__path_ind_common,
            &mut _self.runtime_total.__path_ind_common,
         );
         ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
            &mut _self.runtime_new.path_indices_0_1.to_rel_index_write(&mut _self.runtime_new.__path_ind_common),
            &mut _self.runtime_delta.path_indices_0_1.to_rel_index_write(&mut _self.runtime_delta.__path_ind_common),
            &mut _self.runtime_total.path_indices_0_1.to_rel_index_write(&mut _self.runtime_total.__path_ind_common),
         );
         ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
            &mut _self.runtime_new.__path_id_ind_common,
            &mut _self.runtime_delta.__path_id_ind_common,
            &mut _self.runtime_total.__path_id_ind_common,
         );
         ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
            &mut _self.runtime_new.path_id_indices_0.to_rel_index_write(&mut _self.runtime_new.__path_id_ind_common),
            &mut _self
               .runtime_delta
               .path_id_indices_0
               .to_rel_index_write(&mut _self.runtime_delta.__path_id_ind_common),
            &mut _self
               .runtime_total
               .path_id_indices_0
               .to_rel_index_write(&mut _self.runtime_total.__path_id_ind_common),
         );
         ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
            &mut _self
               .runtime_new
               .path_id_indices_0_1_2
               .to_rel_index_write(&mut _self.runtime_new.__path_id_ind_common),
            &mut _self
               .runtime_delta
               .path_id_indices_0_1_2
               .to_rel_index_write(&mut _self.runtime_delta.__path_id_ind_common),
            &mut _self
               .runtime_total
               .path_id_indices_0_1_2
               .to_rel_index_write(&mut _self.runtime_total.__path_id_ind_common),
         );
         ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
            &mut _self.runtime_new.__provenance_ind_common,
            &mut _self.runtime_delta.__provenance_ind_common,
            &mut _self.runtime_total.__provenance_ind_common,
         );
         ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
            &mut _self
               .runtime_new
               .provenance_indices_0_1
               .to_rel_index_write(&mut _self.runtime_new.__provenance_ind_common),
            &mut _self
               .runtime_delta
               .provenance_indices_0_1
               .to_rel_index_write(&mut _self.runtime_delta.__provenance_ind_common),
            &mut _self
               .runtime_total
               .provenance_indices_0_1
               .to_rel_index_write(&mut _self.runtime_total.__provenance_ind_common),
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
            use ascent::internal::CRelIndexRead;
            use ascent::internal::CRelIndexReadAll;
            use ascent::internal::CRelIndexWrite;
            use ascent::internal::Freezable;
            use ascent::internal::{RelIndexRead, RelIndexReadAll, ToRelIndex0, TupleOfBorrowed};
            use ascent::rayon::iter::ParallelBridge;
            use ascent::rayon::iter::ParallelIterator;
            use core::cmp::PartialEq;
            _self.runtime_delta.__path_ind_common = ::std::mem::take(&mut _self.runtime_total.__path_ind_common);
            _self.runtime_total.__path_ind_common = Default::default();
            _self.runtime_new.__path_ind_common = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut _self.runtime_new.__path_ind_common,
               &mut _self.runtime_delta.__path_ind_common,
               &mut _self.runtime_total.__path_ind_common,
            );
            _self.runtime_delta.path_indices_0_1 = ::std::mem::take(&mut _self.runtime_total.path_indices_0_1);
            _self.runtime_total.path_indices_0_1 = Default::default();
            _self.runtime_new.path_indices_0_1 = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut _self.runtime_new.path_indices_0_1.to_rel_index_write(&mut _self.runtime_new.__path_ind_common),
               &mut _self.runtime_delta.path_indices_0_1.to_rel_index_write(&mut _self.runtime_delta.__path_ind_common),
               &mut _self.runtime_total.path_indices_0_1.to_rel_index_write(&mut _self.runtime_total.__path_ind_common),
            );
            _self.runtime_delta.__path_id_ind_common = ::std::mem::take(&mut _self.runtime_total.__path_id_ind_common);
            _self.runtime_total.__path_id_ind_common = Default::default();
            _self.runtime_new.__path_id_ind_common = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut _self.runtime_new.__path_id_ind_common,
               &mut _self.runtime_delta.__path_id_ind_common,
               &mut _self.runtime_total.__path_id_ind_common,
            );
            _self.runtime_delta.path_id_indices_0 = ::std::mem::take(&mut _self.runtime_total.path_id_indices_0);
            _self.runtime_total.path_id_indices_0 = Default::default();
            _self.runtime_new.path_id_indices_0 = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut _self.runtime_new.path_id_indices_0.to_rel_index_write(&mut _self.runtime_new.__path_id_ind_common),
               &mut _self
                  .runtime_delta
                  .path_id_indices_0
                  .to_rel_index_write(&mut _self.runtime_delta.__path_id_ind_common),
               &mut _self
                  .runtime_total
                  .path_id_indices_0
                  .to_rel_index_write(&mut _self.runtime_total.__path_id_ind_common),
            );
            _self.runtime_delta.path_id_indices_0_1_2 =
               ::std::mem::take(&mut _self.runtime_total.path_id_indices_0_1_2);
            _self.runtime_total.path_id_indices_0_1_2 = Default::default();
            _self.runtime_new.path_id_indices_0_1_2 = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut _self
                  .runtime_new
                  .path_id_indices_0_1_2
                  .to_rel_index_write(&mut _self.runtime_new.__path_id_ind_common),
               &mut _self
                  .runtime_delta
                  .path_id_indices_0_1_2
                  .to_rel_index_write(&mut _self.runtime_delta.__path_id_ind_common),
               &mut _self
                  .runtime_total
                  .path_id_indices_0_1_2
                  .to_rel_index_write(&mut _self.runtime_total.__path_id_ind_common),
            );
            _self.runtime_delta.__provenance_ind_common =
               ::std::mem::take(&mut _self.runtime_total.__provenance_ind_common);
            _self.runtime_total.__provenance_ind_common = Default::default();
            _self.runtime_new.__provenance_ind_common = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut _self.runtime_new.__provenance_ind_common,
               &mut _self.runtime_delta.__provenance_ind_common,
               &mut _self.runtime_total.__provenance_ind_common,
            );
            _self.runtime_delta.provenance_indices_0_1 =
               ::std::mem::take(&mut _self.runtime_total.provenance_indices_0_1);
            _self.runtime_total.provenance_indices_0_1 = Default::default();
            _self.runtime_new.provenance_indices_0_1 = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut _self
                  .runtime_new
                  .provenance_indices_0_1
                  .to_rel_index_write(&mut _self.runtime_new.__provenance_ind_common),
               &mut _self
                  .runtime_delta
                  .provenance_indices_0_1
                  .to_rel_index_write(&mut _self.runtime_delta.__provenance_ind_common),
               &mut _self
                  .runtime_total
                  .provenance_indices_0_1
                  .to_rel_index_write(&mut _self.runtime_total.__provenance_ind_common),
            );
            _self.runtime_total.__edge_id_ind_common.freeze();
            _self.runtime_total.edge_id_indices_none.freeze();
            _self.scc_1_exec();
         }
         true
      }
      #[allow(unused_assignments, unused_variables, dead_code)]
      pub fn scc_2_exec(&mut self) -> bool {
         use ascent::internal::CRelIndexRead;
         use ascent::internal::CRelIndexReadAll;
         use ascent::internal::CRelIndexWrite;
         use ascent::internal::Freezable;
         use ascent::internal::{RelIndexRead, RelIndexReadAll, ToRelIndex0, TupleOfBorrowed};
         use ascent::rayon::iter::ParallelBridge;
         use ascent::rayon::iter::ParallelIterator;
         use core::cmp::PartialEq;
         let _self = self;
         let _scc_start_time = ::ascent::internal::Instant::now();
         let __changed = std::sync::atomic::AtomicBool::new(false);
         _self.runtime_total.__path_ind_common.freeze();
         _self.runtime_delta.__path_ind_common.freeze();
         _self.runtime_total.path_indices_0_1.freeze();
         _self.runtime_delta.path_indices_0_1.freeze();
         _self.runtime_total.__path_id_ind_common.freeze();
         _self.runtime_delta.__path_id_ind_common.freeze();
         _self.runtime_total.path_id_indices_0.freeze();
         _self.runtime_delta.path_id_indices_0.freeze();
         _self.runtime_total.path_id_indices_0_1_2.freeze();
         _self.runtime_delta.path_id_indices_0_1_2.freeze();
         _self.runtime_total.__provenance_ind_common.freeze();
         _self.runtime_delta.__provenance_ind_common.freeze();
         _self.runtime_total.provenance_indices_0_1.freeze();
         _self.runtime_delta.provenance_indices_0_1.freeze();
         ascent::internal::comment(
            "path, path_id, provenance <-- edge_id_indices_1_total, path_id_indices_0_delta [SIMPLE JOIN]",
         );
         if _self.runtime_delta.path_id_indices_0.to_rel_index(&_self.runtime_delta.__path_id_ind_common).len() > 0 {
            if _self.runtime_total.edge_id_indices_1.to_rel_index(&_self.runtime_total.__edge_id_ind_common).len()
               <= _self.runtime_delta.path_id_indices_0.to_rel_index(&_self.runtime_delta.__path_id_ind_common).len()
            {
               _self
                  .runtime_total
                  .edge_id_indices_1
                  .to_rel_index(&_self.runtime_total.__edge_id_ind_common)
                  .c_iter_all()
                  .for_each(|(__cl1_joined_columns, __cl1_tuple_indices)| {
                     let mut __gen_bang = false;
                     let __cl1_joined_columns = __cl1_joined_columns.tuple_of_borrowed();
                     let y = __cl1_joined_columns.0;
                     if let Some(__matching) = _self
                        .runtime_delta
                        .path_id_indices_0
                        .to_rel_index(&_self.runtime_delta.__path_id_ind_common)
                        .c_index_get(&(y.clone(),))
                     {
                        __cl1_tuple_indices.for_each(|cl1_val| {
                           let cl1_val = cl1_val.tuple_of_borrowed();
                           let x: &i32 = cl1_val.0;
                           let eid: &usize = cl1_val.1;
                           __matching.clone().for_each(|__val| {
                              let mut __dep_changed = false;
                              let mut __default_id = 0;
                              let __val = __val.tuple_of_borrowed();
                              let z: &i32 = __val.0;
                              let pid: &usize = __val.1;
                              let __new_row: (i32, i32) =
                                 (ascent::internal::Convert::convert(x), ascent::internal::Convert::convert(z));
                              let mut new_id = 0;
                              if !::ascent::internal::RelFullIndexRead::contains_key(
                                 &_self
                                    .runtime_total
                                    .path_indices_0_1
                                    .to_rel_index(&_self.runtime_total.__path_ind_common),
                                 &__new_row,
                              ) && !::ascent::internal::RelFullIndexRead::contains_key(
                                 &_self
                                    .runtime_delta
                                    .path_indices_0_1
                                    .to_rel_index(&_self.runtime_delta.__path_ind_common),
                                 &__new_row,
                              ) {
                                 new_id = {
                                    use std::hash::{Hash, Hasher};
                                    let mut hasher = ::std::hash::DefaultHasher::new();
                                    __new_row.hash(&mut hasher);
                                    hasher.finish() as usize
                                 };
                                 if ::ascent::internal::CRelFullIndexWrite::insert_if_not_present(
                                    &_self
                                       .runtime_new
                                       .path_indices_0_1
                                       .to_c_rel_index_write(&_self.runtime_new.__path_ind_common),
                                    &__new_row,
                                    (),
                                 ) {
                                    let __new_row_to_be_pushed = (__new_row.0, __new_row.1);
                                    _self.path.push(__new_row_to_be_pushed);
                                    __default_id = new_id;
                                    __changed.store(true, std::sync::atomic::Ordering::Relaxed);
                                 } else {
                                    return;
                                 }
                              } else {
                                 return;
                              }
                              let __new_row: (i32, i32, usize) = (
                                 ascent::internal::Convert::convert(x),
                                 ascent::internal::Convert::convert(z),
                                 ascent::internal::Convert::convert(new_id),
                              );
                              let mut __new_path_id = 0;
                              if !::ascent::internal::RelFullIndexRead::contains_key(
                                 &_self
                                    .runtime_total
                                    .path_id_indices_0_1_2
                                    .to_rel_index(&_self.runtime_total.__path_id_ind_common),
                                 &__new_row,
                              ) && !::ascent::internal::RelFullIndexRead::contains_key(
                                 &_self
                                    .runtime_delta
                                    .path_id_indices_0_1_2
                                    .to_rel_index(&_self.runtime_delta.__path_id_ind_common),
                                 &__new_row,
                              ) {
                                 __new_path_id = {
                                    use std::hash::{Hash, Hasher};
                                    let mut hasher = ::std::hash::DefaultHasher::new();
                                    __new_row.hash(&mut hasher);
                                    hasher.finish() as usize
                                 };
                                 if ::ascent::internal::CRelFullIndexWrite::insert_if_not_present(
                                    &_self
                                       .runtime_new
                                       .path_id_indices_0_1_2
                                       .to_c_rel_index_write(&_self.runtime_new.__path_id_ind_common),
                                    &__new_row,
                                    (),
                                 ) {
                                    let __new_row_to_be_pushed =
                                       (__new_row.0.clone(), __new_row.1.clone(), __new_row.2.clone());
                                    __new_path_id = _self.path_id.push(__new_row_to_be_pushed);
                                    __default_id = __new_path_id;
                                    ::ascent::internal::CRelIndexWrite::index_insert(
                                       &_self
                                          .runtime_new
                                          .path_id_indices_0
                                          .to_c_rel_index_write(&_self.runtime_new.__path_id_ind_common),
                                       (__new_row.0.clone(),),
                                       (__new_row.1.clone(), __new_row.2.clone()),
                                    );
                                    __changed.store(true, std::sync::atomic::Ordering::Relaxed);
                                 } else {
                                 }
                              } else {
                              }
                              let __new_row: (Tag, Tag) = (Tag("path", new_id), Tag("edge", *eid));
                              let mut __new_provenance = 0;
                              if !::ascent::internal::RelFullIndexRead::contains_key(
                                 &_self
                                    .runtime_total
                                    .provenance_indices_0_1
                                    .to_rel_index(&_self.runtime_total.__provenance_ind_common),
                                 &__new_row,
                              ) && !::ascent::internal::RelFullIndexRead::contains_key(
                                 &_self
                                    .runtime_delta
                                    .provenance_indices_0_1
                                    .to_rel_index(&_self.runtime_delta.__provenance_ind_common),
                                 &__new_row,
                              ) {
                                 __new_provenance = {
                                    use std::hash::{Hash, Hasher};
                                    let mut hasher = ::std::hash::DefaultHasher::new();
                                    __new_row.hash(&mut hasher);
                                    hasher.finish() as usize
                                 };
                                 if ::ascent::internal::CRelFullIndexWrite::insert_if_not_present(
                                    &_self
                                       .runtime_new
                                       .provenance_indices_0_1
                                       .to_c_rel_index_write(&_self.runtime_new.__provenance_ind_common),
                                    &__new_row,
                                    (),
                                 ) {
                                    let __new_row_to_be_pushed = (__new_row.0, __new_row.1);
                                    __new_provenance = _self.provenance.push(__new_row_to_be_pushed);
                                    __default_id = __new_provenance;
                                    __changed.store(true, std::sync::atomic::Ordering::Relaxed);
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
                  .runtime_delta
                  .path_id_indices_0
                  .to_rel_index(&_self.runtime_delta.__path_id_ind_common)
                  .c_iter_all()
                  .for_each(|(__cl1_joined_columns, __cl1_tuple_indices)| {
                     let mut __gen_bang = false;
                     let __cl1_joined_columns = __cl1_joined_columns.tuple_of_borrowed();
                     let y = __cl1_joined_columns.0;
                     if let Some(__matching) = _self
                        .runtime_total
                        .edge_id_indices_1
                        .to_rel_index(&_self.runtime_total.__edge_id_ind_common)
                        .c_index_get(&(y.clone(),))
                     {
                        __cl1_tuple_indices.for_each(|cl1_val| {
                           let cl1_val = cl1_val.tuple_of_borrowed();
                           let z: &i32 = cl1_val.0;
                           let pid: &usize = cl1_val.1;
                           __matching.clone().for_each(|__val| {
                              let mut __dep_changed = false;
                              let mut __default_id = 0;
                              let __val = __val.tuple_of_borrowed();
                              let x: &i32 = __val.0;
                              let eid: &usize = __val.1;
                              let __new_row: (i32, i32) =
                                 (ascent::internal::Convert::convert(x), ascent::internal::Convert::convert(z));
                              let mut new_id = 0;
                              if !::ascent::internal::RelFullIndexRead::contains_key(
                                 &_self
                                    .runtime_total
                                    .path_indices_0_1
                                    .to_rel_index(&_self.runtime_total.__path_ind_common),
                                 &__new_row,
                              ) && !::ascent::internal::RelFullIndexRead::contains_key(
                                 &_self
                                    .runtime_delta
                                    .path_indices_0_1
                                    .to_rel_index(&_self.runtime_delta.__path_ind_common),
                                 &__new_row,
                              ) {
                                 new_id = {
                                    use std::hash::{Hash, Hasher};
                                    let mut hasher = ::std::hash::DefaultHasher::new();
                                    __new_row.hash(&mut hasher);
                                    hasher.finish() as usize
                                 };
                                 if ::ascent::internal::CRelFullIndexWrite::insert_if_not_present(
                                    &_self
                                       .runtime_new
                                       .path_indices_0_1
                                       .to_c_rel_index_write(&_self.runtime_new.__path_ind_common),
                                    &__new_row,
                                    (),
                                 ) {
                                    let __new_row_to_be_pushed = (__new_row.0, __new_row.1);
                                    _self.path.push(__new_row_to_be_pushed);
                                    __default_id = new_id;
                                    __changed.store(true, std::sync::atomic::Ordering::Relaxed);
                                 } else {
                                    return;
                                 }
                              } else {
                                 return;
                              }
                              let __new_row: (i32, i32, usize) = (
                                 ascent::internal::Convert::convert(x),
                                 ascent::internal::Convert::convert(z),
                                 ascent::internal::Convert::convert(new_id),
                              );
                              let mut __new_path_id = 0;
                              if !::ascent::internal::RelFullIndexRead::contains_key(
                                 &_self
                                    .runtime_total
                                    .path_id_indices_0_1_2
                                    .to_rel_index(&_self.runtime_total.__path_id_ind_common),
                                 &__new_row,
                              ) && !::ascent::internal::RelFullIndexRead::contains_key(
                                 &_self
                                    .runtime_delta
                                    .path_id_indices_0_1_2
                                    .to_rel_index(&_self.runtime_delta.__path_id_ind_common),
                                 &__new_row,
                              ) {
                                 __new_path_id = {
                                    use std::hash::{Hash, Hasher};
                                    let mut hasher = ::std::hash::DefaultHasher::new();
                                    __new_row.hash(&mut hasher);
                                    hasher.finish() as usize
                                 };
                                 if ::ascent::internal::CRelFullIndexWrite::insert_if_not_present(
                                    &_self
                                       .runtime_new
                                       .path_id_indices_0_1_2
                                       .to_c_rel_index_write(&_self.runtime_new.__path_id_ind_common),
                                    &__new_row,
                                    (),
                                 ) {
                                    let __new_row_to_be_pushed =
                                       (__new_row.0.clone(), __new_row.1.clone(), __new_row.2.clone());
                                    __new_path_id = _self.path_id.push(__new_row_to_be_pushed);
                                    __default_id = __new_path_id;
                                    ::ascent::internal::CRelIndexWrite::index_insert(
                                       &_self
                                          .runtime_new
                                          .path_id_indices_0
                                          .to_c_rel_index_write(&_self.runtime_new.__path_id_ind_common),
                                       (__new_row.0.clone(),),
                                       (__new_row.1.clone(), __new_row.2.clone()),
                                    );
                                    __changed.store(true, std::sync::atomic::Ordering::Relaxed);
                                 } else {
                                 }
                              } else {
                              }
                              let __new_row: (Tag, Tag) = (Tag("path", new_id), Tag("edge", *eid));
                              let mut __new_provenance = 0;
                              if !::ascent::internal::RelFullIndexRead::contains_key(
                                 &_self
                                    .runtime_total
                                    .provenance_indices_0_1
                                    .to_rel_index(&_self.runtime_total.__provenance_ind_common),
                                 &__new_row,
                              ) && !::ascent::internal::RelFullIndexRead::contains_key(
                                 &_self
                                    .runtime_delta
                                    .provenance_indices_0_1
                                    .to_rel_index(&_self.runtime_delta.__provenance_ind_common),
                                 &__new_row,
                              ) {
                                 __new_provenance = {
                                    use std::hash::{Hash, Hasher};
                                    let mut hasher = ::std::hash::DefaultHasher::new();
                                    __new_row.hash(&mut hasher);
                                    hasher.finish() as usize
                                 };
                                 if ::ascent::internal::CRelFullIndexWrite::insert_if_not_present(
                                    &_self
                                       .runtime_new
                                       .provenance_indices_0_1
                                       .to_c_rel_index_write(&_self.runtime_new.__provenance_ind_common),
                                    &__new_row,
                                    (),
                                 ) {
                                    let __new_row_to_be_pushed = (__new_row.0, __new_row.1);
                                    __new_provenance = _self.provenance.push(__new_row_to_be_pushed);
                                    __default_id = __new_provenance;
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
         _self.runtime_total.__path_ind_common.unfreeze();
         _self.runtime_delta.__path_ind_common.unfreeze();
         _self.runtime_total.path_indices_0_1.unfreeze();
         _self.runtime_delta.path_indices_0_1.unfreeze();
         _self.runtime_total.__path_id_ind_common.unfreeze();
         _self.runtime_delta.__path_id_ind_common.unfreeze();
         _self.runtime_total.path_id_indices_0.unfreeze();
         _self.runtime_delta.path_id_indices_0.unfreeze();
         _self.runtime_total.path_id_indices_0_1_2.unfreeze();
         _self.runtime_delta.path_id_indices_0_1_2.unfreeze();
         _self.runtime_total.__provenance_ind_common.unfreeze();
         _self.runtime_delta.__provenance_ind_common.unfreeze();
         _self.runtime_total.provenance_indices_0_1.unfreeze();
         _self.runtime_delta.provenance_indices_0_1.unfreeze();
         ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
            &mut _self.runtime_new.__path_ind_common,
            &mut _self.runtime_delta.__path_ind_common,
            &mut _self.runtime_total.__path_ind_common,
         );
         ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
            &mut _self.runtime_new.path_indices_0_1.to_rel_index_write(&mut _self.runtime_new.__path_ind_common),
            &mut _self.runtime_delta.path_indices_0_1.to_rel_index_write(&mut _self.runtime_delta.__path_ind_common),
            &mut _self.runtime_total.path_indices_0_1.to_rel_index_write(&mut _self.runtime_total.__path_ind_common),
         );
         ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
            &mut _self.runtime_new.__path_id_ind_common,
            &mut _self.runtime_delta.__path_id_ind_common,
            &mut _self.runtime_total.__path_id_ind_common,
         );
         ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
            &mut _self.runtime_new.path_id_indices_0.to_rel_index_write(&mut _self.runtime_new.__path_id_ind_common),
            &mut _self
               .runtime_delta
               .path_id_indices_0
               .to_rel_index_write(&mut _self.runtime_delta.__path_id_ind_common),
            &mut _self
               .runtime_total
               .path_id_indices_0
               .to_rel_index_write(&mut _self.runtime_total.__path_id_ind_common),
         );
         ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
            &mut _self
               .runtime_new
               .path_id_indices_0_1_2
               .to_rel_index_write(&mut _self.runtime_new.__path_id_ind_common),
            &mut _self
               .runtime_delta
               .path_id_indices_0_1_2
               .to_rel_index_write(&mut _self.runtime_delta.__path_id_ind_common),
            &mut _self
               .runtime_total
               .path_id_indices_0_1_2
               .to_rel_index_write(&mut _self.runtime_total.__path_id_ind_common),
         );
         ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
            &mut _self.runtime_new.__provenance_ind_common,
            &mut _self.runtime_delta.__provenance_ind_common,
            &mut _self.runtime_total.__provenance_ind_common,
         );
         ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
            &mut _self
               .runtime_new
               .provenance_indices_0_1
               .to_rel_index_write(&mut _self.runtime_new.__provenance_ind_common),
            &mut _self
               .runtime_delta
               .provenance_indices_0_1
               .to_rel_index_write(&mut _self.runtime_delta.__provenance_ind_common),
            &mut _self
               .runtime_total
               .provenance_indices_0_1
               .to_rel_index_write(&mut _self.runtime_total.__provenance_ind_common),
         );
         _self.scc_iters[2usize] += 1;
         let need_break = !__changed.load(std::sync::atomic::Ordering::Relaxed);
         _self.scc_times[2usize] += _scc_start_time.elapsed();
         need_break
      }
      #[allow(unused_assignments, unused_variables, dead_code)]
      pub fn scc_2(&mut self) -> bool {
         ascent::internal::comment("scc 2");
         {
            macro_rules! __check_return_conditions {
               () => {};
            }
            let _self = self;
            use ascent::internal::CRelIndexRead;
            use ascent::internal::CRelIndexReadAll;
            use ascent::internal::CRelIndexWrite;
            use ascent::internal::Freezable;
            use ascent::internal::{RelIndexRead, RelIndexReadAll, ToRelIndex0, TupleOfBorrowed};
            use ascent::rayon::iter::ParallelBridge;
            use ascent::rayon::iter::ParallelIterator;
            use core::cmp::PartialEq;
            _self.runtime_delta.__path_ind_common = ::std::mem::take(&mut _self.runtime_total.__path_ind_common);
            _self.runtime_total.__path_ind_common = Default::default();
            _self.runtime_new.__path_ind_common = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut _self.runtime_new.__path_ind_common,
               &mut _self.runtime_delta.__path_ind_common,
               &mut _self.runtime_total.__path_ind_common,
            );
            _self.runtime_delta.path_indices_0_1 = ::std::mem::take(&mut _self.runtime_total.path_indices_0_1);
            _self.runtime_total.path_indices_0_1 = Default::default();
            _self.runtime_new.path_indices_0_1 = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut _self.runtime_new.path_indices_0_1.to_rel_index_write(&mut _self.runtime_new.__path_ind_common),
               &mut _self.runtime_delta.path_indices_0_1.to_rel_index_write(&mut _self.runtime_delta.__path_ind_common),
               &mut _self.runtime_total.path_indices_0_1.to_rel_index_write(&mut _self.runtime_total.__path_ind_common),
            );
            _self.runtime_delta.__path_id_ind_common = ::std::mem::take(&mut _self.runtime_total.__path_id_ind_common);
            _self.runtime_total.__path_id_ind_common = Default::default();
            _self.runtime_new.__path_id_ind_common = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut _self.runtime_new.__path_id_ind_common,
               &mut _self.runtime_delta.__path_id_ind_common,
               &mut _self.runtime_total.__path_id_ind_common,
            );
            _self.runtime_delta.path_id_indices_0 = ::std::mem::take(&mut _self.runtime_total.path_id_indices_0);
            _self.runtime_total.path_id_indices_0 = Default::default();
            _self.runtime_new.path_id_indices_0 = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut _self.runtime_new.path_id_indices_0.to_rel_index_write(&mut _self.runtime_new.__path_id_ind_common),
               &mut _self
                  .runtime_delta
                  .path_id_indices_0
                  .to_rel_index_write(&mut _self.runtime_delta.__path_id_ind_common),
               &mut _self
                  .runtime_total
                  .path_id_indices_0
                  .to_rel_index_write(&mut _self.runtime_total.__path_id_ind_common),
            );
            _self.runtime_delta.path_id_indices_0_1_2 =
               ::std::mem::take(&mut _self.runtime_total.path_id_indices_0_1_2);
            _self.runtime_total.path_id_indices_0_1_2 = Default::default();
            _self.runtime_new.path_id_indices_0_1_2 = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut _self
                  .runtime_new
                  .path_id_indices_0_1_2
                  .to_rel_index_write(&mut _self.runtime_new.__path_id_ind_common),
               &mut _self
                  .runtime_delta
                  .path_id_indices_0_1_2
                  .to_rel_index_write(&mut _self.runtime_delta.__path_id_ind_common),
               &mut _self
                  .runtime_total
                  .path_id_indices_0_1_2
                  .to_rel_index_write(&mut _self.runtime_total.__path_id_ind_common),
            );
            _self.runtime_delta.__provenance_ind_common =
               ::std::mem::take(&mut _self.runtime_total.__provenance_ind_common);
            _self.runtime_total.__provenance_ind_common = Default::default();
            _self.runtime_new.__provenance_ind_common = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut _self.runtime_new.__provenance_ind_common,
               &mut _self.runtime_delta.__provenance_ind_common,
               &mut _self.runtime_total.__provenance_ind_common,
            );
            _self.runtime_delta.provenance_indices_0_1 =
               ::std::mem::take(&mut _self.runtime_total.provenance_indices_0_1);
            _self.runtime_total.provenance_indices_0_1 = Default::default();
            _self.runtime_new.provenance_indices_0_1 = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut _self
                  .runtime_new
                  .provenance_indices_0_1
                  .to_rel_index_write(&mut _self.runtime_new.__provenance_ind_common),
               &mut _self
                  .runtime_delta
                  .provenance_indices_0_1
                  .to_rel_index_write(&mut _self.runtime_delta.__provenance_ind_common),
               &mut _self
                  .runtime_total
                  .provenance_indices_0_1
                  .to_rel_index_write(&mut _self.runtime_total.__provenance_ind_common),
            );
            _self.runtime_total.__edge_id_ind_common.freeze();
            _self.runtime_total.edge_id_indices_1.freeze();
            loop {
               let need_break = _self.scc_2_exec();
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
         self.update_indices_edge();
         self.update_indices_edge_id();
         self.update_indices_edge_raw();
         self.update_indices_path();
         self.update_indices_path_id();
         self.update_indices_provenance();
         self.update_indices_duration += before.elapsed();
      }
      #[allow(noop_method_call, suspicious_double_ref_op)]
      pub fn update_indices_edge(&mut self) {
         use ascent::internal::CRelIndexWrite;
         use ascent::internal::ToRelIndex0;
         use ascent::rayon::iter::{IntoParallelIterator, ParallelIterator};
         (0..self.edge.len()).into_par_iter().for_each(|_i| {
            let tuple = &self.edge[_i];
            let selection_tuple = (tuple.0.clone(), tuple.1.clone());
            let rel_ind = &self.runtime_total.edge_indices_0_1;
            ascent::internal::CRelIndexWrite::index_insert(
               &rel_ind.to_c_rel_index_write(&self.runtime_total.__edge_ind_common),
               selection_tuple,
               (),
            );
         });
      }
      #[allow(noop_method_call, suspicious_double_ref_op)]
      pub fn update_indices_edge_id(&mut self) {
         use ascent::internal::CRelIndexWrite;
         use ascent::internal::ToRelIndex0;
         use ascent::rayon::iter::{IntoParallelIterator, ParallelIterator};
         (0..self.edge_id.len()).into_par_iter().for_each(|_i| {
            let tuple = &self.edge_id[_i];
            let selection_tuple = (tuple.0.clone(), tuple.1.clone(), tuple.2.clone());
            let rel_ind = &self.runtime_total.edge_id_indices_0_1_2;
            ascent::internal::CRelIndexWrite::index_insert(
               &rel_ind.to_c_rel_index_write(&self.runtime_total.__edge_id_ind_common),
               selection_tuple,
               (),
            );
            let selection_tuple = (tuple.1.clone(),);
            let rel_ind = &self.runtime_total.edge_id_indices_1;
            ascent::internal::CRelIndexWrite::index_insert(
               &rel_ind.to_c_rel_index_write(&self.runtime_total.__edge_id_ind_common),
               selection_tuple,
               (tuple.0.clone(), tuple.2.clone()),
            );
            let selection_tuple = ();
            let rel_ind = &self.runtime_total.edge_id_indices_none;
            ascent::internal::CRelIndexWrite::index_insert(
               &rel_ind.to_c_rel_index_write(&self.runtime_total.__edge_id_ind_common),
               selection_tuple,
               (tuple.0.clone(), tuple.1.clone(), tuple.2.clone()),
            );
         });
      }
      #[allow(noop_method_call, suspicious_double_ref_op)]
      pub fn update_indices_edge_raw(&mut self) {
         use ascent::internal::CRelIndexWrite;
         use ascent::internal::ToRelIndex0;
         use ascent::rayon::iter::{IntoParallelIterator, ParallelIterator};
         (0..self.edge_raw.len()).into_par_iter().for_each(|_i| {
            let tuple = &self.edge_raw[_i];
            let selection_tuple = (tuple.0.clone(), tuple.1.clone());
            let rel_ind = &self.runtime_total.edge_raw_indices_0_1;
            ascent::internal::CRelIndexWrite::index_insert(
               &rel_ind.to_c_rel_index_write(&self.runtime_total.__edge_raw_ind_common),
               selection_tuple,
               (),
            );
            let selection_tuple = ();
            let rel_ind = &self.runtime_total.edge_raw_indices_none;
            ascent::internal::CRelIndexWrite::index_insert(
               &rel_ind.to_c_rel_index_write(&self.runtime_total.__edge_raw_ind_common),
               selection_tuple,
               (tuple.0.clone(), tuple.1.clone()),
            );
         });
      }
      #[allow(noop_method_call, suspicious_double_ref_op)]
      pub fn update_indices_path(&mut self) {
         use ascent::internal::CRelIndexWrite;
         use ascent::internal::ToRelIndex0;
         use ascent::rayon::iter::{IntoParallelIterator, ParallelIterator};
         (0..self.path.len()).into_par_iter().for_each(|_i| {
            let tuple = &self.path[_i];
            let selection_tuple = (tuple.0.clone(), tuple.1.clone());
            let rel_ind = &self.runtime_total.path_indices_0_1;
            ascent::internal::CRelIndexWrite::index_insert(
               &rel_ind.to_c_rel_index_write(&self.runtime_total.__path_ind_common),
               selection_tuple,
               (),
            );
         });
      }
      #[allow(noop_method_call, suspicious_double_ref_op)]
      pub fn update_indices_path_id(&mut self) {
         use ascent::internal::CRelIndexWrite;
         use ascent::internal::ToRelIndex0;
         use ascent::rayon::iter::{IntoParallelIterator, ParallelIterator};
         (0..self.path_id.len()).into_par_iter().for_each(|_i| {
            let tuple = &self.path_id[_i];
            let selection_tuple = (tuple.0.clone(),);
            let rel_ind = &self.runtime_total.path_id_indices_0;
            ascent::internal::CRelIndexWrite::index_insert(
               &rel_ind.to_c_rel_index_write(&self.runtime_total.__path_id_ind_common),
               selection_tuple,
               (tuple.1.clone(), tuple.2.clone()),
            );
            let selection_tuple = (tuple.0.clone(), tuple.1.clone(), tuple.2.clone());
            let rel_ind = &self.runtime_total.path_id_indices_0_1_2;
            ascent::internal::CRelIndexWrite::index_insert(
               &rel_ind.to_c_rel_index_write(&self.runtime_total.__path_id_ind_common),
               selection_tuple,
               (),
            );
         });
      }
      #[allow(noop_method_call, suspicious_double_ref_op)]
      pub fn update_indices_provenance(&mut self) {
         use ascent::internal::CRelIndexWrite;
         use ascent::internal::ToRelIndex0;
         use ascent::rayon::iter::{IntoParallelIterator, ParallelIterator};
         (0..self.provenance.len()).into_par_iter().for_each(|_i| {
            let tuple = &self.provenance[_i];
            let selection_tuple = (tuple.0.clone(), tuple.1.clone());
            let rel_ind = &self.runtime_total.provenance_indices_0_1;
            ascent::internal::CRelIndexWrite::index_insert(
               &rel_ind.to_c_rel_index_write(&self.runtime_total.__provenance_ind_common),
               selection_tuple,
               (),
            );
         });
      }
      #[deprecated = "Explicit call to update_indices not required anymore."]
      pub fn update_indices(&mut self) { self.update_indices_priv(); }
      fn type_constraints() {
         let _type_constraints: ascent::internal::TypeConstraints<i32>;
         let _par_constraints: ascent::internal::ParTypeConstraints<i32>;
         let _type_constraints: ascent::internal::TypeConstraints<usize>;
         let _par_constraints: ascent::internal::ParTypeConstraints<usize>;
         let _type_constraints: ascent::internal::TypeConstraints<Tag>;
         let _par_constraints: ascent::internal::ParTypeConstraints<Tag>;
      }
      pub fn summary() -> &'static str {
         "scc 0, is_looping: false:\n  edge, edge_id <-- edge_raw_indices_none_total\n  dynamic relations: edge, edge_id\nscc 1, is_looping: false:\n  path, path_id, provenance <-- edge_id_indices_none_total\n  dynamic relations: path, path_id, provenance\nscc 2, is_looping: true:\n  path, path_id, provenance <-- edge_id_indices_1_total, path_id_indices_0_delta [SIMPLE JOIN]\n  dynamic relations: path, path_id, provenance\n"
      }
      pub fn relation_sizes_summary(&self) -> String {
         use std::fmt::Write;
         let mut res = String::new();
         writeln!(&mut res, "{} size: {}", "edge", self.edge.len()).unwrap();
         writeln!(&mut res, "{} size: {}", "edge_id", self.edge_id.len()).unwrap();
         writeln!(&mut res, "{} size: {}", "edge_raw", self.edge_raw.len()).unwrap();
         writeln!(&mut res, "{} size: {}", "path", self.path.len()).unwrap();
         writeln!(&mut res, "{} size: {}", "path_id", self.path_id.len()).unwrap();
         writeln!(&mut res, "{} size: {}", "provenance", self.provenance.len()).unwrap();
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
   impl Default for TCWhereRuntime {
      fn default() -> Self {
         let mut _self = TCWhereRuntime {
            edge: Default::default(),
            __edge_ind_common: Default::default(),
            edge_indices_0_1: Default::default(),
            edge_id: Default::default(),
            __edge_id_ind_common: Default::default(),
            edge_id_indices_0_1_2: Default::default(),
            edge_id_indices_1: Default::default(),
            edge_id_indices_none: Default::default(),
            edge_raw: Default::default(),
            __edge_raw_ind_common: Default::default(),
            edge_raw_indices_0_1: Default::default(),
            edge_raw_indices_none: Default::default(),
            path: Default::default(),
            __path_ind_common: Default::default(),
            path_indices_0_1: Default::default(),
            path_id: Default::default(),
            __path_id_ind_common: Default::default(),
            path_id_indices_0: Default::default(),
            path_id_indices_0_1_2: Default::default(),
            provenance: Default::default(),
            __provenance_ind_common: Default::default(),
            provenance_indices_0_1: Default::default(),
         };
         _self
      }
   }
   impl Default for TCWhere {
      fn default() -> Self {
         let mut _self = TCWhere {
            edge: Default::default(),
            edge_id: Default::default(),
            edge_raw: Default::default(),
            path: Default::default(),
            path_id: Default::default(),
            provenance: Default::default(),
            scc_times: [std::time::Duration::ZERO; 3usize],
            scc_iters: [0; 3usize],
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
