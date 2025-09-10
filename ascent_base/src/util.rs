//! internal utility functions defined here
//! 
//! CAUTION: anything defined here is subject to change in semver-compatible releases

/// update `reference` in-place using the provided closure
pub fn update<T: Default>(reference: &mut T, f: impl FnOnce(T) -> T) {
   let ref_taken = std::mem::take(reference);
   let new_val = f(ref_taken);
   *reference = new_val;
}

#[test]
fn test_update(){
   let mut vec = vec![1, 2, 3];
   update(&mut vec, |mut v| {v.push(4); v});
   assert_eq!(vec, vec![1, 2, 3, 4]);
}

// compute the hash of the given tuple
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