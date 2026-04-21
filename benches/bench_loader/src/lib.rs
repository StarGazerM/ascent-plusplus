//! Fast mmap + byte-level CSV/TSV loader for integration benchmarks.
//!
//! Each `load_N(dir, name)` tries `<dir>/<name>` first; if that misses and
//! `name` has no extension, falls back to `<dir>/<name>.csv`, then
//! `<dir>/<name>.facts` (the Souffle/Polonius convention). Delimiter is
//! auto-detected per file from the first line (`\t` or `,`).
//!
//! Returns an empty vector on missing files — mirrors the SRDatalog
//! `fs::read_to_string().unwrap_or_default()` behaviour so callers don't
//! need to know which optional inputs a dataset has.

use std::fs::File;
use std::path::{Path, PathBuf};

fn try_open(dir: &str, name: &str) -> Option<memmap2::Mmap> {
   let candidates: Vec<PathBuf> = if Path::new(name).extension().is_some() {
      vec![PathBuf::from(format!("{dir}/{name}"))]
   } else {
      vec![
         PathBuf::from(format!("{dir}/{name}")),
         PathBuf::from(format!("{dir}/{name}.csv")),
         PathBuf::from(format!("{dir}/{name}.facts")),
      ]
   };
   for p in candidates {
      if let Ok(f) = File::open(&p) {
         if let Ok(m) = unsafe { memmap2::Mmap::map(&f) } {
            return Some(m);
         }
      }
   }
   None
}

#[inline]
fn atoi(s: &[u8]) -> i32 {
   let (neg, s) = if !s.is_empty() && s[0] == b'-' { (true, &s[1..]) } else { (false, s) };
   let mut n: i32 = 0;
   for &b in s {
      n = n * 10 + (b - b'0') as i32;
   }
   if neg { -n } else { n }
}

fn for_each_row<F: FnMut(&[&[u8]])>(bytes: &[u8], arity: usize, mut f: F) {
   // Detect delimiter from first non-empty line.
   let first_nl = bytes.iter().position(|&b| b == b'\n').unwrap_or(bytes.len());
   let first_line = &bytes[..first_nl];
   let delim = if first_line.contains(&b',') { b',' } else { b'\t' };

   let mut fields: Vec<&[u8]> = Vec::with_capacity(arity);
   let mut i = 0;
   while i < bytes.len() {
      // End of line.
      let mut j = i;
      while j < bytes.len() && bytes[j] != b'\n' {
         j += 1;
      }
      if j > i {
         // Split [i..j] on delim.
         fields.clear();
         let mut start = i;
         let mut k = i;
         while k < j {
            if bytes[k] == delim {
               fields.push(&bytes[start..k]);
               start = k + 1;
            }
            k += 1;
         }
         fields.push(&bytes[start..j]);
         if fields.len() >= arity {
            f(&fields[..arity]);
         }
      }
      i = j + 1;
   }
}

macro_rules! loader {
   ($fn:ident, $n:expr, ($($idx:tt),+), ($($t:ty),+)) => {
      #[allow(clippy::needless_range_loop)]
      pub fn $fn(dir: &str, name: &str) -> Vec<($($t,)+)> {
         let Some(mmap) = try_open(dir, name) else { return Vec::new(); };
         let bytes: &[u8] = &mmap;
         let mut out = Vec::with_capacity(bytes.len() / 8);
         for_each_row(bytes, $n, |f| {
            out.push(( $(atoi(f[$idx]),)+ ));
         });
         out
      }
   };
}

loader!(load_1, 1, (0), (i32));
loader!(load_2, 2, (0, 1), (i32, i32));
loader!(load_3, 3, (0, 1, 2), (i32, i32, i32));
loader!(load_4, 4, (0, 1, 2, 3), (i32, i32, i32, i32));
loader!(load_5, 5, (0, 1, 2, 3, 4), (i32, i32, i32, i32, i32));
loader!(load_6, 6, (0, 1, 2, 3, 4, 5), (i32, i32, i32, i32, i32, i32));
loader!(load_7, 7, (0, 1, 2, 3, 4, 5, 6), (i32, i32, i32, i32, i32, i32, i32));
loader!(load_8, 8, (0, 1, 2, 3, 4, 5, 6, 7), (i32, i32, i32, i32, i32, i32, i32, i32));
