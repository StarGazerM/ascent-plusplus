use rayon::prelude::*;

use crate::internal::{CRelIndexRead, CRelIndexReadAll, ParCombIter};
use crate::rel_index_read::RelIndexCombined;

impl<'a, Ind1, Ind2, K, V> CRelIndexRead<'a> for RelIndexCombined<'a, Ind1, Ind2>
where
   Ind1: CRelIndexRead<'a, Key = K, Value = V>,
   Ind2: CRelIndexRead<'a, Key = K, Value = V>,
   K: Send + 'a,
   V: Send + 'a,
{
   type Key = K;

   type Value = V;

   // type IteratorType = rayon::iter::Chain<
   //    rayon::iter::Flatten<rayon::option::IntoIter<<Ind1 as CRelIndexRead<'a>>::IteratorType>>,
   //    rayon::iter::Flatten<rayon::option::IntoIter<<Ind2 as CRelIndexRead<'a>>::IteratorType>>,
   // >;

   fn c_index_get(&'a self, key: &Self::Key) -> Option<impl ParallelIterator<Item = Self::Value> + Clone + 'a> {
      match (self.ind1.c_index_get(key), self.ind2.c_index_get(key)) {
         (None, None) => None,
         (iter1, iter2) => {
            let res = iter1.into_par_iter().flatten().chain(iter2.into_par_iter().flatten());
            Some(res)
         },
      }
   }
}


impl<'a, Ind1, Ind2, K: Send + 'a, V: Send + 'a> CRelIndexReadAll<'a>
   for RelIndexCombined<'a, Ind1, Ind2>
where
   Ind1: CRelIndexReadAll<'a, Key = K, Value = V>,
   Ind2: CRelIndexReadAll<'a, Key = K, Value = V>,
{
   type Key = K;
   type Value = V;

   // type ValueIteratorType = VTI;
   // type AllIteratorType = rayon::iter::Chain<Ind1::AllIteratorType, Ind2::AllIteratorType>;

   fn c_iter_all(&'a self) -> impl ParallelIterator<Item = (Self::Key, impl ParallelIterator<Item = Self::Value> + 'a)> + 'a {
      // 1. Map over the first iterator and wrap its inner iterator in the enum.
      let iter1 = self.ind1.c_iter_all().map(|(k, v_par_iter)| {
         (k, ParCombIter::First(v_par_iter))
   });

   // 2. Do the same for the second iterator.
   let iter2 = self.ind2.c_iter_all().map(|(k, v_par_iter)| {
         (k, ParCombIter::Second(v_par_iter))
   });

   // 3. Now that both iterators yield the exact same item type, they can be chained.
   iter1.chain(iter2)
   }
}
