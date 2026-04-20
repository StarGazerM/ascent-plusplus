//! Fast CSV loader: mmap + byte-level parsing. No `fs::read_to_string`,
//! no `Vec<&str>` per line, no UTF-8 validation. ~5-10× faster than the
//! naive loader on large files.

use std::fs::File;
use std::path::Path;

/// Load a two-column comma- or tab-separated CSV of `(i32, i32)` tuples.
pub fn load_pairs<P: AsRef<Path>>(path: P) -> Vec<(i32, i32)> {
   let file = File::open(&path).unwrap_or_else(|e| panic!("open {:?}: {e}", path.as_ref()));
   let mmap = unsafe { memmap2::Mmap::map(&file).expect("mmap") };
   let bytes: &[u8] = &mmap;

   // Determine delimiter from the first line.
   let first_nl = bytes.iter().position(|&b| b == b'\n').unwrap_or(bytes.len());
   let delim = if bytes[..first_nl].contains(&b',') { b',' } else { b'\t' };

   let mut out = Vec::with_capacity(bytes.len() / 8);
   let mut i = 0;
   while i < bytes.len() {
      // Find end of line.
      let mut j = i;
      while j < bytes.len() && bytes[j] != b'\n' {
         j += 1;
      }
      if j > i {
         // Find delimiter inside [i..j].
         let mut k = i;
         while k < j && bytes[k] != delim {
            k += 1;
         }
         if k < j {
            let a = atoi(&bytes[i..k]);
            let b = atoi(&bytes[k + 1..j]);
            out.push((a, b));
         }
      }
      i = j + 1;
   }
   out
}

/// Parse a nonnegative or signed decimal integer from bytes (no UTF-8
/// validation, no overflow check — trusts the benchmark CSVs).
#[inline]
fn atoi(s: &[u8]) -> i32 {
   let (neg, s) = if !s.is_empty() && s[0] == b'-' { (true, &s[1..]) } else { (false, s) };
   let mut n: i32 = 0;
   for &b in s {
      n = n * 10 + (b - b'0') as i32;
   }
   if neg { -n } else { n }
}
