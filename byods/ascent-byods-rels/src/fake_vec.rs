use std::marker::PhantomData;
use std::ops::Index;

use crate::union_find::EqRel;

pub struct FakeVec<T> {
   _phantom: PhantomData<T>,
}

impl<T> Default for FakeVec<T> {
   fn default() -> Self { Self { _phantom: PhantomData } }
}

impl<T> FakeVec<T> {
   #[inline(always)]
   pub fn push(&self, _: T) {}

   pub fn is_empty(&self) -> bool { self.len() == 0 }

   pub fn len(&self) -> usize { 0 }

   pub fn iter(&self) -> std::iter::Empty<&T> { std::iter::empty() }
}

impl<T> Index<usize> for FakeVec<T> {
   type Output = T;

   fn index(&self, _index: usize) -> &Self::Output { panic!("FakeVec is empty!") }
}

// a fake vector implemented by a equivalence relation
pub struct VecEqRel {
   pub eqrel: EqRel<usize>,
}

impl Default for VecEqRel {
   fn default() -> Self {
      Self { eqrel: EqRel::default() }
   }
}

impl VecEqRel {
   pub fn push(&mut self, tp: (usize, usize, usize)) {
      self.eqrel.add(tp.0, tp.1);
      // self.eqrel.add(tp.0, tp.2);
      // self.eqrel.add(tp.1, tp.2);
   }

   pub fn is_empty(&self) -> bool {
      self.eqrel.sets.is_empty()
   }

   pub fn len(&self) -> usize {
      self.eqrel.sets.len()
   }

   pub fn iter(&self) -> impl Iterator<Item = (usize, usize, usize)> {
      println!("Iterating over FakeVecEqRel, this will force the materialization of the equivalence relation");
      self.eqrel.iter_all().map(|(x, y)| {
         (*x, *y, *self.eqrel.get_dominant_elem(x).unwrap())
      }).collect::<Vec<_>>().into_iter()
   }
}
