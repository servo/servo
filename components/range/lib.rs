/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(step_trait)]

#![deny(unsafe_code)]

extern crate heapsize;
#[macro_use] extern crate heapsize_derive;
extern crate num_traits;
#[macro_use] extern crate serde;

use std::cmp::{self, max, min};
use std::fmt;
use std::iter;
use std::marker::PhantomData;
use std::ops;

pub trait Int:
    Copy
    + ops::Add<Self, Output=Self>
    + ops::Sub<Self, Output=Self>
    + cmp::Ord
{
    fn zero() -> Self;
    fn one() -> Self;
    fn max_value() -> Self;
    fn from_usize(n: usize) -> Option<Self>;
}
impl Int for isize {
    #[inline]
    fn zero() -> isize { 0 }
    #[inline]
    fn one() -> isize { 1 }
    #[inline]
    fn max_value() -> isize { ::std::isize::MAX }
    #[inline]
    fn from_usize(n: usize) -> Option<isize> { num_traits::NumCast::from(n) }
}
impl Int for usize {
    #[inline]
    fn zero() -> usize { 0 }
    #[inline]
    fn one() -> usize { 1 }
    #[inline]
    fn max_value() -> usize { ::std::usize::MAX }
    #[inline]
    fn from_usize(n: usize) -> Option<usize> { Some(n) }
}

/// An index type to be used by a `Range`
pub trait RangeIndex: Int + fmt::Debug {
    type Index;
    fn new(x: Self::Index) -> Self;
    fn get(self) -> Self::Index;
}

impl RangeIndex for isize {
    type Index = isize;
    #[inline]
    fn new(x: isize) -> isize { x }

    #[inline]
    fn get(self) -> isize { self }
}

impl RangeIndex for usize {
    type Index = usize;
    #[inline]
    fn new(x: usize) -> usize { x }

    #[inline]
    fn get(self) -> usize { self }
}

/// Implements a range index type with operator overloads
#[macro_export]
macro_rules! int_range_index {
    ($(#[$attr:meta])* struct $Self_:ident($T:ty)) => (
        #[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Copy)]
        $(#[$attr])*
        pub struct $Self_(pub $T);

        impl $Self_ {
            #[inline]
            pub fn to_usize(self) -> usize {
                self.get() as usize
            }
        }

        impl RangeIndex for $Self_ {
            type Index = $T;
            #[inline]
            fn new(x: $T) -> $Self_ {
                $Self_(x)
            }

            #[inline]
            fn get(self) -> $T {
                match self { $Self_(x) => x }
            }
        }

        impl $crate::Int for $Self_ {
            #[inline]
            fn zero() -> $Self_ { $Self_($crate::Int::zero()) }
            #[inline]
            fn one() -> $Self_ { $Self_($crate::Int::one()) }
            #[inline]
            fn max_value() -> $Self_ { $Self_($crate::Int::max_value()) }
            #[inline]
            fn from_usize(n: usize) -> Option<$Self_> { $crate::Int::from_usize(n).map($Self_) }
        }

        impl ::std::ops::Add<$Self_> for $Self_ {
            type Output = $Self_;

            #[inline]
            fn add(self, other: $Self_) -> $Self_ {
                $Self_(self.get() + other.get())
            }
        }

        impl ::std::ops::Sub<$Self_> for $Self_ {
            type Output = $Self_;

            #[inline]
            fn sub(self, other: $Self_) -> $Self_ {
                $Self_(self.get() - other.get())
            }
        }

        impl ::std::ops::Neg for $Self_ {
            type Output = $Self_;

            #[inline]
            fn neg(self) -> $Self_ {
                $Self_(-self.get())
            }
        }
    )
}

/// A range of indices
#[derive(Clone, Copy, Deserialize, HeapSizeOf, Serialize)]
pub struct Range<I> {
    begin: I,
    length: I,
}

impl<I: RangeIndex> fmt::Debug for Range<I> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{:?} .. {:?})", self.begin(), self.end())
    }
}

/// An iterator over each index in a range
pub struct EachIndex<T, I> {
    it: ops::Range<T>,
    phantom: PhantomData<I>,
}

pub fn each_index<T: Int, I: RangeIndex<Index=T>>(start: I, stop: I) -> EachIndex<T, I> {
    EachIndex { it: start.get()..stop.get(), phantom: PhantomData }
}

