pub struct Range {
    priv off: u16,
    priv len: u16
}

pub pure fn Range(off: uint, len: uint) -> Range {
    assert off <= u16::max_value as uint;
    assert len <= u16::max_value as uint;

    Range {
        off: off as u16,
        len: len as u16
    }
}

pub pure fn empty() -> Range { Range(0,0) }

enum RangeRelation {
    OverlapsBegin(/* overlap */ uint),
    OverlapsEnd(/* overlap */ uint),
    ContainedBy,
    Contains,
    Coincides,
    EntirelyBefore,
    EntirelyAfter
}

pub impl Range {
    pub pure fn begin() -> uint { self.off as uint }
    pub pure fn length() -> uint { self.len as uint }
    pub pure fn end() -> uint { (self.off as uint) + (self.len as uint) }

    pub pure fn eachi(cb: fn&(uint) -> bool) {
        do uint::range(self.off as uint, 
                       (self.off as uint) + (self.len as uint)) |i| {
            cb(i)
        }
    }

    pub pure fn is_valid_for_string(s: &str) -> bool {
        self.begin() < s.len() && self.end() <= s.len() && self.length() <= s.len()
    }

    pub pure fn shift_by(i: int) -> Range { 
        Range(((self.off as int) + i) as uint, self.len as uint)
    }

    pub pure fn extend_by(i: int) -> Range { 
        Range(self.off as uint, ((self.len as int) + i) as uint)
    }

    pub pure fn adjust_by(off_i: int, len_i: int) -> Range {
        Range(((self.off as int) + off_i) as uint, ((self.len as int) + len_i) as uint)
    }
}

pub pure fn empty_mut() -> MutableRange { MutableRange::new(0, 0) }

pub struct MutableRange {
    priv off: uint,
    priv len: uint
}

pub impl MutableRange {
    static pub pure fn new(off: uint, len: uint) -> MutableRange {
        MutableRange { off: off, len: len }
    }

    static pub pure fn empty() -> MutableRange {
        MutableRange::new(0, 0)
    }
}

pub impl MutableRange {
    pure fn begin(&const self) -> uint { self.off  }
    pure fn length(&const self) -> uint { self.len }
    pure fn end(&const self) -> uint { self.off + self.len }
    pure fn eachi(&const self, cb: fn&(uint) -> bool) {
        do uint::range(self.off, self.off + self.len) |i| { cb(i) }
    }

    pure fn contains(&const self, i: uint) -> bool {
        i >= self.begin() && i < self.end()
    }

    pure fn as_immutable(&const self) -> Range {
        Range(self.begin(), self.length())
    }

    pure fn is_valid_for_string(&const self, s: &str) -> bool {
        self.begin() < s.len() && self.end() <= s.len() && self.length() <= s.len()
    }

    fn shift_by(&mut self, i: int) { 
        self.off = ((self.off as int) + i) as uint;
    }

    fn extend_by(&mut self, i: int) { 
        self.len = ((self.len as int) + i) as uint;
    }

    fn extend_to(&mut self, i: uint) { 
        self.len = i - self.off;
    }

    fn adjust_by(&mut self, off_i: int, len_i: int) {
        self.off = ((self.off as int) + off_i) as uint;
        self.len = ((self.len as int) + len_i) as uint;
    }

    fn reset(&mut self, off_i: uint, len_i: uint) {
        self.off = off_i;
        self.len = len_i;
    }

    /// Computes the relationship between two ranges (`self` and `other`),
    /// from the point of view of `self`. So, 'EntirelyBefore' means
    /// that the `self` range is entirely before `other` range.
    pure fn relation_to_range(&const self, other: &const MutableRange) -> RangeRelation {
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
        fail fmt!("relation_to_range(): didn't classify self=%?, other=%?",
                  self, other);
    }

    fn repair_after_coalesced_range(&mut self, other: &const MutableRange) {
        let relation = self.relation_to_range(other);
        debug!("repair_after_coalesced_range: possibly repairing range %?", self);
        debug!("repair_after_coalesced_range: relation of original range and coalesced range(%?): %?",
               other, relation);
        match relation {
            EntirelyBefore => { },
            EntirelyAfter =>  { self.shift_by(-(other.length() as int)); },
            Coincides | ContainedBy =>   { self.reset(other.begin(), 1); },
            Contains =>      { self.extend_by(-(other.length() as int)); },
            OverlapsBegin(overlap) => { self.extend_by(1 - (overlap as int)); },
            OverlapsEnd(overlap) => 
            { self.reset(other.begin(), self.length() - overlap + 1); }
        };
        debug!("repair_after_coalesced_range: new range: ---- %?", self);
    }
}
