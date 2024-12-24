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
   ::ascent::rel::rel_codegen! { TCIncremental_edge , (i32 , i32) , [[] , [0] , [0 , 1]] , par , () }
   ::ascent::rel::rel_codegen! { TCIncremental_path , (i32 , i32) , [[] , [0 , 1] , [1]] , par , () }
   ::ascent::rel::rel_codegen! { TCIncremental_outside , () , [[]] , par , () }
   ::ascent::rel::rel_codegen! { TCIncremental_edge_incremental , (i32 , i32) , [[] , [0 , 1]] , par , () }
   ::ascent::rel::rel_codegen! { TCIncremental_path_incremental , (i32 , i32) , [[] , [0 , 1]] , par , () }
   struct TCIncremental {
      #[doc = "\nlogical indices: edge_indices_0; edge_indices_0_1; edge_indices_none"]
      pub edge: ::ascent::rel::rel!(TCIncremental_edge, (i32, i32), [[], [0], [0, 1]], par, ()),
      pub __edge_ind_common: ::ascent::rel::rel_ind_common!(TCIncremental_edge, (i32, i32), [[], [0], [0, 1]], par, ()),
      pub edge_indices_0:
         ::ascent::rel::rel_ind!(TCIncremental_edge, (i32, i32), [[], [0], [0, 1]], par, (), [0], (i32,), (i32,)),
      pub edge_indices_0_1:
         ::ascent::rel::rel_full_ind!(TCIncremental_edge, (i32, i32), [[], [0], [0, 1]], par, (), (i32, i32), ()),
      pub edge_indices_none:
         ::ascent::rel::rel_ind!(TCIncremental_edge, (i32, i32), [[], [0], [0, 1]], par, (), [], (), (i32, i32)),
      #[doc = "\nlogical indices: edge_incremental_indices_0_1; edge_incremental_indices_none"]
      pub edge_incremental: ::ascent::rel::rel!(TCIncremental_edge_incremental, (i32, i32), [[], [0, 1]], par, ()),
      pub __edge_incremental_ind_common:
         ::ascent::rel::rel_ind_common!(TCIncremental_edge_incremental, (i32, i32), [[], [0, 1]], par, ()),
      pub edge_incremental_indices_0_1: ::ascent::rel::rel_full_ind!(
         TCIncremental_edge_incremental,
         (i32, i32),
         [[], [0, 1]],
         par,
         (),
         (i32, i32),
         ()
      ),
      pub edge_incremental_indices_none:
         ::ascent::rel::rel_ind!(TCIncremental_edge_incremental, (i32, i32), [[], [0, 1]], par, (), [], (), (i32, i32)),
      #[doc = "\nlogical indices: outside_indices_none"]
      pub outside: ::ascent::rel::rel!(TCIncremental_outside, (), [[]], par, ()),
      pub __outside_ind_common: ::ascent::rel::rel_ind_common!(TCIncremental_outside, (), [[]], par, ()),
      pub outside_indices_none: ::ascent::rel::rel_full_ind!(TCIncremental_outside, (), [[]], par, (), (), ()),
      #[doc = "\nlogical indices: path_indices_0_1; path_indices_1; path_indices_none"]
      pub path: ::ascent::rel::rel!(TCIncremental_path, (i32, i32), [[], [0, 1], [1]], par, ()),
      pub __path_ind_common: ::ascent::rel::rel_ind_common!(TCIncremental_path, (i32, i32), [[], [0, 1], [1]], par, ()),
      pub path_indices_0_1:
         ::ascent::rel::rel_full_ind!(TCIncremental_path, (i32, i32), [[], [0, 1], [1]], par, (), (i32, i32), ()),
      pub path_indices_1:
         ::ascent::rel::rel_ind!(TCIncremental_path, (i32, i32), [[], [0, 1], [1]], par, (), [1], (i32,), (i32,)),
      pub path_indices_none:
         ::ascent::rel::rel_ind!(TCIncremental_path, (i32, i32), [[], [0, 1], [1]], par, (), [], (), (i32, i32)),
      #[doc = "\nlogical indices: path_incremental_indices_0_1; path_incremental_indices_none"]
      pub path_incremental: ::ascent::rel::rel!(TCIncremental_path_incremental, (i32, i32), [[], [0, 1]], par, ()),
      pub __path_incremental_ind_common:
         ::ascent::rel::rel_ind_common!(TCIncremental_path_incremental, (i32, i32), [[], [0, 1]], par, ()),
      pub path_incremental_indices_0_1: ::ascent::rel::rel_full_ind!(
         TCIncremental_path_incremental,
         (i32, i32),
         [[], [0, 1]],
         par,
         (),
         (i32, i32),
         ()
      ),
      pub path_incremental_indices_none:
         ::ascent::rel::rel_ind!(TCIncremental_path_incremental, (i32, i32), [[], [0, 1]], par, (), [], (), (i32, i32)),
      scc_times: [std::time::Duration; 1usize],
      scc_iters: [usize; 1usize],
      pub update_time_nanos: std::sync::atomic::AtomicU64,
      pub update_indices_duration: std::time::Duration,
   }
   impl TCIncremental {
      #[allow(unused_imports, noop_method_call, suspicious_double_ref_op)]
      #[doc = "Runs the Ascent program to a fixed point."]
      pub fn run(&mut self) { self.run_with_init_flag(true); }
      pub fn run_with_init_flag(&mut self, init_flag: bool) {
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
         if init_flag {
            self.update_indices_priv()
         };
         let _self = self;
         ascent::internal::comment("scc 0");
         {
            let _scc_start_time = ::ascent::internal::Instant::now();
            let mut __edge_ind_common_delta: ::ascent::rel::rel_ind_common!(
               TCIncremental_edge,
               (i32, i32),
               [[], [0], [0, 1]],
               par,
               ()
            ) = ::std::mem::take(&mut _self.__edge_ind_common);
            let mut __edge_ind_common_total: ::ascent::rel::rel_ind_common!(
               TCIncremental_edge,
               (i32, i32),
               [[], [0], [0, 1]],
               par,
               ()
            ) = Default::default();
            let mut __edge_ind_common_new: ::ascent::rel::rel_ind_common!(
               TCIncremental_edge,
               (i32, i32),
               [[], [0], [0, 1]],
               par,
               ()
            ) = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut __edge_ind_common_new,
               &mut __edge_ind_common_delta,
               &mut __edge_ind_common_total,
            );
            let mut edge_indices_0_delta: ::ascent::rel::rel_ind!(
               TCIncremental_edge,
               (i32, i32),
               [[], [0], [0, 1]],
               par,
               (),
               [0],
               (i32,),
               (i32,)
            ) = ::std::mem::take(&mut _self.edge_indices_0);
            let mut edge_indices_0_total: ::ascent::rel::rel_ind!(
               TCIncremental_edge,
               (i32, i32),
               [[], [0], [0, 1]],
               par,
               (),
               [0],
               (i32,),
               (i32,)
            ) = Default::default();
            let mut edge_indices_0_new: ::ascent::rel::rel_ind!(
               TCIncremental_edge,
               (i32, i32),
               [[], [0], [0, 1]],
               par,
               (),
               [0],
               (i32,),
               (i32,)
            ) = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut edge_indices_0_new.to_rel_index_write(&mut __edge_ind_common_new),
               &mut edge_indices_0_delta.to_rel_index_write(&mut __edge_ind_common_delta),
               &mut edge_indices_0_total.to_rel_index_write(&mut __edge_ind_common_total),
            );
            let mut edge_indices_0_1_delta: ::ascent::rel::rel_full_ind!(
               TCIncremental_edge,
               (i32, i32),
               [[], [0], [0, 1]],
               par,
               (),
               (i32, i32),
               ()
            ) = ::std::mem::take(&mut _self.edge_indices_0_1);
            let mut edge_indices_0_1_total: ::ascent::rel::rel_full_ind!(
               TCIncremental_edge,
               (i32, i32),
               [[], [0], [0, 1]],
               par,
               (),
               (i32, i32),
               ()
            ) = Default::default();
            let mut edge_indices_0_1_new: ::ascent::rel::rel_full_ind!(
               TCIncremental_edge,
               (i32, i32),
               [[], [0], [0, 1]],
               par,
               (),
               (i32, i32),
               ()
            ) = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut edge_indices_0_1_new.to_rel_index_write(&mut __edge_ind_common_new),
               &mut edge_indices_0_1_delta.to_rel_index_write(&mut __edge_ind_common_delta),
               &mut edge_indices_0_1_total.to_rel_index_write(&mut __edge_ind_common_total),
            );
            let mut edge_indices_none_delta: ::ascent::rel::rel_ind!(
               TCIncremental_edge,
               (i32, i32),
               [[], [0], [0, 1]],
               par,
               (),
               [],
               (),
               (i32, i32)
            ) = ::std::mem::take(&mut _self.edge_indices_none);
            let mut edge_indices_none_total: ::ascent::rel::rel_ind!(
               TCIncremental_edge,
               (i32, i32),
               [[], [0], [0, 1]],
               par,
               (),
               [],
               (),
               (i32, i32)
            ) = Default::default();
            let mut edge_indices_none_new: ::ascent::rel::rel_ind!(
               TCIncremental_edge,
               (i32, i32),
               [[], [0], [0, 1]],
               par,
               (),
               [],
               (),
               (i32, i32)
            ) = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut edge_indices_none_new.to_rel_index_write(&mut __edge_ind_common_new),
               &mut edge_indices_none_delta.to_rel_index_write(&mut __edge_ind_common_delta),
               &mut edge_indices_none_total.to_rel_index_write(&mut __edge_ind_common_total),
            );
            let mut __edge_incremental_ind_common_delta: ::ascent::rel::rel_ind_common!(
               TCIncremental_edge_incremental,
               (i32, i32),
               [[], [0, 1]],
               par,
               ()
            ) = ::std::mem::take(&mut _self.__edge_incremental_ind_common);
            let mut __edge_incremental_ind_common_total: ::ascent::rel::rel_ind_common!(
               TCIncremental_edge_incremental,
               (i32, i32),
               [[], [0, 1]],
               par,
               ()
            ) = Default::default();
            let mut __edge_incremental_ind_common_new: ::ascent::rel::rel_ind_common!(
               TCIncremental_edge_incremental,
               (i32, i32),
               [[], [0, 1]],
               par,
               ()
            ) = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut __edge_incremental_ind_common_new,
               &mut __edge_incremental_ind_common_delta,
               &mut __edge_incremental_ind_common_total,
            );
            let mut edge_incremental_indices_0_1_delta: ::ascent::rel::rel_full_ind!(
               TCIncremental_edge_incremental,
               (i32, i32),
               [[], [0, 1]],
               par,
               (),
               (i32, i32),
               ()
            ) = ::std::mem::take(&mut _self.edge_incremental_indices_0_1);
            let mut edge_incremental_indices_0_1_total: ::ascent::rel::rel_full_ind!(
               TCIncremental_edge_incremental,
               (i32, i32),
               [[], [0, 1]],
               par,
               (),
               (i32, i32),
               ()
            ) = Default::default();
            let mut edge_incremental_indices_0_1_new: ::ascent::rel::rel_full_ind!(
               TCIncremental_edge_incremental,
               (i32, i32),
               [[], [0, 1]],
               par,
               (),
               (i32, i32),
               ()
            ) = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut edge_incremental_indices_0_1_new.to_rel_index_write(&mut __edge_incremental_ind_common_new),
               &mut edge_incremental_indices_0_1_delta.to_rel_index_write(&mut __edge_incremental_ind_common_delta),
               &mut edge_incremental_indices_0_1_total.to_rel_index_write(&mut __edge_incremental_ind_common_total),
            );
            let mut edge_incremental_indices_none_delta: ::ascent::rel::rel_ind!(
               TCIncremental_edge_incremental,
               (i32, i32),
               [[], [0, 1]],
               par,
               (),
               [],
               (),
               (i32, i32)
            ) = ::std::mem::take(&mut _self.edge_incremental_indices_none);
            let mut edge_incremental_indices_none_total: ::ascent::rel::rel_ind!(
               TCIncremental_edge_incremental,
               (i32, i32),
               [[], [0, 1]],
               par,
               (),
               [],
               (),
               (i32, i32)
            ) = Default::default();
            let mut edge_incremental_indices_none_new: ::ascent::rel::rel_ind!(
               TCIncremental_edge_incremental,
               (i32, i32),
               [[], [0, 1]],
               par,
               (),
               [],
               (),
               (i32, i32)
            ) = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut edge_incremental_indices_none_new.to_rel_index_write(&mut __edge_incremental_ind_common_new),
               &mut edge_incremental_indices_none_delta.to_rel_index_write(&mut __edge_incremental_ind_common_delta),
               &mut edge_incremental_indices_none_total.to_rel_index_write(&mut __edge_incremental_ind_common_total),
            );
            let mut __outside_ind_common_delta: ::ascent::rel::rel_ind_common!(
               TCIncremental_outside,
               (),
               [[]],
               par,
               ()
            ) = ::std::mem::take(&mut _self.__outside_ind_common);
            let mut __outside_ind_common_total: ::ascent::rel::rel_ind_common!(
               TCIncremental_outside,
               (),
               [[]],
               par,
               ()
            ) = Default::default();
            let mut __outside_ind_common_new: ::ascent::rel::rel_ind_common!(TCIncremental_outside, (), [[]], par, ()) =
               Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut __outside_ind_common_new,
               &mut __outside_ind_common_delta,
               &mut __outside_ind_common_total,
            );
            let mut outside_indices_none_delta: ::ascent::rel::rel_full_ind!(
               TCIncremental_outside,
               (),
               [[]],
               par,
               (),
               (),
               ()
            ) = ::std::mem::take(&mut _self.outside_indices_none);
            let mut outside_indices_none_total: ::ascent::rel::rel_full_ind!(
               TCIncremental_outside,
               (),
               [[]],
               par,
               (),
               (),
               ()
            ) = Default::default();
            let mut outside_indices_none_new: ::ascent::rel::rel_full_ind!(
               TCIncremental_outside,
               (),
               [[]],
               par,
               (),
               (),
               ()
            ) = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut outside_indices_none_new.to_rel_index_write(&mut __outside_ind_common_new),
               &mut outside_indices_none_delta.to_rel_index_write(&mut __outside_ind_common_delta),
               &mut outside_indices_none_total.to_rel_index_write(&mut __outside_ind_common_total),
            );
            let mut __path_ind_common_delta: ::ascent::rel::rel_ind_common!(
               TCIncremental_path,
               (i32, i32),
               [[], [0, 1], [1]],
               par,
               ()
            ) = ::std::mem::take(&mut _self.__path_ind_common);
            let mut __path_ind_common_total: ::ascent::rel::rel_ind_common!(
               TCIncremental_path,
               (i32, i32),
               [[], [0, 1], [1]],
               par,
               ()
            ) = Default::default();
            let mut __path_ind_common_new: ::ascent::rel::rel_ind_common!(
               TCIncremental_path,
               (i32, i32),
               [[], [0, 1], [1]],
               par,
               ()
            ) = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut __path_ind_common_new,
               &mut __path_ind_common_delta,
               &mut __path_ind_common_total,
            );
            let mut path_indices_0_1_delta: ::ascent::rel::rel_full_ind!(
               TCIncremental_path,
               (i32, i32),
               [[], [0, 1], [1]],
               par,
               (),
               (i32, i32),
               ()
            ) = ::std::mem::take(&mut _self.path_indices_0_1);
            let mut path_indices_0_1_total: ::ascent::rel::rel_full_ind!(
               TCIncremental_path,
               (i32, i32),
               [[], [0, 1], [1]],
               par,
               (),
               (i32, i32),
               ()
            ) = Default::default();
            let mut path_indices_0_1_new: ::ascent::rel::rel_full_ind!(
               TCIncremental_path,
               (i32, i32),
               [[], [0, 1], [1]],
               par,
               (),
               (i32, i32),
               ()
            ) = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut path_indices_0_1_new.to_rel_index_write(&mut __path_ind_common_new),
               &mut path_indices_0_1_delta.to_rel_index_write(&mut __path_ind_common_delta),
               &mut path_indices_0_1_total.to_rel_index_write(&mut __path_ind_common_total),
            );
            let mut path_indices_1_delta: ::ascent::rel::rel_ind!(
               TCIncremental_path,
               (i32, i32),
               [[], [0, 1], [1]],
               par,
               (),
               [1],
               (i32,),
               (i32,)
            ) = ::std::mem::take(&mut _self.path_indices_1);
            let mut path_indices_1_total: ::ascent::rel::rel_ind!(
               TCIncremental_path,
               (i32, i32),
               [[], [0, 1], [1]],
               par,
               (),
               [1],
               (i32,),
               (i32,)
            ) = Default::default();
            let mut path_indices_1_new: ::ascent::rel::rel_ind!(
               TCIncremental_path,
               (i32, i32),
               [[], [0, 1], [1]],
               par,
               (),
               [1],
               (i32,),
               (i32,)
            ) = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut path_indices_1_new.to_rel_index_write(&mut __path_ind_common_new),
               &mut path_indices_1_delta.to_rel_index_write(&mut __path_ind_common_delta),
               &mut path_indices_1_total.to_rel_index_write(&mut __path_ind_common_total),
            );
            let mut path_indices_none_delta: ::ascent::rel::rel_ind!(
               TCIncremental_path,
               (i32, i32),
               [[], [0, 1], [1]],
               par,
               (),
               [],
               (),
               (i32, i32)
            ) = ::std::mem::take(&mut _self.path_indices_none);
            let mut path_indices_none_total: ::ascent::rel::rel_ind!(
               TCIncremental_path,
               (i32, i32),
               [[], [0, 1], [1]],
               par,
               (),
               [],
               (),
               (i32, i32)
            ) = Default::default();
            let mut path_indices_none_new: ::ascent::rel::rel_ind!(
               TCIncremental_path,
               (i32, i32),
               [[], [0, 1], [1]],
               par,
               (),
               [],
               (),
               (i32, i32)
            ) = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut path_indices_none_new.to_rel_index_write(&mut __path_ind_common_new),
               &mut path_indices_none_delta.to_rel_index_write(&mut __path_ind_common_delta),
               &mut path_indices_none_total.to_rel_index_write(&mut __path_ind_common_total),
            );
            let mut __path_incremental_ind_common_delta: ::ascent::rel::rel_ind_common!(
               TCIncremental_path_incremental,
               (i32, i32),
               [[], [0, 1]],
               par,
               ()
            ) = ::std::mem::take(&mut _self.__path_incremental_ind_common);
            let mut __path_incremental_ind_common_total: ::ascent::rel::rel_ind_common!(
               TCIncremental_path_incremental,
               (i32, i32),
               [[], [0, 1]],
               par,
               ()
            ) = Default::default();
            let mut __path_incremental_ind_common_new: ::ascent::rel::rel_ind_common!(
               TCIncremental_path_incremental,
               (i32, i32),
               [[], [0, 1]],
               par,
               ()
            ) = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut __path_incremental_ind_common_new,
               &mut __path_incremental_ind_common_delta,
               &mut __path_incremental_ind_common_total,
            );
            let mut path_incremental_indices_0_1_delta: ::ascent::rel::rel_full_ind!(
               TCIncremental_path_incremental,
               (i32, i32),
               [[], [0, 1]],
               par,
               (),
               (i32, i32),
               ()
            ) = ::std::mem::take(&mut _self.path_incremental_indices_0_1);
            let mut path_incremental_indices_0_1_total: ::ascent::rel::rel_full_ind!(
               TCIncremental_path_incremental,
               (i32, i32),
               [[], [0, 1]],
               par,
               (),
               (i32, i32),
               ()
            ) = Default::default();
            let mut path_incremental_indices_0_1_new: ::ascent::rel::rel_full_ind!(
               TCIncremental_path_incremental,
               (i32, i32),
               [[], [0, 1]],
               par,
               (),
               (i32, i32),
               ()
            ) = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut path_incremental_indices_0_1_new.to_rel_index_write(&mut __path_incremental_ind_common_new),
               &mut path_incremental_indices_0_1_delta.to_rel_index_write(&mut __path_incremental_ind_common_delta),
               &mut path_incremental_indices_0_1_total.to_rel_index_write(&mut __path_incremental_ind_common_total),
            );
            let mut path_incremental_indices_none_delta: ::ascent::rel::rel_ind!(
               TCIncremental_path_incremental,
               (i32, i32),
               [[], [0, 1]],
               par,
               (),
               [],
               (),
               (i32, i32)
            ) = ::std::mem::take(&mut _self.path_incremental_indices_none);
            let mut path_incremental_indices_none_total: ::ascent::rel::rel_ind!(
               TCIncremental_path_incremental,
               (i32, i32),
               [[], [0, 1]],
               par,
               (),
               [],
               (),
               (i32, i32)
            ) = Default::default();
            let mut path_incremental_indices_none_new: ::ascent::rel::rel_ind!(
               TCIncremental_path_incremental,
               (i32, i32),
               [[], [0, 1]],
               par,
               (),
               [],
               (),
               (i32, i32)
            ) = Default::default();
            ::ascent::internal::RelIndexMerge::init(
               &mut path_incremental_indices_none_new.to_rel_index_write(&mut __path_incremental_ind_common_new),
               &mut path_incremental_indices_none_delta.to_rel_index_write(&mut __path_incremental_ind_common_delta),
               &mut path_incremental_indices_none_total.to_rel_index_write(&mut __path_incremental_ind_common_total),
            );
            #[allow(unused_assignments, unused_variables)]
            loop {
               let __changed = std::sync::atomic::AtomicBool::new(false);
               __edge_ind_common_total.freeze();
               __edge_ind_common_delta.freeze();
               edge_indices_0_total.freeze();
               edge_indices_0_delta.freeze();
               edge_indices_0_1_total.freeze();
               edge_indices_0_1_delta.freeze();
               edge_indices_none_total.freeze();
               edge_indices_none_delta.freeze();
               __edge_incremental_ind_common_total.freeze();
               __edge_incremental_ind_common_delta.freeze();
               edge_incremental_indices_0_1_total.freeze();
               edge_incremental_indices_0_1_delta.freeze();
               edge_incremental_indices_none_total.freeze();
               edge_incremental_indices_none_delta.freeze();
               __outside_ind_common_total.freeze();
               __outside_ind_common_delta.freeze();
               outside_indices_none_total.freeze();
               outside_indices_none_delta.freeze();
               __path_ind_common_total.freeze();
               __path_ind_common_delta.freeze();
               path_indices_0_1_total.freeze();
               path_indices_0_1_delta.freeze();
               path_indices_1_total.freeze();
               path_indices_1_delta.freeze();
               path_indices_none_total.freeze();
               path_indices_none_delta.freeze();
               __path_incremental_ind_common_total.freeze();
               __path_incremental_ind_common_delta.freeze();
               path_incremental_indices_0_1_total.freeze();
               path_incremental_indices_0_1_delta.freeze();
               path_incremental_indices_none_total.freeze();
               path_incremental_indices_none_delta.freeze();
               ascent::internal::comment("edge_incremental <-- outside_indices_none_delta, let ⋯, let ⋯");
               if outside_indices_none_delta.to_rel_index(&__outside_ind_common_delta).len() > 0 {
                  if let Some(__matching) =
                     outside_indices_none_delta.to_rel_index(&__outside_ind_common_delta).c_index_get(&())
                  {
                     __matching.for_each(|__val| {
                        let mut __dep_changed = false;
                        let mut __default_id = 0;
                        let x = 0;
                        let y = 0;
                        let __new_row: (i32, i32) =
                           (ascent::internal::Convert::convert(x), ascent::internal::Convert::convert(y));
                        let mut __new_edge_incremental = 0;
                        if !::ascent::internal::RelFullIndexRead::contains_key(
                           &edge_incremental_indices_0_1_total.to_rel_index(&__edge_incremental_ind_common_total),
                           &__new_row,
                        ) && !::ascent::internal::RelFullIndexRead::contains_key(
                           &edge_incremental_indices_0_1_delta.to_rel_index(&__edge_incremental_ind_common_delta),
                           &__new_row,
                        ) {
                           if ::ascent::internal::CRelFullIndexWrite::insert_if_not_present(
                              &edge_incremental_indices_0_1_new
                                 .to_c_rel_index_write(&__edge_incremental_ind_common_new),
                              &__new_row,
                              (),
                           ) {
                              __new_edge_incremental =
                                 _self.edge_incremental.push((__new_row.0.clone(), __new_row.1.clone()));
                              __default_id = __new_edge_incremental;
                              ::ascent::internal::CRelIndexWrite::index_insert(
                                 &edge_incremental_indices_none_new
                                    .to_c_rel_index_write(&__edge_incremental_ind_common_new),
                                 (),
                                 (__new_row.0.clone(), __new_row.1.clone()),
                              );
                              __changed.store(true, std::sync::atomic::Ordering::Relaxed);
                           } else {
                           }
                        } else {
                        }
                     });
                  }
               }
               ascent::internal::comment("edge <-- edge_incremental_indices_none_delta");
               if edge_incremental_indices_none_delta.to_rel_index(&__edge_incremental_ind_common_delta).len() > 0 {
                  if let Some(__matching) = edge_incremental_indices_none_delta
                     .to_rel_index(&__edge_incremental_ind_common_delta)
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
                        let mut __new_edge = 0;
                        if !::ascent::internal::RelFullIndexRead::contains_key(
                           &edge_indices_0_1_total.to_rel_index(&__edge_ind_common_total),
                           &__new_row,
                        ) && !::ascent::internal::RelFullIndexRead::contains_key(
                           &edge_indices_0_1_delta.to_rel_index(&__edge_ind_common_delta),
                           &__new_row,
                        ) {
                           if ::ascent::internal::CRelFullIndexWrite::insert_if_not_present(
                              &edge_indices_0_1_new.to_c_rel_index_write(&__edge_ind_common_new),
                              &__new_row,
                              (),
                           ) {
                              __new_edge = _self.edge.push((__new_row.0.clone(), __new_row.1.clone()));
                              __default_id = __new_edge;
                              ::ascent::internal::CRelIndexWrite::index_insert(
                                 &edge_indices_0_new.to_c_rel_index_write(&__edge_ind_common_new),
                                 (__new_row.0.clone(),),
                                 (__new_row.1.clone(),),
                              );
                              ::ascent::internal::CRelIndexWrite::index_insert(
                                 &edge_indices_none_new.to_c_rel_index_write(&__edge_ind_common_new),
                                 (),
                                 (__new_row.0.clone(), __new_row.1.clone()),
                              );
                              __changed.store(true, std::sync::atomic::Ordering::Relaxed);
                           } else {
                           }
                        } else {
                        }
                     });
                  }
               }
               ascent::internal::comment("edge_incremental <-- edge_indices_none_delta");
               if edge_indices_none_delta.to_rel_index(&__edge_ind_common_delta).len() > 0 {
                  if let Some(__matching) =
                     edge_indices_none_delta.to_rel_index(&__edge_ind_common_delta).c_index_get(&())
                  {
                     __matching.for_each(|__val| {
                        let mut __dep_changed = false;
                        let mut __default_id = 0;
                        let __val = __val.tuple_of_borrowed();
                        let x: &i32 = __val.0;
                        let y: &i32 = __val.1;
                        let __new_row: (i32, i32) =
                           (ascent::internal::Convert::convert(x), ascent::internal::Convert::convert(y));
                        let mut __new_edge_incremental = 0;
                        if !::ascent::internal::RelFullIndexRead::contains_key(
                           &edge_incremental_indices_0_1_total.to_rel_index(&__edge_incremental_ind_common_total),
                           &__new_row,
                        ) && !::ascent::internal::RelFullIndexRead::contains_key(
                           &edge_incremental_indices_0_1_delta.to_rel_index(&__edge_incremental_ind_common_delta),
                           &__new_row,
                        ) {
                           if ::ascent::internal::CRelFullIndexWrite::insert_if_not_present(
                              &edge_incremental_indices_0_1_new
                                 .to_c_rel_index_write(&__edge_incremental_ind_common_new),
                              &__new_row,
                              (),
                           ) {
                              __new_edge_incremental =
                                 _self.edge_incremental.push((__new_row.0.clone(), __new_row.1.clone()));
                              __default_id = __new_edge_incremental;
                              ::ascent::internal::CRelIndexWrite::index_insert(
                                 &edge_incremental_indices_none_new
                                    .to_c_rel_index_write(&__edge_incremental_ind_common_new),
                                 (),
                                 (__new_row.0.clone(), __new_row.1.clone()),
                              );
                              __changed.store(true, std::sync::atomic::Ordering::Relaxed);
                           } else {
                           }
                        } else {
                        }
                     });
                  }
               }
               ascent::internal::comment("path_incremental <-- outside_indices_none_delta, let ⋯, let ⋯");
               if outside_indices_none_delta.to_rel_index(&__outside_ind_common_delta).len() > 0 {
                  if let Some(__matching) =
                     outside_indices_none_delta.to_rel_index(&__outside_ind_common_delta).c_index_get(&())
                  {
                     __matching.for_each(|__val| {
                        let mut __dep_changed = false;
                        let mut __default_id = 0;
                        let x = 0;
                        let y = 0;
                        let __new_row: (i32, i32) =
                           (ascent::internal::Convert::convert(x), ascent::internal::Convert::convert(y));
                        let mut __new_path_incremental = 0;
                        if !::ascent::internal::RelFullIndexRead::contains_key(
                           &path_incremental_indices_0_1_total.to_rel_index(&__path_incremental_ind_common_total),
                           &__new_row,
                        ) && !::ascent::internal::RelFullIndexRead::contains_key(
                           &path_incremental_indices_0_1_delta.to_rel_index(&__path_incremental_ind_common_delta),
                           &__new_row,
                        ) {
                           if ::ascent::internal::CRelFullIndexWrite::insert_if_not_present(
                              &path_incremental_indices_0_1_new
                                 .to_c_rel_index_write(&__path_incremental_ind_common_new),
                              &__new_row,
                              (),
                           ) {
                              __new_path_incremental =
                                 _self.path_incremental.push((__new_row.0.clone(), __new_row.1.clone()));
                              __default_id = __new_path_incremental;
                              ::ascent::internal::CRelIndexWrite::index_insert(
                                 &path_incremental_indices_none_new
                                    .to_c_rel_index_write(&__path_incremental_ind_common_new),
                                 (),
                                 (__new_row.0.clone(), __new_row.1.clone()),
                              );
                              __changed.store(true, std::sync::atomic::Ordering::Relaxed);
                           } else {
                           }
                        } else {
                        }
                     });
                  }
               }
               ascent::internal::comment("path <-- path_incremental_indices_none_delta");
               if path_incremental_indices_none_delta.to_rel_index(&__path_incremental_ind_common_delta).len() > 0 {
                  if let Some(__matching) = path_incremental_indices_none_delta
                     .to_rel_index(&__path_incremental_ind_common_delta)
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
                        let mut __new_path = 0;
                        if !::ascent::internal::RelFullIndexRead::contains_key(
                           &path_indices_0_1_total.to_rel_index(&__path_ind_common_total),
                           &__new_row,
                        ) && !::ascent::internal::RelFullIndexRead::contains_key(
                           &path_indices_0_1_delta.to_rel_index(&__path_ind_common_delta),
                           &__new_row,
                        ) {
                           if ::ascent::internal::CRelFullIndexWrite::insert_if_not_present(
                              &path_indices_0_1_new.to_c_rel_index_write(&__path_ind_common_new),
                              &__new_row,
                              (),
                           ) {
                              __new_path = _self.path.push((__new_row.0.clone(), __new_row.1.clone()));
                              __default_id = __new_path;
                              ::ascent::internal::CRelIndexWrite::index_insert(
                                 &path_indices_1_new.to_c_rel_index_write(&__path_ind_common_new),
                                 (__new_row.1.clone(),),
                                 (__new_row.0.clone(),),
                              );
                              ::ascent::internal::CRelIndexWrite::index_insert(
                                 &path_indices_none_new.to_c_rel_index_write(&__path_ind_common_new),
                                 (),
                                 (__new_row.0.clone(), __new_row.1.clone()),
                              );
                              __changed.store(true, std::sync::atomic::Ordering::Relaxed);
                           } else {
                           }
                        } else {
                        }
                     });
                  }
               }
               ascent::internal::comment("path_incremental <-- path_indices_none_delta");
               if path_indices_none_delta.to_rel_index(&__path_ind_common_delta).len() > 0 {
                  if let Some(__matching) =
                     path_indices_none_delta.to_rel_index(&__path_ind_common_delta).c_index_get(&())
                  {
                     __matching.for_each(|__val| {
                        let mut __dep_changed = false;
                        let mut __default_id = 0;
                        let __val = __val.tuple_of_borrowed();
                        let x: &i32 = __val.0;
                        let y: &i32 = __val.1;
                        let __new_row: (i32, i32) =
                           (ascent::internal::Convert::convert(x), ascent::internal::Convert::convert(y));
                        let mut __new_path_incremental = 0;
                        if !::ascent::internal::RelFullIndexRead::contains_key(
                           &path_incremental_indices_0_1_total.to_rel_index(&__path_incremental_ind_common_total),
                           &__new_row,
                        ) && !::ascent::internal::RelFullIndexRead::contains_key(
                           &path_incremental_indices_0_1_delta.to_rel_index(&__path_incremental_ind_common_delta),
                           &__new_row,
                        ) {
                           if ::ascent::internal::CRelFullIndexWrite::insert_if_not_present(
                              &path_incremental_indices_0_1_new
                                 .to_c_rel_index_write(&__path_incremental_ind_common_new),
                              &__new_row,
                              (),
                           ) {
                              __new_path_incremental =
                                 _self.path_incremental.push((__new_row.0.clone(), __new_row.1.clone()));
                              __default_id = __new_path_incremental;
                              ::ascent::internal::CRelIndexWrite::index_insert(
                                 &path_incremental_indices_none_new
                                    .to_c_rel_index_write(&__path_incremental_ind_common_new),
                                 (),
                                 (__new_row.0.clone(), __new_row.1.clone()),
                              );
                              __changed.store(true, std::sync::atomic::Ordering::Relaxed);
                           } else {
                           }
                        } else {
                        }
                     });
                  }
               }
               ascent::internal::comment("outside <-- edge_indices_none_delta");
               if edge_indices_none_delta.to_rel_index(&__edge_ind_common_delta).len() > 0 {
                  if let Some(__matching) =
                     edge_indices_none_delta.to_rel_index(&__edge_ind_common_delta).c_index_get(&())
                  {
                     __matching.for_each(|__val| {
                        let mut __dep_changed = false;
                        let mut __default_id = 0;
                        let __val = __val.tuple_of_borrowed();
                        let x: &i32 = __val.0;
                        let y: &i32 = __val.1;
                        let __new_row: () = ();
                        let mut __new_outside = 0;
                        if !::ascent::internal::RelFullIndexRead::contains_key(
                           &outside_indices_none_total.to_rel_index(&__outside_ind_common_total),
                           &__new_row,
                        ) && !::ascent::internal::RelFullIndexRead::contains_key(
                           &outside_indices_none_delta.to_rel_index(&__outside_ind_common_delta),
                           &__new_row,
                        ) {
                           if ::ascent::internal::CRelFullIndexWrite::insert_if_not_present(
                              &outside_indices_none_new.to_c_rel_index_write(&__outside_ind_common_new),
                              &__new_row,
                              (),
                           ) {
                              __new_outside = _self.outside.push(());
                              __default_id = __new_outside;
                              __changed.store(true, std::sync::atomic::Ordering::Relaxed);
                           } else {
                           }
                        } else {
                        }
                     });
                  }
               }
               ascent::internal::comment("outside <-- path_indices_none_delta");
               if path_indices_none_delta.to_rel_index(&__path_ind_common_delta).len() > 0 {
                  if let Some(__matching) =
                     path_indices_none_delta.to_rel_index(&__path_ind_common_delta).c_index_get(&())
                  {
                     __matching.for_each(|__val| {
                        let mut __dep_changed = false;
                        let mut __default_id = 0;
                        let __val = __val.tuple_of_borrowed();
                        let x: &i32 = __val.0;
                        let y: &i32 = __val.1;
                        let __new_row: () = ();
                        let mut __new_outside = 0;
                        if !::ascent::internal::RelFullIndexRead::contains_key(
                           &outside_indices_none_total.to_rel_index(&__outside_ind_common_total),
                           &__new_row,
                        ) && !::ascent::internal::RelFullIndexRead::contains_key(
                           &outside_indices_none_delta.to_rel_index(&__outside_ind_common_delta),
                           &__new_row,
                        ) {
                           if ::ascent::internal::CRelFullIndexWrite::insert_if_not_present(
                              &outside_indices_none_new.to_c_rel_index_write(&__outside_ind_common_new),
                              &__new_row,
                              (),
                           ) {
                              __new_outside = _self.outside.push(());
                              __default_id = __new_outside;
                              __changed.store(true, std::sync::atomic::Ordering::Relaxed);
                           } else {
                           }
                        } else {
                        }
                     });
                  }
               }
               ascent::internal::comment("path <-- edge_indices_none_delta");
               if edge_indices_none_delta.to_rel_index(&__edge_ind_common_delta).len() > 0 {
                  if let Some(__matching) =
                     edge_indices_none_delta.to_rel_index(&__edge_ind_common_delta).c_index_get(&())
                  {
                     __matching.for_each(|__val| {
                        let mut __dep_changed = false;
                        let mut __default_id = 0;
                        let __val = __val.tuple_of_borrowed();
                        let x: &i32 = __val.0;
                        let y: &i32 = __val.1;
                        let __new_row: (i32, i32) =
                           (ascent::internal::Convert::convert(x), ascent::internal::Convert::convert(y));
                        let mut __new_path = 0;
                        if !::ascent::internal::RelFullIndexRead::contains_key(
                           &path_indices_0_1_total.to_rel_index(&__path_ind_common_total),
                           &__new_row,
                        ) && !::ascent::internal::RelFullIndexRead::contains_key(
                           &path_indices_0_1_delta.to_rel_index(&__path_ind_common_delta),
                           &__new_row,
                        ) {
                           if ::ascent::internal::CRelFullIndexWrite::insert_if_not_present(
                              &path_indices_0_1_new.to_c_rel_index_write(&__path_ind_common_new),
                              &__new_row,
                              (),
                           ) {
                              __new_path = _self.path.push((__new_row.0.clone(), __new_row.1.clone()));
                              __default_id = __new_path;
                              ::ascent::internal::CRelIndexWrite::index_insert(
                                 &path_indices_1_new.to_c_rel_index_write(&__path_ind_common_new),
                                 (__new_row.1.clone(),),
                                 (__new_row.0.clone(),),
                              );
                              ::ascent::internal::CRelIndexWrite::index_insert(
                                 &path_indices_none_new.to_c_rel_index_write(&__path_ind_common_new),
                                 (),
                                 (__new_row.0.clone(), __new_row.1.clone()),
                              );
                              __changed.store(true, std::sync::atomic::Ordering::Relaxed);
                           } else {
                           }
                        } else {
                        }
                     });
                  }
               }
               ascent::internal::comment(
                  "path <-- path_indices_1_delta, edge_indices_0_total+delta [SIMPLE JOIN] [NOT REORDERABLE]",
               );
               if path_indices_1_delta.to_rel_index(&__path_ind_common_delta).len() > 0 {
                  path_indices_1_delta.to_rel_index(&__path_ind_common_delta).c_iter_all().for_each(
                     |(__cl1_joined_columns, __cl1_tuple_indices)| {
                        let __cl1_joined_columns = __cl1_joined_columns.tuple_of_borrowed();
                        let y = __cl1_joined_columns.0;
                        if let Some(__matching) = ascent::internal::RelIndexCombined::new(
                           &edge_indices_0_total.to_rel_index(&__edge_ind_common_total),
                           &edge_indices_0_delta.to_rel_index(&__edge_ind_common_delta),
                        )
                        .c_index_get(&(y.clone(),))
                        {
                           __cl1_tuple_indices.for_each(|cl1_val| {
                              let cl1_val = cl1_val.tuple_of_borrowed();
                              let x: &i32 = cl1_val.0;
                              __matching.clone().for_each(|__val| {
                                 let mut __dep_changed = false;
                                 let mut __default_id = 0;
                                 let __val = __val.tuple_of_borrowed();
                                 let z: &i32 = __val.0;
                                 let __new_row: (i32, i32) =
                                    (ascent::internal::Convert::convert(x), ascent::internal::Convert::convert(z));
                                 let mut __new_path = 0;
                                 if !::ascent::internal::RelFullIndexRead::contains_key(
                                    &path_indices_0_1_total.to_rel_index(&__path_ind_common_total),
                                    &__new_row,
                                 ) && !::ascent::internal::RelFullIndexRead::contains_key(
                                    &path_indices_0_1_delta.to_rel_index(&__path_ind_common_delta),
                                    &__new_row,
                                 ) {
                                    if ::ascent::internal::CRelFullIndexWrite::insert_if_not_present(
                                       &path_indices_0_1_new.to_c_rel_index_write(&__path_ind_common_new),
                                       &__new_row,
                                       (),
                                    ) {
                                       __new_path = _self.path.push((__new_row.0.clone(), __new_row.1.clone()));
                                       __default_id = __new_path;
                                       ::ascent::internal::CRelIndexWrite::index_insert(
                                          &path_indices_1_new.to_c_rel_index_write(&__path_ind_common_new),
                                          (__new_row.1.clone(),),
                                          (__new_row.0.clone(),),
                                       );
                                       ::ascent::internal::CRelIndexWrite::index_insert(
                                          &path_indices_none_new.to_c_rel_index_write(&__path_ind_common_new),
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
               __edge_ind_common_total.unfreeze();
               __edge_ind_common_delta.unfreeze();
               edge_indices_0_total.unfreeze();
               edge_indices_0_delta.unfreeze();
               edge_indices_0_1_total.unfreeze();
               edge_indices_0_1_delta.unfreeze();
               edge_indices_none_total.unfreeze();
               edge_indices_none_delta.unfreeze();
               __edge_incremental_ind_common_total.unfreeze();
               __edge_incremental_ind_common_delta.unfreeze();
               edge_incremental_indices_0_1_total.unfreeze();
               edge_incremental_indices_0_1_delta.unfreeze();
               edge_incremental_indices_none_total.unfreeze();
               edge_incremental_indices_none_delta.unfreeze();
               __outside_ind_common_total.unfreeze();
               __outside_ind_common_delta.unfreeze();
               outside_indices_none_total.unfreeze();
               outside_indices_none_delta.unfreeze();
               __path_ind_common_total.unfreeze();
               __path_ind_common_delta.unfreeze();
               path_indices_0_1_total.unfreeze();
               path_indices_0_1_delta.unfreeze();
               path_indices_1_total.unfreeze();
               path_indices_1_delta.unfreeze();
               path_indices_none_total.unfreeze();
               path_indices_none_delta.unfreeze();
               __path_incremental_ind_common_total.unfreeze();
               __path_incremental_ind_common_delta.unfreeze();
               path_incremental_indices_0_1_total.unfreeze();
               path_incremental_indices_0_1_delta.unfreeze();
               path_incremental_indices_none_total.unfreeze();
               path_incremental_indices_none_delta.unfreeze();
               ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
                  &mut __edge_ind_common_new,
                  &mut __edge_ind_common_delta,
                  &mut __edge_ind_common_total,
               );
               ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
                  &mut edge_indices_0_new.to_rel_index_write(&mut __edge_ind_common_new),
                  &mut edge_indices_0_delta.to_rel_index_write(&mut __edge_ind_common_delta),
                  &mut edge_indices_0_total.to_rel_index_write(&mut __edge_ind_common_total),
               );
               ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
                  &mut edge_indices_0_1_new.to_rel_index_write(&mut __edge_ind_common_new),
                  &mut edge_indices_0_1_delta.to_rel_index_write(&mut __edge_ind_common_delta),
                  &mut edge_indices_0_1_total.to_rel_index_write(&mut __edge_ind_common_total),
               );
               ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
                  &mut edge_indices_none_new.to_rel_index_write(&mut __edge_ind_common_new),
                  &mut edge_indices_none_delta.to_rel_index_write(&mut __edge_ind_common_delta),
                  &mut edge_indices_none_total.to_rel_index_write(&mut __edge_ind_common_total),
               );
               ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
                  &mut __edge_incremental_ind_common_new,
                  &mut __edge_incremental_ind_common_delta,
                  &mut __edge_incremental_ind_common_total,
               );
               ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
                  &mut edge_incremental_indices_0_1_new.to_rel_index_write(&mut __edge_incremental_ind_common_new),
                  &mut edge_incremental_indices_0_1_delta.to_rel_index_write(&mut __edge_incremental_ind_common_delta),
                  &mut edge_incremental_indices_0_1_total.to_rel_index_write(&mut __edge_incremental_ind_common_total),
               );
               ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
                  &mut edge_incremental_indices_none_new.to_rel_index_write(&mut __edge_incremental_ind_common_new),
                  &mut edge_incremental_indices_none_delta.to_rel_index_write(&mut __edge_incremental_ind_common_delta),
                  &mut edge_incremental_indices_none_total.to_rel_index_write(&mut __edge_incremental_ind_common_total),
               );
               ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
                  &mut __outside_ind_common_new,
                  &mut __outside_ind_common_delta,
                  &mut __outside_ind_common_total,
               );
               ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
                  &mut outside_indices_none_new.to_rel_index_write(&mut __outside_ind_common_new),
                  &mut outside_indices_none_delta.to_rel_index_write(&mut __outside_ind_common_delta),
                  &mut outside_indices_none_total.to_rel_index_write(&mut __outside_ind_common_total),
               );
               ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
                  &mut __path_ind_common_new,
                  &mut __path_ind_common_delta,
                  &mut __path_ind_common_total,
               );
               ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
                  &mut path_indices_0_1_new.to_rel_index_write(&mut __path_ind_common_new),
                  &mut path_indices_0_1_delta.to_rel_index_write(&mut __path_ind_common_delta),
                  &mut path_indices_0_1_total.to_rel_index_write(&mut __path_ind_common_total),
               );
               ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
                  &mut path_indices_1_new.to_rel_index_write(&mut __path_ind_common_new),
                  &mut path_indices_1_delta.to_rel_index_write(&mut __path_ind_common_delta),
                  &mut path_indices_1_total.to_rel_index_write(&mut __path_ind_common_total),
               );
               ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
                  &mut path_indices_none_new.to_rel_index_write(&mut __path_ind_common_new),
                  &mut path_indices_none_delta.to_rel_index_write(&mut __path_ind_common_delta),
                  &mut path_indices_none_total.to_rel_index_write(&mut __path_ind_common_total),
               );
               ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
                  &mut __path_incremental_ind_common_new,
                  &mut __path_incremental_ind_common_delta,
                  &mut __path_incremental_ind_common_total,
               );
               ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
                  &mut path_incremental_indices_0_1_new.to_rel_index_write(&mut __path_incremental_ind_common_new),
                  &mut path_incremental_indices_0_1_delta.to_rel_index_write(&mut __path_incremental_ind_common_delta),
                  &mut path_incremental_indices_0_1_total.to_rel_index_write(&mut __path_incremental_ind_common_total),
               );
               ::ascent::internal::RelIndexMerge::merge_delta_to_total_new_to_delta(
                  &mut path_incremental_indices_none_new.to_rel_index_write(&mut __path_incremental_ind_common_new),
                  &mut path_incremental_indices_none_delta.to_rel_index_write(&mut __path_incremental_ind_common_delta),
                  &mut path_incremental_indices_none_total.to_rel_index_write(&mut __path_incremental_ind_common_total),
               );
               _self.scc_iters[0usize] += 1;
               if !__changed.load(std::sync::atomic::Ordering::Relaxed) {
                  break;
               }
               __check_return_conditions!();
            }
            _self.__edge_ind_common = __edge_ind_common_total;
            _self.edge_indices_0 = edge_indices_0_total;
            _self.edge_indices_0_1 = edge_indices_0_1_total;
            _self.edge_indices_none = edge_indices_none_total;
            _self.__edge_incremental_ind_common = __edge_incremental_ind_common_total;
            _self.edge_incremental_indices_0_1 = edge_incremental_indices_0_1_total;
            _self.edge_incremental_indices_none = edge_incremental_indices_none_total;
            _self.__outside_ind_common = __outside_ind_common_total;
            _self.outside_indices_none = outside_indices_none_total;
            _self.__path_ind_common = __path_ind_common_total;
            _self.path_indices_0_1 = path_indices_0_1_total;
            _self.path_indices_1 = path_indices_1_total;
            _self.path_indices_none = path_indices_none_total;
            _self.__path_incremental_ind_common = __path_incremental_ind_common_total;
            _self.path_incremental_indices_0_1 = path_incremental_indices_0_1_total;
            _self.path_incremental_indices_none = path_incremental_indices_none_total;
            _self.scc_times[0usize] += _scc_start_time.elapsed();
         }
      }
      #[allow(noop_method_call, suspicious_double_ref_op)]
      pub fn update_indices_priv(&mut self) {
         let before = ::ascent::internal::Instant::now();
         self.update_indices_edge();
         self.update_indices_edge_incremental();
         self.update_indices_outside();
         self.update_indices_path();
         self.update_indices_path_incremental();
         self.update_indices_duration += before.elapsed();
      }
      #[allow(noop_method_call, suspicious_double_ref_op)]
      pub fn update_indices_edge(&mut self) {
         use ascent::internal::CRelIndexWrite;
         use ascent::internal::ToRelIndex0;
         use ascent::rayon::iter::{IntoParallelIterator, ParallelIterator};
         (0..self.edge.len()).into_par_iter().for_each(|_i| {
            let tuple = &self.edge[_i];
            let selection_tuple = (tuple.0.clone(),);
            let rel_ind = &self.edge_indices_0;
            ascent::internal::CRelIndexWrite::index_insert(
               &rel_ind.to_c_rel_index_write(&self.__edge_ind_common),
               selection_tuple,
               (tuple.1.clone(),),
            );
            let selection_tuple = (tuple.0.clone(), tuple.1.clone());
            let rel_ind = &self.edge_indices_0_1;
            ascent::internal::CRelIndexWrite::index_insert(
               &rel_ind.to_c_rel_index_write(&self.__edge_ind_common),
               selection_tuple,
               (),
            );
            let selection_tuple = ();
            let rel_ind = &self.edge_indices_none;
            ascent::internal::CRelIndexWrite::index_insert(
               &rel_ind.to_c_rel_index_write(&self.__edge_ind_common),
               selection_tuple,
               (tuple.0.clone(), tuple.1.clone()),
            );
         });
      }
      #[allow(noop_method_call, suspicious_double_ref_op)]
      pub fn update_indices_edge_incremental(&mut self) {
         use ascent::internal::CRelIndexWrite;
         use ascent::internal::ToRelIndex0;
         use ascent::rayon::iter::{IntoParallelIterator, ParallelIterator};
         (0..self.edge_incremental.len()).into_par_iter().for_each(|_i| {
            let tuple = &self.edge_incremental[_i];
            let selection_tuple = (tuple.0.clone(), tuple.1.clone());
            let rel_ind = &self.edge_incremental_indices_0_1;
            ascent::internal::CRelIndexWrite::index_insert(
               &rel_ind.to_c_rel_index_write(&self.__edge_incremental_ind_common),
               selection_tuple,
               (),
            );
            let selection_tuple = ();
            let rel_ind = &self.edge_incremental_indices_none;
            ascent::internal::CRelIndexWrite::index_insert(
               &rel_ind.to_c_rel_index_write(&self.__edge_incremental_ind_common),
               selection_tuple,
               (tuple.0.clone(), tuple.1.clone()),
            );
         });
      }
      #[allow(noop_method_call, suspicious_double_ref_op)]
      pub fn update_indices_outside(&mut self) {
         use ascent::internal::CRelIndexWrite;
         use ascent::internal::ToRelIndex0;
         use ascent::rayon::iter::{IntoParallelIterator, ParallelIterator};
         (0..self.outside.len()).into_par_iter().for_each(|_i| {
            let tuple = &self.outside[_i];
            let selection_tuple = ();
            let rel_ind = &self.outside_indices_none;
            ascent::internal::CRelIndexWrite::index_insert(
               &rel_ind.to_c_rel_index_write(&self.__outside_ind_common),
               selection_tuple,
               (),
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
            let rel_ind = &self.path_indices_0_1;
            ascent::internal::CRelIndexWrite::index_insert(
               &rel_ind.to_c_rel_index_write(&self.__path_ind_common),
               selection_tuple,
               (),
            );
            let selection_tuple = (tuple.1.clone(),);
            let rel_ind = &self.path_indices_1;
            ascent::internal::CRelIndexWrite::index_insert(
               &rel_ind.to_c_rel_index_write(&self.__path_ind_common),
               selection_tuple,
               (tuple.0.clone(),),
            );
            let selection_tuple = ();
            let rel_ind = &self.path_indices_none;
            ascent::internal::CRelIndexWrite::index_insert(
               &rel_ind.to_c_rel_index_write(&self.__path_ind_common),
               selection_tuple,
               (tuple.0.clone(), tuple.1.clone()),
            );
         });
      }
      #[allow(noop_method_call, suspicious_double_ref_op)]
      pub fn update_indices_path_incremental(&mut self) {
         use ascent::internal::CRelIndexWrite;
         use ascent::internal::ToRelIndex0;
         use ascent::rayon::iter::{IntoParallelIterator, ParallelIterator};
         (0..self.path_incremental.len()).into_par_iter().for_each(|_i| {
            let tuple = &self.path_incremental[_i];
            let selection_tuple = (tuple.0.clone(), tuple.1.clone());
            let rel_ind = &self.path_incremental_indices_0_1;
            ascent::internal::CRelIndexWrite::index_insert(
               &rel_ind.to_c_rel_index_write(&self.__path_incremental_ind_common),
               selection_tuple,
               (),
            );
            let selection_tuple = ();
            let rel_ind = &self.path_incremental_indices_none;
            ascent::internal::CRelIndexWrite::index_insert(
               &rel_ind.to_c_rel_index_write(&self.__path_incremental_ind_common),
               selection_tuple,
               (tuple.0.clone(), tuple.1.clone()),
            );
         });
      }
      #[deprecated = "Explicit call to update_indices not required anymore."]
      pub fn update_indices(&mut self) { self.update_indices_priv(); }
      fn type_constraints() {
         let _type_constraints: ascent::internal::TypeConstraints<i32>;
         let _par_constraints: ascent::internal::ParTypeConstraints<i32>;
      }
      pub fn summary() -> &'static str {
         "scc 0, is_looping: true:\n  edge_incremental <-- outside_indices_none_delta, let ⋯, let ⋯\n  edge <-- edge_incremental_indices_none_delta\n  edge_incremental <-- edge_indices_none_delta\n  path_incremental <-- outside_indices_none_delta, let ⋯, let ⋯\n  path <-- path_incremental_indices_none_delta\n  path_incremental <-- path_indices_none_delta\n  outside <-- edge_indices_none_delta\n  outside <-- path_indices_none_delta\n  path <-- edge_indices_none_delta\n  path <-- path_indices_1_delta, edge_indices_0_total+delta [SIMPLE JOIN] [NOT REORDERABLE]\n  dynamic relations: edge, edge_incremental, outside, path, path_incremental\n"
      }
      pub fn relation_sizes_summary(&self) -> String {
         use std::fmt::Write;
         let mut res = String::new();
         writeln!(&mut res, "{} size: {}", "edge", self.edge.len()).unwrap();
         writeln!(&mut res, "{} size: {}", "edge_incremental", self.edge_incremental.len()).unwrap();
         writeln!(&mut res, "{} size: {}", "outside", self.outside.len()).unwrap();
         writeln!(&mut res, "{} size: {}", "path", self.path.len()).unwrap();
         writeln!(&mut res, "{} size: {}", "path_incremental", self.path_incremental.len()).unwrap();
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
   impl Default for TCIncremental {
      fn default() -> Self {
         let mut _self = TCIncremental {
            edge: Default::default(),
            __edge_ind_common: Default::default(),
            edge_indices_0: Default::default(),
            edge_indices_0_1: Default::default(),
            edge_indices_none: Default::default(),
            edge_incremental: Default::default(),
            __edge_incremental_ind_common: Default::default(),
            edge_incremental_indices_0_1: Default::default(),
            edge_incremental_indices_none: Default::default(),
            outside: Default::default(),
            __outside_ind_common: Default::default(),
            outside_indices_none: Default::default(),
            path: Default::default(),
            __path_ind_common: Default::default(),
            path_indices_0_1: Default::default(),
            path_indices_1: Default::default(),
            path_indices_none: Default::default(),
            path_incremental: Default::default(),
            __path_incremental_ind_common: Default::default(),
            path_incremental_indices_0_1: Default::default(),
            path_incremental_indices_none: Default::default(),
            scc_times: [std::time::Duration::ZERO; 1usize],
            scc_iters: [0; 1usize],
            update_time_nanos: Default::default(),
            update_indices_duration: std::time::Duration::default(),
         };
         _self
      }
   };
}
