//! Provides definitions required for the `ascent` macro(s), plus traits that custom relations need to implement.

pub use crate::convert::*;

use std::sync::atomic::AtomicI32;
use std::sync::Arc;
use std::time::Duration;
use std::hash::{BuildHasherDefault, Hash};
use std::collections::{HashMap, HashSet};

use cfg_if::cfg_if;
pub use instant::Instant;

use ascent_base::Lattice;
use rustc_hash::FxHasher;

pub use crate::rel_index_read::RelIndexCombined;
pub use crate::rel_index_read::RelIndexRead;
pub use crate::rel_index_read::RelIndexReadAll;

pub type RelIndexType<K> = RelIndexType1<K, usize>;

pub type LatticeIndexType<K, V> = HashMap<K, HashSet<V, BuildHasherDefault<FxHasher>>, BuildHasherDefault<FxHasher>>;

pub(crate) type HashBrownRelFullIndexType<K, V> = hashbrown::HashMap<K, V, BuildHasherDefault<FxHasher>>;
pub type RelFullIndexType<K, V> = HashBrownRelFullIndexType<K, V>;

pub struct LatticeMap<K, V>(pub hashbrown::HashMap<K, V, BuildHasherDefault<FxHasher>>);
pub type LatticeFullIndexType<K, V> = LatticeMap<K, V>;

pub type RelNoIndexType = Vec<usize>;

cfg_if! {
   if #[cfg(feature = "par")] {
      pub use crate::c_rel_index_read::CRelIndexRead;
      pub use crate::c_rel_index_read::CRelIndexReadAll;

      pub use crate::c_rel_index::shards_count;

      pub use crate::c_rel_index::CRelIndex;
      pub use crate::c_rel_full_index::CRelFullIndex;
      pub use crate::c_lat_index::CLatIndex;
      pub use crate::c_rel_no_index::CRelNoIndex;
      pub use crate::c_rel_index::DashMapViewParIter;
   }
}

pub use crate::to_rel_index::{ToRelIndex0, ToRelIndex};
pub use crate::tuple_of_borrowed::TupleOfBorrowed;


pub trait Freezable {
   fn freeze(&mut self) { }
   fn unfreeze(&mut self) { }
}

pub trait RelIndexWrite: Sized {
   type Key;
   type Value;
   fn index_insert(&mut self, key: Self::Key, value: Self::Value);
}

pub trait RelIndexMerge: Sized {
   fn move_index_contents(from: &mut Self, to: &mut Self);
   fn merge_delta_to_total_new_to_delta(new: &mut Self, delta: &mut Self, total: &mut Self) {
      Self::move_index_contents(delta, total);
      std::mem::swap(new, delta);
   }

   /// Called once at the start of the SCC
   #[allow(unused_variables)]
   fn init(new: &mut Self, delta: &mut Self, total: &mut Self) { }
}

pub trait RelIndexDelete : Sized {
   type Key;
   type Value;
   fn remove_if_present(&mut self, key: &Self::Key, value: &Self::Value) -> bool;
}

pub trait CRelIndexWrite{
   type Key;
   type Value;
   fn index_insert(&self, key: Self::Key, value: Self::Value);
}

pub trait RelFullIndexRead<'a> {
   type Key;
   fn contains_key(&'a self, key: &Self::Key) -> bool;
}

pub trait Counter {
   fn inc(&mut self);
}

pub trait AtomicCounter {
   fn inc_atomic(&self);
   fn add_atomic(&self, value: i32);
}

impl Counter for i32 {
   fn inc(&mut self) {
      *self += 1;
   }
}

impl Counter for usize {
   fn inc(&mut self) {
      *self += 1;
   }
}

impl AtomicCounter for AtomicI32 {
   fn inc_atomic(&self) {
      self.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
   }

   fn add_atomic(&self, value: i32) {
      self.fetch_add(value, std::sync::atomic::Ordering::Relaxed);
   }
}

#[derive(Clone, Debug)]
pub struct FullRelCounter {
   pub counter: Arc<AtomicI32>
}

impl AtomicCounter for FullRelCounter {
   fn inc_atomic(&self) {
      self.counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
   }

   fn add_atomic(&self, value: i32) {
      self.counter.fetch_add(value, std::sync::atomic::Ordering::Relaxed);
   }
}

impl Into<FullRelCounter> for i32 {
   fn into(self) -> FullRelCounter {
      FullRelCounter{counter: Arc::new(AtomicI32::new(self))}
   }
}

impl Into<FullRelCounter> for usize {
   fn into(self) -> FullRelCounter {
      FullRelCounter{counter: Arc::new(AtomicI32::new(self as i32))}
   }
}


