//! DD-backend shim for the `ascent_m_par` / `ascent_run_m_par` macros the
//! upstream tests use. Both variants always route to the DD backend here;
//! `par` isn't a valid combination with `#![backend(dd)]` so we simply ignore
//! any distinction between par and non-par when running under DD.
//!
//! `lat_to_vec` is re-exported for test-surface compatibility.

#[allow(dead_code)]
pub fn lat_to_vec<T>(vec: Vec<T>) -> Vec<T> { vec }

#[macro_export]
macro_rules! ascent_m_par {
   ($($tt: tt)*) => {
      ::ascent::ascent! { #![backend(dd)] $($tt)* }
   };
}

#[macro_export]
macro_rules! ascent_run_m_par {
   ($($tt: tt)*) => {
      ::ascent::ascent_run! { #![backend(dd)] $($tt)* }
   };
}
