// the full relation of Phantom relation

use std::{
   hash::{BuildHasherDefault, Hash},
   time::Instant,
};

use rustc_hash::FxHasher;
pub struct PhantomRelIndexType<K, V>(hashbrown::HashMap<K, Vec<V>, BuildHasherDefault<FxHasher>>);
pub struct PhantomRelIndexFullType<K, V>(hashbrown::HashMap<K, V, BuildHasherDefault<FxHasher>>);
pub struct PhantomNoIndexType<V>(Vec<V>);
use ascent::rayon;

use ascent::internal::{
   CRelIndexRead, Freezable, RelFullIndexRead, RelFullIndexWrite, RelIndexMerge, RelIndexRead, RelIndexReadAll,
   RelIndexWrite, MOVE_FULL_INDEX_CONTENTS_TOTAL_TIME,
};

use ascent::dashmap::RwLock;

use ascent::internal::{
   CRelFullIndex, CRelFullIndexWrite, CRelIndex, CRelIndexReadAll, CRelIndexReadAllParIter, CRelIndexWrite,
   CRelNoIndex, DashMapViewParIter,
};

impl<K, V> Default for PhantomRelIndexType<K, V> {
   fn default() -> Self { Self(hashbrown::HashMap::default()) }
}

impl<K: Clone, V: Clone> Clone for PhantomRelIndexType<K, V> {
   fn clone(&self) -> Self { Self(self.0.clone()) }
}

impl<K: Eq + Hash, V> RelIndexWrite for PhantomRelIndexType<K, V> {
   type Key = K;
   type Value = V;

   fn index_insert(&mut self, key: K, value: V) {}
}

impl<K: Eq + Hash, V> PhantomRelIndexType<K, V> {
   pub fn insert(&mut self, key: K, value: V) { self.0.entry(key).or_insert_with(Vec::new).push(value); }
}

impl<K: Eq + Hash, V> FromIterator<(K, V)> for PhantomRelIndexType<K, V> {
   fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
      let mut vec = Self::default();
      for (k, v) in iter {
         vec.insert(k, v);
      }
      vec
   }
}

impl<K: Eq + Hash, V> RelIndexMerge for PhantomRelIndexType<K, V> {
   fn move_index_contents(from: &mut PhantomRelIndexType<K, V>, to: &mut PhantomRelIndexType<K, V>) {
      let before = Instant::now();
      //    std::mem::swap(from, to);
      from.0.clear();
      unsafe {
         MOVE_FULL_INDEX_CONTENTS_TOTAL_TIME += before.elapsed();
      }
   }
}

impl RelIndexWrite for PhantomNoIndexType<usize> {
   type Key = ();
   type Value = usize;

   fn index_insert(&mut self, _: (), value: usize) { self.0.push(value); }
}

impl<K: Eq + Hash, V> RelIndexWrite for PhantomRelIndexFullType<K, V> {
   type Key = K;
   type Value = V;

   #[inline(always)]
   fn index_insert(&mut self, key: K, value: V) { }
}

impl<K: Eq + Hash, V> RelIndexMerge for PhantomRelIndexFullType<K, V> {
   fn move_index_contents(from: &mut Self, to: &mut Self) {
      let before = Instant::now();
      //    std::mem::swap(from, to);
      from.0.clear();
      unsafe {
         MOVE_FULL_INDEX_CONTENTS_TOTAL_TIME += before.elapsed();
      }
   }
}

impl<K: Clone + Hash + Eq, V> RelFullIndexWrite for PhantomRelIndexFullType<K, V> {
   type Key = K;
   type Value = V;

   #[inline(always)]
   fn insert_if_not_present(&mut self, key: &K, value: V) -> bool { true }
}

impl<'a, K: Hash + Eq, V> RelFullIndexRead<'a> for PhantomRelIndexFullType<K, V> {
   type Key = K;