pub trait RelFullIndexWrite {
   type Key: Clone;
   type Value;
   /// if an entry for `key` does not exist, inserts `v` for it and returns true.
   fn insert_if_not_present(&mut self, key: &Self::Key, v: Self::Value) -> bool;
}

pub trait CRelFullIndexWrite {
   type Key: Clone;
   type Value;
   /// if an entry for `key` does not exist, inserts `v` for it and returns true.
   fn insert_if_not_present(&self, key: &Self::Key, v: Self::Value) -> bool;
}

pub trait RelFullIndexDelete {
   type Key: Clone;
   type Value;
   /// if an entry for `key` exists, removes it and returns true.
   fn remove_if_present(&mut self, key: &Self::Key) -> bool;
}


pub type RelIndexType1<K, V> = HashMap<K, Vec<V>, BuildHasherDefault<FxHasher>>;

pub static mut MOVE_REL_INDEX_CONTENTS_TOTAL_TIME : Duration = Duration::ZERO;
pub static mut INDEX_INSERT_TOTAL_TIME : Duration = Duration::ZERO;

impl<K: Eq + Hash, V> RelIndexWrite for RelIndexType1<K, V>{
   type Key = K;
   type Value = V;

   fn index_insert(&mut self, key: K, value: V) {
      // let before = Instant::now();
      use std::collections::hash_map::Entry::*;
      match self.entry(key){
         Occupied(mut vec) => vec.get_mut().push(value),
         Vacant(vacant) => {
            let mut vec = Vec::with_capacity(4);
            vec.push(value);
            vacant.insert(vec);
         },
      }
      // unsafe {
      //    INDEX_INSERT_TOTAL_TIME += before.elapsed();
      // }
   }
}

impl<K: Eq + Hash, V> RelIndexMerge for RelIndexType1<K, V> {
   fn move_index_contents(from: &mut RelIndexType1<K, V>, to: &mut RelIndexType1<K, V>) {
      let before = Instant::now();
      if from.len() > to.len() {
         std::mem::swap(from, to);
      }
      use std::collections::hash_map::Entry::*;
      for (k, mut v) in from.drain() {
         match to.entry(k) {
            Occupied(existing) => {
               let existing = existing.into_mut();
               if v.len() > existing.len() {
                  std::mem::swap(&mut v, existing);
               }
               existing.append(&mut v);
            },
            Vacant(vacant) => {
               vacant.insert(v);
            },
         }
      }
      unsafe {
         MOVE_REL_INDEX_CONTENTS_TOTAL_TIME += before.elapsed();
      }
   }
}

impl<K: Eq + Hash, V: Eq> RelIndexDelete for RelIndexType1<K, V> {
   type Key = K;
   type Value = V;

   fn remove_if_present(&mut self, key: &Self::Key, value: &Self::Value) -> bool {
      if let Some(vec) = self.get_mut(key) {
         let index = vec.iter().position(|v| v == value);
         if let Some(index) = index {
            vec.swap_remove(index);
            if vec.is_empty() {
               self.remove(key);
            }
            return true;
         }
      }
      false
   }
}

impl RelIndexWrite for RelNoIndexType {
   type Key = ();
   type Value = usize;

   fn index_insert(&mut self, _key: Self::Key, tuple_index: usize) {
      self.push(tuple_index);
   }
}

impl RelIndexMerge for RelNoIndexType {
   fn move_index_contents(ind1: &mut Self, ind2: &mut Self) {
      ind2.append(ind1);
   }
}

impl<K: Eq + Hash, V: Hash + Eq> RelIndexWrite for LatticeIndexType<K, V>{
   type Key = K;
   type Value = V;

   #[inline(always)]
   fn index_insert(&mut self, key: Self::Key, tuple_index: V) {
      self.entry(key).or_default().insert(tuple_index);
   }
}

impl<K: Eq + Hash, V: Hash + Eq> RelIndexMerge for LatticeIndexType<K, V>{
   #[inline(always)]
   fn move_index_contents(hm1: &mut LatticeIndexType<K, V>, hm2: &mut LatticeIndexType<K, V>) {
      for (k,v) in hm1.drain(){
         let set = hm2.entry(k).or_default();
         set.extend(v);
      }
   }
}


pub static mut MOVE_FULL_INDEX_CONTENTS_TOTAL_TIME : Duration = Duration::ZERO;
pub static mut MOVE_NO_INDEX_CONTENTS_TOTAL_TIME : Duration = Duration::ZERO;

