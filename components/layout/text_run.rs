/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::cmp::max;
use std::slice::Iter;
use std::sync::Arc;

use app_units::Au;
use fonts::{
    ByteIndex, FontMetrics, FontRef, GlyphRun, GlyphStore, RunMetrics, ShapingFlags, ShapingOptions,
};
use log::debug;
use range::Range;
use serde::{Deserialize, Serialize};
use style::str::char_is_whitespace;
use unicode_bidi as bidi;
use webrender_api::FontInstanceKey;
use xi_unicode::LineBreakLeafIter;

thread_local! {
    static INDEX_OF_FIRST_GLYPH_RUN_CACHE: Cell<Option<(*const TextRun, ByteIndex, usize)>> =
        const { Cell::new(None) }
}

/// A single "paragraph" of text in one font size and style.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TextRun {
    /// The UTF-8 string represented by this text run.
    pub text: Arc<String>,
    pub pt_size: Au,
    pub font_metrics: FontMetrics,
    pub font_key: FontInstanceKey,
    /// The glyph runs that make up this text run.
    pub glyphs: Arc<Vec<GlyphRun>>,
    pub bidi_level: bidi::Level,
    pub extra_word_spacing: Au,
}

impl Drop for TextRun {
    fn drop(&mut self) {
        // Invalidate the glyph run cache if it was our text run that got freed.
        INDEX_OF_FIRST_GLYPH_RUN_CACHE.with(|index_of_first_glyph_run_cache| {
            if let Some((text_run_ptr, _, _)) = index_of_first_glyph_run_cache.get() {
                if text_run_ptr == (self as *const TextRun) {
                    index_of_first_glyph_run_cache.set(None);
                }
            }
        })
    }
}

pub struct NaturalWordSliceIterator<'a> {
    glyphs: &'a [GlyphRun],
    index: usize,
    range: Range<ByteIndex>,
    reverse: bool,
}

/// A "slice" of a text run is a series of contiguous glyphs that all belong to the same glyph
/// store. Line breaking strategies yield these.
pub struct TextRunSlice<'a> {
    /// The glyph store that the glyphs in this slice belong to.
    pub glyphs: &'a GlyphStore,
    /// The byte index that this slice begins at, relative to the start of the *text run*.
    pub offset: ByteIndex,
    /// The range that these glyphs encompass, relative to the start of the *glyph store*.
    pub range: Range<ByteIndex>,
}

impl<'a> TextRunSlice<'a> {
    /// Returns the range that these glyphs encompass, relative to the start of the *text run*.
    #[inline]
    pub fn text_run_range(&self) -> Range<ByteIndex> {
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
        let slice_glyphs;
        if self.reverse {
            if self.index == 0 {
                return None;
            }
            self.index -= 1;
            slice_glyphs = &self.glyphs[self.index];
        } else {
            if self.index >= self.glyphs.len() {
                return None;
            }
            slice_glyphs = &self.glyphs[self.index];
            self.index += 1;
        }

        let mut byte_range = self.range.intersect(&slice_glyphs.range);
        let slice_range_begin = slice_glyphs.range.begin();
        byte_range.shift_by(-slice_range_begin);

        if !byte_range.is_empty() {
            Some(TextRunSlice {
                glyphs: &slice_glyphs.glyph_store,
                offset: slice_range_begin,
                range: byte_range,
            })
        } else {
            None
        }
    }
}

pub struct CharacterSliceIterator<'a> {
    text: &'a str,
    glyph_run: Option<&'a GlyphRun>,
    glyph_run_iter: Iter<'a, GlyphRun>,
    range: Range<ByteIndex>,
}

impl<'a> Iterator for CharacterSliceIterator<'a> {
    type Item = TextRunSlice<'a>;

    // inline(always) due to the inefficient rt failures messing up inline heuristics, I think.
    #[inline(always)]
    fn next(&mut self) -> Option<TextRunSlice<'a>> {
        let glyph_run = self.glyph_run?;

        debug_assert!(!self.range.is_empty());
        let byte_start = self.range.begin();
        let byte_len = match self.text[byte_start.to_usize()..].chars().next() {
            Some(ch) => ByteIndex(ch.len_utf8() as isize),
            None => unreachable!(), // XXX refactor?
        };

        self.range.adjust_by(byte_len, -byte_len);
        if self.range.is_empty() {
            // We're done.
            self.glyph_run = None
        } else if self.range.intersect(&glyph_run.range).is_empty() {
            // Move on to the next glyph run.
            self.glyph_run = self.glyph_run_iter.next();
        }

        let index_within_glyph_run = byte_start - glyph_run.range.begin();
        Some(TextRunSlice {
            glyphs: &glyph_run.glyph_store,
            offset: glyph_run.range.begin(),
            range: Range::new(index_within_glyph_run, byte_len),
        })
    }
}

