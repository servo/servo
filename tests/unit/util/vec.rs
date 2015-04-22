/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::fmt::Debug;
use util::vec::BinarySearchMethods;

#[cfg(test)]
fn test_find_all_elems<T: PartialEq + PartialOrd + Eq + Ord>(arr: &[T]) {
    let mut i = 0;
    while i < arr.len() {
        assert!(test_match(&arr[i], arr.binary_search_(&arr[i])));
        i += 1;
    }
}

#[cfg(test)]
fn test_miss_all_elems<T: PartialEq + PartialOrd + Eq + Ord + Debug>(arr: &[T], misses: &[T]) {
    let mut i = 0;
    while i < misses.len() {
        let res = arr.binary_search_(&misses[i]);
        println!("{:?} == {:?} ?", misses[i], res);
        assert!(!test_match(&misses[i], arr.binary_search_(&misses[i])));
        i += 1;
    }
}

#[cfg(test)]
fn test_match<T: PartialEq>(b: &T, a: Option<&T>) -> bool {
    match a {
        None => false,
        Some(t) => t == b
    }
}

#[test]
fn should_find_all_elements() {
    let arr_odd = [1_i32, 2, 4, 6, 7, 8, 9];
    let arr_even = [1_i32, 2, 5, 6, 7, 8, 9, 42];
    let arr_double = [1_i32, 1, 2, 2, 6, 8, 22];
    let arr_one = [234986325_i32];
    let arr_two = [3044_i32, 8393];
    let arr_three = [12_i32, 23, 34];

    test_find_all_elems(&arr_odd);
    test_find_all_elems(&arr_even);
    test_find_all_elems(&arr_double);
    test_find_all_elems(&arr_one);
    test_find_all_elems(&arr_two);
    test_find_all_elems(&arr_three);
}

#[test]
fn should_not_find_missing_elements() {
    let arr_odd = [1_i32, 2, 4, 6, 7, 8, 9];
    let arr_even = [1_i32, 2, 5, 6, 7, 8, 9, 42];
    let arr_double = [1_i32, 1, 2, 2, 6, 8, 22];
    let arr_one = [234986325_i32];
    let arr_two = [3044_i32, 8393];
    let arr_three = [12_i32, 23, 34];

    test_miss_all_elems(&arr_odd, &[-22, 0, 3, 5, 34938, 10, 11, 12]);
    test_miss_all_elems(&arr_even, &[-1, 0, 3, 34938, 10, 11, 12]);
    test_miss_all_elems(&arr_double, &[-1, 0, 3, 4, 34938, 10, 11, 12, 234, 234, 33]);
    test_miss_all_elems(&arr_one, &[-1, 0, 3, 34938, 10, 11, 12, 234, 234, 33]);
    test_miss_all_elems(&arr_two, &[-1, 0, 3, 34938, 10, 11, 12, 234, 234, 33]);
    test_miss_all_elems(&arr_three, &[-2, 0, 1, 2, 3, 34938, 10, 11, 234, 234, 33]);
}
