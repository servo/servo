/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use font_context::FontContext;
use geometry::Au;
use text::glyph::GlyphStore;
use font::{Font, FontDescriptor, RunMetrics};
use servo_util::range::Range;
use extra::arc::ARC;

/// A text run.
pub struct TextRun {
    text: ~str,
    font: @mut Font,
    underline: bool,
    glyphs: ~[ARC<GlyphStore>],
}

/// This is a hack until TextRuns are normally sendable, or we instead use ARC<TextRun> everywhere.
pub struct SendableTextRun {
    text: ~str,
    font: FontDescriptor,
    underline: bool,
    priv glyphs: ~[ARC<GlyphStore>],
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
            underline: self.underline,
            glyphs: self.glyphs.clone(),
        }
    }
}

impl<'self> TextRun {
    pub fn new(font: @mut Font, text: ~str, underline: bool) -> TextRun {
        let glyphs = TextRun::break_and_shape(font, text);

        let run = TextRun {
            text: text,
            font: font,
            underline: underline,
            glyphs: glyphs,
        };
        return run;
    }

    pub fn teardown(&self) {
        self.font.teardown();
    }

    pub fn break_and_shape(font: @mut Font, text: &str) -> ~[ARC<GlyphStore>] {
        // TODO(Issue #230): do a better job. See Gecko's LineBreaker.

        let mut glyphs = ~[];
        let mut byte_i = 0u;
        let mut cur_slice_is_whitespace = false;
        let mut byte_last_boundary = 0;
        while byte_i < text.len() {
            let range = text.char_range_at(byte_i);
            let ch = range.ch;
            let next = range.next;

            // Slices alternate between whitespace and non-whitespace,
            // representing line break opportunities.
            let can_break_before = if cur_slice_is_whitespace {
                match ch {
                    ' ' | '\t' | '\n' => false,
                    _ => {
                        cur_slice_is_whitespace = false;
                        true
                    }
                }
            } else {
                match ch {
                    ' ' | '\t' | '\n' => {
                        cur_slice_is_whitespace = true;
                        true
                    },
                    _ => false
                }
            };

            // Create a glyph store for this slice if it's nonempty.
            if can_break_before && byte_i > byte_last_boundary {
                let slice = text.slice(byte_last_boundary, byte_i).to_owned();
                debug!("creating glyph store for slice %? (ws? %?), %? - %? in run %?",
                        slice, !cur_slice_is_whitespace, byte_last_boundary, byte_i, text);
                glyphs.push(font.shape_text(slice, !cur_slice_is_whitespace));
                byte_last_boundary = byte_i;
            }

            byte_i = next;
        }

        // Create a glyph store for the final slice if it's nonempty.
        if byte_i > byte_last_boundary {
            let slice = text.slice(byte_last_boundary, text.len()).to_owned();
            debug!("creating glyph store for final slice %? (ws? %?), %? - %? in run %?",
                slice, cur_slice_is_whitespace, byte_last_boundary, text.len(), text);
            glyphs.push(font.shape_text(slice, cur_slice_is_whitespace));
        }

        glyphs
    }

    pub fn serialize(&self) -> SendableTextRun {
        SendableTextRun {
            text: copy self.text,
            font: self.font.get_descriptor(),
            underline: self.underline,
            glyphs: self.glyphs.clone(),
        }
    }

    pub fn char_len(&self) -> uint {
        do self.glyphs.iter().fold(0u) |len, slice_glyphs| {
            len + slice_glyphs.get().char_len()
        }
    }

    pub fn glyphs(&'self self) -> &'self ~[ARC<GlyphStore>] { &self.glyphs }

    pub fn range_is_trimmable_whitespace(&self, range: &Range) -> bool {
        for self.iter_slices_for_range(range) |slice_glyphs, _, _| {
            if !slice_glyphs.is_whitespace() { return false; }
        }
        true
    }

    pub fn metrics_for_range(&self, range: &Range) -> RunMetrics {
        self.font.measure_text(self, range)
    }

    pub fn metrics_for_slice(&self, glyphs: &GlyphStore, slice_range: &Range) -> RunMetrics {
        self.font.measure_text_for_slice(glyphs, slice_range)
    }

    pub fn min_width_for_range(&self, range: &Range) -> Au {
        let mut max_piece_width = Au(0);
        debug!("iterating outer range %?", range);
        for self.iter_slices_for_range(range) |glyphs, offset, slice_range| {
            debug!("iterated on %?[%?]", offset, slice_range);
            let metrics = self.font.measure_text_for_slice(glyphs, slice_range);
            max_piece_width = Au::max(max_piece_width, metrics.advance_width);
        }
        max_piece_width
    }

    pub fn iter_slices_for_range(&self,
                                 range: &Range,
                                 f: &fn(&GlyphStore, uint, &Range) -> bool)
                                 -> bool {
        let mut offset = 0;
        for self.glyphs.iter().advance |slice_glyphs| {
            // Determine the range of this slice that we need.
            let slice_range = Range::new(offset, slice_glyphs.get().char_len());
            let mut char_range = range.intersect(&slice_range);
            char_range.shift_by(-(offset.to_int()));

            let unwrapped_glyphs = slice_glyphs.get();
            if !char_range.is_empty() {
                if !f(unwrapped_glyphs, offset, &char_range) { break }
            }
            offset += unwrapped_glyphs.char_len();
        }
        true
    }

    pub fn iter_natural_lines_for_range(&self, range: &Range, f: &fn(&Range) -> bool) -> bool {
        let mut clump = Range::new(range.begin(), 0);
        let mut in_clump = false;

        for self.iter_slices_for_range(range) |glyphs, offset, slice_range| {
            match (glyphs.is_whitespace(), in_clump) {
                (false, true)  => { clump.extend_by(slice_range.length().to_int()); }
                (false, false) => {
                    in_clump = true;
                    clump = *slice_range;
                    clump.shift_by(offset.to_int());
                }
                (true, false)  => { /* chomp whitespace */ }
                (true, true)   => {
                    in_clump = false;
                    // The final whitespace clump is not included.
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
}
