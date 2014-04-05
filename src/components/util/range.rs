/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::cmp::{max, min};
use std::iter;
use std::fmt;

pub enum RangeRelation {
    OverlapsBegin(/* overlap */ uint),
    OverlapsEnd(/* overlap */ uint),
    ContainedBy,
    Contains,
    Coincides,
    EntirelyBefore,
    EntirelyAfter
}

#[deriving(Clone)]
pub struct Range {
    off: uint,
    len: uint
}

impl fmt::Show for Range {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f.buf, "[{} .. {})", self.begin(), self.end())
    }
}

impl Range {
    #[inline]
    pub fn new(off: uint, len: uint) -> Range {
        Range {
            off: off,
            len: len,
        }
    }

    #[inline]
    pub fn empty() -> Range {
        Range::new(0, 0)
    }
}

impl Range {
    #[inline]
    pub fn begin(&self) -> uint { self.off  }
    #[inline]
    pub fn length(&self) -> uint { self.len }
    #[inline]
    pub fn end(&self) -> uint { self.off + self.len }

    #[inline]
    pub fn eachi(&self) -> iter::Range<uint> {
        range(self.off, self.off + self.len)
    }

    #[inline]
    pub fn contains(&self, i: uint) -> bool {
        i >= self.begin() && i < self.end()
    }

    #[inline]
    pub fn is_valid_for_string(&self, s: &str) -> bool {
        self.begin() < s.len() && self.end() <= s.len() && self.length() <= s.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline]
    pub fn shift_by(&mut self, i: int) {
        self.off = ((self.off as int) + i) as uint;
    }

    #[inline]
    pub fn extend_by(&mut self, i: int) {
        self.len = ((self.len as int) + i) as uint;
    }

    #[inline]
    pub fn extend_to(&mut self, i: uint) {
        self.len = i - self.off;
    }

    #[inline]
    pub fn adjust_by(&mut self, off_i: int, len_i: int) {
        self.off = ((self.off as int) + off_i) as uint;
        self.len = ((self.len as int) + len_i) as uint;
    }

    #[inline]
    pub fn reset(&mut self, off_i: uint, len_i: uint) {
        self.off = off_i;
        self.len = len_i;
    }

    #[inline]
    pub fn intersect(&self, other: &Range) -> Range {
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
    pub fn relation_to_range(&self, other: &Range) -> RangeRelation {
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
        fail!("relation_to_range(): didn't classify self={:?}, other={:?}",
              self, other);
    }

    #[inline]
    pub fn repair_after_coalesced_range(&mut self, other: &Range) {
        let relation = self.relation_to_range(other);
        debug!("repair_after_coalesced_range: possibly repairing range {:?}", self);
        debug!("repair_after_coalesced_range: relation of original range and coalesced range({:?}): {:?}",
               other, relation);
        match relation {
            EntirelyBefore => { },
            EntirelyAfter =>  { self.shift_by(-(other.length() as int)); },
            Coincides | ContainedBy =>   { self.reset(other.begin(), 1); },
            Contains =>      { self.extend_by(-(other.length() as int)); },
            OverlapsBegin(overlap) => { self.extend_by(1 - (overlap as int)); },
            OverlapsEnd(overlap) => {
                let len = self.length() - overlap + 1;
                self.reset(other.begin(), len);
            }
        };
        debug!("repair_after_coalesced_range: new range: ---- {:?}", self);
    }
}
