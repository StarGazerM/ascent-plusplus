/// A macro to count the number of elements in a tuple literal, expanding
/// directly to an integer literal (e.g., `3`) at compile time.
///
/// This uses a "token tree muncher" pattern. It recursively processes each
/// element of the tuple, building up a list of `()` tokens. A final helper
/// macro then matches on the list of `()` tokens to produce a hardcoded
/// integer literal.
///
/// Usage: `count_tuple_elements!((item1, item2, ...))`
#[macro_export]
macro_rules! count_tuple_elements {
    // This is the public-facing entry point.
    // It matches a tuple literal, with an optional trailing comma.
    ( ($($items:expr),* $(,)?) ) => {
        // It kicks off the recursive muncher, starting with an empty
        // accumulator `[]` and passing along all the matched items.
        // A comma is added to each item to normalize the input for the muncher.
        count_tuple_elements_muncher!([]; $($items,)*)
    };
}

/// The internal "muncher" that recursively counts items.
#[macro_export]
macro_rules! count_tuple_elements_muncher {
    // Base case: No items are left to munch.
    // The accumulator `[$($acc:tt)*]` now holds one `()` for each item.
    // We call the final conversion macro to turn this into a literal.
    ([$($acc:tt)*];) => {
        count_tuple_elements_to_literal!([$($acc)*])
    };

    // Recursive step: At least one item remains.
    // 1. `$head:expr`: Match the next item.
    // 2. `$($tail:tt)*`: Capture the rest of the items.
    // 3. `count_tuple_elements_muncher!([$($acc)* ()]; ...)`:
    //    Recursively call the muncher, adding one `()` to the accumulator
    //    and passing the rest of the tail to be processed.
    ([$($acc:tt)*]; $head:expr, $($tail:tt)*) => {
        count_tuple_elements_muncher!([$($acc)* ()]; $($tail)*)
    };
}

/// Converts the accumulated `()` tokens into a final integer literal.
/// NOTE: This has a hardcoded limit. To count larger tuples, you must add more lines.
#[macro_export]
macro_rules! count_tuple_elements_to_literal {
   ([]) => {
      0usize
   };
   ([()]) => {
      1usize
   };
   ([() ()]) => {
      2usize
   };
   ([() () ()]) => {
      3usize
   };
   ([() () () ()]) => {
      4usize
   };
   ([() () () () ()]) => {
      5usize
   };
   ([() () () () () ()]) => {
      6usize
   };
   ([() () () () () () ()]) => {
      7usize
   };
   ([() () () () () () () ()]) => {
      8usize
   };
   ([() () () () () () () () ()]) => {
      9usize
   };
   ([() () () () () () () () () ()]) => {
      10usize
   };
   ([() () () () () () () () () () ()]) => {
      11usize
   };
   ([() () () () () () () () () () () ()]) => {
      12usize
   };
}

#[macro_export]
macro_rules! empty_to_rel_index {
   ($name: ident) => {
      impl<R> ToRelIndex<R> for $name {
         type RelIndex<'a>
            = &'a Self
         where
            Self: 'a,
            R: 'a;

         fn to_rel_index<'a>(&'a self, _rel: &'a R) -> Self::RelIndex<'a> {
            self
         }

         type RelIndexWrite<'a>
            = &'a mut Self
         where
            Self: 'a,
            R: 'a;

         fn to_rel_index_write<'a>(&'a mut self, _rel: &'a mut R) -> Self::RelIndexWrite<'a> {
            self
         }
      }
   };
}
pub use empty_to_rel_index;

#[macro_export]
macro_rules! empty_rel_index_write {
   ($name: ident, $key: ty, $value: ty) => {
      impl<'a> RelIndexWrite for $name {
         type Key = $key;
         type Value = $value;
         fn index_insert(&mut self, _key: Self::Key, _value: Self::Value) {}
      }
   };
}
pub use empty_rel_index_write;
