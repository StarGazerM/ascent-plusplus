/// calculate the id of a tuple
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
pub fn calc_id<T: Hash>(t: &T) -> usize {
   let mut s = DefaultHasher::new();
   t.hash(&mut s);
   s.finish() as usize
}

#[test]
fn test_calc_id() {
   let t = (1, 2, 3);
   assert_eq!(calc_id(&t), 646939227381880718);
}

/// collect all the items in the input iterator as a vector
pub fn collect<'a, N: 'a>(inp: impl Iterator<Item = (&'a N,)>) -> impl Iterator<Item = Vec<N>>
where N: Clone {
   std::iter::once(inp.map(|tuple| tuple.0.clone()).collect())
}

pub use usize as eclass_id;

// WARN: Generated from Gemini, do not trust this macro
#[macro_export]
macro_rules! id_vec {
   // ## Macro Matcher ##
   // It matches the store expression, a comma, and then the list of tuples.
   ( $calc_id:ident, [ $( ( $($fields:expr),* ) ),* $(,)? ] ) => {
        // ## Macro Expansion ##
        // It expands into a `vec!` literal.
        vec![
            // The `$()*` block repeats for each tuple in the input.
            $(
                // For each tuple, it generates a new, larger tuple:
                // ( id_from_calc_id, original_field_1, original_field_2, ... )
                (
                    $($fields,)* // Splice in the original fields
                    $calc_id( &( $($fields),* ) ), // Call calc_id on the original tuple
                )
            ),*
        ]
    };
}

#[macro_export]
macro_rules! slog_gen {
   ($struct_name:ident, { $($prev_code:tt)* }, { $($new_code:tt)* }, $slog_macro:ident) => {
        $slog_macro! {
            (struct $struct_name)
            $($prev_code)*
            $($new_code)*
        }
    };
}
