use std::hash::Hash;
use std::marker::PhantomData;

use ascent::internal::{
   RelFullIndexRead, RelFullIndexWrite, RelIndexMerge, RelIndexRead, RelIndexReadAll, RelIndexWrite, ToRelIndex,
};

use crate::ceqrel_ind::ref_to_singleton_tuple_ref;
use crate::eqrel_ind::EqRelIndCommon;
use crate::iterator_from_dyn::IteratorFromDyn;

pub struct ToEqRelCanonicalInd0_1_2<T>(PhantomData<T>);

impl<T> Default for ToEqRelCanonicalInd0_1_2<T> {
   fn default() -> Self {
      Self(Default::default())
   }
}

pub struct EqRelCanonicalInd0_1_2<'a, T: Clone + Hash + Eq>(pub(crate) &'a EqRelIndCommon<T>);

pub struct EqRelCanonicalInd0_1_2Write<'a, T: Clone + Hash + Eq>(&'a mut EqRelIndCommon<T>);
impl<T: Clone + Hash + Eq> RelIndexWrite for EqRelCanonicalInd0_1_2Write<'_, T> {
   type Key = (T, T, T);
   type Value = ();

   fn index_insert(&mut self, key: Self::Key, value: Self::Value) {
      self.0.index_insert((key.0, key.1), value)
   }
}

impl<T: Clone + Hash + Eq> RelIndexMerge for EqRelCanonicalInd0_1_2<'_, T> {
   fn move_index_contents(_from: &mut Self, _to: &mut Self) {
      //noop
   }
}

impl<T: Clone + Hash + Eq> RelIndexMerge for EqRelCanonicalInd0_1_2Write<'_, T> {
   fn move_index_contents(_from: &mut Self, _to: &mut Self) {
      //noop
   }
}

impl<T: Clone + Hash + Eq> RelFullIndexWrite for EqRelCanonicalInd0_1_2Write<'_, T> {
   type Key = (T, T, T);
   type Value = ();
   fn insert_if_not_present(&mut self, key: &Self::Key, v: Self::Value) -> bool {
      self.0.insert_if_not_present(&(key.0.clone(), key.1.clone()), v)
   }
}

impl<'a, T: Clone + Hash + Eq> RelIndexRead<'a> for EqRelCanonicalInd0_1_2<'a, T> {
   type Key = (T, T, T);
   type Value = ();
   // type IteratorType = std::iter::Once<()>;
   fn index_get(&'a self, key: &Self::Key) -> Option<impl Iterator<Item = Self::Value> + Clone + 'a> {
      self.0.index_get(&(key.0.clone(), key.1.clone())).map(|iter| IteratorFromDyn::new(move || iter.clone()))
   }

   fn len_estimate(&self) -> usize {
      self.0.len_estimate()
   }
}

impl<'a, T: Clone + Hash + Eq> RelIndexReadAll<'a> for EqRelCanonicalInd0_1_2<'a, T> {
   type Key = (&'a T, &'a T, &'a T);
   type Value = ();
   // type ValueIteratorType = std::iter::Once<()>;
   // type AllIteratorType = Box<dyn Iterator<Item = (Self::Key, Self::ValueIteratorType)> + 'a>;
   fn iter_all(&'a self) -> impl Iterator<Item = (Self::Key, impl Iterator<Item = Self::Value> + 'a)> + 'a {
      self.0.iter_all().map(|(key, val)| {
         let (x, y) = key;
         let x_canonical = self.0.combined.get_dominant_elem(x).unwrap();
         ((x, y, x_canonical), val)
      })
   }
}

impl<'a, T: Clone + Hash + Eq> RelFullIndexRead<'a> for EqRelCanonicalInd0_1_2<'a, T> {
   type Key = (T, T, T);
   fn contains_key(&'a self, key: &Self::Key) -> bool {
      self.0.contains_key(&(key.0.clone(), key.1.clone()))
   }
}

impl<T: Clone + Hash + Eq> ToRelIndex<EqRelIndCommon<T>> for ToEqRelCanonicalInd0_1_2<T> {
   type RelIndex<'a>
      = EqRelCanonicalInd0_1_2<'a, T>
   where
      T: 'a;
   fn to_rel_index<'a>(&'a self, rel: &'a EqRelIndCommon<T>) -> Self::RelIndex<'a> {
      EqRelCanonicalInd0_1_2(rel)
   }

