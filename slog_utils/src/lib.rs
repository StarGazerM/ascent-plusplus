
/// calculate the id of a tuple
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
pub fn calc_id<T: Hash>(t: &T) -> usize {
   let mut s = DefaultHasher::new();
   t.hash(&mut s);
   s.finish() as usize
}

#[test]
fn test_calc_id(){
   let t = (1, 2, 3);
   assert_eq!(calc_id(&t), 646939227381880718);
}

/// collect all the items in the input iterator as a vector
pub fn collect<'a, N: 'a>(inp: impl Iterator<Item = (&'a N,)>) -> impl Iterator<Item = Vec<N>>
where N: Clone
{
   std::iter::once(inp.map(|tuple| tuple.0.clone()).collect())
}

pub use usize as eclass_id;