impl<T: Int, I: RangeIndex<Index=T>> Iterator for EachIndex<T, I>
where T: Int + iter::Step, for<'a> &'a T: ops::Add<&'a T, Output = T> {
    type Item = I;

    #[inline]
    fn next(&mut self) -> Option<I> {
        self.it.next().map(RangeIndex::new)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.it.size_hint()
    }
}

impl<I: RangeIndex> Range<I> {
    /// Create a new range from beginning and length offsets. This could be
    /// denoted as `[begin, begin + length)`.
    ///
    /// ~~~ignore
    ///    |-- begin ->|-- length ->|
    ///    |           |            |
    /// <- o - - - - - +============+ - - - ->
    /// ~~~
    #[inline]
    pub fn new(begin: I, length: I) -> Range<I> {
        Range { begin: begin, length: length }
    }

    #[inline]
    pub fn empty() -> Range<I> {
        Range::new(Int::zero(), Int::zero())
    }

    /// The index offset to the beginning of the range.
    ///
    /// ~~~ignore
    ///    |-- begin ->|
    ///    |           |
    /// <- o - - - - - +============+ - - - ->
    /// ~~~
    #[inline]
    pub fn begin(&self) -> I { self.begin  }

    /// The index offset from the beginning to the end of the range.
    ///
    /// ~~~ignore
    ///                |-- length ->|
    ///                |            |
    /// <- o - - - - - +============+ - - - ->
    /// ~~~
    #[inline]
    pub fn length(&self) -> I { self.length }

    /// The index offset to the end of the range.
    ///
    /// ~~~ignore
    ///    |--------- end --------->|
    ///    |                        |
    /// <- o - - - - - +============+ - - - ->
    /// ~~~
    #[inline]
    pub fn end(&self) -> I { self.begin + self.length }

    /// `true` if the index is between the beginning and the end of the range.
    ///
    /// ~~~ignore
    ///        false        true      false
    ///          |           |          |
    /// <- o - - + - - +=====+======+ - + - ->
    /// ~~~
    #[inline]
    pub fn contains(&self, i: I) -> bool {
        i >= self.begin() && i < self.end()
    }

    /// `true` if the offset from the beginning to the end of the range is zero.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.length() == Int::zero()
    }

    /// Shift the entire range by the supplied index delta.
    ///
    /// ~~~ignore
    ///                     |-- delta ->|
    ///                     |           |
    /// <- o - +============+ - - - - - | - - - ->
    ///                                 |
    /// <- o - - - - - - - +============+ - - - ->
    /// ~~~
    #[inline]
    pub fn shift_by(&mut self, delta: I) {
        self.begin = self.begin + delta;
    }

    /// Extend the end of the range by the supplied index delta.
    ///
    /// ~~~ignore
    ///                     |-- delta ->|
    ///                     |           |
    /// <- o - - - - - +====+ - - - - - | - - - ->
    ///                                 |
    /// <- o - - - - - +================+ - - - ->
    /// ~~~
    #[inline]
    pub fn extend_by(&mut self, delta: I) {
        self.length = self.length + delta;
    }

    /// Move the end of the range to the target index.
    ///
    /// ~~~ignore
    ///                               target
    ///                                 |
    /// <- o - - - - - +====+ - - - - - | - - - ->
    ///                                 |
    /// <- o - - - - - +================+ - - - ->
    /// ~~~
    #[inline]
    pub fn extend_to(&mut self, target: I) {
        self.length = target - self.begin;
    }

    /// Adjust the beginning offset and the length by the supplied deltas.
    #[inline]
    pub fn adjust_by(&mut self, begin_delta: I, length_delta: I) {
        self.begin = self.begin + begin_delta;
        self.length = self.length + length_delta;
    }

    /// Set the begin and length values.
    #[inline]
    pub fn reset(&mut self, begin: I, length: I) {
        self.begin = begin;
        self.length = length;
    }

    #[inline]
    pub fn intersect(&self, other: &Range<I>) -> Range<I> {
        let begin = max(self.begin(), other.begin());
        let end = min(self.end(), other.end());

        if end < begin {
            Range::empty()
        } else {
            Range::new(begin, end - begin)
        }
    }
}

/// Methods for `Range`s with indices based on integer values
impl<T: Int, I: RangeIndex<Index=T>> Range<I> {
    /// Returns an iterater that increments over `[begin, end)`.
    #[inline]
    pub fn each_index(&self) -> EachIndex<T, I> {
        each_index(self.begin(), self.end())
    }
}
