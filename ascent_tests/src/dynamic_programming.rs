//  Longest Palindromic Substring
//  Given a string s, return the longest palindromic substring in s.
//  Example 1:
//  Input: s = "babad"
//  Output: "bab"
//  Note: "aba" is also a valid answer.
//  Example 2:
//  Input: s = "cbbd"
//  Output: "bb"

use std::{
   collections::{BTreeMap, BTreeSet, HashMap, HashSet},
   mem,
};

use ascent::{ascent_run, ascent_run_par, Lattice};

type Palindrome = (usize, usize);

fn longest_palindromic(s: &str) -> String {
   let mut new_palindromes: Vec<Palindrome> = vec![];

   for i in 0..s.len() {
      let p_odd = (i, i);
      let p_even = (i, i + 1);
      // palindromes_set.insert(p.clone());
      new_palindromes.push(p_odd);
      new_palindromes.push(p_even);
   }

   // let mut longest_len = 0;
   loop {
      // println!("longest_len: {:?}", longest_len);
      let mut next_palindromes: Vec<Palindrome> = vec![];

      for (start, end) in new_palindromes.iter() {
         if *start > 0 && *end < s.len() && s.as_bytes()[start - 1] == s.as_bytes()[*end] {
            next_palindromes.push((start - 1, end + 1));
         }
      }
      if next_palindromes.len() == 0 {
         break;
      }
      // longest_len += 1;
      mem::swap(&mut new_palindromes, &mut next_palindromes);
   }

   // return the longest of new_palindromes
   new_palindromes.sort_by(|a, b| (b.1 - b.0).cmp(&(a.1 - a.0)));
   let (l_s, l_e) = new_palindromes[0];
   s[l_s..l_e].to_string()
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct PalindromeR(usize, usize);

impl PalindromeR {
   fn size(&self) -> usize { self.1 - self.0 }
}

impl PartialOrd for PalindromeR {
   fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> { Some(self.size().cmp(&other.size())) }
}

impl Ord for PalindromeR {
   fn cmp(&self, other: &Self) -> std::cmp::Ordering { self.size().cmp(&other.size()) }
}

impl Lattice for PalindromeR {
   fn meet_mut(&mut self, other: Self) -> bool {
      // compare the length of the two palindromes
      if (self.1 - self.0) > (other.1 - other.0) {
         *self = other;
         true
      } else {
         false
      }
   }

   fn join_mut(&mut self, other: Self) -> bool {
      // compare the length of the two palindromes
      if (self.1 - self.0) < (other.1 - other.0) {
         *self = other;
         true
      } else {
         false
      }
   }
}

fn longest_palindromic_ascent(s: &str) -> String {
   let res = ascent_run! {
       #[ds(ascent_byods_rels::linear::linear)]
       relation palindromes(usize, usize); // this relation can be linear
       lattice longest_palindromic(PalindromeR);

       palindromes(i,i), palindromes(i,i+1) <-- for i in 0..s.len();

       palindromes(start-1, end+1) <--
           palindromes(start, end),
           if *start > 0 && *end < s.len(),
           if s.as_bytes()[start - 1] == s.as_bytes()[*end];

       longest_palindromic(PalindromeR(*from, *to)) <--
           palindromes(from, to);
   };

   let PalindromeR(l_s, l_e) = res.longest_palindromic[0].0;
   println!("{:?} : l_s: {:?}, l_e: {:?}", res.longest_palindromic.len(), l_s, l_e);
   s[l_s..l_e].to_string()
}

#[test]
fn test_longest_palindromic() {
   let s = "babad";
   let result = longest_palindromic(s);
   let result_ascent = longest_palindromic_ascent(s);
   println!("result: {:?}", result);
   println!("result_ascent: {:?}", result_ascent);
   // assert_eq!(result, "bab".to_string());
   let s = "cbbd";
   let result = longest_palindromic(s);
   let result_ascent = longest_palindromic_ascent(s);
   println!("result: {:?}", result);
   println!("result_ascent: {:?}", result_ascent);
   let s = "a";
   let result = longest_palindromic(s);
   let result_ascent = longest_palindromic_ascent(s);
   println!("result: {:?}", result);
   println!("result_ascent: {:?}", result_ascent);
   // assert_eq!(result, "bb".to_string());
   let s = "aacabdkacaa";
   let result = longest_palindromic(s);
   let result_ascent = longest_palindromic_ascent(s);
   println!("result: {:?}", result);
   println!("result_ascent: {:?}", result_ascent);
}

// Given a string containing just the characters '(' and ')', return the length of the longest valid (well-formed) parentheses
// substring
// .
// Example 1:
// Input: s = "(()"
// Output: 2
// Explanation: The longest valid parentheses substring is "()".
// Example 2:
// Input: s = ")()())"
// Output: 4
// Explanation: The longest valid parentheses substring is "()()".
// Example 3:
// Input: s = ""
// Output: 0
// Constraints:
//     0 <= s.length <= 3 * 104
//     s[i] is '(', or ')'.

fn longest_valid_parentheses(s: String) -> i32 {
   let mut stack: Vec<i32> = vec![];
   let mut max_length = 0;

   // Push a boundary index to handle edge cases
   stack.push(-1);

   for (i, c) in s.chars().enumerate() {
      if c == '(' {
         // Push the index of '('
         stack.push(i as i32);
      } else {
         // Pop the stack for ')'
         stack.pop();
         if stack.is_empty() {
            // Push the current index as a new boundary
            stack.push(i as i32);
         } else {
            // Calculate the valid substring length
            let valid_length = (i as i32) - stack.last().unwrap();
            max_length = max_length.max(valid_length);
         }
      }
   }

   max_length
}

fn longest_valid_parentheses_ascent(s: String) -> i32 {
   let res = ascent_run! {
      lattice ranges(usize, usize);
      // a range not made by sequential ranges
      relation longest_range(usize);
      // init longest_range to 0
      longest_range(0);

      // init ranges for every "()" in string
      ranges(i, i+2) <-- for i in 0..s.len(), if s.as_bytes()[i] == b'(' && i + 1 < s.len() && s.as_bytes()[i + 1] == b')';
      // if a range's outside is wrapped by "()", it is a range
      ranges(from-1, to+1) <--
          ranges(from, to),
          if *from > 0 && *to < s.len(),
          if s.as_bytes()[from - 1] == b'(' && s.as_bytes()[*to] == b')';
      // if two range are connected, they are a range
      ranges(from, to) <-- ranges(from, mid), ranges(mid, to);
      // send every change to compare with longest_range
      longest_range(*to - *from) <-- ranges(from, to);
   };

   // let lv = res.longest_range[0].read().unwrap().0 as i32;
   // lv
   res.longest_range[0].0 as i32
   // 0
}

fn longest_valid_parentheses_ascent1(s: String) -> i32 {
   let res = ascent_run! {
      #![measure_rule_times]
      lattice longest_range(usize);
      longest_range(0);
      // relation input_string(usize, u8);
      // input_string(i, c) <-- for i in 1..s.len() + 1, let c = s.as_bytes()[i - 1];

      // the longest range ends at i is j, inclusive
      lattice longest_ranges_ends(usize, usize);

      // case 0: "" empty is a range has size 0
      // init longest_range to 0
      longest_ranges_ends(i + 1, 0) <-- for i in 0..s.len();

      // case 1 : ( {range} ) is a range
      longest_ranges_ends(i + 1, j + 2) <--
         longest_ranges_ends(i, j),
         if i < &s.len() && i > j,
         if s.as_bytes()[*i] == b')' && s.as_bytes()[i - j - 1] == b'(';
         // input_string(i + 1, b')'),
         // input_string(i - j, b'(');

      // case 2 : {range} {range} is a range
      // if two ranges are connected, update the later range's size
      longest_ranges_ends(i, size1 + size2) <--
         longest_ranges_ends(i, size1),
         longest_ranges_ends(i - size1, size2);

      longest_range(size) <-- longest_ranges_ends(_, size);
   };
   // println!("longest_range: {:?}", res.);

   res.longest_range[0].0 as i32
}

// generate a huge random string has 1000000 characters, which are all '('/ ')'
fn generate_random_paren_string() -> String {
   use rand::Rng;
   let mut rng = rand::thread_rng();
   let mut s = String::new();
   for _ in 0..100000 {
      let c = if rng.gen_bool(0.5) { '(' } else { ')' };
      s.push(c);
   }
   s
}

#[test]
fn test_longest_valid_parentheses() {
   let s = "(()".to_string();
   let result = longest_valid_parentheses_ascent(s);
   println!("result: {:?}", result);
   assert_eq!(result, 2);
   let s = ")()())".to_string();
   let result = longest_valid_parentheses_ascent(s);
   println!("result: {:?}", result);
   assert_eq!(result, 4);
   let s = "".to_string();
   let result = longest_valid_parentheses_ascent(s);
   println!("result: {:?}", result);
   assert_eq!(result, 0);
   let s = "(()))())(".to_string();
   let result = longest_valid_parentheses_ascent(s);
   println!("result: {:?}", result);
   // assert_eq!(result, 4);
   // let s = ")())()(()()))".to_string();
   // let result = longest_valid_parentheses_ascent(s);
   // println!("result: {:?}", result);
   // assert_eq!(result, 8);

   // let s = generate_random_paren_string();
   // save the string to test_data/longest_valid_parentheses_rand.txt
   // std::fs::write("test_data/longest_valid_parentheses_rand.txt", s.clone()).unwrap();

   // let s = std::fs::read_to_string("test_data/longest_valid_parentheses_rand.txt").unwrap();
   // // time the function
   // let start = std::time::Instant::now();
   // let result = longest_valid_parentheses(s.clone());
   // let duration = start.elapsed();
   // println!("result: {:?}, duration: {:?}", result, duration);
   // let start = std::time::Instant::now();
   // let result = longest_valid_parentheses_ascent(s.clone());
   // let duration = start.elapsed();
   // println!("result: {:?}, duration: {:?}", result, duration);
}
