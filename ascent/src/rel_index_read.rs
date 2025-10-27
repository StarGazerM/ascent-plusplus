// use core::slice::Iter;
use std::collections::HashSet;
// use std::hash::BuildHasherDefault;
// use std::iter::Chain;
use itertools::Either;

// use rustc_hash::FxHasher;

use crate::internal::*;

#[allow(clippy::len_without_is_empty)]
pub trait RelIndexRead<'a> {
   type Key;
   type Value;
   // type IteratorType: Iterator<Item = Self::Value> + Clone + 'a;
   // fn index_get(&'a self, key: &Self::Key) -> Option<Self::IteratorType>;
   fn index_get(&'a self, key: &Self::Key) -> Option<impl Iterator<Item = Self::Value> + Clone + 'a>;
   fn len_estimate(&'a self) -> usize;
   fn combine_delta(&'a self) -> bool { true }

   /// Is the relation **definitely** empty?
   ///
   /// It is OK for implementations to return `false` even if the relation may be empty,
   /// as this is used to enable certain optimizations.
   fn is_empty(&'a self) -> bool { false }
}

pub trait RelIndexReadAll<'a> {
   type Key: 'a;
   type Value;
   // type ValueIteratorType: Iterator<Item = Self::Value> + 'a;
   // type AllIteratorType: Iterator<Item = (Self::Key, Self::ValueIteratorType)> + 'a;
   // fn iter_all(&'a self) -> Self::AllIteratorType;
   fn iter_all(&'a self) -> impl Iterator<Item = (Self::Key, impl Iterator<Item = Self::Value> + 'a)> + 'a;
   fn combine_delta(&'a self) -> bool { true }
}

impl<'a, K: Eq + std::hash::Hash + 'a, V: Clone + 'a> RelIndexRead<'a> for RelIndexType1<K, V> {
   // type IteratorType = core::slice::Iter<'a, V>;
   type Key = K;
   type Value = &'a V;

   #[inline]
   fn index_get(&'a self, key: &K) -> Option<impl Iterator<Item = Self::Value> + Clone + 'a> {
      let v = self.get(key)?;
      Some(v.iter())
   }

   #[inline(always)]
   fn len_estimate(&self) -> usize { Self::len(self) }

   #[inline(always)]
   fn is_empty(&'a self) -> bool { Self::is_empty(self) }
}

impl<'a, K: Eq + std::hash::Hash + 'a, V: 'a + Clone> RelIndexReadAll<'a> for RelIndexType1<K, V> {
   type Key = &'a K;
   type Value = &'a V;
   // type ValueIteratorType = core::slice::Iter<'a, V>;

   // type AllIteratorType = std::iter::Map<
   //    std::collections::hash_map::Iter<'a, K, Vec<V>>,
   //    for<'aa, 'bb> fn((&'aa K, &'bb Vec<V>)) -> (&'aa K, Iter<'bb, V>),
   // >;

   fn iter_all(&'a self) -> impl Iterator<Item = (Self::Key, impl Iterator<Item = Self::Value> + 'a)> + 'a {
      self.iter().map(|(k, v)| (k, v.iter()))
   }
}

impl<'a, K: Eq + std::hash::Hash, V: 'a + Clone> RelIndexRead<'a> for HashBrownRelFullIndexType<K, V> {
   // type IteratorType = std::iter::Once<&'a V>;
   type Key = K;
   type Value = &'a V;

   #[inline]
   fn index_get(&'a self, key: &K) -> Option<impl Iterator<Item = Self::Value> + Clone + 'a> {
      let res = self.get(key)?;
      Some(std::iter::once(res))
   }

   #[inline(always)]
   fn len_estimate(&self) -> usize { Self::len(self) }

   #[inline(always)]
   fn is_empty(&'a self) -> bool { Self::is_empty(self) }
}

impl<'a, K: Eq + std::hash::Hash + 'a, V: 'a + Clone> RelIndexReadAll<'a> for HashBrownRelFullIndexType<K, V> {
   type Key = &'a K;
   type Value = &'a V;
   // type ValueIteratorType = std::iter::Once<&'a V>;

   // type AllIteratorType = std::iter::Map<
   //    hashbrown::hash_map::Iter<'a, K, V>,
   //    for<'aa, 'bb> fn((&'aa K, &'bb V)) -> (&'aa K, std::iter::Once<&'bb V>),
   // >;

   fn iter_all(&'a self) -> impl Iterator<Item = (Self::Key, impl Iterator<Item = Self::Value> + 'a)> + 'a {
      self.iter().map(|(k, v)| (k, std::iter::once(v)))
   }
}

impl<'a, K: Eq + std::hash::Hash, V: 'a + Clone> RelIndexRead<'a> for LatticeIndexType<K, V> {
   // type IteratorType = std::collections::hash_set::Iter<'a, V>;
   type Key = K;
   type Value = &'a V;

   #[inline]
   fn index_get(&'a self, key: &K) -> Option<impl Iterator<Item = Self::Value> + Clone + 'a> {
      let res: Option<std::collections::hash_set::Iter<'a, V>> = self.get(key).map(HashSet::iter);
      res
   }

   #[inline(always)]
   fn len_estimate(&self) -> usize { Self::len(self) }
   #[inline(always)]
   fn is_empty(&'a self) -> bool { Self::is_empty(self) }
}

impl<'a, K: Eq + std::hash::Hash + 'a, V: 'a + Clone> RelIndexReadAll<'a> for LatticeIndexType<K, V> {
   type Key = &'a K;
   type Value = &'a V;
   // type ValueIteratorType = std::collections::hash_set::Iter<'a, V>;

   // type AllIteratorType = std::iter::Map<
   //    std::collections::hash_map::Iter<'a, K, HashSet<V, BuildHasherDefault<FxHasher>>>,
   //    for<'aa, 'bb> fn(
   //       (&'aa K, &'bb std::collections::HashSet<V, BuildHasherDefault<FxHasher>>),
   //    ) -> (&'aa K, std::collections::hash_set::Iter<'bb, V>),
   // >;

   #[inline]
   fn iter_all(&'a self) -> impl Iterator<Item = (Self::Key, impl Iterator<Item = Self::Value> + 'a)> + 'a {
      self.iter().map(|(k, v)| (k, v.iter()))
   }
}

pub struct RelIndexCombined<'a, Ind1, Ind2> {
   pub ind1: &'a Ind1,
   pub ind2: &'a Ind2,
}

impl<'a, Ind1, Ind2> RelIndexCombined<'a, Ind1, Ind2> {
   #[inline]
   pub fn new(ind1: &'a Ind1, ind2: &'a Ind2) -> Self { Self { ind1, ind2 } }
}

impl<'a, Ind1, Ind2, K, V> RelIndexRead<'a> for RelIndexCombined<'a, Ind1, Ind2>
where
   Ind1: RelIndexRead<'a, Key = K, Value = V>,
   Ind2: RelIndexRead<'a, Key = K, Value = V>,
{
   type Key = K;
   type Value = V;

   // type IteratorType = Either<
   //    Chain<
   //       std::iter::Flatten<std::option::IntoIter<Ind1::IteratorType>>,
   //       std::iter::Flatten<std::option::IntoIter<Ind2::IteratorType>>,
   //    >,
   //    Ind1::IteratorType,
   // >;

   fn index_get(&'a self, key: &Self::Key) -> Option<impl Iterator<Item = Self::Value> + Clone + 'a> {
      let iter1_opt = self.ind1.index_get(key);
      let iter2_opt = self.ind2.index_get(key);
      if iter1_opt.is_none() && iter2_opt.is_none() {
         None
      } else {
         if self.ind1.combine_delta() && iter1_opt.is_none() {   
            let combined = iter1_opt.into_iter().flatten()
                                 .chain(iter2_opt.into_iter().flatten());
            Some(Either::Left(combined))
         } else {
            self.ind1.index_get(key).map(Either::Right)
         }
      }
   }

   #[inline(always)]
   fn len_estimate(&self) -> usize { self.ind1.len_estimate() + self.ind2.len_estimate() }

   #[inline]
   fn is_empty(&'a self) -> bool { self.ind1.is_empty() && self.ind2.is_empty() }
}

// impl <'a, Ind> RelIndexRead<'a> for RelIndexCombined<'a, Ind, Ind>
// where Ind: RelIndexTrait2<'a> {
//     type Key = Ind::Key;

//     type IteratorType = EitherIter<Ind::IteratorType, Chain<Ind::IteratorType, Ind::IteratorType>>;

//    fn index_get(&'a self, key: &Self::Key) -> Option<Self::IteratorType> {
//       let res: Option<EitherIter<<Ind as RelIndexTrait2>::IteratorType, Chain<<Ind as RelIndexTrait2>::IteratorType, <Ind as RelIndexTrait2>::IteratorType>>> =
//       match (self.ind1.index_get(key), self.ind2.index_get(key)) {
//          (None, None) => None,
//          (Some(it), None) | (None, Some(it)) => Some(EitherIter::Left(it)),
//          (Some(it1), Some(it2)) => Some(EitherIter::Right(it1.chain(it2)))
//       };
//       res
//    }

//    #[inline(always)]
//    fn len(&self) -> usize { self.ind1.len() + self.ind2.len() }
// }

// #[derive(Clone)]
// pub enum EitherIter<L, R> {
//    Left(L),
//    Right(R)
// }

// impl <L: Iterator<Item = T>, R: Iterator<Item = T>, T> Iterator for EitherIter<L, R> {
//    type Item = T;

//    fn next(&mut self) -> Option<Self::Item> {
//       match self {
//          EitherIter::Left(l) => l.next(),
//          EitherIter::Right(r) => r.next(),
//       }
//    }
// }



impl<'a, Ind1, Ind2, K: 'a, V: 'a> RelIndexReadAll<'a>
   for RelIndexCombined<'a, Ind1, Ind2>
where
   Ind1: RelIndexReadAll<'a, Key = K, Value = V>,
   Ind2: RelIndexReadAll<'a, Key = K, Value = V>,
{
   type Key = K;
   type Value = V;

   // type ValueIteratorType = VTI;

   // type AllIteratorType = Either<Peekable<Ind1::AllIteratorType>, Ind2::AllIteratorType>;

   fn iter_all(&'a self) -> impl Iterator<Item = (Self::Key, impl Iterator<Item = Self::Value> + 'a)> + 'a {
      // 1. Create the first iterator and make it peekable.
      let mut iter1 = self.ind1.iter_all().map(|(k, v_iter)| {
         (k, CombinedIter::First(v_iter))
     }).peekable();

      // 2. Check if the first iterator has any items without consuming them.
      let ret = if iter1.peek().is_none() {
         // If it's empty, return the second iterator.
         Either::Right(self.ind2.iter_all().map(|(k, v_iter)| {
            (k, CombinedIter::Second(v_iter))
         }))
      } else {
         // If it's not empty, return the first iterator we already created.
         Either::Left(iter1)
      };

      ret
   }
}