   fn contains_key(&'a self, key: &Self::Key) -> bool { false }
}

impl<'a, K: Eq + std::hash::Hash + 'a, V: 'a + Clone> RelIndexReadAll<'a> for PhantomRelIndexType<K, V> {
   type Key = &'a K;
   type Value = &'a V;
   type ValueIteratorType = std::slice::Iter<'a, V>;
   type AllIteratorType = std::iter::Map<
      hashbrown::hash_map::Iter<'a, K, Vec<V>>,
      for<'aa, 'bb> fn((&'aa K, &'bb Vec<V>)) -> (&'aa K, std::slice::Iter<'bb, V>),
   >;

   fn iter_all(&'a self) -> Self::AllIteratorType { self.0.iter().map(|(k, v)| (k, v.iter())) }
}

impl<'a, K: Eq + std::hash::Hash, V: 'a + Clone> RelIndexRead<'a> for PhantomRelIndexType<K, V> {
   type Key = K;
   type Value = &'a V;

   type IteratorType = std::slice::Iter<'a, V>;

   #[inline]
   fn index_get(&'a self, key: &Self::Key) -> Option<Self::IteratorType> { self.0.get(key).map(|v| v.iter()) }

   #[inline(always)]
   fn len(&self) -> usize { self.0.len() }
}

impl<'a, K: Eq + std::hash::Hash, V: 'a + Clone> RelIndexRead<'a> for PhantomRelIndexFullType<K, V> {
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

impl<'a, K: Eq + std::hash::Hash + 'a, V: 'a + Clone> RelIndexReadAll<'a> for PhantomRelIndexFullType<K, V> {
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

pub struct CPhantomRelIndex<K, V>(CRelIndex<K, V>);
pub struct CPhantomRelIndexFull<K, V>(CRelFullIndex<K, V>);
pub struct CPhantomNoIndex<V>(CRelNoIndex<V>);

impl<K: Clone + Hash + Eq, V> Freezable for CPhantomRelIndex<K, V> {
   fn freeze(&mut self) { self.0.freeze(); }

   fn unfreeze(&mut self) { self.0.unfreeze(); }
}

impl<K: Clone + Hash + Eq, V> Default for CPhantomRelIndex<K, V> {
   fn default() -> Self { Self(CRelIndex::default()) }
}

impl<'a, K: 'a + Clone + Hash + Eq, V: 'a> RelIndexRead<'a> for CPhantomRelIndex<K, V> {
   type Key = K;
   type Value = &'a V;

   type IteratorType = std::slice::Iter<'a, V>;

   fn index_get(&'a self, key: &Self::Key) -> Option<Self::IteratorType> { self.0.index_get(key) }

   fn len(&self) -> usize { self.0.len() }
}

impl<'a, K: 'a + Clone + Hash + Eq, V: 'a + Sync> CRelIndexRead<'a> for CPhantomRelIndex<K, V> {
   type Key = K;
   type Value = &'a V;

   type IteratorType = rayon::slice::Iter<'a, V>;

   fn c_index_get(&'a self, key: &Self::Key) -> Option<Self::IteratorType> { self.0.c_index_get(key) }
}

impl<'a, K: 'a + Clone + Hash + Eq + Send + Sync, V: 'a + Send + Sync> RelIndexWrite for CPhantomRelIndex<K, V> {
   type Key = K;
   type Value = V;