   type RelIndexWrite<'a>
      = EqRelCanonicalInd0_1_2Write<'a, T>
   where
      T: 'a;
   fn to_rel_index_write<'a>(&'a mut self, rel: &'a mut EqRelIndCommon<T>) -> Self::RelIndexWrite<'a> {
      EqRelCanonicalInd0_1_2Write(rel)
   }
}

pub struct EqRelCanonicalInd0<'a, T: Clone + Hash + Eq>(pub(crate) &'a EqRelIndCommon<T>);

pub struct ToEqRelCanonicalInd0<T>(PhantomData<T>);

impl<T> Default for ToEqRelCanonicalInd0<T> {
   fn default() -> Self {
      Self(Default::default())
   }
}

impl<T: Clone + Hash + Eq> ToRelIndex<EqRelIndCommon<T>> for ToEqRelCanonicalInd0<T> {
   type RelIndex<'a>
      = EqRelCanonicalInd0<'a, T>
   where
      T: 'a;
   fn to_rel_index<'a>(&'a self, rel: &'a EqRelIndCommon<T>) -> Self::RelIndex<'a> {
      EqRelCanonicalInd0(rel)
   }

   type RelIndexWrite<'a>
      = EqRelCanonicalInd0<'a, T>
   where
      T: 'a;
   fn to_rel_index_write<'a>(&'a mut self, rel: &'a mut EqRelIndCommon<T>) -> Self::RelIndexWrite<'a> {
      EqRelCanonicalInd0(rel)
   }
}

impl<'a, T: Clone + Hash + Eq> RelIndexRead<'a> for EqRelCanonicalInd0<'a, T> {
   type Key = (T,);
   type Value = (&'a T, &'a T);

   // type IteratorType = IteratorFromDyn<'a, (&'a T, &'a T)>;

   fn index_get(&'a self, key: &Self::Key) -> Option<impl Iterator<Item = Self::Value> + Clone + 'a> {
      let _ = self.0.set_of_added(&key.0)?;
      let key = key.clone();
      let producer =
         move || self.0.set_of_added(&key.0).unwrap().map(|x| (x, self.0.combined.get_dominant_elem(x).unwrap()));
      Some(IteratorFromDyn::new(producer))
   }

   fn len_estimate(&self) -> usize {
      self.0.combined.elem_ids.len()
   }
}

impl<'a, T: Clone + Hash + Eq> RelIndexReadAll<'a> for EqRelCanonicalInd0<'a, T> {
   type Key = &'a (T,);
   type Value = (&'a T, &'a T);

   // WARNING: this not zero cost
   // type ValueIteratorType = Box<dyn Iterator<Item = Self::Value> + 'a>;
   // type AllIteratorType = Box<dyn Iterator<Item = (Self::Key, Self::ValueIteratorType)> + 'a>;

   fn iter_all(&'a self) -> impl Iterator<Item = (Self::Key, impl Iterator<Item = Self::Value> + 'a)> + 'a {
      self.0.combined.sets.iter().flat_map(move |s| {
         s.iter().map(move |x| {
            (ref_to_singleton_tuple_ref(x), s.iter().map(move |y| (y, self.0.combined.get_dominant_elem(y).unwrap())))
         })
      })
   }
}

impl<T: Clone + Hash + Eq> RelIndexWrite for EqRelCanonicalInd0<'_, T> {
   type Key = (T,);
   type Value = (T, T);
   fn index_insert(&mut self, _key: Self::Key, _value: Self::Value) {
      // noop
   }
}

impl<T: Clone + Hash + Eq> RelIndexMerge for EqRelCanonicalInd0<'_, T> {
   fn move_index_contents(_from: &mut Self, _to: &mut Self) {
      //noop
   }
}

pub struct EqRelCanonicalInd2<'a, T: Clone + Hash + Eq>(&'a EqRelIndCommon<T>);

#[derive(Default)]
pub struct ToEqRelCanonicalInd2<T>(PhantomData<T>);

impl<T: Clone + Hash + Eq> ToRelIndex<EqRelIndCommon<T>> for ToEqRelCanonicalInd2<T> {
   type RelIndex<'a>
      = EqRelCanonicalInd2<'a, T>
   where
      T: 'a;
   fn to_rel_index<'a>(&'a self, rel: &'a EqRelIndCommon<T>) -> Self::RelIndex<'a> {
      EqRelCanonicalInd2(rel)
   }

   type RelIndexWrite<'a>
      = EqRelCanonicalInd2<'a, T>
   where
      T: 'a;
   fn to_rel_index_write<'a>(&'a mut self, rel: &'a mut EqRelIndCommon<T>) -> Self::RelIndexWrite<'a> {
      EqRelCanonicalInd2(rel)
   }
}

