use arc = std::arc;
use arc::ARC;
use au = gfx::geometry;
use font::{RunMetrics, Font};
use font_cache::FontCache;
use geom::point::Point2D;
use geom::size::Size2D;
use gfx::geometry::au;
use glyph::GlyphStore;
use layout::context::LayoutContext;
use libc::{c_void};
use servo_util::color;
use shaper::shape_textrun;
use std::arc;

pub struct TextRun {
    text: ~str,
    font: @Font,
    priv glyphs: GlyphStore,
}

pub struct TextRange {
    priv off: u16,
    priv len: u16
}

pure fn TextRange(off: uint, len :uint) -> TextRange {
    assert off <= u16::max_value as uint;
    assert len <= u16::max_value as uint;

    TextRange {
        off: off as u16,
        len: len as u16
    }
}

impl TextRange {
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

    pub pure fn shift_by(i: int) -> TextRange { 
        TextRange(((self.off as int) + i) as uint, self.len as uint)
    }

    pub pure fn extend_by(i: int) -> TextRange { 
        TextRange(self.off as uint, ((self.len as int) + i) as uint)
    }

    pub pure fn adjust_by(off_i: int, len_i: int) -> TextRange {
        TextRange(((self.off as int) + off_i) as uint, ((self.len as int) + len_i) as uint)
    }
}

pub struct MutableTextRange {
    priv mut off: uint,
    priv mut len: uint
}

pure fn MutableTextRange(off: uint, len :uint) -> MutableTextRange {
    MutableTextRange { off: off, len: len }
}

impl MutableTextRange {
    pub pure fn begin() -> uint { self.off  }
    pub pure fn length() -> uint { self.len }
    pub pure fn end() -> uint { self.off + self.len }
    pub pure fn eachi(cb: fn&(uint) -> bool) {
        do uint::range(self.off, self.off + self.len) |i| { cb(i) }
    }