   fn index_insert(&mut self, key: K, value: V) {}
}

impl<'a, K: 'a + Clone + Hash + Eq + Send + Sync, V: 'a + Send + Sync> RelIndexMerge for CPhantomRelIndex<K, V> {
   fn move_index_contents(from: &mut Self, to: &mut Self) {
      let before = Instant::now();

      from.0.clear();

      unsafe {
         MOVE_FULL_INDEX_CONTENTS_TOTAL_TIME += before.elapsed();
      }
   }
}

impl<'a, K: 'a + Clone + Hash + Eq, V: Clone + 'a> RelIndexReadAll<'a> for CPhantomRelIndex<K, V> {
   type Key = &'a K;
   type Value = &'a V;
   type ValueIteratorType = std::slice::Iter<'a, V>;
   type AllIteratorType = Box<dyn Iterator<Item = (&'a K, Self::ValueIteratorType)> + 'a>;

   fn iter_all(&'a self) -> Self::AllIteratorType { self.0.iter_all() }
}

impl<'a, K: 'a + Clone + Hash + Eq + Sync + Send, V: Clone + 'a + Sync + Send> CRelIndexReadAll<'a>
   for CPhantomRelIndex<K, V>
{
   type Key = &'a K;
   type Value = &'a V;
   type ValueIteratorType = rayon::slice::Iter<'a, V>;

   type AllIteratorType = CRelIndexReadAllParIter<'a, K, V, BuildHasherDefault<FxHasher>>;

   #[inline]
   fn c_iter_all(&'a self) -> Self::AllIteratorType { self.0.c_iter_all() }
}

impl<'a, K: 'a + Clone + Hash + Eq, V: 'a> CRelIndexWrite for CPhantomRelIndex<K, V> {
   type Key = K;
   type Value = V;

   fn index_insert(&self, key: K, value: V) {}
}

// c full index

impl<K: Clone + Hash + Eq, V> Freezable for CPhantomRelIndexFull<K, V> {
   fn freeze(&mut self) { self.0.freeze(); }

   fn unfreeze(&mut self) { self.0.unfreeze(); }
}

impl<K: Clone + Hash + Eq, V> Default for CPhantomRelIndexFull<K, V> {
   fn default() -> Self { Self(CRelFullIndex::default()) }
}

impl<'a, K: 'a + Clone + Hash + Eq, V: 'a> RelIndexRead<'a> for CPhantomRelIndexFull<K, V> {
   type Key = K;
   type Value = &'a V;

   type IteratorType = std::iter::Once<&'a V>;

   fn index_get(&'a self, key: &Self::Key) -> Option<Self::IteratorType> { self.0.index_get(key) }

   fn len(&self) -> usize { self.0.len() }
}

impl<'a, K: 'a + Clone + Hash + Eq, V: 'a + Sync> CRelIndexRead<'a> for CPhantomRelIndexFull<K, V> {
   type Key = K;
   type Value = &'a V;

   type IteratorType = rayon::iter::Once<&'a V>;

   fn c_index_get(&'a self, key: &Self::Key) -> Option<Self::IteratorType> { self.0.c_index_get(key) }
}

impl<'a, K: 'a + Clone + Hash + Eq, V: 'a> RelFullIndexRead<'a> for CPhantomRelIndexFull<K, V> {
   type Key = K;

   #[inline(always)]
   fn contains_key(&self, key: &Self::Key) -> bool { false }
}

impl<'a, K: 'a + Clone + Hash + Eq, V: 'a> RelFullIndexWrite for CPhantomRelIndexFull<K, V> {
   type Key = K;
   type Value = V;

   fn insert_if_not_present(&mut self, key: &Self::Key, value: V) -> bool { true }
}

impl<'a, K: 'a + Clone + Hash + Eq, V: 'a> CRelFullIndexWrite for CPhantomRelIndexFull<K, V> {
   type Key = K;
   type Value = V;

   fn insert_if_not_present(&self, key: &Self::Key, value: V) -> bool { true }
}

impl<'a, K: 'a + Clone + Hash + Eq, V: 'a + Clone> RelIndexReadAll<'a> for CPhantomRelIndexFull<K, V> {
   type Key = &'a K;
   type Value = V;
   type ValueIteratorType = std::iter::Once<V>;
   type AllIteratorType = Box<dyn Iterator<Item = (&'a K, Self::ValueIteratorType)> + 'a>;

   fn iter_all(&'a self) -> Self::AllIteratorType { self.0.iter_all() }
}

impl<'a, K: 'a + Clone + Hash + Eq + Send + Sync, V: 'a + Clone + Send + Sync> CRelIndexReadAll<'a>
   for CPhantomRelIndexFull<K, V>
{
   type Key = &'a K;
   type Value = &'a V;

   type ValueIteratorType = rayon::iter::Once<&'a V>;
   // type AllIteratorType = Box<dyn Iterator<Item = (&'a K, Self::ValueIteratorType)> + 'a>;

   type AllIteratorType = rayon::iter::Map<
      DashMapViewParIter<'a, K, V, BuildHasherDefault<FxHasher>>,
      for<'aa, 'bb> fn((&'aa K, &'bb V)) -> (&'aa K, rayon::iter::Once<&'bb V>),
   >;

   #[inline]
   fn c_iter_all(&'a self) -> Self::AllIteratorType { self.0.c_iter_all() }
}

impl<'a, K: 'a + Clone + Hash + Eq + Send + Sync, V: 'a + Send + Sync> RelIndexWrite for CPhantomRelIndexFull<K, V> {
   type Key = K;
   type Value = V;

   fn index_insert(&mut self, key: K, value: V) {}
}

impl<'a, K: 'a + Clone + Hash + Eq + Send + Sync, V: 'a + Send + Sync> RelIndexMerge for CPhantomRelIndexFull<K, V> {
   fn move_index_contents(from: &mut Self, to: &mut Self) {
      let before = Instant::now();

      from.0.clear();

      unsafe {
         MOVE_FULL_INDEX_CONTENTS_TOTAL_TIME += before.elapsed();
      }
   }
}

impl<'a, K: 'a + Clone + Hash + Eq, V: 'a> CRelIndexWrite for CPhantomRelIndexFull<K, V> {
   type Key = K;
   type Value = V;

   fn index_insert(&self, key: K, value: V) {}
}

// c no index
impl<V> Default for CPhantomNoIndex<V> {
   fn default() -> Self { Self(CRelNoIndex::default()) }
}

impl<V> Freezable for CPhantomNoIndex<V> {
   fn freeze(&mut self) { self.0.freeze(); }

   fn unfreeze(&mut self) { self.0.unfreeze(); }
}

impl<'a, V: 'a> RelIndexRead<'a> for CPhantomNoIndex<V> {
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

impl<'a, V: 'a + Sync + Send> CRelIndexRead<'a> for CPhantomNoIndex<V> {
   type Key = ();
   type Value = &'a V;

   type IteratorType =
      rayon::iter::FlatMap<rayon::slice::Iter<'a, RwLock<Vec<V>>>, fn(&RwLock<Vec<V>>) -> rayon::slice::Iter<V>>;

   fn c_index_get(&'a self, _: &Self::Key) -> Option<Self::IteratorType> { self.0.c_index_get(&()) }
}

impl<'a, V: 'a> RelIndexWrite for CPhantomNoIndex<V> {
   type Key = ();
   type Value = V;

   fn index_insert(&mut self, _: Self::Key, value: Self::Value) {}
}

impl<'a, V: 'a> RelIndexMerge for CPhantomNoIndex<V> {
   fn move_index_contents(from: &mut Self, to: &mut Self) {
      let before = Instant::now();

      from.0.clear();

      unsafe {
         MOVE_FULL_INDEX_CONTENTS_TOTAL_TIME += before.elapsed();
      }
   }
}

impl<'a, V: 'a> CRelIndexWrite for CPhantomNoIndex<V> {
   type Key = ();
   type Value = V;

   fn index_insert(&self, _: Self::Key, value: Self::Value) {}
}

impl<'a, V: 'a> RelIndexReadAll<'a> for CPhantomNoIndex<V> {
   type Key = &'a ();
   type Value = &'a V;

   type ValueIteratorType = <Self as RelIndexRead<'a>>::IteratorType;

   type AllIteratorType = std::iter::Once<(&'a (), Self::ValueIteratorType)>;

   fn iter_all(&'a self) -> Self::AllIteratorType { self.0.iter_all() }
}

impl<'a, V: 'a + Sync + Send> CRelIndexReadAll<'a> for CPhantomNoIndex<V> {
   type Key = &'a ();
   type Value = &'a V;

   type ValueIteratorType = <Self as CRelIndexRead<'a>>::IteratorType;

   type AllIteratorType = rayon::iter::Once<(&'a (), Self::ValueIteratorType)>;

   fn c_iter_all(&'a self) -> Self::AllIteratorType { self.0.c_iter_all() }
}

impl<K, V> Default for PhantomRelIndexFullType<K, V> {
   fn default() -> Self { Self(hashbrown::HashMap::default()) }
}
