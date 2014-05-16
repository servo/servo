/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use font::{Font, FontDescriptor, RunMetrics, FontStyle, FontMetrics};
use servo_util::geometry::Au;
use servo_util::range::Range;
use std::slice::Items;
use style::computed_values::text_decoration;
use sync::Arc;
use text::glyph::{CharIndex, GlyphStore};

/// A text run.
#[deriving(Clone)]
pub struct TextRun {
    pub text: Arc<~str>,
    pub font_descriptor: FontDescriptor,
    pub font_metrics: FontMetrics,
    pub font_style: FontStyle,
    pub decoration: text_decoration::T,
    // An Arc pointing to a Vec of Arcs?! Wat.
    pub glyphs: Arc<Vec<Arc<GlyphStore>>>,
}

pub struct SliceIterator<'a> {
    glyph_iter: Items<'a, Arc<GlyphStore>>,
    range:      Range<CharIndex>,
    offset:     CharIndex,
}

impl<'a> Iterator<(&'a GlyphStore, CharIndex, Range<CharIndex>)> for SliceIterator<'a> {
    // inline(always) due to the inefficient rt failures messing up inline heuristics, I think.
    #[inline(always)]
    fn next(&mut self) -> Option<(&'a GlyphStore, CharIndex, Range<CharIndex>)> {
        loop {
            let slice_glyphs = self.glyph_iter.next();
            if slice_glyphs.is_none() {
                return None;
            }
            let slice_glyphs = slice_glyphs.unwrap();

            let slice_range = Range::new(self.offset, slice_glyphs.char_len());
            let mut char_range = self.range.intersect(&slice_range);
            char_range.shift_by(-self.offset);

            let old_offset = self.offset;
            self.offset = self.offset + slice_glyphs.char_len();
            if !char_range.is_empty() {
                return Some((&**slice_glyphs, old_offset, char_range))
            }
        }
    }
}

pub struct LineIterator<'a> {
    range:  Range<CharIndex>,
    clump:  Option<Range<CharIndex>>,
    slices: SliceIterator<'a>,
}

impl<'a> Iterator<Range<CharIndex>> for LineIterator<'a> {
    fn next(&mut self) -> Option<Range<CharIndex>> {
        // Loop until we hit whitespace and are in a clump.
        loop {
            match self.slices.next() {
                Some((glyphs, offset, slice_range)) => {
                    match (glyphs.is_whitespace(), self.clump) {
                        (false, Some(ref mut c))  => {
                            c.extend_by(slice_range.length());
                        }
                        (false, None) => {
                            let mut c = slice_range;
                            c.shift_by(offset);
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

impl<'a> TextRun {
    pub fn new(font: &mut Font, text: ~str, decoration: text_decoration::T) -> TextRun {
        let glyphs = TextRun::break_and_shape(font, text);

        let run = TextRun {
            text: Arc::new(text),
            font_style: font.style.clone(),
            font_metrics: font.metrics.clone(),
            font_descriptor: font.get_descriptor(),
            decoration: decoration,
            glyphs: Arc::new(glyphs),
        };
        return run;
    }

    pub fn break_and_shape(font: &mut Font, text: &str) -> Vec<Arc<GlyphStore>> {
        // TODO(Issue #230): do a better job. See Gecko's LineBreaker.

        let mut glyphs = vec!();
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
                glyphs.push(font.shape_text(slice, !cur_slice_is_whitespace));
                byte_last_boundary = byte_i;
            }

            byte_i = next;
        }

        // Create a glyph store for the final slice if it's nonempty.
        if byte_i > byte_last_boundary {
            let slice = text.slice_from(byte_last_boundary).to_owned();
            debug!("creating glyph store for final slice {} (ws? {}), {} - {} in run {}",
                slice, cur_slice_is_whitespace, byte_last_boundary, text.len(), text);
            glyphs.push(font.shape_text(slice, cur_slice_is_whitespace));
        }

        glyphs
    }

    pub fn char_len(&self) -> CharIndex {
        self.glyphs.iter().fold(CharIndex(0), |len, slice_glyphs| {
            len + slice_glyphs.char_len()
        })
    }

    pub fn glyphs(&'a self) -> &'a Vec<Arc<GlyphStore>> {
        &*self.glyphs
    }

    pub fn range_is_trimmable_whitespace(&self, range: &Range<CharIndex>) -> bool {
        for (slice_glyphs, _, _) in self.iter_slices_for_range(range) {
            if !slice_glyphs.is_whitespace() { return false; }
        }
        true
    }

    pub fn ascent(&self) -> Au {
        self.font_metrics.ascent
    }

    pub fn descent(&self) -> Au {
        self.font_metrics.descent
    }

    pub fn advance_for_range(&self, range: &Range<CharIndex>) -> Au {
        // TODO(Issue #199): alter advance direction for RTL
        // TODO(Issue #98): using inter-char and inter-word spacing settings  when measuring text
        self.iter_slices_for_range(range)
            .fold(Au(0), |advance, (glyphs, _, slice_range)| {
                advance + glyphs.advance_for_char_range(&slice_range)
            })
    }

    pub fn metrics_for_range(&self, range: &Range<CharIndex>) -> RunMetrics {
        RunMetrics::new(self.advance_for_range(range),
                        self.font_metrics.ascent,
                        self.font_metrics.descent)
    }

    pub fn metrics_for_slice(&self, glyphs: &GlyphStore, slice_range: &Range<CharIndex>) -> RunMetrics {
        RunMetrics::new(glyphs.advance_for_char_range(slice_range),
                        self.font_metrics.ascent,
                        self.font_metrics.descent)
    }

    pub fn min_width_for_range(&self, range: &Range<CharIndex>) -> Au {
        let mut max_piece_width = Au(0);
        debug!("iterating outer range {:?}", range);
        for (_, offset, slice_range) in self.iter_slices_for_range(range) {
            debug!("iterated on {:?}[{:?}]", offset, slice_range);
            max_piece_width = Au::max(max_piece_width, self.advance_for_range(&slice_range));
        }
        max_piece_width
    }

    pub fn iter_slices_for_range(&'a self, range: &Range<CharIndex>) -> SliceIterator<'a> {
        SliceIterator {
            glyph_iter: self.glyphs.iter(),
            range:      *range,
            offset:     CharIndex(0),
        }
    }

    pub fn iter_natural_lines_for_range(&'a self, range: &Range<CharIndex>) -> LineIterator<'a> {
        LineIterator {
            range:  *range,
            clump:  None,
            slices: self.iter_slices_for_range(range),
        }
    }
}
