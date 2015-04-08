/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::cmp::{PartialOrd, PartialEq, Ordering};
use std::iter::range_step;

/// FIXME(pcwalton): Workaround for lack of unboxed closures. This is called in
/// performance-critical code, so a closure is insufficient.
pub trait Comparator<K,T> {
    fn compare(&self, key: &K, value: &T) -> Ordering;
}

pub trait BinarySearchMethods<'a, T: Ord + PartialOrd + PartialEq> {
    fn binary_search_(&self, key: &T) -> Option<&'a T>;
    fn binary_search_index(&self, key: &T) -> Option<usize>;
}

pub trait FullBinarySearchMethods<T> {
    fn binary_search_index_by<K,C:Comparator<K,T>>(&self, key: &K, cmp: C) -> Option<usize>;
}

impl<'a, T: Ord + PartialOrd + PartialEq> BinarySearchMethods<'a, T> for &'a [T] {
    fn binary_search_(&self, key: &T) -> Option<&'a T> {
        self.binary_search_index(key).map(|i| &self[i])
    }

    fn binary_search_index(&self, key: &T) -> Option<usize> {
        self.binary_search_index_by(key, DefaultComparator)
    }
}

impl<'a, T> FullBinarySearchMethods<T> for &'a [T] {
    fn binary_search_index_by<K,C:Comparator<K,T>>(&self, key: &K, cmp: C) -> Option<usize> {
        if self.len() == 0 {
            return None;
        }

        let mut low : isize = 0;
        let mut high : isize = (self.len() as isize) - 1;

        while low <= high {
            // http://googleresearch.blogspot.com/2006/06/extra-extra-read-all-about-it-nearly.html
            let mid = ((low as usize) + (high as usize)) >> 1;
            let midv = &self[mid];

            match cmp.compare(key, midv) {
                Ordering::Greater => low = (mid as isize) + 1,
                Ordering::Less => high = (mid as isize) - 1,
                Ordering::Equal => return Some(mid),
            }
        }
        return None;
    }
}

struct DefaultComparator;

impl<T:PartialEq + PartialOrd + Ord> Comparator<T,T> for DefaultComparator {
    fn compare(&self, key: &T, value: &T) -> Ordering {
        (*key).cmp(value)
    }
}


// TODO(pcwalton): Speed up with SIMD, or better yet, find some way to not do this.
pub fn byte_swap(data: &mut [u8]) {
    let length = data.len();
    for i in range_step(0, length, 4) {
        let r = data[i + 2];
        data[i + 2] = data[i + 0];
        data[i + 0] = r;
    }
}