    pub pure fn as_immutable() -> TextRange {
        TextRange(self.begin(), self.length())
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

// This is a hack until TextRuns are normally sendable, or
// we instead use ARC<TextRun> everywhere.
pub struct SendableTextRun {
    text: ~str,
    font_descriptor: (),
    priv glyphs: GlyphStore,
}

pub fn serialize(_cache: @FontCache, run: &TextRun) -> ~SendableTextRun {
    ~SendableTextRun {
        text: copy run.text,
        // TODO: actually serialize a font descriptor thingy
        font_descriptor: (),
        glyphs: copy run.glyphs,
    }
}

pub fn deserialize(cache: @FontCache, run: &SendableTextRun) -> @TextRun {
    @TextRun {
        text: copy run.text,
        // TODO: actually deserialize a font descriptor thingy
        font: cache.get_test_font(),
        glyphs: copy run.glyphs
    }
}

trait TextRunMethods {
    pure fn glyphs(&self) -> &self/GlyphStore;
    fn iter_indivisible_pieces_for_range(&self, range: TextRange, f: fn&(TextRange) -> bool);
    // TODO: needs to take box style as argument, or move to TextBox.
    // see Gecko's IsTrimmableSpace methods for details.
    pure fn range_is_trimmable_whitespace(&self, range: TextRange) -> bool;

    fn metrics_for_range(&self, range: TextRange) -> RunMetrics;
    fn min_width_for_range(&self, range: TextRange) -> au;
    fn iter_natural_lines_for_range(&self, range: TextRange, f: fn&(TextRange) -> bool);
}

impl TextRun : TextRunMethods {
    pure fn glyphs(&self) -> &self/GlyphStore { &self.glyphs }

    pure fn range_is_trimmable_whitespace(&self, range: TextRange) -> bool {
        let mut i = range.begin();
        while i < range.end() {
            // jump i to each new char
            let {ch, next} = str::char_range_at(self.text, i);
            match ch {
                ' ' | '\t' | '\r'  => {},
                _ => { return false; }
            }
            i = next;
        }
        return true;
    }

    fn metrics_for_range(&self, range: TextRange) -> RunMetrics {
        self.font.measure_text(self, range.begin(), range.length())
    }

    fn min_width_for_range(&self, range: TextRange) -> au {    
        assert range.is_valid_for_string(self.text);

        let mut max_piece_width = au(0);
        for self.iter_indivisible_pieces_for_range(range) |piece_range| {
            let metrics = self.font.measure_text(self, piece_range.begin(), piece_range.length());
            max_piece_width = au::max(max_piece_width, metrics.advance_width);
        }
        return max_piece_width;
    }

    fn iter_natural_lines_for_range(&self, range: TextRange, f: fn(TextRange) -> bool) {
        assert range.is_valid_for_string(self.text);

        let clump = MutableTextRange(range.begin(), 0);
        let mut in_clump = false;

        // clump non-linebreaks of nonzero length
        for range.eachi |i| {
            match (self.glyphs.char_is_newline(i), in_clump) {
                (false, true)  => { clump.extend_by(1); }
                (false, false) => { in_clump = true; clump.reset(i, 1); }
                (true, false) => { /* chomp whitespace */ }
                (true, true)  => {
                    in_clump = false;
                    // don't include the linebreak 'glyph'
                    // (we assume there's one GlyphEntry for a newline, and no actual glyphs)
                    if !f(clump.as_immutable()) { break }
                }
            }
        }
        
        // flush any remaining chars as a line
        if in_clump {
            clump.extend_to(range.end());
            f(clump.as_immutable());
        }
    }

    fn iter_indivisible_pieces_for_range(&self, range: TextRange, f: fn(TextRange) -> bool) {
        assert range.is_valid_for_string(self.text);

        let clump = MutableTextRange(range.begin(), 0);
        loop {
            // find next non-whitespace byte index, then clump all whitespace before it.
            if clump.end() == range.end() { break }
            match str::find_from(self.text, clump.begin(), |c| !char::is_whitespace(c)) {
                Some(nonws_char_offset) => {
                    clump.extend_to(nonws_char_offset);
                    if !f(clump.as_immutable()) { break }
                    clump.reset(clump.end(), 0);
                },
                None => {
                    // nothing left, flush last piece containing only whitespace
                    if clump.end() < range.end() {
                        clump.extend_to(range.end());
                        f(clump.as_immutable());
                    }
                }
            };

            // find next whitespace byte index, then clump all non-whitespace before it.
            if clump.end() == range.end() { break }
            match str::find_from(self.text, clump.begin(), |c| char::is_whitespace(c)) {
                Some(ws_char_offset) => {
                    clump.extend_to(ws_char_offset);
                    if !f(clump.as_immutable()) { break }
                    clump.reset(clump.end(), 0);
                }
                None => {
                    // nothing left, flush last piece containing only non-whitespaces
                    if clump.end() < range.end() {
                        clump.extend_to(range.end());
                        f(clump.as_immutable());
                    }
                }
            }
        }
    }
}
 
fn TextRun(font: @Font, text: ~str) -> TextRun {
    let glyph_store = GlyphStore(text.len());
    let run = TextRun {
        text: text,
        font: font,
        glyphs: glyph_store,
    };

    shape_textrun(&run);
    return run;
}

// this test can't run until LayoutContext is removed as an argument
// to min_width_for_range.
/*
#[test]
fn test_calc_min_break_width() {

    fn test_min_width_for_run(text: ~str, width: au) {
        let flib = FontCache();
        let font = flib.get_test_font();
        let run = TextRun(font, text);
        run.min_width_for_range(0, text.len())
    }

    test_min_width_for_run(~"firecracker", au::from_px(84));
    test_min_width_for_run(~"firecracker yumyum", au::from_px(84));
    test_min_width_for_run(~"yumyum firecracker", au::from_px(84));
    test_min_width_for_run(~"yumyum firecracker yumyum", au::from_px(84));
}
*/

#[test]
#[ignore]
fn test_iter_indivisible_pieces() {
    fn test_pieces(text: ~str, res: ~[~str]) {
        let flib = FontCache();
        let font = flib.get_test_font();
        let run = TextRun(font, copy text);
        let mut slices : ~[~str] = ~[];
        for run.iter_indivisible_pieces_for_range(TextRange(0, text.len())) |subrange| {
            slices.push(str::slice(text, subrange.begin(), subrange.length()));
        }
        assert slices == res;
    }

    test_pieces(~"firecracker yumyum woopwoop", ~[~"firecracker", ~" ", ~"yumyum", ~" ", ~"woopwoop"]);
    test_pieces(~"firecracker yumyum     ", ~[~"firecracker", ~" ", ~"yumyum", ~"     "]);
    test_pieces(~"     firecracker yumyum", ~[~"     ", ~"firecracker", ~" ", ~"yumyum"]);
    test_pieces(~"  ", ~[~"  "]);
    test_pieces(~"", ~[]);
}

