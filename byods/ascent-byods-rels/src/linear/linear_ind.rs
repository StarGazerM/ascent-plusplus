// the full relation of linear relation

use std::{
   hash::{BuildHasherDefault, Hash},
   time::Instant,
};

use rustc_hash::FxHasher;
pub struct LinearRelIndexType<K, V>(hashbrown::HashMap<K, Vec<V>, BuildHasherDefault<FxHasher>>);
pub struct LinearRelIndexFullType<K, V>(hashbrown::HashMap<K, V, BuildHasherDefault<FxHasher>>);
pub struct LinearNoIndexType<V>(Vec<V>);

use ascent::internal::{
   AtomicCounter, CRelIndexRead, Freezable, RelFullIndexRead, RelFullIndexWrite, RelIndexMerge, RelIndexRead, RelIndexReadAll, RelIndexWrite, MOVE_FULL_INDEX_CONTENTS_TOTAL_TIME
};

use ascent::dashmap::RwLock;

use ascent::internal::{
   CRelFullIndex, CRelFullIndexWrite, CRelIndex, CRelIndexReadAll, CRelIndexReadAllParIter, CRelIndexWrite,
   CRelNoIndex, DashMapViewParIter,
};

impl<K, V> Default for LinearRelIndexType<K, V> {
   fn default() -> Self { Self(hashbrown::HashMap::default()) }
}

impl<K: Clone, V: Clone> Clone for LinearRelIndexType<K, V> {
   fn clone(&self) -> Self { Self(self.0.clone()) }
}

impl<K: Eq + Hash, V> RelIndexWrite for LinearRelIndexType<K, V> {
   type Key = K;
   type Value = V;

   fn index_insert(&mut self, key: K, value: V) {
      // let before = Instant::now();
      use hashbrown::hash_map::Entry::*;
      match self.0.entry(key) {
         Occupied(mut vec) => vec.get_mut().push(value),
         Vacant(vacant) => {
            let mut vec = Vec::with_capacity(4);
            vec.push(value);
            vacant.insert(vec);
         }
      }
      // unsafe {
      //    INDEX_INSERT_TOTAL_TIME += before.elapsed();
      // }
   }
}

impl<K: Eq + Hash, V> RelIndexMerge for LinearRelIndexType<K, V> {
   fn move_index_contents(from: &mut LinearRelIndexType<K, V>, to: &mut LinearRelIndexType<K, V>) {
      let before = Instant::now();
      std::mem::swap(from, to);
      from.0.clear();
      unsafe {
         MOVE_FULL_INDEX_CONTENTS_TOTAL_TIME += before.elapsed();
      }
   }
}

impl RelIndexWrite for LinearNoIndexType<usize> {
   type Key = ();
   type Value = usize;

   fn index_insert(&mut self, _: (), value: usize) { self.0.push(value); }
}

impl<K: Eq + Hash, V> RelIndexWrite for LinearRelIndexFullType<K, V> {
   type Key = K;
   type Value = V;

   #[inline(always)]
   fn index_insert(&mut self, key: K, value: V) { self.0.insert(key, value); }
}

impl<K: Eq + Hash, V> RelIndexMerge for LinearRelIndexFullType<K, V> {
   fn move_index_contents(from: &mut Self, to: &mut Self) {
      let before = Instant::now();
      std::mem::swap(from, to);
      from.0.clear();
      unsafe {
         MOVE_FULL_INDEX_CONTENTS_TOTAL_TIME += before.elapsed();
      }
   }
}

impl<K: Clone + Hash + Eq, V> RelFullIndexWrite for LinearRelIndexFullType<K, V> {
   type Key = K;
   type Value = V;

   #[inline(always)]
   fn insert_if_not_present(&mut self, key: &K, value: V) -> bool {
      match self.0.raw_entry_mut().from_key(key) {
         hashbrown::hash_map::RawEntryMut::Occupied(_) => false,
         hashbrown::hash_map::RawEntryMut::Vacant(vacant) => {
            vacant.insert(key.clone(), value);
            true
         }
      }
   }
}

impl<'a, K: Hash + Eq, V> RelFullIndexRead<'a> for LinearRelIndexFullType<K, V> {
   type Key = K;

   fn contains_key(&'a self, key: &Self::Key) -> bool { self.0.contains_key(key) }
}

impl<'a, K: Eq + std::hash::Hash + 'a, V: 'a + Clone> RelIndexReadAll<'a> for LinearRelIndexType<K, V> {
   type Key = &'a K;
   type Value = &'a V;
   type ValueIteratorType = std::slice::Iter<'a, V>;
   type AllIteratorType = std::iter::Map<
      hashbrown::hash_map::Iter<'a, K, Vec<V>>,
      for<'aa, 'bb> fn((&'aa K, &'bb Vec<V>)) -> (&'aa K, std::slice::Iter<'bb, V>),
   >;

