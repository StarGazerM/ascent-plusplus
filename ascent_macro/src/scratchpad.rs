#![allow(unused_imports)]
use std::fmt::Debug;
use std::hash::Hash;
use std::ops::Deref;
use std::{clone, cmp::max, rc::Rc};

#[allow(dead_code)]
pub trait Atom:
   From<usize> + Into<usize> + Copy + Clone + std::fmt::Debug + Eq + Ord + Hash + Sync + Send + 'static
{
   fn index(self) -> usize;
}

#[allow(dead_code)]
pub trait FactTypes: Copy + Clone + Debug {
   type Origin: Atom;
   type Loan: Atom;
   type Point: Atom;
   type Variable: Atom;
   type Path: Atom;
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum List<T> {
   Cons(T, Rc<List<T>>),
   Nil,
}

macro_rules! cons {
   ($h: expr, $t: expr) => {
      Rc::new(List::Cons($h, $t))
   };
}

macro_rules! nil {
   () => {
      Rc::new(List::Nil)
   };
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct Res(&'static str);

#[warn(warnings)]
#[allow(unused_imports)]
#[allow(dead_code)]
#[allow(redundant_semicolons)]
#[cfg(test)]
fn _test<T: FactTypes>() {
   use ascent::Dual;
   use ascent::aggregators::*;
   use ascent::lattice::set::Set;

   use ascent::rel as custom_ds;
   ::ascent::rel::rel_codegen! { AscentProgram_c , (i32 , i32) , [[0] , [0 , 1]] , ser , () }
   ::ascent::rel::rel_codegen! { AscentProgram_a , (i32 , i32) , [[0 , 1] , [1]] , ser , () }
   ::ascent::rel::rel_codegen! { AscentProgram_b , (i32 , i32) , [[0] , [0 , 1]] , ser , () }
   
}
