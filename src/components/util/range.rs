/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::cmp::{max, min};
use std::iter;
use std::fmt;
use std::num;
use std::num::Bounded;

#[deriving(Show)]
pub enum RangeRelation<T> {
    OverlapsBegin(/* overlap */ T),
    OverlapsEnd(/* overlap */ T),
    ContainedBy,
    Contains,
    Coincides,
    EntirelyBefore,
    EntirelyAfter
}

#[deriving(Clone)]
pub struct Range<T> {
    off: T,
    len: T,
}

impl<T: Int + TotalOrd + Signed> fmt::Show for Range<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f.buf, "[{} .. {})", self.begin(), self.end())
    }
}

impl<T: Int + TotalOrd + Signed> Range<T> {
    #[inline]
    pub fn new(off: T, len: T) -> Range<T> {
        Range {
            off: off,
            len: len,
        }
    }

    #[inline]
    pub fn empty() -> Range<T> {
        Range::new(num::zero(), num::zero())
    }

    #[inline]
    pub fn begin(&self) -> T { self.off  }
    #[inline]
    pub fn length(&self) -> T { self.len }
    #[inline]
    pub fn end(&self) -> T { self.off + self.len }

    #[inline]
    pub fn eachi(&self) -> iter::Range<T> {
        range(self.off, self.off + self.len)
    }

    #[inline]
    pub fn contains(&self, i: T) -> bool {
        i >= self.begin() && i < self.end()
    }

    #[inline]
    pub fn is_valid_for_string(&self, s: &str) -> bool {
        let s_len = s.len();
        match num::cast(s_len) {
            Some(len) => {
                self.begin() < len && self.end() <= len && self.length() <= len
            },
            None => {
                debug!("Range<T>::is_valid_for_string: string length (len={}) is longer than the max \
                        value for T (max={})", s_len, { let val: T = Bounded::max_value(); val });
                false
            },
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len.is_zero()
    }

    #[inline]
    pub fn shift_by(&mut self, i: T) {
        self.off = self.off + i;
    }

    #[inline]
    pub fn extend_by(&mut self, i: T) {
        self.len = self.len + i;
    }

    #[inline]
    pub fn extend_to(&mut self, i: T) {
        self.len = i - self.off;
    }

    #[inline]
    pub fn adjust_by(&mut self, off_i: T, len_i: T) {
        self.off = self.off + off_i;
        self.len = self.len + len_i;
    }

    #[inline]
    pub fn reset(&mut self, off_i: T, len_i: T) {
        self.off = off_i;
        self.len = len_i;
    }

    #[inline]
    pub fn intersect(&self, other: &Range<T>) -> Range<T> {
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
    pub fn relation_to_range(&self, other: &Range<T>) -> RangeRelation<T> {
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
    pub fn repair_after_coalesced_range(&mut self, other: &Range<T>) {
        let relation = self.relation_to_range(other);
        debug!("repair_after_coalesced_range: possibly repairing range {}", *self);
        debug!("repair_after_coalesced_range: relation of original range and coalesced range {}: {}",
               *other, relation);
        match relation {
            EntirelyBefore => {},
            EntirelyAfter => { self.shift_by(-other.length()); },
            Coincides | ContainedBy => { self.reset(other.begin(), num::one()); },
            Contains => { self.extend_by(-other.length()); },
            OverlapsBegin(overlap) => { self.extend_by(num::one::<T>() - overlap); },
            OverlapsEnd(overlap) => {
                let len = self.length() - overlap + num::one();
                self.reset(other.begin(), len);
            }
        };
        debug!("repair_after_coalesced_range: new range: ---- {}", *self);
    }
}