   fn iter_all(&'a self) -> Self::AllIteratorType { self.0.iter().map(|(k, v)| (k, v.iter())) }
}

impl<'a, K: Eq + std::hash::Hash, V: 'a + Clone> RelIndexRead<'a> for LinearRelIndexType<K, V> {
    type Key = K;
    type Value = &'a V;

    type IteratorType = std::slice::Iter<'a, V>;

    #[inline]
    fn index_get(&'a self, key: &Self::Key) -> Option<Self::IteratorType> { self.0.get(key).map(|v| v.iter()) }

    #[inline(always)]
    fn len(&self) -> usize { self.0.len() }
}

impl<'a, K: Eq + std::hash::Hash, V: 'a + Clone> RelIndexRead<'a> for LinearRelIndexFullType<K, V> {
   type IteratorType = std::iter::Once<&'a V>;
   type Key = K;
   type Value = &'a V;

   #[inline]
   fn index_get(&'a self, key: &Self::Key) -> Option<Self::IteratorType> {
      let res = self.0.get(key)?;
      Some(std::iter::once(res))
   }

   #[inline(always)]
   fn len(&self) -> usize { self.0.len() }
}

impl<'a, K: Eq + std::hash::Hash + 'a, V: 'a + Clone> RelIndexReadAll<'a> for LinearRelIndexFullType<K, V> {
   type Key = &'a K;
   type Value = &'a V;
   type ValueIteratorType = std::iter::Once<&'a V>;
   type AllIteratorType = std::iter::Map<
      hashbrown::hash_map::Iter<'a, K, V>,
      for<'aa, 'bb> fn((&'aa K, &'bb V)) -> (&'aa K, std::iter::Once<&'bb V>),
   >;

   fn iter_all(&'a self) -> Self::AllIteratorType { self.0.iter().map(|(k, v)| (k, std::iter::once(v))) }
}

// par: TODO: use cfg_if
// wrapper for original par rel

pub struct CLinearRelIndex<K, V>(CRelIndex<K, V>);
pub struct CLinearRelIndexFull<K, V>(CRelFullIndex<K, V>);
pub struct CLinearNoIndex<V>(CRelNoIndex<V>);

impl<K: Clone + Hash + Eq, V> Freezable for CLinearRelIndex<K, V> {
   fn freeze(&mut self) { self.0.freeze(); }

   fn unfreeze(&mut self) { self.0.unfreeze(); }
}

impl<K: Clone + Hash + Eq, V> Default for CLinearRelIndex<K, V> {
   fn default() -> Self { Self(CRelIndex::default()) }
}

impl<'a, K: 'a + Clone + Hash + Eq, V: 'a> RelIndexRead<'a> for CLinearRelIndex<K, V> {
   type Key = K;
   type Value = &'a V;

   type IteratorType = std::slice::Iter<'a, V>;

   fn index_get(&'a self, key: &Self::Key) -> Option<Self::IteratorType> { self.0.index_get(key) }

   fn len(&self) -> usize { self.0.len() }
}

impl<'a, K: 'a + Clone + Hash + Eq, V: 'a + Sync> CRelIndexRead<'a> for CLinearRelIndex<K, V> {
   type Key = K;
   type Value = &'a V;

   type IteratorType = ::rayon::slice::Iter<'a, V>;

   fn c_index_get(&'a self, key: &Self::Key) -> Option<Self::IteratorType> { self.0.c_index_get(key) }
}

impl<'a, K: 'a + Clone + Hash + Eq + Send + Sync, V: 'a + Send + Sync> RelIndexWrite for CLinearRelIndex<K, V> {
   type Key = K;
   type Value = V;

