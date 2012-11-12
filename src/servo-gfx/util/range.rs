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
      
    /// Computes the relationship between two ranges (`self` and `other`),
    /// from the point of view of `self`. So, 'EntirelyBefore' means
    /// that the `self` range is entirely before `other` range.
    fn relation_to_range(&self, other: Range) -> RangeRelation {
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

    fn repair_after_coalesced_range(&self, other: Range) -> Range {
        let relation = self.relation_to_range(other);
        debug!("repair_after_coalesced_range: possibly repairing range %?", self);
        debug!("repair_after_coalesced_range: relation of original range and coalesced range(%?): %?",
               other, relation);
        let new_range = match relation {
            EntirelyBefore => { *self },
            EntirelyAfter =>  { self.shift_by(-(other.length() as int)) },
            Coincides | ContainedBy =>   { Range(other.begin(), 1) },
            Contains =>      { self.extend_by(-(other.length() as int)) },
            OverlapsBegin(overlap) => { self.extend_by(1 - (overlap as int)) },
            OverlapsEnd(overlap) => 
            { Range(other.begin(), self.length() - overlap + 1) }
        };
        debug!("repair_after_coalesced_range: new range: ---- %?", new_range);
        new_range
    }
}

pub pure fn empty_mut() -> MutableRange { MutableRange::new(0, 0) }

pub struct MutableRange {
    priv mut off: uint,
    priv mut len: uint
}

pub impl MutableRange {
    static pub pure fn new(off: uint, len: uint) -> MutableRange {
        MutableRange { off: off, len: len }
    }

    static pub pure fn empty() -> MutableRange {
        MutableRange::new(0, 0)
    }
}

impl MutableRange {
    pub pure fn begin() -> uint { self.off  }
    pub pure fn length() -> uint { self.len }
    pub pure fn end() -> uint { self.off + self.len }
    pub pure fn eachi(cb: fn&(uint) -> bool) {
        do uint::range(self.off, self.off + self.len) |i| { cb(i) }
    }

    fn relation_to_range(&self, other: &MutableRange) -> RangeRelation {
        self.as_immutable().relation_to_range(other.as_immutable())
    }

    pub pure fn as_immutable() -> Range {
        Range(self.begin(), self.length())
    }

    pub pure fn is_valid_for_string(s: &str) -> bool {
        self.begin() < s.len() && self.end() <= s.len() && self.length() <= s.len()
    }

    pub fn shift_by(i: int) { 
        self.off = ((self.off as int) + i) as uint;
    }

    pub fn extend_by(i: int) { 
        self.len = ((self.len as int) + i) as uint;
    }

    pub fn extend_to(i: uint) { 
        self.len = i - self.off;
    }

    pub fn adjust_by(off_i: int, len_i: int) {
        self.off = ((self.off as int) + off_i) as uint;
        self.len = ((self.len as int) + len_i) as uint;
    }

    pub fn reset(off_i: uint, len_i: uint) {
        self.off = off_i;
        self.len = len_i;
    }
}
