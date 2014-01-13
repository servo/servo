/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::cmp::{Ord, Eq};

pub trait BinarySearchMethods<'a, T: Ord + Eq> {
    fn binary_search(&self, key: &T) -> Option<&'a T>;
    fn binary_search_index(&self, key: &T) -> Option<uint>;
}

impl<'a, T: Ord + Eq> BinarySearchMethods<'a, T> for &'a [T] {
    fn binary_search(&self, key: &T) -> Option<&'a T> {
        self.binary_search_index(key).map(|i| &self[i])
    }

    fn binary_search_index(&self, key: &T) -> Option<uint> {
        if self.len() == 0 {
            return None;
        }

        let mut low : int = 0;
        let mut high : int = (self.len() as int) - 1;

        while (low <= high) {
            // http://googleresearch.blogspot.com/2006/06/extra-extra-read-all-about-it-nearly.html
            let mid : int = (((low as uint) + (high as uint)) >> 1) as int;
            let midv = &self[mid];

            if (midv < key) {
                low = mid + 1;
            } else if (midv > key) {
                high = mid - 1;
            } else {
                return Some(mid as uint);
            }
        }
        return None;
    }
}

#[cfg(test)]
fn test_find_all_elems<T: Eq + Ord>(arr: &[T]) {
    let mut i = 0;
    while i < arr.len() {
        assert!(test_match(&arr[i], arr.binary_search(&arr[i])));
        i += 1;
    }
}

#[cfg(test)]
fn test_miss_all_elems<T: Eq + Ord>(arr: &[T], misses: &[T]) {
    let mut i = 0;
    while i < misses.len() {
        let res = arr.binary_search(&misses[i]);
        debug!("{:?} == {:?} ?", misses[i], res);
        assert!(!test_match(&misses[i], arr.binary_search(&misses[i])));
        i += 1;
    }
}

#[cfg(test)]
fn test_match<T: Eq>(b: &T, a: Option<&T>) -> bool {
    match a {
        None => false,
        Some(t) => t == b
    }
} 

pub fn zip_copies<A: Clone, B: Clone>(avec: &[A], bvec: &[B]) -> ~[(A,B)] {
    avec.iter().map(|x| x.clone())
        .zip(bvec.iter().map(|x| x.clone()))
        .collect()
}

#[test]
fn should_find_all_elements() {
    let arr_odd = [1, 2, 4, 6, 7, 8, 9];
    let arr_even = [1, 2, 5, 6, 7, 8, 9, 42];
    let arr_double = [1, 1, 2, 2, 6, 8, 22];
    let arr_one = [234986325];
    let arr_two = [3044, 8393];
    let arr_three = [12, 23, 34];

    test_find_all_elems(arr_odd);
    test_find_all_elems(arr_even);
    test_find_all_elems(arr_double);
    test_find_all_elems(arr_one);
    test_find_all_elems(arr_two);
    test_find_all_elems(arr_three);
}

#[test]
fn should_not_find_missing_elements() {
    let arr_odd = [1, 2, 4, 6, 7, 8, 9];
    let arr_even = [1, 2, 5, 6, 7, 8, 9, 42];
    let arr_double = [1, 1, 2, 2, 6, 8, 22];
    let arr_one = [234986325];
    let arr_two = [3044, 8393];
    let arr_three = [12, 23, 34];

    test_miss_all_elems(arr_odd, [-22, 0, 3, 5, 34938, 10, 11, 12]);
    test_miss_all_elems(arr_even, [-1, 0, 3, 34938, 10, 11, 12]);
    test_miss_all_elems(arr_double, [-1, 0, 3, 4, 34938, 10, 11, 12, 234, 234, 33]);
    test_miss_all_elems(arr_one, [-1, 0, 3, 34938, 10, 11, 12, 234, 234, 33]);
    test_miss_all_elems(arr_two, [-1, 0, 3, 34938, 10, 11, 12, 234, 234, 33]);
    test_miss_all_elems(arr_three, [-2, 0, 1, 2, 3, 34938, 10, 11, 234, 234, 33]);
}
