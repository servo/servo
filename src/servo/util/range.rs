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

pub pure fn empty_mut() -> MutableRange { MutableRange(0,0) }

pub struct MutableRange {
    priv mut off: uint,
    priv mut len: uint
}

pure fn MutableRange(off: uint, len :uint) -> MutableRange {
    MutableRange { off: off, len: len }
}

impl MutableRange {
    pub pure fn begin() -> uint { self.off  }
    pub pure fn length() -> uint { self.len }
    pub pure fn end() -> uint { self.off + self.len }
    pub pure fn eachi(cb: fn&(uint) -> bool) {
        do uint::range(self.off, self.off + self.len) |i| { cb(i) }
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
