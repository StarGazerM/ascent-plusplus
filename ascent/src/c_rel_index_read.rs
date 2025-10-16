use rayon::iter::ParallelIterator;

pub trait CRelIndexRead<'a> {
   type Key: Send + 'a;
   type Value: Send + 'a;
   // type IteratorType: ParallelIterator<Item = Self::Value> + Clone + 'a;
   fn c_index_get(&'a self, key: &Self::Key) -> Option<impl ParallelIterator<Item = Self::Value> + Clone + 'a>;
}

pub trait CRelIndexReadAll<'a> {
   type Key: Send + 'a;
   type Value: Send + 'a;
   // type ValueIteratorType: ParallelIterator<Item = Self::Value> + 'a;
   // type AllIteratorType: ParallelIterator<Item = (Self::Key, Self::ValueIteratorType)> + 'a;
   fn c_iter_all(&'a self) -> impl ParallelIterator<Item = (Self::Key, impl ParallelIterator<Item = Self::Value> + 'a)> + 'a;
}
