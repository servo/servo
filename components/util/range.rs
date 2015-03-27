/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::cmp::{max, min};
use std::iter;
use std::fmt;
use std::num;
use std::num::Int;
use std::marker::PhantomData;

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

        impl ::std::num::Int for $Self_ {
            fn zero() -> $Self_ { $Self_(0) }
            fn one() -> $Self_ { $Self_(1) }
            fn min_value() -> $Self_ { $Self_(::std::num::Int::min_value()) }
            fn max_value() -> $Self_ { $Self_(::std::num::Int::max_value()) }
            fn count_ones(self) -> u32 { self.get().count_ones() }
            fn leading_zeros(self) -> u32 { self.get().leading_zeros() }
            fn trailing_zeros(self) -> u32 { self.get().trailing_zeros() }
            fn rotate_left(self, n: u32) -> $Self_ { $Self_(self.get().rotate_left(n)) }
            fn rotate_right(self, n: u32) -> $Self_ { $Self_(self.get().rotate_right(n)) }
            fn swap_bytes(self) -> $Self_ { $Self_(self.get().swap_bytes()) }
            fn checked_add(self, other: $Self_) -> Option<$Self_> {
                self.get().checked_add(other.get()).map($Self_)
            }
            fn checked_sub(self, other: $Self_) -> Option<$Self_> {
                self.get().checked_sub(other.get()).map($Self_)
            }
            fn checked_mul(self, other: $Self_) -> Option<$Self_> {
                self.get().checked_mul(other.get()).map($Self_)
            }
            fn checked_div(self, other: $Self_) -> Option<$Self_> {
                self.get().checked_div(other.get()).map($Self_)
            }
        }

        impl Add<$Self_> for $Self_ {
            type Output = $Self_;

            #[inline]
            fn add(self, other: $Self_) -> $Self_ {
                $Self_(self.get() + other.get())
            }
        }

        impl Sub<$Self_> for $Self_ {
            type Output = $Self_;

            #[inline]
            fn sub(self, other: $Self_) -> $Self_ {
                $Self_(self.get() - other.get())
            }
        }

        impl Mul<$Self_> for $Self_ {
            type Output = $Self_;

            #[inline]
            fn mul(self, other: $Self_) -> $Self_ {
                $Self_(self.get() * other.get())
            }
        }

        impl Neg for $Self_ {
            type Output = $Self_;

            #[inline]
            fn neg(self) -> $Self_ {
                $Self_(-self.get())
            }
        }

        impl ToPrimitive for $Self_ {
            fn to_i64(&self) -> Option<i64> {
                Some(self.get() as i64)
            }

            fn to_u64(&self) -> Option<u64> {
                Some(self.get() as u64)
            }
        }

        impl ::std::num::NumCast for $Self_ {
            fn from<T: ToPrimitive>(n: T) -> Option<$Self_> {
                n.to_isize().map($Self_)
            }
        }

        impl Div<$Self_> for $Self_ {
            type Output = $Self_;
            fn div(self, other: $Self_) -> $Self_ {
                $Self_(self.get() / other.get())
            }
        }

        impl Rem<$Self_> for $Self_ {
            type Output = $Self_;
            fn rem(self, other: $Self_) -> $Self_ {
                $Self_(self.get() % other.get())
            }
        }

        impl Not for $Self_ {
            type Output = $Self_;
            fn not(self) -> $Self_ {
                $Self_(!self.get())
            }
        }

        impl BitAnd<$Self_> for $Self_ {
            type Output = $Self_;
            fn bitand(self, other: $Self_) -> $Self_ {
                $Self_(self.get() & other.get())
            }
        }

        impl BitOr<$Self_> for $Self_ {
            type Output = $Self_;
            fn bitor(self, other: $Self_) -> $Self_ {
                $Self_(self.get() | other.get())
            }
        }

        impl BitXor<$Self_> for $Self_ {
            type Output = $Self_;
            fn bitxor(self, other: $Self_) -> $Self_ {
                $Self_(self.get() ^ other.get())
            }
        }

        impl Shl<usize> for $Self_ {
            type Output = $Self_;
            fn shl(self, n: usize) -> $Self_ {
                $Self_(self.get() << n)
            }
        }

        impl Shr<usize> for $Self_ {
            type Output = $Self_;
            fn shr(self, n: usize) -> $Self_ {
                $Self_(self.get() >> n)
            }
        }

        impl ::std::num::wrapping::WrappingOps for $Self_ {
            fn wrapping_add(self, rhs: $Self_) -> $Self_ {
                $Self_(self.get().wrapping_add(rhs.get()))
            }
            fn wrapping_sub(self, rhs: $Self_) -> $Self_ {
                $Self_(self.get().wrapping_sub(rhs.get()))
            }
            fn wrapping_mul(self, rhs: $Self_) -> $Self_ {
                $Self_(self.get().wrapping_mul(rhs.get()))
            }
        }

        impl ::std::num::wrapping::OverflowingOps for $Self_ {
            fn overflowing_add(self, rhs: $Self_) -> ($Self_, bool) {
                let (x, b) = self.get().overflowing_add(rhs.get());
                ($Self_(x), b)
            }
            fn overflowing_sub(self, rhs: $Self_) -> ($Self_, bool) {
                let (x, b) = self.get().overflowing_sub(rhs.get());
                ($Self_(x), b)
            }
            fn overflowing_mul(self, rhs: $Self_) -> ($Self_, bool) {
                let (x, b) = self.get().overflowing_mul(rhs.get());
                ($Self_(x), b)
            }
        }
    )
}

/// A range of indices
#[derive(Clone, RustcEncodable, Copy)]
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
    it: iter::Range<T>,
    phantom: PhantomData<I>,
}

pub fn each_index<T: Int, I: RangeIndex<Index=T>>(start: I, stop: I) -> EachIndex<T, I> {
    EachIndex { it: iter::range(start.get(), stop.get()), phantom: PhantomData }
}

impl<T: Int, I: RangeIndex<Index=T>> Iterator for EachIndex<T, I> {
    type Item = I;

    #[inline]
    fn next(&mut self) -> Option<I> {
        self.it.next().map(|i| RangeIndex::new(i))
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

    #[inline]
    pub fn is_valid_for_string(&self, s: &str) -> bool {
        let s_len = s.len();
        match num::cast::<usize, T>(s_len) {
            Some(len) => {
                let len = RangeIndex::new(len);
                self.begin() < len
                && self.end() <= len
                && self.length() <= len
            },
            None => {
                debug!("Range<T>::is_valid_for_string: string length \
                        (len={:?}) is longer than the max value for the range \
                        index (max={:?})", s_len,
                        {
                            let max: T = Int::max_value();
                            let val: I = RangeIndex::new(max);
                            val
                        });
                false
            },
        }
    }
}
