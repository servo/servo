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

        debug!("enter min_width_for_range(o=%?, l=%?)", offset, length);

        let mut max_piece_width = au(0);
        // TODO: use a real font reference
        let font = ctx.font_cache.get_test_font();
        for self.iter_indivisible_pieces_for_range(offset, length) |piece_offset, piece_len| {
            let metrics = font.measure_text(&self, piece_offset, piece_len);
            if metrics.advance > max_piece_width {
                max_piece_width = metrics.advance;
            }
        };

        debug!("exit min_width_for_range(o=%?, l=%?)", offset, length);
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

        debug!("enter iter_indivisible_pieces_for_range(o=%?, l=%?)", offset, length);

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
        debug!("exit iter_indivisible_pieces_for_range(o=%?, l=%?)", offset, length);
    }
}
 
fn TextRun(font: &Font, +text: ~str) -> TextRun {
    let glyph_store = GlyphStore(text.len());
    let run = TextRun {
        text: text,
        glyphs: glyph_store,
    };

    shape_textrun(font, &run);
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
    fn test_pieces(+text: ~str, +res: ~[~str]) {
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

