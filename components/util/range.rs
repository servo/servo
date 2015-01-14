/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::cmp::{max, min};
use std::iter;
use std::fmt;
use std::num;
use std::num::Int;

/// An index type to be used by a `Range`
pub trait RangeIndex<T>: Int + fmt::Show {
    fn new(x: T) -> Self;
    fn get(self) -> T;
}

impl RangeIndex<int> for int {
    #[inline]
    fn new(x: int) -> int { x }

    #[inline]
    fn get(self) -> int { self }
}

/// Implements a range index type with operator overloads
#[macro_export]
macro_rules! int_range_index {
    ($(#[$attr:meta])* struct $Self:ident($T:ty)) => (
        #[deriving(Clone, PartialEq, PartialOrd, Eq, Ord, Show, Copy)]
        $(#[$attr])*
        pub struct $Self(pub $T);

        impl $Self {
            #[inline]
            pub fn to_uint(self) -> uint {
                self.get() as uint
            }
        }

        impl RangeIndex<$T> for $Self {
            #[inline]
            fn new(x: $T) -> $Self {
                $Self(x)
            }

            #[inline]
            fn get(self) -> $T {
                match self { $Self(x) => x }
            }
        }

        impl ::std::num::Int for $Self {
            fn zero() -> $Self { $Self(0) }
            fn one() -> $Self { $Self(1) }
            fn min_value() -> $Self { $Self(::std::num::Int::min_value()) }
            fn max_value() -> $Self { $Self(::std::num::Int::max_value()) }
            fn count_ones(self) -> uint { self.get().count_ones() }
            fn leading_zeros(self) -> uint { self.get().leading_zeros() }
            fn trailing_zeros(self) -> uint { self.get().trailing_zeros() }
            fn rotate_left(self, n: uint) -> $Self { $Self(self.get().rotate_left(n)) }
            fn rotate_right(self, n: uint) -> $Self { $Self(self.get().rotate_right(n)) }
            fn swap_bytes(self) -> $Self { $Self(self.get().swap_bytes()) }
            fn checked_add(self, other: $Self) -> Option<$Self> {
                self.get().checked_add(other.get()).map($Self)
            }
            fn checked_sub(self, other: $Self) -> Option<$Self> {
                self.get().checked_sub(other.get()).map($Self)
            }
            fn checked_mul(self, other: $Self) -> Option<$Self> {
                self.get().checked_mul(other.get()).map($Self)
            }
            fn checked_div(self, other: $Self) -> Option<$Self> {
                self.get().checked_div(other.get()).map($Self)
            }
        }

        impl Add<$Self, $Self> for $Self {
            #[inline]
            fn add(&self, other: &$Self) -> $Self {
                $Self(self.get() + other.get())
            }
        }

        impl Sub<$Self, $Self> for $Self {
            #[inline]
            fn sub(&self, other: &$Self) -> $Self {
                $Self(self.get() - other.get())
            }
        }

        impl Mul<$Self, $Self> for $Self {
            #[inline]
            fn mul(&self, other: &$Self) -> $Self {
                $Self(self.get() * other.get())
            }
        }

        impl Neg<$Self> for $Self {
            #[inline]
            fn neg(&self) -> $Self {
                $Self(-self.get())
            }
        }

        impl ::std::num::One for $Self {
            fn one() -> $Self {
                $Self(1)
            }
        }

        impl ToPrimitive for $Self {
            fn to_i64(&self) -> Option<i64> {
                Some(self.get() as i64)
            }

            fn to_u64(&self) -> Option<u64> {
                Some(self.get() as u64)
            }
        }

        impl ::std::num::NumCast for $Self {
            fn from<T: ToPrimitive>(n: T) -> Option<$Self> {
                n.to_int().map($Self)
            }
        }

        impl Div<$Self, $Self> for $Self {
            fn div(&self, other: &$Self) -> $Self {
                $Self(self.get() / other.get())
            }
        }

        impl Rem<$Self, $Self> for $Self {
            fn rem(&self, other: &$Self) -> $Self {
                $Self(self.get() % other.get())
            }
        }

        impl Not<$Self> for $Self {
            fn not(&self) -> $Self {
                $Self(!self.get())
            }
        }

        impl BitAnd<$Self, $Self> for $Self {
            fn bitand(&self, other: &$Self) -> $Self {
                $Self(self.get() & other.get())
            }
        }

        impl BitOr<$Self, $Self> for $Self {
            fn bitor(&self, other: &$Self) -> $Self {
                $Self(self.get() | other.get())
            }
        }

        impl BitXor<$Self, $Self> for $Self {
            fn bitxor(&self, other: &$Self) -> $Self {
                $Self(self.get() ^ other.get())
            }
        }

        impl Shl<uint, $Self> for $Self {
            fn shl(&self, n: &uint) -> $Self {
                $Self(self.get() << *n)
            }
        }

        impl Shr<uint, $Self> for $Self {
            fn shr(&self, n: &uint) -> $Self {
                $Self(self.get() >> *n)
            }
        }
    )
}

/// A range of indices
#[deriving(Clone, Encodable, Copy)]
pub struct Range<I> {
    begin: I,
    length: I,
}

impl<I: RangeIndex<T>, T> fmt::Show for Range<I> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{} .. {})", self.begin(), self.end())
    }
}

/// An iterator over each index in a range
pub struct EachIndex<T, I> {
    it: iter::Range<T>,
}

pub fn each_index<T: Int, I: RangeIndex<T>>(start: I, stop: I) -> EachIndex<T, I> {
    EachIndex { it: iter::range(start.get(), stop.get()) }
}

impl<T: Int, I: RangeIndex<T>> Iterator<I> for EachIndex<T, I> {
    #[inline]
    fn next(&mut self) -> Option<I> {
        self.it.next().map(|i| RangeIndex::new(i))
    }

    #[inline]
    fn size_hint(&self) -> (uint, Option<uint>) {
        self.it.size_hint()
    }
}

impl<I: RangeIndex<T>, T> Range<I> {
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
impl<T: Int, I: RangeIndex<T>> Range<I> {
    /// Returns an iterater that increments over `[begin, end)`.
    #[inline]
    pub fn each_index(&self) -> EachIndex<T, I> {
        each_index(self.begin(), self.end())
    }

    #[inline]
    pub fn is_valid_for_string(&self, s: &str) -> bool {
        let s_len = s.len();
        match num::cast::<uint, T>(s_len) {
            Some(len) => {
                let len = RangeIndex::new(len);
                self.begin() < len
                && self.end() <= len
                && self.length() <= len
            },
            None => {
                debug!("Range<T>::is_valid_for_string: string length (len={}) is longer than the \
                        max value for the range index (max={})", s_len,
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
