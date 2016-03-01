/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::marker::PhantomData;
use std::ops;
use super::smallvec::VecLike;

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
