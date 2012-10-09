use arc = std::arc;
use arc::ARC;
use au = gfx::geometry;
use font::Font;
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
    priv glyphs: GlyphStore,
}

impl TextRun {
    pure fn glyphs(&self) -> &self/GlyphStore { &self.glyphs }

    fn min_width_for_range(ctx: &LayoutContext, offset: uint, length: uint) -> au {    
        assert length > 0;
        assert offset < self.text.len();
        assert offset + length <= self.text.len();

        let mut max_piece_width = au(0);
        // TODO: use a real font reference
        let font = ctx.font_cache.get_test_font();
        for self.iter_indivisible_pieces_for_range(offset, length) |piece_offset, piece_len| {

            let metrics = font.measure_text(&self, piece_offset, piece_len);
            if metrics.advance > max_piece_width {
                max_piece_width = metrics.advance;
            }
        };
        return max_piece_width;
    }

    fn iter_natural_lines_for_range(&self, offset: uint, length: uint, f: fn(uint, uint) -> bool) {
        assert length > 0;
        assert offset < self.text.len();
        assert offset + length <= self.text.len();

        let mut clump_start = offset;
        let mut clump_end = offset;
        let mut in_clump = false;

        // clump non-linebreaks of nonzero length
        for uint::range(offset, offset + length) |i| {
            match (self.glyphs.char_is_newline(i), in_clump) {
                (false, true)  => { clump_end = i; }
                (false, false) => { in_clump = true; clump_start = i; clump_end = i; }
                (true, false) => { /* chomp whitespace */ }
                (true, true)  => {
                    in_clump = false;
                    // don't include the linebreak 'glyph'
                    // (we assume there's one GlyphEntry for a newline, and no actual glyphs)
                    if !f(clump_start, clump_end - clump_start + 1) { break }
                }
            }
        }
        
        // flush any remaining chars as a line
        if in_clump {
            clump_end = offset + length - 1;
            f(clump_start, clump_end - clump_start + 1);
        }
    }

    pure fn iter_indivisible_pieces_for_range(&self, offset: uint, length: uint, f: fn(uint, uint) -> bool) {
        assert length > 0;
        assert offset < self.text.len();
        assert offset + length <= self.text.len();

        //TODO: need a more sophisticated model of words and possible breaks
        let text = str::view(self.text, offset, length);

        let mut clump_start = offset;

        loop {
            // clump contiguous non-whitespace
            match str::find_from(text, clump_start, |c| !char::is_whitespace(c)) {
                Some(clump_end) => {
                    if !f(clump_start, clump_end - clump_start + 1) { break }
                    clump_start = clump_end + 1;
                    // reached end
                    if clump_start == offset + length { break }
                },
                None => {
                    // nothing left, flush last piece containing only spaces
                    if clump_start < offset + length {
                        let clump_end = offset + length - 1;
                        f(clump_start, clump_end - clump_start + 1);
                    }
                    break
                }
            };

            // clump contiguous whitespace
            match str::find_from(text, clump_start, |c| char::is_whitespace(c)) {
                Some(clump_end) => {
                    if !f(clump_start, clump_end - clump_start + 1) { break }
                    clump_start = clump_end + 1;
                    // reached end
                    if clump_start == offset + length { break }
                }
                None => {
                    // nothing left, flush last piece containing only spaces
                    if clump_start < offset + length {
                        let clump_end = offset + length - 1;
                        f(clump_start, clump_end - clump_start + 1);
                    }
                    break
                }
            }
        }
    }
}
 
fn TextRun(font: &Font, text: ~str) -> TextRun {
    let glyph_store = GlyphStore(text);
    let run = TextRun {
        text: text,
        glyphs: glyph_store,
    };

    shape_textrun(font, &run);
    return run;
}

/// Iterates over all the indivisible substrings
#[test]
fn test_calc_min_break_width1() {
    let flib = FontCache();
    let font = flib.get_test_font();
    let actual = calc_min_break_width(font, ~"firecracker");
    let expected = au::from_px(84);
    assert expected == actual;
}

#[test]
fn test_calc_min_break_width2() {
    let flib = FontCache();
    let font = flib.get_test_font();
    let actual = calc_min_break_width(font, ~"firecracker yumyum");
    let expected = au::from_px(84);
    assert expected == actual;
}

#[test]
fn test_calc_min_break_width3() {
    let flib = FontCache();
    let font = flib.get_test_font();
    let actual = calc_min_break_width(font, ~"yumyum firecracker");
    let expected = au::from_px(84);
    assert expected == actual;
}

#[test]
fn test_calc_min_break_width4() {
    let flib = FontCache();
    let font = flib.get_test_font();
    let actual = calc_min_break_width(font, ~"yumyum firecracker yumyum");
    let expected = au::from_px(84);
    assert expected == actual;
}

#[test]
fn test_iter_indivisible_slices() {
    let flib = FontCache();
    let font = flib.get_test_font();
    let mut slices = ~[];
    for iter_indivisible_slices(font, "firecracker yumyum woopwoop") |slice| {
        slices += [slice];
    }
    assert slices == ~["firecracker", "yumyum", "woopwoop"];
}

#[test]
fn test_iter_indivisible_slices_trailing_whitespace() {
    let flib = FontCache();
    let font = flib.get_test_font();
    let mut slices = ~[];
    for iter_indivisible_slices(font, "firecracker  ") |slice| {
        slices += [slice];
    }
    assert slices == ~["firecracker"];
}

#[test]
fn test_iter_indivisible_slices_leading_whitespace() {
    let flib = FontCache();
    let font = flib.get_test_font();
    let mut slices = ~[];
    for iter_indivisible_slices(font, "  firecracker") |slice| {
        slices += [slice];
    }
    assert slices == ~["firecracker"];
}

#[test]
fn test_iter_indivisible_slices_empty() {
    let flib = FontCache();
    let font = flib.get_test_font();
    let mut slices = ~[];
    for iter_indivisible_slices(font, "") |slice| {
        slices += [slice];
    }
    assert slices == ~[];
}

#[test]
fn test_split() {
    let flib = FontCache();
    let font = flib.get_test_font();
    let run = TextRun(font, ~"firecracker yumyum");
    let break_runs = run.split(font, run.min_break_width());
    assert break_runs.first().text == ~"firecracker";
    assert break_runs.second().text == ~"yumyum";
}

#[test]
fn test_split2() {
    let flib = FontCache();
    let font = flib.get_test_font();
    let run = TextRun(font, ~"firecracker yum yum yum yum yum");
    let break_runs = run.split(font, run.min_break_width());
    assert break_runs.first().text == ~"firecracker";
    assert break_runs.second().text == ~"yum yum yum yum yum";
}

/* Causes ICE during compilation. See Rust Issue #3592 */
/*
#[test]
fn test_split3() {
    let flib = FontCache();
    let font = flib.get_test_font();
    let run = TextRun(font, ~"firecracker firecracker");
    let px = au::from_px(10);
    let break_runs = run.split(font, run.min_break_width() + px);
    assert break_runs.first().text == ~"firecracker";
    assert break_runs.second().text == ~"firecracker";

}*/

#[test]
#[ignore(cfg(target_os = "macos"))]
fn should_calculate_the_total_size() {
    let flib = FontCache();
    let font = flib.get_test_font();
    let run = TextRun(font, ~"firecracker");
    let expected = Size2D(au::from_px(84), au::from_px(20));
    assert run.size() == expected;
}

