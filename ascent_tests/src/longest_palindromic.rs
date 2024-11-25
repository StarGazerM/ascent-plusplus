
//  Longest Palindromic Substring
//  Given a string s, return the longest palindromic substring in s.
//  Example 1:
//  Input: s = "babad"
//  Output: "bab"
//  Note: "aba" is also a valid answer.
//  Example 2:
//  Input: s = "cbbd"
//  Output: "bb"

use std::{collections::{BTreeMap, BTreeSet}, mem};

use ascent::{ascent_run, Lattice};

use crate::se;

type Palindrome = (usize, usize);

fn longest_palindromic(s: &str) -> String {
    let mut new_palindromes: Vec<Palindrome> = vec![];

    for i in 0..s.len() {
        let p_odd = (i, i);
        let p_even = (i, i+1);
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
    fn size(&self) -> usize {
        self.1 - self.0
    }
}

impl PartialOrd for PalindromeR {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.size().cmp(&other.size()))
    }
}

impl Ord for PalindromeR {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.size().cmp(&other.size())
    }
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
    println!("{:?} : l_s: {:?}, l_e: {:?}",res.longest_palindromic.len(), l_s, l_e);
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

