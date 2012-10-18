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
    pure fn iter_indivisible_pieces_for_range(&self, offset: uint, length: uint, f: fn(uint, uint) -> bool);
    // TODO: needs to take box style as argument, or move to TextBox.
    // see Gecko's IsTrimmableSpace methods for details.
    pure fn range_is_trimmable_whitespace(&self, offset: uint, length: uint) -> bool;

    fn metrics_for_range(offset: uint, length: uint) -> RunMetrics;
    fn min_width_for_range(offset: uint, length: uint) -> au;
    fn iter_natural_lines_for_range(&self, offset: uint, length: uint, f: fn(uint, uint) -> bool);
}

impl TextRun : TextRunMethods {
    pure fn glyphs(&self) -> &self/GlyphStore { &self.glyphs }

    pure fn range_is_trimmable_whitespace(&self, offset: uint, length: uint) -> bool {
        let mut i = offset;
        while i < offset + length {
            let {ch, next} = str::char_range_at(self.text, i);
            match ch {
                ' ' | '\t' | '\r'  => {},
                _ => { return false; }
            }
            i = next;
        }
        return true;
    }

    fn metrics_for_range(offset: uint, length: uint) -> RunMetrics {
        self.font.measure_text(&self, offset, length)
    }

    fn min_width_for_range(offset: uint, length: uint) -> au {    
        assert length > 0;
        assert offset < self.text.len();
        assert offset + length <= self.text.len();

        let mut max_piece_width = au(0);
        for self.iter_indivisible_pieces_for_range(offset, length) |piece_offset, piece_len| {
            let metrics = self.font.measure_text(&self, piece_offset, piece_len);
            if metrics.advance_width > max_piece_width {
                max_piece_width = metrics.advance_width;
            }
        };

        return max_piece_width;
    }

    fn iter_natural_lines_for_range(&self, offset: uint, length: uint, f: fn(uint, uint) -> bool) {
        assert length > 0;
        assert offset < self.text.len();
        assert offset + length <= self.text.len();

        let mut clump_offset = offset;
        let mut clump_length = 0;
        let mut in_clump = false;

        // clump non-linebreaks of nonzero length
        for uint::range(offset, offset + length) |i| {
            match (self.glyphs.char_is_newline(i), in_clump) {
                (false, true)  => { clump_length += 1; }
                (false, false) => { in_clump = true; clump_offset = i; clump_length = 1; }
                (true, false) => { /* chomp whitespace */ }
                (true, true)  => {
                    in_clump = false;
                    // don't include the linebreak 'glyph'
                    // (we assume there's one GlyphEntry for a newline, and no actual glyphs)
                    if !f(clump_offset, clump_length) { break }
                }
            }
        }
        
        // flush any remaining chars as a line
        if in_clump {
            clump_length = (offset + length) - clump_offset;
            f(clump_offset, clump_length);
        }
    }

    pure fn iter_indivisible_pieces_for_range(&self, offset: uint, length: uint, f: fn(uint, uint) -> bool) {
        assert length > 0;
        assert offset < self.text.len();
        assert offset + length <= self.text.len();

        let mut clump_offset = offset;
        let mut clump_length;

        loop {
            // find next non-whitespace byte index, then clump all whitespace before it.
            match str::find_from(self.text, clump_offset, |c| !char::is_whitespace(c)) {
                Some(nonws_char_offset) => {
                    clump_length = nonws_char_offset - clump_offset;
                    if !f(clump_offset, clump_length) { break }
                    clump_offset += clump_length;
                    // reached end
                    if clump_offset == offset + length { break }
                },
                None => {
                    // nothing left, flush last piece containing only whitespace
                    if clump_offset < offset + length {
                        let clump_length = (offset + length) - clump_offset;
                        f(clump_offset, clump_length);
                    }
                    break
                }
            };

            // find next whitespace byte index, then clump all non-whitespace before it.
            match str::find_from(self.text, clump_offset, |c| char::is_whitespace(c)) {
                Some(ws_char_offset) => {
                    clump_length = ws_char_offset - clump_offset;
                    if !f(clump_offset, clump_length) { break }
                    clump_offset += clump_length;
                    // reached end
                    if clump_offset == offset + length { break }
                }
                None => {
                    // nothing left, flush last piece containing only non-whitespaces
                    if clump_offset < offset + length {
                        let clump_length = (offset + length) - clump_offset;
                        f(clump_offset, clump_length);
                    }
                    break
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
        for run.iter_indivisible_pieces_for_range(0, text.len()) |offset, length| {
            slices.push(str::slice(text, offset, length));
        }
        assert slices == res;
    }

    test_pieces(~"firecracker yumyum woopwoop", ~[~"firecracker", ~" ", ~"yumyum", ~" ", ~"woopwoop"]);
    test_pieces(~"firecracker yumyum     ", ~[~"firecracker", ~" ", ~"yumyum", ~"     "]);
    test_pieces(~"     firecracker yumyum", ~[~"     ", ~"firecracker", ~" ", ~"yumyum"]);
    test_pieces(~"  ", ~[~"  "]);
    test_pieces(~"", ~[]);
}