impl<'a, T: Clone + Hash + Eq> RelIndexRead<'a> for EqRelCanonicalInd2<'a, T> {
   type Key = (T,);
   type Value = (&'a T, &'a T);

   // type IteratorType = std::iter::Once<(&'a T, &'a T)>;

   fn index_get(&'a self, key: &Self::Key) -> Option<impl Iterator<Item = Self::Value> + Clone + 'a> {
      self.0.combined.get_dominant_elem(&key.0).and_then(|canonical_key| {
         if &key.0 == canonical_key { Some(std::iter::once((canonical_key, canonical_key))) } else { None }
      })
   }

   fn len_estimate(&self) -> usize {
      1
   }
}

impl<'a, T: Clone + Hash + Eq> RelIndexReadAll<'a> for EqRelCanonicalInd2<'a, T> {
   type Key = (&'a T,);
   type Value = (&'a T, &'a T);

   // type ValueIteratorType = Box<dyn Iterator<Item = Self::Value> + 'a>;
   // type AllIteratorType = Box<dyn Iterator<Item = (Self::Key, Self::ValueIteratorType)> + 'a>;

   fn iter_all(&'a self) -> impl Iterator<Item = (Self::Key, impl Iterator<Item = Self::Value> + 'a)> + 'a {
      self.0.combined.dominant_elements().map(|canonical_elem| {
         let key = (canonical_elem,);
         (key, std::iter::once((canonical_elem, canonical_elem)))
      })
   }
}

impl<T: Clone + Hash + Eq> RelIndexWrite for EqRelCanonicalInd2<'_, T> {
   type Key = (T,);
   type Value = (T, T);
   fn index_insert(&mut self, _key: Self::Key, _value: Self::Value) {
      // noop
   }
}

impl<T: Clone + Hash + Eq> RelIndexMerge for EqRelCanonicalInd2<'_, T> {
   fn move_index_contents(_from: &mut Self, _to: &mut Self) {
      //noop
   }
}

pub struct EqRelCanonicalIndNone<'a, T: Clone + Hash + Eq>(&'a EqRelIndCommon<T>);

impl<'a, T: Clone + Hash + Eq> RelIndexRead<'a> for EqRelCanonicalIndNone<'a, T> {
   type Key = ();

   type Value = (&'a T, &'a T, &'a T);

   // type IteratorType = IteratorFromDyn<'a, (&'a T, &'a T, &'a T)>;

   fn index_get(&'a self, _key: &Self::Key) -> Option<impl Iterator<Item = Self::Value> + Clone + 'a> {
      Some(IteratorFromDyn::new(|| {
         // Map (&T, &T) to (&T, &T, &T) by duplicating the first element.
         // iter_all_added? or iter_all? maybe we can iter over dominant elements?
         self.0.iter_all_added().map(|(a, b)| (a, b, self.0.combined.get_dominant_elem(a).unwrap()))
      }))
   }

   fn len_estimate(&self) -> usize {
      1
   }
}
impl<'a, T: Clone + Hash + Eq> RelIndexReadAll<'a> for EqRelCanonicalIndNone<'a, T> {
   type Key = ();

   type Value = (&'a T, &'a T, &'a T);

   // type ValueIteratorType = IteratorFromDyn<'a, (&'a T, &'a T, &'a T)>;

   // type AllIteratorType = std::option::IntoIter<(Self::Key, Self::ValueIteratorType)>;

   fn iter_all(&'a self) -> impl Iterator<Item = (Self::Key, impl Iterator<Item = Self::Value> + 'a)> + 'a {
      self.index_get(&()).map(|iter| ((), iter)).into_iter()
   }
}

impl<T: Clone + Hash + Eq> RelIndexWrite for EqRelCanonicalIndNone<'_, T> {
   type Key = ();
   type Value = (T, T, T);
   fn index_insert(&mut self, _key: Self::Key, _value: Self::Value) { /* noop */
   }
}

impl<T: Clone + Hash + Eq> RelIndexMerge for EqRelCanonicalIndNone<'_, T> {
   fn move_index_contents(_from: &mut Self, _to: &mut Self) { /* noop */
   }
}

#[derive(Default)]
pub struct ToEqRelCanonicalIndNone<T>(PhantomData<T>);

