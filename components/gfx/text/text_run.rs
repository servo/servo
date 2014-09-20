/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use font::{Font, RunMetrics, FontMetrics};
use servo_util::geometry::Au;
use servo_util::range::Range;
use servo_util::vec::{Comparator, FullBinarySearchMethods};
use std::slice::Items;
use sync::Arc;
use text::glyph::{CharIndex, GlyphStore};
use font::FontHandleMethods;
use platform::font_template::FontTemplateData;

/// A single "paragraph" of text in one font size and style.
#[deriving(Clone)]
pub struct TextRun {
    pub text: Arc<String>,
    pub font_template: Arc<FontTemplateData>,
    pub pt_size: f64,
    pub font_metrics: FontMetrics,
    /// The glyph runs that make up this text run.
    pub glyphs: Arc<Vec<GlyphRun>>,
}

/// A single series of glyphs within a text run.
#[deriving(Clone)]
pub struct GlyphRun {
    /// The glyphs.
    glyph_store: Arc<GlyphStore>,
    /// The range of characters in the containing run.
    range: Range<CharIndex>,
}

pub struct SliceIterator<'a> {
    glyph_iter: Items<'a, GlyphRun>,
    range:      Range<CharIndex>,
}

struct CharIndexComparator;

impl Comparator<CharIndex,GlyphRun> for CharIndexComparator {
    fn compare(&self, key: &CharIndex, value: &GlyphRun) -> Ordering {
        if *key < value.range.begin() {
            Less
        } else if *key >= value.range.end() {
            Greater
        } else {
            Equal
        }
    }
}

impl<'a> Iterator<(&'a GlyphStore, CharIndex, Range<CharIndex>)> for SliceIterator<'a> {
    // inline(always) due to the inefficient rt failures messing up inline heuristics, I think.
    #[inline(always)]
    fn next(&mut self) -> Option<(&'a GlyphStore, CharIndex, Range<CharIndex>)> {
        let slice_glyphs = self.glyph_iter.next();
        if slice_glyphs.is_none() {
            return None;
        }
        let slice_glyphs = slice_glyphs.unwrap();

        let mut char_range = self.range.intersect(&slice_glyphs.range);
        let slice_range_begin = slice_glyphs.range.begin();
        char_range.shift_by(-slice_range_begin);
        if !char_range.is_empty() {
            return Some((&*slice_glyphs.glyph_store, slice_range_begin, char_range))
        }

        return None;
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
                        let mut c = self.clump.take().unwrap();
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
    pub fn new(font: &mut Font, text: String) -> TextRun {
        let glyphs = TextRun::break_and_shape(font, text.as_slice());
        let run = TextRun {
            text: Arc::new(text),
            font_metrics: font.metrics.clone(),
            font_template: font.handle.get_template(),
            pt_size: font.pt_size,
            glyphs: Arc::new(glyphs),
        };
        return run;
    }

    pub fn break_and_shape(font: &mut Font, text: &str) -> Vec<GlyphRun> {
        // TODO(Issue #230): do a better job. See Gecko's LineBreaker.
        let mut glyphs = vec!();
        let (mut byte_i, mut char_i) = (0u, CharIndex(0));
        let mut cur_slice_is_whitespace = false;
        let (mut byte_last_boundary, mut char_last_boundary) = (0, CharIndex(0));
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
                let slice = text.slice(byte_last_boundary, byte_i).to_string();
                debug!("creating glyph store for slice {} (ws? {}), {} - {} in run {}",
                        slice, !cur_slice_is_whitespace, byte_last_boundary, byte_i, text);
                glyphs.push(GlyphRun {
                    glyph_store: font.shape_text(slice, !cur_slice_is_whitespace),
                    range: Range::new(char_last_boundary, char_i - char_last_boundary),
                });
                byte_last_boundary = byte_i;
                char_last_boundary = char_i;
            }

            byte_i = next;
            char_i = char_i + CharIndex(1);
        }

        // Create a glyph store for the final slice if it's nonempty.
        if byte_i > byte_last_boundary {
            let slice = text.slice_from(byte_last_boundary).to_string();
            debug!("creating glyph store for final slice {} (ws? {}), {} - {} in run {}",
                slice, cur_slice_is_whitespace, byte_last_boundary, text.len(), text);
            glyphs.push(GlyphRun {
                glyph_store: font.shape_text(slice, cur_slice_is_whitespace),
                range: Range::new(char_last_boundary, char_i - char_last_boundary),
            });
        }

        glyphs
    }

    pub fn char_len(&self) -> CharIndex {
        match self.glyphs.last() {
            None => CharIndex(0),
            Some(ref glyph_run) => glyph_run.range.end(),
        }
    }

    pub fn glyphs(&'a self) -> &'a Vec<GlyphRun> {
        &*self.glyphs
    }

    pub fn range_is_trimmable_whitespace(&self, range: &Range<CharIndex>) -> bool {
        self.iter_slices_for_range(range).all(|(slice_glyphs, _, _)| {
            slice_glyphs.is_whitespace()
        })
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
        debug!("iterating outer range {:?}", range);
        self.iter_slices_for_range(range).fold(Au(0), |max_piece_width, (_, offset, slice_range)| {
            debug!("iterated on {:?}[{:?}]", offset, slice_range);
            Au::max(max_piece_width, self.advance_for_range(&slice_range))
        })
    }

    /// Returns the index of the first glyph run containing the given character index.
    fn index_of_first_glyph_run_containing(&self, index: CharIndex) -> Option<uint> {
        self.glyphs.as_slice().binary_search_index_by(&index, CharIndexComparator)
    }

    pub fn iter_slices_for_range(&'a self, range: &Range<CharIndex>) -> SliceIterator<'a> {
        let index = match self.index_of_first_glyph_run_containing(range.begin()) {
            None => self.glyphs.len(),
            Some(index) => index,
        };
        SliceIterator {
            glyph_iter: self.glyphs.slice_from(index).iter(),
            range:      *range,
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
