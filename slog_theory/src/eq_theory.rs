// use ascent::lattice;
// use slog::slog_source;
// slog_source! {
//   eq_theory (unification_rel):
//   unify_eclass(x, x, x) <-- unify_eclass_type(x);
// }

// use ascent_byods_rels::{eqrel_ind::EqRelIndCommon, union_find::EqRel};

// pub fn canonicalize_eclass<'a>(
//    full: &'a EqRelIndCommon<usize>, delta: &'a EqRelIndCommon<usize>, x: &'a usize,
// ) -> &'a usize {
//    full.combined.get_dominant_elem(x).unwrap_or(delta.combined.get_dominant_elem(x).unwrap_or(x))
// }

// use std::cell::RefCell;
// use std::rc::Rc;
// use std::{
//    cmp::Ordering,
//    fmt::{Debug, Formatter},
//    hash::{Hash, Hasher},
// };

// #[derive(Clone, Default)]
// pub struct Quotient<T: Clone + Hash + Eq + PartialOrd + Debug> {
//    pub set: Rc<RefCell<EqRel<T>>>,
//    pub repr: T,
// }

// // impl<T: Clone + Hash + Eq + PartialOrd + Debug + Default> Default for Quotient<T> {
// //    fn default() -> Self {
// //       Self { set: Rc::new(RefCell::new(EqRel::default())), repr: T::default() }
// //    }
// // }

// impl<T: Clone + Hash + Eq + PartialOrd + Debug> Quotient<T> {
//    // Frequent read operations - use borrow() for shared access
//    pub fn contains(&self, other: &Self) -> bool {
//       self.set.borrow().contains(&self.repr, &other.repr)
//    }

//    pub fn get_canonical(&self) -> T {
//       self.set.borrow().get_dominant_elem(&self.repr).cloned().unwrap_or_else(|| self.repr.clone())
//    }
// }

// impl<T: Clone + Hash + Eq + PartialOrd + Debug> Hash for Quotient<T> {
//    fn hash<H: Hasher>(&self, state: &mut H) {
//       self.set.borrow().get_dominant_elem(&self.repr).hash(state)
//    }
// }

// impl<T: Clone + Hash + Eq + PartialOrd + Debug> PartialEq for Quotient<T> {
//    fn eq(&self, other: &Self) -> bool {
//       // check if use the same disjoint set
//       Rc::ptr_eq(&self.set, &other.set) && self.set.borrow().contains(&self.repr, &other.repr)
//    }
// }

// impl<T: Clone + Hash + Eq + PartialOrd + Debug> Eq for Quotient<T> {}

// impl<T: Clone + Hash + Eq + PartialOrd + Debug> Debug for Quotient<T> {
//    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//       // only print the repr
//       write!(f, "{:?}", self.repr)
//    }
// }

// impl<T: Clone + Hash + Eq + PartialOrd + Debug> PartialOrd for Quotient<T> {
//    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
//       // check if use the same disjoint set
//       if Rc::ptr_eq(&self.set, &other.set) {
//          let set_borrow = self.set.borrow();
//          let self_canonical = set_borrow.get_dominant_elem(&self.repr).unwrap();
//          let other_canonical = set_borrow.get_dominant_elem(&other.repr).unwrap();
//          if self_canonical == other_canonical {
//             if &self.repr == self_canonical && &other.repr == other_canonical {
//                Some(Ordering::Equal)
//             } else if &self.repr == self_canonical && &other.repr != other_canonical {
//                Some(Ordering::Less)
//             } else if &self.repr != self_canonical && &other.repr == other_canonical {
//                Some(Ordering::Greater)
//             } else if self_canonical == other_canonical {
//                Some(Ordering::Equal)
//             } else {
//                None
//             }
//          } else {
//             None
//          }
//       } else {
//          None
//       }
//    }
// }

// impl<T: Clone + Hash + Eq + PartialOrd + Debug> lattice::Lattice for Quotient<T> {
//    fn meet_mut(&mut self, _other: Self) -> bool {
      
//    }

//    fn join_mut(&mut self, _other: Self) -> bool {
      
//    }
// }

// mod tests {
//    use ascent::Lattice;

// #[allow(unused_imports)]
//    use super::*;

//    #[test]
//    fn test_quotient() {
//       // Build with interior mutability
//       let set = Rc::new(RefCell::new(EqRel::default()));
//       let mut quotient = Quotient { set: Rc::clone(&set), repr: 1 };

//       // Write operations (rare) - use borrow_mut()
//       {
//          let mut eqrel = set.borrow_mut();
//          eqrel.add(1, 2);
//          eqrel.add(2, 3);
//          eqrel.add(3, 4);
//       } // borrow_mut() dropped here

//       // Read operations (frequent) - use borrow()
//       assert!(quotient.contains(&Quotient { set: Rc::clone(&set), repr: 2 }));
//       println!("Canonical of 1: {:?}", quotient.get_canonical());

//    }
// }
