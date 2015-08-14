/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::smallvec::VecLike;

use std::cmp::{PartialOrd, PartialEq, Ordering};
use std::marker::PhantomData;
use std::ops;

/// FIXME(pcwalton): Workaround for lack of unboxed closures. This is called in
/// performance-critical code, so a closure is insufficient.
pub trait Comparator<K,T> {
    fn compare(&self, key: &K, value: &T) -> Ordering;
}

pub trait BinarySearchMethods<T: Ord + PartialOrd + PartialEq> {
    fn binary_search_(&self, key: &T) -> Option<&T>;
    fn binary_search_index(&self, key: &T) -> Option<usize>;
}

pub trait FullBinarySearchMethods<T> {
    fn binary_search_index_by<K,C:Comparator<K,T>>(&self, key: &K, cmp: C) -> Option<usize>;
}

impl<T: Ord + PartialOrd + PartialEq> BinarySearchMethods<T> for [T] {
    fn binary_search_(&self, key: &T) -> Option<&T> {
        self.binary_search_index(key).map(|i| &self[i])
    }

    fn binary_search_index(&self, key: &T) -> Option<usize> {
        self.binary_search_index_by(key, DefaultComparator)
    }
}

impl<T> FullBinarySearchMethods<T> for [T] {
    fn binary_search_index_by<K,C:Comparator<K,T>>(&self, key: &K, cmp: C) -> Option<usize> {
        if self.is_empty() {
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
    for i in (0..length).step_by(4) {
        let r = data[i + 2];
        data[i + 2] = data[i + 0];
        data[i + 0] = r;
    }
}

/// A `VecLike` that only tracks whether or not something was ever pushed to it.
pub struct ForgetfulSink<T> {
    empty: bool,
    _data: PhantomData<T>,
}

impl<T> ForgetfulSink<T> {
    pub fn new() -> ForgetfulSink<T> {
        ForgetfulSink {
            empty: true,
            _data: PhantomData,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.empty
    }
}

impl<T> ops::Deref for ForgetfulSink<T> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        unreachable!()
    }
}

impl<T> ops::DerefMut for ForgetfulSink<T> {
    fn deref_mut(&mut self) -> &mut [T] {
        unreachable!()
    }
}

macro_rules! impl_index {
    ($index_type: ty, $output_type: ty) => {
        impl<T> ops::Index<$index_type> for ForgetfulSink<T> {
            type Output = $output_type;
            fn index(&self, _index: $index_type) -> &$output_type {
                unreachable!()
            }
        }

        impl<T> ops::IndexMut<$index_type> for ForgetfulSink<T> {
            fn index_mut(&mut self, _index: $index_type) -> &mut $output_type {
                unreachable!()
            }
        }
    }
}

impl_index!(usize, T);
impl_index!(ops::Range<usize>, [T]);
impl_index!(ops::RangeFrom<usize>, [T]);
impl_index!(ops::RangeTo<usize>, [T]);
impl_index!(ops::RangeFull, [T]);

impl<T> VecLike<T> for ForgetfulSink<T> {
    #[inline]
    fn len(&self) -> usize {
        unreachable!()
    }

    #[inline]
    fn push(&mut self, _value: T) {
        self.empty = false;
    }
}
