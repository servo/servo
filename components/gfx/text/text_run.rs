/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use font::{Font, FontHandleMethods, FontMetrics, IS_WHITESPACE_SHAPING_FLAG, RunMetrics};
use font::{ShapingOptions};
use platform::font_template::FontTemplateData;
use text::glyph::{CharIndex, GlyphStore};

use util::geometry::Au;
use util::range::Range;
use util::vec::{Comparator, FullBinarySearchMethods};

use std::cmp::{Ordering, max};
use std::slice::Iter;
use std::sync::Arc;

/// A single "paragraph" of text in one font size and style.
#[derive(Clone)]
pub struct TextRun {
    /// The UTF-8 string represented by this text run.
    pub text: Arc<String>,
    pub font_template: Arc<FontTemplateData>,
    pub actual_pt_size: Au,
    pub font_metrics: FontMetrics,
    /// The glyph runs that make up this text run.
    pub glyphs: Arc<Vec<GlyphRun>>,
    pub bidi_level: u8,
}

/// A single series of glyphs within a text run.
#[derive(Clone)]
pub struct GlyphRun {
    /// The glyphs.
    pub glyph_store: Arc<GlyphStore>,
    /// The range of characters in the containing run.
    pub range: Range<CharIndex>,
}

pub struct NaturalWordSliceIterator<'a> {
    glyph_iter: Iter<'a, GlyphRun>,
    range: Range<CharIndex>,
}

struct CharIndexComparator;

impl Comparator<CharIndex,GlyphRun> for CharIndexComparator {
    fn compare(&self, key: &CharIndex, value: &GlyphRun) -> Ordering {
        if *key < value.range.begin() {
            Ordering::Less
        } else if *key >= value.range.end() {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }
}

/// A "slice" of a text run is a series of contiguous glyphs that all belong to the same glyph
/// store. Line breaking strategies yield these.
pub struct TextRunSlice<'a> {
    /// The glyph store that the glyphs in this slice belong to.
    pub glyphs: &'a GlyphStore,
    /// The character index that this slice begins at, relative to the start of the *text run*.
    pub offset: CharIndex,
    /// The range that these glyphs encompass, relative to the start of the *glyph store*.
    pub range: Range<CharIndex>,
}

impl<'a> TextRunSlice<'a> {
    /// Returns the range that these glyphs encompass, relative to the start of the *text run*.
    #[inline]
    pub fn text_run_range(&self) -> Range<CharIndex> {
        let mut range = self.range;
        range.shift_by(self.offset);
        range
    }
}

impl<'a> Iterator for NaturalWordSliceIterator<'a> {
    type Item = TextRunSlice<'a>;

    // inline(always) due to the inefficient rt failures messing up inline heuristics, I think.
    #[inline(always)]
    fn next(&mut self) -> Option<TextRunSlice<'a>> {
        let slice_glyphs = self.glyph_iter.next();
        if slice_glyphs.is_none() {
            return None;
        }
        let slice_glyphs = slice_glyphs.unwrap();

        let mut char_range = self.range.intersect(&slice_glyphs.range);
        let slice_range_begin = slice_glyphs.range.begin();
        char_range.shift_by(-slice_range_begin);

        if !char_range.is_empty() {
            Some(TextRunSlice {
                glyphs: &*slice_glyphs.glyph_store,
                offset: slice_range_begin,
                range: char_range,
            })
        } else {
            None
        }
    }
}

pub struct CharacterSliceIterator<'a> {
    glyph_run: Option<&'a GlyphRun>,
    glyph_run_iter: Iter<'a, GlyphRun>,
    range: Range<CharIndex>,
}

impl<'a> Iterator for CharacterSliceIterator<'a> {
    type Item = TextRunSlice<'a>;

    // inline(always) due to the inefficient rt failures messing up inline heuristics, I think.
    #[inline(always)]
    fn next(&mut self) -> Option<TextRunSlice<'a>> {
        let glyph_run = match self.glyph_run {
            None => return None,
            Some(glyph_run) => glyph_run,
        };

        debug_assert!(!self.range.is_empty());
        let index_to_return = self.range.begin();
        self.range.adjust_by(CharIndex(1), CharIndex(-1));
        if self.range.is_empty() {
            // We're done.
            self.glyph_run = None
        } else if self.range.intersect(&glyph_run.range).is_empty() {
            // Move on to the next glyph run.
            self.glyph_run = self.glyph_run_iter.next();
        }

        let index_within_glyph_run = index_to_return - glyph_run.range.begin();
        Some(TextRunSlice {
            glyphs: &*glyph_run.glyph_store,
            offset: glyph_run.range.begin(),
            range: Range::new(index_within_glyph_run, CharIndex(1)),
        })
    }
}

pub struct LineIterator<'a> {
    range: Range<CharIndex>,
    clump: Option<Range<CharIndex>>,
    slices: NaturalWordSliceIterator<'a>,
}

impl<'a> Iterator for LineIterator<'a> {
    type Item = Range<CharIndex>;

    fn next(&mut self) -> Option<Range<CharIndex>> {
        // Loop until we hit whitespace and are in a clump.
        loop {
            match self.slices.next() {
                Some(slice) => {
                    match (slice.glyphs.is_whitespace(), self.clump) {
                        (false, Some(ref mut c))  => {
                            c.extend_by(slice.range.length());
                        }
                        (false, None) => {
                            let mut range = slice.range;
                            range.shift_by(slice.offset);
                            self.clump = Some(range);
                        }
                        (true, None)    => { /* chomp whitespace */ }
                        (true, Some(clump)) => {
                            self.clump = None;
                            // The final whitespace clump is not included.
                            return Some(clump);
                        }
                    }
                }
                None => {
                    // Flush any remaining characters as a line.
                    if self.clump.is_some() {
                        let mut range = self.clump.take().unwrap();
                        range.extend_to(self.range.end());
                        return Some(range);
                    } else {
                        return None;
                    }
                }
            }
        }
    }
}