impl<T: Clone + Hash + Eq> ToRelIndex<EqRelIndCommon<T>> for ToEqRelCanonicalIndNone<T> {
   type RelIndex<'a>
      = EqRelCanonicalIndNone<'a, T>
   where
      T: 'a;
   fn to_rel_index<'a>(&'a self, rel: &'a EqRelIndCommon<T>) -> Self::RelIndex<'a> {
      EqRelCanonicalIndNone(rel)
   }

   type RelIndexWrite<'a>
      = EqRelCanonicalIndNone<'a, T>
   where
      T: 'a;
   fn to_rel_index_write<'a>(&'a mut self, rel: &'a mut EqRelIndCommon<T>) -> Self::RelIndexWrite<'a> {
      EqRelCanonicalIndNone(rel)
   }
}

pub struct EqRelCanonicalInd0_1<'a, T: Clone + Hash + Eq>(&'a EqRelIndCommon<T>);

pub struct ToEqRelCanonicalInd0_1<T>(PhantomData<T>);

impl<T> Default for ToEqRelCanonicalInd0_1<T> {
   fn default() -> Self {
      Self(Default::default())
   }
}

impl<'a, T: Clone + Hash + Eq> RelIndexRead<'a> for EqRelCanonicalInd0_1<'a, T> {
   type Key = (T, T);
   type Value = (&'a T,);

   // type IteratorType = std::iter::Chain<std::iter::Once<(&'a T,)>, std::iter::Empty<(&'a T,)>>;

   fn index_get(&'a self, key: &Self::Key) -> Option<impl Iterator<Item = Self::Value> + Clone + 'a> {
      // let _ = self.0.set_of_added(&key.0)?;
      if &key.0 == &key.1 {
         self
            .0
            .combined
            .get_dominant_elem(&key.0)
            .map(|canonical_key| std::iter::once((canonical_key,)).chain(std::iter::empty()))
      } else {
         // check if the are the same element
         if self.0.combined.contains(&key.0, &key.1) {
            self
               .0
               .combined
               .get_dominant_elem(&key.0)
               .map(|canonical_key| std::iter::once((canonical_key,)).chain(std::iter::empty()))
         } else {
            None
         }
      }
   }

   fn len_estimate(&self) -> usize {
      self.0.combined.elem_ids.len()
   }

   fn combine_delta(&self) -> bool {
      false
   }
}

impl<'a, T: Clone + Hash + Eq> RelIndexReadAll<'a> for EqRelCanonicalInd0_1<'a, T> {
   type Key = (&'a T, &'a T);
   type Value = (&'a T,);

   // type ValueIteratorType = std::iter::Once<(&'a T,)>;

   // type AllIteratorType = Box<dyn Iterator<Item = (Self::Key, Self::ValueIteratorType)> + 'a>;

   fn iter_all(&'a self) -> impl Iterator<Item = (Self::Key, impl Iterator<Item = Self::Value> + 'a)> + 'a {
      self.0.combined.sets.iter().flat_map(move |s| {
         s.iter().flat_map(move |x| {
            s.iter().map(move |y| {
               let canonical = self.0.combined.get_dominant_elem(x).unwrap();
               ((x, y), std::iter::once((canonical,)))
            })
         })
      })
   }

   fn combine_delta(&self) -> bool {
      false
   }
}

impl<T: Clone + Hash + Eq> RelIndexWrite for EqRelCanonicalInd0_1<'_, T> {
   type Key = (T, T);
   type Value = (T,);
   fn index_insert(&mut self, _key: Self::Key, _value: Self::Value) {
      // noop
   }
}

impl<T: Clone + Hash + Eq> RelIndexMerge for EqRelCanonicalInd0_1<'_, T> {
   fn move_index_contents(_from: &mut Self, _to: &mut Self) {
      //noop
   }
}

impl<T: Clone + Hash + Eq> ToRelIndex<EqRelIndCommon<T>> for ToEqRelCanonicalInd0_1<T> {
   type RelIndex<'a>
      = EqRelCanonicalInd0_1<'a, T>
   where
      T: 'a;
   fn to_rel_index<'a>(&'a self, rel: &'a EqRelIndCommon<T>) -> Self::RelIndex<'a> {
      EqRelCanonicalInd0_1(rel)
   }

   type RelIndexWrite<'a>
      = EqRelCanonicalInd0_1<'a, T>
   where
      T: 'a;
   fn to_rel_index_write<'a>(&'a mut self, rel: &'a mut EqRelIndCommon<T>) -> Self::RelIndexWrite<'a> {
      EqRelCanonicalInd0_1(rel)
   }
}