   fn index_insert(&mut self, key: K, value: V) { self.0.index_insert(key, value) }
}

impl<'a, K: 'a + Clone + Hash + Eq + Send + Sync, V: 'a + Send + Sync> RelIndexMerge for CLinearRelIndex<K, V> {
   fn move_index_contents(from: &mut Self, to: &mut Self) {
      let before = Instant::now();

      std::mem::swap(&mut from.0, &mut to.0);
      from.0.clear();

      unsafe {
         MOVE_FULL_INDEX_CONTENTS_TOTAL_TIME += before.elapsed();
      }
   }
}

impl<'a, K: 'a + Clone + Hash + Eq, V: Clone + 'a> RelIndexReadAll<'a> for CLinearRelIndex<K, V> {
   type Key = &'a K;
   type Value = &'a V;
   type ValueIteratorType = std::slice::Iter<'a, V>;
   type AllIteratorType = Box<dyn Iterator<Item = (&'a K, Self::ValueIteratorType)> + 'a>;

   fn iter_all(&'a self) -> Self::AllIteratorType { self.0.iter_all() }
}

impl<'a, K: 'a + Clone + Hash + Eq + Sync + Send, V: Clone + 'a + Sync + Send> CRelIndexReadAll<'a>
   for CLinearRelIndex<K, V>
{
   type Key = &'a K;
   type Value = &'a V;
   type ValueIteratorType = ::rayon::slice::Iter<'a, V>;

   type AllIteratorType = CRelIndexReadAllParIter<'a, K, V, BuildHasherDefault<FxHasher>>;

   #[inline]
   fn c_iter_all(&'a self) -> Self::AllIteratorType { self.0.c_iter_all() }
}

impl<'a, K: 'a + Clone + Hash + Eq, V: 'a> CRelIndexWrite for CLinearRelIndex<K, V> {
   type Key = K;
   type Value = V;

   fn index_insert(&self, key: K, value: V) { self.0.index_insert(key, value) }
}

// c full index

impl<K: Clone + Hash + Eq, V> Freezable for CLinearRelIndexFull<K, V> {
   fn freeze(&mut self) { self.0.freeze(); }

   fn unfreeze(&mut self) { self.0.unfreeze(); }
}

impl<K: Clone + Hash + Eq, V> Default for CLinearRelIndexFull<K, V> {
   fn default() -> Self { Self(CRelFullIndex::default()) }
}

impl<'a, K: 'a + Clone + Hash + Eq, V: 'a> RelIndexRead<'a> for CLinearRelIndexFull<K, V> {
   type Key = K;
   type Value = &'a V;

   type IteratorType = std::iter::Once<&'a V>;

   fn index_get(&'a self, key: &Self::Key) -> Option<Self::IteratorType> { self.0.index_get(key) }

   fn len(&self) -> usize { self.0.len() }
}

impl<'a, K: 'a + Clone + Hash + Eq, V: 'a + Sync> CRelIndexRead<'a> for CLinearRelIndexFull<K, V> {
   type Key = K;
   type Value = &'a V;

   type IteratorType = ::rayon::iter::Once<&'a V>;

   fn c_index_get(&'a self, key: &Self::Key) -> Option<Self::IteratorType> { self.0.c_index_get(key) }
}

impl<'a, K: 'a + Clone + Hash + Eq, V: 'a + AtomicCounter> RelFullIndexRead<'a> for CLinearRelIndexFull<K, V> {
   type Key = K;

   #[inline(always)]
   fn contains_key(&self, key: &Self::Key) -> bool { self.0.contains_key(key) }
}

impl<'a, K: 'a + Clone + Hash + Eq, V: 'a> RelFullIndexWrite for CLinearRelIndexFull<K, V> {
   type Key = K;
   type Value = V;

   fn insert_if_not_present(&mut self, key: &Self::Key, value: V) -> bool { self.0.insert_if_not_present(key, value) }
}

impl<'a, K: 'a + Clone + Hash + Eq, V: 'a> CRelFullIndexWrite for CLinearRelIndexFull<K, V> {
   type Key = K;
   type Value = V;

   fn insert_if_not_present(&self, key: &Self::Key, value: V) -> bool { self.0.insert_if_not_present(key, value) }
}

impl<'a, K: 'a + Clone + Hash + Eq, V: 'a + Clone> RelIndexReadAll<'a> for CLinearRelIndexFull<K, V> {
   type Key = &'a K;
   type Value = V;
   type ValueIteratorType = std::iter::Once<V>;
   type AllIteratorType = Box<dyn Iterator<Item = (&'a K, Self::ValueIteratorType)> + 'a>;

   fn iter_all(&'a self) -> Self::AllIteratorType { self.0.iter_all() }
}

impl<'a, K: 'a + Clone + Hash + Eq + Send + Sync, V: 'a + Clone + Send + Sync> CRelIndexReadAll<'a>
   for CLinearRelIndexFull<K, V>
{
   type Key = &'a K;
   type Value = &'a V;

   type ValueIteratorType = ::rayon::iter::Once<&'a V>;
   // type AllIteratorType = Box<dyn Iterator<Item = (&'a K, Self::ValueIteratorType)> + 'a>;

   type AllIteratorType = ::rayon::iter::Map<
      DashMapViewParIter<'a, K, V, BuildHasherDefault<FxHasher>>,
      for<'aa, 'bb> fn((&'aa K, &'bb V)) -> (&'aa K, ::rayon::iter::Once<&'bb V>),
   >;

   #[inline]
   fn c_iter_all(&'a self) -> Self::AllIteratorType { self.0.c_iter_all() }
}

impl<'a, K: 'a + Clone + Hash + Eq + Send + Sync, V: 'a + Send + Sync> RelIndexWrite for CLinearRelIndexFull<K, V> {
   type Key = K;
   type Value = V;

   fn index_insert(&mut self, key: K, value: V) { self.0.index_insert(key, value) }
}

impl<'a, K: 'a + Clone + Hash + Eq + Send + Sync, V: 'a + Send + Sync> RelIndexMerge for CLinearRelIndexFull<K, V> {
   fn move_index_contents(from: &mut Self, to: &mut Self) {
      let before = Instant::now();

      std::mem::swap(&mut from.0, &mut to.0);
      from.0.clear();

      unsafe {
         MOVE_FULL_INDEX_CONTENTS_TOTAL_TIME += before.elapsed();
      }
   }
}

impl<'a, K: 'a + Clone + Hash + Eq, V: 'a> CRelIndexWrite for CLinearRelIndexFull<K, V> {
   type Key = K;
   type Value = V;

   fn index_insert(&self, key: K, value: V) { self.0.index_insert(key, value) }
}

// c no index
impl<V> Default for CLinearNoIndex<V> {
   fn default() -> Self { Self(CRelNoIndex::default()) }
}

impl<V> Freezable for CLinearNoIndex<V> {
   fn freeze(&mut self) { self.0.freeze(); }

   fn unfreeze(&mut self) { self.0.unfreeze(); }
}

impl<'a, V: 'a> RelIndexRead<'a> for CLinearNoIndex<V> {
   type Key = ();
   type Value = &'a V;

   type IteratorType = std::iter::FlatMap<
      std::slice::Iter<'a, RwLock<Vec<V>>>,
      std::slice::Iter<'a, V>,
      fn(&RwLock<Vec<V>>) -> std::slice::Iter<V>,
   >;

   fn index_get(&'a self, _: &Self::Key) -> Option<Self::IteratorType> { self.0.index_get(&()) }

   fn len(&self) -> usize { self.0.len() }
}

impl<'a, V: 'a + Sync + Send> CRelIndexRead<'a> for CLinearNoIndex<V> {
   type Key = ();
   type Value = &'a V;

   type IteratorType =
      ::rayon::iter::FlatMap<::rayon::slice::Iter<'a, RwLock<Vec<V>>>, fn(&RwLock<Vec<V>>) -> ::rayon::slice::Iter<V>>;

   fn c_index_get(&'a self, _: &Self::Key) -> Option<Self::IteratorType> { self.0.c_index_get(&()) }
}

impl<'a, V: 'a> RelIndexWrite for CLinearNoIndex<V> {
   type Key = ();
   type Value = V;

   fn index_insert(&mut self, _: Self::Key, value: Self::Value) { self.0.index_insert((), value) }
}

impl<'a, V: 'a> RelIndexMerge for CLinearNoIndex<V> {
   fn move_index_contents(from: &mut Self, to: &mut Self) {
      let before = Instant::now();

      std::mem::swap(&mut from.0, &mut to.0);
      from.0.clear();

      unsafe {
         MOVE_FULL_INDEX_CONTENTS_TOTAL_TIME += before.elapsed();
      }
   }
}

impl<'a, V: 'a> CRelIndexWrite for CLinearNoIndex<V> {
   type Key = ();
   type Value = V;

   fn index_insert(&self, _: Self::Key, value: Self::Value) { self.0.index_insert((), value) }
}

impl<'a, V: 'a> RelIndexReadAll<'a> for CLinearNoIndex<V> {
   type Key = &'a ();
   type Value = &'a V;

   type ValueIteratorType = <Self as RelIndexRead<'a>>::IteratorType;

   type AllIteratorType = std::iter::Once<(&'a (), Self::ValueIteratorType)>;

   fn iter_all(&'a self) -> Self::AllIteratorType { self.0.iter_all() }
}

impl<'a, V: 'a + Sync + Send> CRelIndexReadAll<'a> for CLinearNoIndex<V> {
   type Key = &'a ();
   type Value = &'a V;

   type ValueIteratorType = <Self as CRelIndexRead<'a>>::IteratorType;

   type AllIteratorType = ::rayon::iter::Once<(&'a (), Self::ValueIteratorType)>;

   fn c_iter_all(&'a self) -> Self::AllIteratorType { self.0.c_iter_all() }
}

impl <K,V> Default for LinearRelIndexFullType<K,V> {
   fn default() -> Self { Self(hashbrown::HashMap::default()) }
}