impl<'a> TextRun {
    pub fn new(font: &mut Font, text: String, options: &ShapingOptions, bidi_level: u8) -> TextRun {
        let glyphs = TextRun::break_and_shape(font, &text, options);
        let run = TextRun {
            text: Arc::new(text),
            font_metrics: font.metrics.clone(),
            font_template: font.handle.template(),
            actual_pt_size: font.actual_pt_size,
            glyphs: Arc::new(glyphs),
            bidi_level: bidi_level,
        };
        return run;
    }

    pub fn break_and_shape(font: &mut Font, text: &str, options: &ShapingOptions)
                           -> Vec<GlyphRun> {
        // TODO(Issue #230): do a better job. See Gecko's LineBreaker.
        let mut glyphs = vec!();
        let (mut byte_i, mut char_i) = (0, CharIndex(0));
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
                let slice = &text[byte_last_boundary .. byte_i];
                debug!("creating glyph store for slice {} (ws? {}), {} - {} in run {}",
                        slice, !cur_slice_is_whitespace, byte_last_boundary, byte_i, text);

                let mut options = *options;
                if !cur_slice_is_whitespace {
                    options.flags.insert(IS_WHITESPACE_SHAPING_FLAG);
                }

                glyphs.push(GlyphRun {
                    glyph_store: font.shape_text(slice, &options),
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
            let slice = &text[byte_last_boundary..];
            debug!("creating glyph store for final slice {} (ws? {}), {} - {} in run {}",
                slice, cur_slice_is_whitespace, byte_last_boundary, text.len(), text);

            let mut options = *options;
            if cur_slice_is_whitespace {
                options.flags.insert(IS_WHITESPACE_SHAPING_FLAG);
            }

            glyphs.push(GlyphRun {
                glyph_store: font.shape_text(slice, &options),
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
        self.natural_word_slices_in_range(range).all(|slice| slice.glyphs.is_whitespace())
    }

    pub fn ascent(&self) -> Au {
        self.font_metrics.ascent
    }

    pub fn descent(&self) -> Au {
        self.font_metrics.descent
    }

    pub fn advance_for_range(&self, range: &Range<CharIndex>) -> Au {
        // TODO(Issue #199): alter advance direction for RTL
        // TODO(Issue #98): using inter-char and inter-word spacing settings when measuring text
        self.natural_word_slices_in_range(range)
            .fold(Au(0), |advance, slice| {
                advance + slice.glyphs.advance_for_char_range(&slice.range)
            })
    }

    pub fn metrics_for_range(&self, range: &Range<CharIndex>) -> RunMetrics {
        RunMetrics::new(self.advance_for_range(range),
                        self.font_metrics.ascent,
                        self.font_metrics.descent)
    }

    pub fn metrics_for_slice(&self, glyphs: &GlyphStore, slice_range: &Range<CharIndex>)
                             -> RunMetrics {
        RunMetrics::new(glyphs.advance_for_char_range(slice_range),
                        self.font_metrics.ascent,
                        self.font_metrics.descent)
    }

    pub fn min_width_for_range(&self, range: &Range<CharIndex>) -> Au {
        debug!("iterating outer range {:?}", range);
        self.natural_word_slices_in_range(range).fold(Au(0), |max_piece_width, slice| {
            debug!("iterated on {:?}[{:?}]", slice.offset, slice.range);
            max(max_piece_width, self.advance_for_range(&slice.range))
        })
    }

    /// Returns the first glyph run containing the given character index.
    pub fn first_glyph_run_containing(&'a self, index: CharIndex) -> Option<&'a GlyphRun> {
        self.index_of_first_glyph_run_containing(index).map(|index| &self.glyphs[index])
    }

    /// Returns the index of the first glyph run containing the given character index.
    fn index_of_first_glyph_run_containing(&self, index: CharIndex) -> Option<usize> {
        (&**self.glyphs).binary_search_index_by(&index, CharIndexComparator)
    }

    /// Returns an iterator that will iterate over all slices of glyphs that represent natural
    /// words in the given range.
    pub fn natural_word_slices_in_range(&'a self, range: &Range<CharIndex>)
                                        -> NaturalWordSliceIterator<'a> {
        let index = match self.index_of_first_glyph_run_containing(range.begin()) {
            None => self.glyphs.len(),
            Some(index) => index,
        };
        NaturalWordSliceIterator {
            glyph_iter: self.glyphs[index..].iter(),
            range: *range,
        }
    }

    /// Returns an iterator that will iterate over all slices of glyphs that represent individual
    /// characters in the given range.
    pub fn character_slices_in_range(&'a self, range: &Range<CharIndex>)
                                     -> CharacterSliceIterator<'a> {
        let index = match self.index_of_first_glyph_run_containing(range.begin()) {
            None => self.glyphs.len(),
            Some(index) => index,
        };
        let mut glyph_run_iter = self.glyphs[index..].iter();
        let first_glyph_run = glyph_run_iter.next();
        CharacterSliceIterator {
            glyph_run: first_glyph_run,
            glyph_run_iter: glyph_run_iter,
            range: *range,
        }
    }

    pub fn iter_natural_lines_for_range(&'a self, range: &Range<CharIndex>) -> LineIterator<'a> {
        LineIterator {
            range: *range,
            clump: None,
            slices: self.natural_word_slices_in_range(range),
        }
    }
}
