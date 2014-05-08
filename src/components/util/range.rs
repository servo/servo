/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::cmp::{max, min};
use std::iter;
use std::fmt;
use std::num;
use std::num::{Bounded, Zero};

/// An index type to be used by a `Range`
pub trait RangeIndex<T>: Eq + Ord
                       + Clone
                       + Copy
                       + Zero
                       + TotalEq
                       + TotalOrd
                       + Add<Self, Self> 
                       + Sub<Self, Self>
                       + Neg<Self>
                       + fmt::Show {
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
macro_rules! range_index {
    ($(#[$attr:meta])* struct $Self:ident($T:ty)) => (
        #[deriving(Clone, Eq, Ord, TotalEq, TotalOrd, Show, Zero)]
        $(#[$attr])*
        pub struct $Self(pub $T);

        impl $Self {
            #[inline]
            pub fn to_uint(self) -> uint {
                self.get() as uint
            }
        }

        impl RangeIndex<$T> for $Self  {
            #[inline]
            fn new(x: $T) -> $Self {
                $Self(x)
            }

            #[inline]
            fn get(self) -> $T {
                match self { $Self(x) => x }
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

        impl Neg<$Self> for $Self {
            #[inline]
            fn neg(&self) -> $Self {
                $Self(-self.get())
            }
        }
    )
}

#[deriving(Show)]
pub enum RangeRelation<I> {
    OverlapsBegin(/* overlap */ I),
    OverlapsEnd(/* overlap */ I),
    ContainedBy,
    Contains,
    Coincides,
    EntirelyBefore,
    EntirelyAfter
}

/// A range of indices
#[deriving(Clone)]
pub struct Range<I> {
    off: I,
    len: I,
}

impl<T: Int, I: fmt::Show + RangeIndex<T>> fmt::Show for Range<I> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f.buf, "[{} .. {})", self.begin(), self.end())
    }
}

/// An iterator over each index in a range
pub struct EachIndex<T, I> {
    it: iter::Range<T>,
}

pub fn each_index<T: Int, I: RangeIndex<T>>(start: I, stop: I) -> EachIndex<T, I> {
    EachIndex {
        it: iter::range(start.get(), stop.get())
    }
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

impl<T: Int, I: RangeIndex<T>> Range<I> {
    #[inline]
    pub fn new(off: I, len: I) -> Range<I> {
        Range { off: off, len: len }
    }

    #[inline]
    pub fn empty() -> Range<I> {
        Range::new(num::zero(), num::zero())
    }

    #[inline]
    pub fn begin(&self) -> I { self.off  }
    #[inline]
    pub fn length(&self) -> I { self.len }
    #[inline]
    pub fn end(&self) -> I { self.off + self.len }

    #[inline]
    pub fn each_index(&self) -> EachIndex<T, I> {
        each_index(self.off, self.off + self.len)
    }

    #[inline]
    pub fn contains(&self, i: I) -> bool {
        i >= self.begin() && i < self.end()
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
                            let max: T = Bounded::max_value();
                            let val: I = RangeIndex::new(max);
                            val
                        });
                false
            },
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len.is_zero()
    }

    #[inline]
    pub fn shift_by(&mut self, i: I) {
        self.off = self.off + i;
    }

    #[inline]
    pub fn extend_by(&mut self, i: I) {
        self.len = self.len + i;
    }

    #[inline]
    pub fn extend_to(&mut self, i: I) {
        self.len = i - self.off;
    }

    #[inline]
    pub fn adjust_by(&mut self, off_i: I, len_i: I) {
        self.off = self.off + off_i;
        self.len = self.len + len_i;
    }

    #[inline]
    pub fn reset(&mut self, off_i: I, len_i: I) {
        self.off = off_i;
        self.len = len_i;
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

    /// Computes the relationship between two ranges (`self` and `other`),
    /// from the point of view of `self`. So, 'EntirelyBefore' means
    /// that the `self` range is entirely before `other` range.
    #[inline]
    pub fn relation_to_range(&self, other: &Range<I>) -> RangeRelation<I> {
        if other.begin() > self.end() {
            return EntirelyBefore;
        }
        if self.begin() > other.end() {
            return EntirelyAfter;
        }
        if self.begin() == other.begin() && self.end() == other.end() {
            return Coincides;
        }
        if self.begin() <= other.begin() && self.end() >= other.end() {
            return Contains;
        }
        if self.begin() >= other.begin() && self.end() <= other.end() {
            return ContainedBy;
        }
        if self.begin() < other.begin() && self.end() < other.end() {
            let overlap = self.end() - other.begin();
            return OverlapsBegin(overlap);
        }
        if self.begin() > other.begin() && self.end() > other.end() {
            let overlap = other.end() - self.begin();
            return OverlapsEnd(overlap);
        }
        fail!("relation_to_range(): didn't classify self={}, other={}",
              self, other);
    }

    #[inline]
    pub fn repair_after_coalesced_range(&mut self, other: &Range<I>) {
        let relation = self.relation_to_range(other);
        debug!("repair_after_coalesced_range: possibly repairing range {}", *self);
        debug!("repair_after_coalesced_range: relation of original range and coalesced range {}: {}",
               *other, relation);
        let _1: I = RangeIndex::new(num::one::<T>());
        match relation {
            EntirelyBefore => {},
            EntirelyAfter => { self.shift_by(-other.length()); },
            Coincides | ContainedBy => { self.reset(other.begin(), _1); },
            Contains => { self.extend_by(-other.length()); },
            OverlapsBegin(overlap) => { self.extend_by(_1 - overlap); },
            OverlapsEnd(overlap) => {
                let len = self.length() - overlap + _1;
                self.reset(other.begin(), len);
            }
        };
        debug!("repair_after_coalesced_range: new range: ---- {}", *self);
    }
}