impl<'a> TextRun {
    /// Constructs a new text run. Also returns if there is a line break at the beginning
    pub fn new(
        font: FontRef,
        text: String,
        options: &ShapingOptions,
        bidi_level: bidi::Level,
        breaker: &mut Option<LineBreakLeafIter>,
    ) -> (TextRun, bool) {
        let (glyphs, break_at_zero) =
            TextRun::break_and_shape(font.clone(), &text, options, breaker);
        (
            TextRun {
                text: Arc::new(text),
                font_metrics: font.metrics.clone(),
                font_key: font.font_key,
                pt_size: font.descriptor.pt_size,
                glyphs: Arc::new(glyphs),
                bidi_level,
                extra_word_spacing: Au(0),
            },
            break_at_zero,
        )
    }

    pub fn break_and_shape(
        font: FontRef,
        text: &str,
        options: &ShapingOptions,
        breaker: &mut Option<LineBreakLeafIter>,
    ) -> (Vec<GlyphRun>, bool) {
        let mut glyphs = vec![];
        let mut slice = 0..0;

        let mut finished = false;
        let mut break_at_zero = false;

        if breaker.is_none() {
            if text.is_empty() {
                return (glyphs, true);
            }
            *breaker = Some(LineBreakLeafIter::new(text, 0));
        }

        let breaker = breaker.as_mut().unwrap();

        let mut push_range = |range: &std::ops::Range<usize>, options: &ShapingOptions| {
            glyphs.push(GlyphRun {
                glyph_store: font.shape_text(&text[range.clone()], options),
                range: Range::new(
                    ByteIndex(range.start as isize),
                    ByteIndex(range.len() as isize),
                ),
            });
        };

        while !finished {
            let (idx, _is_hard_break) = breaker.next(text);
            if idx == text.len() {
                finished = true;
            }
            if idx == 0 {
                break_at_zero = true;
            }

            // Extend the slice to the next UAX#14 line break opportunity.
            slice.end = idx;
            let word = &text[slice.clone()];

            // Split off any trailing whitespace into a separate glyph run.
            let mut whitespace = slice.end..slice.end;
            let mut rev_char_indices = word.char_indices().rev().peekable();
            let ends_with_newline = rev_char_indices.peek().map_or(false, |&(_, c)| c == '\n');
            if let Some((i, _)) = rev_char_indices
                .take_while(|&(_, c)| char_is_whitespace(c))
                .last()
            {
                whitespace.start = slice.start + i;
                slice.end = whitespace.start;
            } else if idx != text.len() && options.flags.contains(ShapingFlags::KEEP_ALL_FLAG) {
                // If there's no whitespace and word-break is set to
                // keep-all, try increasing the slice.
                continue;
            }
            if !slice.is_empty() {
                push_range(&slice, options);
            }
            if !whitespace.is_empty() {
                let mut options = *options;
                options
                    .flags
                    .insert(ShapingFlags::IS_WHITESPACE_SHAPING_FLAG);

                // The breaker breaks after every newline, so either there is none,
                // or there is exactly one at the very end. In the latter case,
                // split it into a different run. That's because shaping considers
                // a newline to have the same advance as a space, but during layout
                // we want to treat the newline as having no advance.
                if ends_with_newline {
                    whitespace.end -= 1;
                    if !whitespace.is_empty() {
                        push_range(&whitespace, &options);
                    }
                    whitespace.start = whitespace.end;
                    whitespace.end += 1;
                }
                push_range(&whitespace, &options);
            }
            slice.start = whitespace.end;
        }
        (glyphs, break_at_zero)
    }

    pub fn ascent(&self) -> Au {
        self.font_metrics.ascent
    }

    pub fn advance_for_range(&self, range: &Range<ByteIndex>) -> Au {
        if range.is_empty() {
            return Au(0);
        }

        // TODO(Issue #199): alter advance direction for RTL
        // TODO(Issue #98): using inter-char and inter-word spacing settings when measuring text
        self.natural_word_slices_in_range(range)
            .fold(Au(0), |advance, slice| {
                advance +
                    slice
                        .glyphs
                        .advance_for_byte_range(&slice.range, self.extra_word_spacing)
            })
    }

    pub fn metrics_for_range(&self, range: &Range<ByteIndex>) -> RunMetrics {
        RunMetrics::new(
            self.advance_for_range(range),
            self.font_metrics.ascent,
            self.font_metrics.descent,
        )
    }

    pub fn metrics_for_slice(
        &self,
        glyphs: &GlyphStore,
        slice_range: &Range<ByteIndex>,
    ) -> RunMetrics {
        RunMetrics::new(
            glyphs.advance_for_byte_range(slice_range, self.extra_word_spacing),
            self.font_metrics.ascent,
            self.font_metrics.descent,
        )
    }

    pub fn min_width_for_range(&self, range: &Range<ByteIndex>) -> Au {
        debug!("iterating outer range {:?}", range);
        self.natural_word_slices_in_range(range)
            .fold(Au(0), |max_piece_width, slice| {
                debug!("iterated on {:?}[{:?}]", slice.offset, slice.range);
                max(max_piece_width, self.advance_for_range(&slice.range))
            })
    }

    pub fn minimum_splittable_inline_size(&self, range: &Range<ByteIndex>) -> Au {
        match self.natural_word_slices_in_range(range).next() {
            None => Au(0),
            Some(slice) => self.advance_for_range(&slice.range),
        }
    }

    /// Returns the index of the first glyph run containing the given character index.
    fn index_of_first_glyph_run_containing(&self, index: ByteIndex) -> Option<usize> {
        let self_ptr = self as *const TextRun;
        INDEX_OF_FIRST_GLYPH_RUN_CACHE.with(|index_of_first_glyph_run_cache| {
            if let Some((last_text_run, last_index, last_result)) =
                index_of_first_glyph_run_cache.get()
            {
                if last_text_run == self_ptr && last_index == index {
                    return Some(last_result);
                }
            }

            if let Ok(result) = self
                .glyphs
                .binary_search_by(|current| current.compare(&index))
            {
                index_of_first_glyph_run_cache.set(Some((self_ptr, index, result)));
                Some(result)
            } else {
                None
            }
        })
    }

    pub fn on_glyph_run_boundary(&self, index: ByteIndex) -> bool {
        if let Some(glyph_index) = self.index_of_first_glyph_run_containing(index) {
            self.glyphs[glyph_index].range.begin() == index
        } else {
            true
        }
    }

    /// Returns the index in the range of the first glyph advancing over given advance
    pub fn range_index_of_advance(&self, range: &Range<ByteIndex>, advance: Au) -> usize {
        // TODO(Issue #199): alter advance direction for RTL
        // TODO(Issue #98): using inter-char and inter-word spacing settings when measuring text
        let mut remaining = advance;
        self.natural_word_slices_in_range(range)
            .map(|slice| {
                let (slice_index, slice_advance) = slice.glyphs.range_index_of_advance(
                    &slice.range,
                    remaining,
                    self.extra_word_spacing,
                );
                remaining -= slice_advance;
                slice_index
            })
            .sum()
    }

    /// Returns an iterator that will iterate over all slices of glyphs that represent natural
    /// words in the given range.
    pub fn natural_word_slices_in_range(
        &'a self,
        range: &Range<ByteIndex>,
    ) -> NaturalWordSliceIterator<'a> {
        let index = match self.index_of_first_glyph_run_containing(range.begin()) {
            None => self.glyphs.len(),
            Some(index) => index,
        };
        NaturalWordSliceIterator {
            glyphs: &self.glyphs[..],
            index,
            range: *range,
            reverse: false,
        }
    }

    /// Returns an iterator that over natural word slices in visual order (left to right or
    /// right to left, depending on the bidirectional embedding level).
    pub fn natural_word_slices_in_visual_order(
        &'a self,
        range: &Range<ByteIndex>,
    ) -> NaturalWordSliceIterator<'a> {
        // Iterate in reverse order if bidi level is RTL.
        let reverse = self.bidi_level.is_rtl();

        let index = if reverse {
            match self.index_of_first_glyph_run_containing(range.end() - ByteIndex(1)) {
                Some(i) => i + 1, // In reverse mode, index points one past the next element.
                None => 0,
            }
        } else {
            match self.index_of_first_glyph_run_containing(range.begin()) {
                Some(i) => i,
                None => self.glyphs.len(),
            }
        };
        NaturalWordSliceIterator {
            glyphs: &self.glyphs[..],
            index,
            range: *range,
            reverse,
        }
    }

    /// Returns an iterator that will iterate over all slices of glyphs that represent individual
    /// characters in the given range.
    pub fn character_slices_in_range(
        &'a self,
        range: &Range<ByteIndex>,
    ) -> CharacterSliceIterator<'a> {
        let index = match self.index_of_first_glyph_run_containing(range.begin()) {
            None => self.glyphs.len(),
            Some(index) => index,
        };
        let mut glyph_run_iter = self.glyphs[index..].iter();
        let first_glyph_run = glyph_run_iter.next();
        CharacterSliceIterator {
            text: &self.text,
            glyph_run: first_glyph_run,
            glyph_run_iter,
            range: *range,
        }
    }
}