impl<K: Eq + Hash, V> RelIndexWrite for HashBrownRelFullIndexType<K, V>{
    type Key = K;
    type Value = V;

   #[inline(always)]
   fn index_insert(&mut self, key: Self::Key, value: V) {
      self.insert(key, value);
   }
}

impl<K: Eq + Hash, V: AtomicCounter> RelIndexMerge for HashBrownRelFullIndexType<K, V> {
   fn move_index_contents(from: &mut Self, to: &mut Self) {
      let before = Instant::now();
      if from.len() > to.len() {
         std::mem::swap(from, to);
      }
      to.reserve(from.len());
      for (k, v) in from.drain() {
         // to.insert(k, v); // TODO could be improved
         match to.raw_entry_mut().from_key(&k) {
            hashbrown::hash_map::RawEntryMut::Occupied(occupied) => {
               let val = occupied.into_mut();
               val.inc_atomic();
            },
            hashbrown::hash_map::RawEntryMut::Vacant(vacant) => {vacant.insert(k, v);},
         }
      }
      unsafe {
         MOVE_FULL_INDEX_CONTENTS_TOTAL_TIME += before.elapsed();
      }
   }
}

impl <K: Clone + Hash + Eq, V: AtomicCounter> RelFullIndexWrite for HashBrownRelFullIndexType<K, V> {
   type Key = K;
   type Value = V;
   #[inline]
   fn insert_if_not_present(&mut self, key: &K, v: V) -> bool {
      match self.raw_entry_mut().from_key(key) {
         hashbrown::hash_map::RawEntryMut::Occupied(occupied) => {
            let val = occupied.into_mut();
            val.inc_atomic();
            false
         },
         hashbrown::hash_map::RawEntryMut::Vacant(vacant) => {vacant.insert(key.clone(), v); true},
      }
   }
}

impl<'a, K: Hash + Eq, V: AtomicCounter> RelFullIndexRead<'a> for HashBrownRelFullIndexType<K, V> {
    type Key = K;

   fn contains_key(&self, key: &Self::Key) -> bool {
      // self.contains_key(key)
      match self.raw_entry().from_key(key) {
        Some((_, exists_v)) => {
         exists_v.inc_atomic();
         true
        },
        None => false,
      }
   }
}


impl<K: Eq + Hash, V> RelIndexWrite for LatticeMap<K, V>{
   type Key = K;
   type Value = V;

  #[inline(always)]
  fn index_insert(&mut self, key: Self::Key, value: V) {
     self.0.insert(key, value);
  }
}

impl<K: Eq + Hash, V> RelIndexMerge for LatticeMap<K, V> {
   fn move_index_contents(from_r: &mut Self, to_r: &mut Self) {
      let before = Instant::now();
      let from = &mut from_r.0;
      let to = &mut to_r.0;
      if from.len() > to.len() {
         std::mem::swap(from, to);
      }
      to.reserve(from.len());
      for (k, v) in from.drain() {
         to.insert(k, v); // TODO could be improved
      }
      unsafe {
         MOVE_FULL_INDEX_CONTENTS_TOTAL_TIME += before.elapsed();
      }
   }
}

impl <K: Clone + Hash + Eq, V> RelFullIndexWrite for LatticeMap<K, V> {
   type Key = K;
   type Value = V;
   #[inline]
   fn insert_if_not_present(&mut self, key: &K, v: V) -> bool {
      match self.0.raw_entry_mut().from_key(key) {
         hashbrown::hash_map::RawEntryMut::Occupied(_) => false,
         hashbrown::hash_map::RawEntryMut::Vacant(vacant) => {vacant.insert(key.clone(), v); true},
      }
   }
}

impl<'a, K: Hash + Eq, V> RelFullIndexRead<'a> for LatticeMap<K, V> {
   type Key = K;

  fn contains_key(&self, key: &Self::Key) -> bool {
     self.0.contains_key(key)
  }
}

impl<K, V> Default for LatticeMap<K, V> {
   fn default() -> Self {
      LatticeMap(hashbrown::HashMap::default())
   }
}

/// type constraints for relation columns
pub struct TypeConstraints<T> where T : Clone + Eq + Hash{_t: ::core::marker::PhantomData<T>}
/// type constraints for a lattice
pub struct LatTypeConstraints<T> where T : Clone + Eq + Hash + Lattice{_t: ::core::marker::PhantomData<T>}

/// type constraints for parallel Ascent
pub struct ParTypeConstraints<T> where T: Send + Sync {_t: ::core::marker::PhantomData<T>}

#[inline(always)]
pub fn comment(_: &str){}
