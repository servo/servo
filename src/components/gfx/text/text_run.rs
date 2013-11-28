/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use font::{Font, FontDescriptor, RunMetrics};
use font_context::FontContext;
use text::glyph::GlyphStore;

use extra::arc::{Arc, MutexArc};
use servo_util::geometry::Au;
use servo_util::range::Range;
use servo_util::sync::MutexArcUtils;
use std::vec::VecIterator;
use style::computed_values::text_decoration;

/// A text run.
pub struct TextRun {
    text: Arc<~str>,
    font: MutexArc<Font>,
    decoration: text_decoration::T,
    glyphs: Arc<~[Arc<GlyphStore>]>,
}

/// The same as a text run, but with a font descriptor instead of a font. This makes them thread
/// safe.
pub struct SendableTextRun {
    text: Arc<~str>,
    font: FontDescriptor,
    decoration: text_decoration::T,
    priv glyphs: Arc<~[Arc<GlyphStore>]>,
}

impl SendableTextRun {
    pub fn deserialize(&self, fctx: @mut FontContext) -> TextRun {
        let font = match fctx.get_font_by_descriptor(&self.font) {
            Ok(f) => f,
            Err(_) => fail!("Font descriptor deserialization failed! desc={:?}", self.font)
        };

        TextRun {
            text: self.text.clone(),
            font: font,
            decoration: self.decoration,
            glyphs: self.glyphs.clone(),
        }
    }
}

pub struct SliceIterator<'self> {
    priv glyph_iter: VecIterator<'self, Arc<GlyphStore>>,
    priv range:      Range,
    priv offset:     uint,
}

impl<'self> Iterator<(&'self GlyphStore, uint, Range)> for SliceIterator<'self> {
    // inline(always) due to the inefficient rt failures messing up inline heuristics, I think.
    #[inline(always)]
    fn next(&mut self) -> Option<(&'self GlyphStore, uint, Range)> {
        loop {
            let slice_glyphs = self.glyph_iter.next();
            if slice_glyphs.is_none() {
                return None;
            }
            let slice_glyphs = slice_glyphs.unwrap().get();

            let slice_range = Range::new(self.offset, slice_glyphs.char_len());
            let mut char_range = self.range.intersect(&slice_range);
            char_range.shift_by(-(self.offset.to_int().unwrap()));

            let old_offset = self.offset;
            self.offset += slice_glyphs.char_len();
            if !char_range.is_empty() {
                return Some((slice_glyphs, old_offset, char_range))
            }
        }
    }
}

pub struct LineIterator<'self> {
    priv range:  Range,
    priv clump:  Option<Range>,
    priv slices: SliceIterator<'self>,
}

impl<'self> Iterator<Range> for LineIterator<'self> {
    fn next(&mut self) -> Option<Range> {
        // Loop until we hit whitespace and are in a clump.
        loop {
            match self.slices.next() {
                Some((glyphs, offset, slice_range)) => {
                    match (glyphs.is_whitespace(), self.clump) {
                        (false, Some(ref mut c))  => {
                            c.extend_by(slice_range.length().to_int().unwrap());
                        }
                        (false, None) => {
                            let mut c = slice_range;
                            c.shift_by(offset.to_int().unwrap());
                            self.clump = Some(c);
                        }
                        (true, None)    => { /* chomp whitespace */ }
                        (true, Some(c)) => {
                            self.clump = None;
                            // The final whitespace clump is not included.
                            return Some(c);
                        }
                    }
                },
                None => {
                    // flush any remaining chars as a line
                    if self.clump.is_some() {
                        let mut c = self.clump.take_unwrap();
                        c.extend_to(self.range.end());
                        return Some(c);
                    } else {
                        return None;
                    }
                }
            }
        }
    }
}

impl<'self> TextRun {
    pub fn new(font: MutexArc<Font>, text: ~str, decoration: text_decoration::T) -> TextRun {
        let glyphs = TextRun::break_and_shape(font.clone(), text);

        let run = TextRun {
            text: Arc::new(text),
            font: font,
            decoration: decoration,
            glyphs: Arc::new(glyphs),
        };
        return run;
    }

    pub fn teardown(&self) {
        self.font.force_access(|font| font.teardown());
    }

    pub fn break_and_shape(font: MutexArc<Font>, text: &str) -> ~[Arc<GlyphStore>] {
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
                debug!("creating glyph store for slice {} (ws? {}), {} - {} in run {}",
                        slice, !cur_slice_is_whitespace, byte_last_boundary, byte_i, text);
                glyphs.push(Font::shape_text(font.clone(), slice, !cur_slice_is_whitespace));
                byte_last_boundary = byte_i;
            }

            byte_i = next;
        }

        // Create a glyph store for the final slice if it's nonempty.
        if byte_i > byte_last_boundary {
            let slice = text.slice_from(byte_last_boundary).to_owned();
            debug!("creating glyph store for final slice {} (ws? {}), {} - {} in run {}",
                slice, cur_slice_is_whitespace, byte_last_boundary, text.len(), text);
            glyphs.push(Font::shape_text(font.clone(), slice, cur_slice_is_whitespace));
        }

        glyphs
    }

    pub fn serialize(&self) -> SendableTextRun {
        SendableTextRun {
            text: self.text.clone(),
            font: self.font.force_access(|font| font.get_descriptor()),
            decoration: self.decoration,
            glyphs: self.glyphs.clone(),
        }
    }

    pub fn char_len(&self) -> uint {
        do self.glyphs.get().iter().fold(0u) |len, slice_glyphs| {
            len + slice_glyphs.get().char_len()
        }
    }

    pub fn glyphs(&'self self) -> &'self ~[Arc<GlyphStore>] {
        self.glyphs.get()
    }

    pub fn range_is_trimmable_whitespace(&self, range: &Range) -> bool {
        for (slice_glyphs, _, _) in self.iter_slices_for_range(range) {
            if !slice_glyphs.is_whitespace() { return false; }
        }
        true
    }

    // NB: Takes a lock on the font temporarily.
    pub fn metrics_for_range(&self, range: &Range) -> RunMetrics {
        self.font.force_access(|font| font.measure_text(self, range))
    }

    // NB: Takes a lock on the font temporarily.
    pub fn metrics_for_slice(&self, glyphs: &GlyphStore, slice_range: &Range) -> RunMetrics {
        self.font.force_access(|font| font.measure_text_for_slice(glyphs, slice_range))
    }

    pub fn min_width_for_range(&self, range: &Range) -> Au {
        let mut max_piece_width = Au(0);
        debug!("iterating outer range {:?}", range);
        self.font.force_access(|font| {
            for (glyphs, offset, slice_range) in self.iter_slices_for_range(range) {
                debug!("iterated on {:?}[{:?}]", offset, slice_range);
                let metrics = font.measure_text_for_slice(glyphs, &slice_range);
                max_piece_width = Au::max(max_piece_width, metrics.advance_width);
            }
        });
        max_piece_width
    }

    pub fn iter_slices_for_range(&'self self, range: &Range) -> SliceIterator<'self> {
        SliceIterator {
            glyph_iter: self.glyphs.get().iter(),
            range:      *range,
            offset:     0,
        }
    }

    pub fn iter_natural_lines_for_range(&'self self, range: &Range) -> LineIterator<'self> {
        LineIterator {
            range:  *range,
            clump:  None,
            slices: self.iter_slices_for_range(range),
        }
    }
}
