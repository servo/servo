/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use font_context::FontContext;
use geometry::Au;
use text::glyph::{BreakTypeNormal, GlyphStore};
use font::{Font, FontDescriptor, RunMetrics};
use servo_util::range::Range;

/// A text run.
pub struct TextRun {
    text: ~str,
    font: @mut Font,
    glyphs: GlyphStore,
}

/// This is a hack until TextRuns are normally sendable, or we instead use ARC<TextRun> everywhere.
pub struct SendableTextRun {
    text: ~str,
    font: FontDescriptor,
    priv glyphs: GlyphStore,
}

impl SendableTextRun {
    pub fn deserialize(&self, fctx: @mut FontContext) -> TextRun {
        let font = match fctx.get_font_by_descriptor(&self.font) {
            Ok(f) => f,
            Err(_) => fail!(fmt!("Font descriptor deserialization failed! desc=%?", self.font))
        };

        TextRun {
            text: copy self.text,
            font: font,
            glyphs: copy self.glyphs
        }
    }
}

pub impl<'self> TextRun {
    fn new(font: @mut Font, text: ~str) -> TextRun {
        let mut glyph_store = GlyphStore::new(str::char_len(text));
        TextRun::compute_potential_breaks(text, &mut glyph_store);
        font.shape_text(text, &mut glyph_store);

        let run = TextRun {
            text: text,
            font: font,
            glyphs: glyph_store,
        };
        return run;
    }

    fn compute_potential_breaks(text: &str, glyphs: &mut GlyphStore) {
        // TODO(Issue #230): do a better job. See Gecko's LineBreaker.

        let mut byte_i = 0u;
        let mut char_j = 0u;
        let mut prev_is_whitespace = false;
        while byte_i < text.len() {
            let range = str::char_range_at(text, byte_i);
            let ch = range.ch;
            let next = range.next;
            // set char properties.
            match ch {
                ' '  => { glyphs.set_char_is_space(char_j); },
                '\t' => { glyphs.set_char_is_tab(char_j); },
                '\n' => { glyphs.set_char_is_newline(char_j); },
                _ => {}
            }

            // set line break opportunities at whitespace/non-whitespace boundaries.
            if prev_is_whitespace {
                match ch {
                    ' ' | '\t' | '\n' => {},
                    _ => {
                        glyphs.set_can_break_before(char_j, BreakTypeNormal);
                        prev_is_whitespace = false;
                    }
                }
            } else {
                match ch {
                    ' ' | '\t' | '\n' => {
                        glyphs.set_can_break_before(char_j, BreakTypeNormal);
                        prev_is_whitespace = true;
                    },
                    _ => { }
                }
            }

            byte_i = next;
            char_j += 1;
        }
    }

    pub fn serialize(&self) -> SendableTextRun {
        SendableTextRun {
            text: copy self.text,
            font: self.font.get_descriptor(),
            glyphs: copy self.glyphs,
        }
    }

    fn char_len(&self) -> uint { self.glyphs.entry_buffer.len() }
    fn glyphs(&'self self) -> &'self GlyphStore { &self.glyphs }

    fn range_is_trimmable_whitespace(&self, range: &Range) -> bool {
        for range.eachi |i| {
            if  !self.glyphs.char_is_space(i) &&
                !self.glyphs.char_is_tab(i)   &&
                !self.glyphs.char_is_newline(i) { return false; }
        }
        return true;
    }

    fn metrics_for_range(&self, range: &Range) -> RunMetrics {
        self.font.measure_text(self, range)
    }

    fn min_width_for_range(&self, range: &Range) -> Au {
        let mut max_piece_width = Au(0);
        debug!("iterating outer range %?", range);
        for self.iter_indivisible_pieces_for_range(range) |piece_range| {
            debug!("iterated on %?", piece_range);
            let metrics = self.font.measure_text(self, piece_range);
            max_piece_width = Au::max(max_piece_width, metrics.advance_width);
        }
        return max_piece_width;
    }

    fn iter_natural_lines_for_range(&self, range: &Range, f: &fn(&Range) -> bool) -> bool {
        let mut clump = Range::new(range.begin(), 0);
        let mut in_clump = false;

        // clump non-linebreaks of nonzero length
        for range.eachi |i| {
            match (self.glyphs.char_is_newline(i), in_clump) {
                (false, true)  => { clump.extend_by(1); }
                (false, false) => { in_clump = true; clump.reset(i, 1); }
                (true, false) => { /* chomp whitespace */ }
                (true, true)  => {
                    in_clump = false;
                    // don't include the linebreak character itself in the clump.
                    if !f(&clump) { break }
                }
            }
        }
        
        // flush any remaining chars as a line
        if in_clump {
            clump.extend_to(range.end());
            f(&clump);
        }

        true
    }

    fn iter_indivisible_pieces_for_range(&self, range: &Range, f: &fn(&Range) -> bool) -> bool {
        let mut clump = Range::new(range.begin(), 0);

        loop {
            // extend clump to non-break-before characters.
            while clump.end() < range.end() 
                && self.glyphs.can_break_before(clump.end()) != BreakTypeNormal {

                clump.extend_by(1);
            }

            // now clump.end() is break-before or range.end()
            if !f(&clump) || clump.end() == range.end() {
                break
            }

            // now clump includes one break-before character, or starts from range.end()
            let end = clump.end(); // FIXME: borrow checker workaround
            clump.reset(end, 1);
        }

        true
    }
}
