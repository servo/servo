/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ops::Range;
use std::sync::Arc;

use fonts::{ShapedText, ShapedTextSlice, ShapedTextSliceType, ShapedTextSlicer, ShapingOptions};
use icu_segmenter::LineBreakOptions;
use style::computed_values::white_space_collapse::T as WhiteSpaceCollapse;
use style::computed_values::word_break::T as WordBreak;
use style::properties::ComputedValues;
use style::str::char_is_whitespace;
use style::values::computed::OverflowWrap;
use unicode_script::Script;

use crate::ArcRefCell;
use crate::flow::inline::line_breaker::LineBreaker;
use crate::flow::inline::text_run::{FontAndScriptInfo, TextRun, TextRunItem, script_is_specific};

/// An entry on the shaping queue that represents text that needs to be shaped.
/// This contains a lot of duplicated data from `TextRunSegment` so that
/// it can outlive a mutable borrow on the owning `TextRun`.
pub(crate) struct ShapingQueueText {
    info: FontAndScriptInfo,
    byte_range: Range<usize>,
    character_range: Range<usize>,
    text_run: ArcRefCell<TextRun>,
    index_in_text_run: usize,
    old_shaped_text: Option<Arc<ShapedText>>,
}

/// A new entry for the [`ShapingQueue`].
pub(crate) enum ShapingQueueEntry {
    PreservedTabOrNewline,
    Text(ShapingQueueText),
}

impl ShapingQueueEntry {
    pub(crate) fn new(
        text_run: ArcRefCell<TextRun>,
        text_run_item: &TextRunItem,
        index_in_text_run: usize,
        old_text_run_line_item: Option<TextRunItem>,
    ) -> Self {
        let text_segment = match text_run_item {
            TextRunItem::LineBreak { .. } | TextRunItem::Tab { .. } => {
                return Self::PreservedTabOrNewline;
            },
            TextRunItem::TextSegment(text_run_segment) => text_run_segment,
        };

        let old_shaped_text = old_text_run_line_item.and_then(|old_text_run_line_item| {
            let TextRunItem::TextSegment(old_text_segment) = old_text_run_line_item else {
                return None;
            };
            if !text_segment.is_compatible_with_old_shaping_result(&old_text_segment) {
                return None;
            }
            old_text_segment.shaped_text
        });

        Self::Text(ShapingQueueText {
            info: text_segment.info.clone(),
            byte_range: text_segment.byte_range.clone(),
            character_range: text_segment.character_range.clone(),
            text_run,
            index_in_text_run,
            old_shaped_text,
        })
    }
}

struct BatchSlicer<'a> {
    slicer: ShapedTextSlicer,
    text: &'a str,
    line_breaker: &'a mut LineBreaker,
    character_offset_origin: usize,
}

impl BatchSlicer<'_> {
    fn slice_shaped_text_at_line_break_opportunities(
        &mut self,
        segment: &ShapingQueueText,
        parent_style: &ComputedValues,
    ) -> (Vec<Arc<ShapedTextSlice>>, bool) {
        // Gather the linebreaks that apply to this segment from the inline formatting context's collection
        // of line breaks. Also add a simulated break at the end of the segment in order to ensure the final
        // piece of text is processed.
        let range = segment.byte_range.clone();
        let linebreaks = self
            .line_breaker
            .advance_to_linebreaks_in_range(segment.byte_range.clone());
        let linebreak_iter = linebreaks.iter().chain(std::iter::once(&range.end));

        let mut break_at_start = false;

        let text_style = parent_style.get_inherited_text();
        let can_break_anywhere = text_style.word_break == WordBreak::BreakAll ||
            text_style.overflow_wrap == OverflowWrap::Anywhere ||
            text_style.overflow_wrap == OverflowWrap::BreakWord;

        let mut last_slice = segment.byte_range.start..segment.byte_range.start;
        let mut current_character_offset =
            segment.character_range.start - self.character_offset_origin;

        let mut runs = Vec::with_capacity(linebreaks.len());
        let mut maybe_push_run = |run: Option<Arc<ShapedTextSlice>>| {
            if let Some(run) = run {
                runs.push(run);
            }
        };

        for break_index in linebreak_iter {
            if *break_index == segment.byte_range.start {
                break_at_start = true;
                continue;
            }

            // Extend the slice to the next UAX#14 line break opportunity.
            let mut slice = last_slice.end..*break_index;
            let word = &self.text[slice.clone()];

            // Split off any trailing whitespace into a separate glyph run.
            let mut whitespace = slice.end..slice.end;
            let rev_char_indices = word.char_indices().rev().peekable();

            let mut slice_type = ShapedTextSliceType::Word;
            let mut ends_with_whitespace = false;
            if let Some((first_white_space_index, first_white_space_character)) = rev_char_indices
                .take_while(|&(_, character)| char_is_whitespace(character))
                .last()
            {
                ends_with_whitespace = true;
                whitespace.start = slice.start + first_white_space_index;

                // If line breaking for a piece of text that has `white-space-collapse:
                // break-spaces` there is a line break opportunity *after* every preserved space,
                // but not before. This means that we should not split off the first whitespace.
                //
                // An exception to this is if the style tells us that we can break in the middle of words.
                if text_style.white_space_collapse == WhiteSpaceCollapse::BreakSpaces &&
                    !can_break_anywhere
                {
                    whitespace.start += first_white_space_character.len_utf8();
                    slice_type = ShapedTextSliceType::WordAndWhiteSpace;
                }

                slice.end = whitespace.start;
            }

            // If there's no whitespace and `word-break` is set to `keep-all`, try increasing the slice.
            // TODO: This should only happen for CJK text.
            if !ends_with_whitespace &&
                *break_index != segment.byte_range.end &&
                text_style.word_break == WordBreak::KeepAll &&
                !can_break_anywhere
            {
                continue;
            }

            // Only advance the last slice if we are not going to try to expand the slice.
            last_slice = slice.start..*break_index;

            // Push the non-whitespace part of the range.
            if !slice.is_empty() {
                current_character_offset += self.text[slice].chars().count();
                maybe_push_run(
                    self.slicer
                        .slice_until_character_offset(current_character_offset, slice_type),
                );
            }

            if whitespace.is_empty() {
                continue;
            }

            // If `white-space-collapse: break-spaces` is active, insert a line breaking opportunity
            // between each white space character in the white space that we trimmed off.
            if text_style.white_space_collapse == WhiteSpaceCollapse::BreakSpaces {
                for _ in self.text[whitespace].chars() {
                    current_character_offset += 1;
                    maybe_push_run(self.slicer.slice_until_character_offset(
                        current_character_offset,
                        ShapedTextSliceType::WhiteSpace,
                    ));
                }
                continue;
            }

            current_character_offset += self.text[whitespace].chars().count();
            maybe_push_run(self.slicer.slice_until_character_offset(
                current_character_offset,
                ShapedTextSliceType::WhiteSpace,
            ));
        }

        (runs, break_at_start)
    }
}

/// The [`ShapingQueue`] is responsible for shaping text during inline formatting context
/// construction. It allows for shaping text across inline box boundaries. When pushing
/// items to the queue, if the items are compatible pieces of text that can be shaped
/// together, they are accumulated. The queue may be flushed in the given situations:
///
/// - An incompatible piece of text (different fonts or certain style properties) is
///   pushed to the queue.
/// - A preserved newline or tab is pushed to the queue.
/// - An inline box breaks shaping via padding, border, margins or a non-`baseline`
///   `vertical-align` property.
/// - Atomic content in the inline formatting context
///
/// Upon flushing, the [`ShapingQueue`] will shape any pending text and assign the
/// resulting [`ShapedTextSlice`]s to the originating [`TextRun`]s.
pub(crate) struct ShapingQueue<'a> {
    /// The queue of items in the current batch that will be shaped together.
    queue: Vec<ShapingQueueText>,
    /// The text that will be used for shaping.
    text: &'a str,
    /// The line breaker that will be used to slice shaping results across on line break boundaries.
    line_breaker: LineBreaker,
    /// The byte range of the text to shape in [`Self::text`] for the current batch.
    /// Only contiguous ranges can be shaped together.
    byte_range: Range<usize>,
    /// The character range of the text to shape in [`Self::text`] for the current batch.
    /// Only contiguous ranges can be shaped together.
    character_range: Range<usize>,
    /// The resolved script for the current batch. This is used to gradually turn non-specific
    /// scripts into a resolved value for shaping.
    resolved_script: Option<Script>,
}

impl<'a> ShapingQueue<'a> {
    pub(crate) fn new(text: &'a str, line_break_options: LineBreakOptions) -> Self {
        Self {
            queue: Default::default(),
            text,
            line_breaker: LineBreaker::new(text, line_break_options),
            byte_range: Default::default(),
            character_range: Default::default(),
            resolved_script: None,
        }
    }

    fn compatible_old_shaping_result(&self, character_count: usize) -> Option<Arc<ShapedText>> {
        let old_shaped_text = self.queue.first()?.old_shaped_text.as_ref()?;
        if old_shaped_text.character_count() != character_count {
            return None;
        }

        if !self.queue.iter().all(|entry| {
            entry
                .old_shaped_text
                .as_ref()
                .is_some_and(|entry_old_shaped_text| {
                    Arc::ptr_eq(old_shaped_text, entry_old_shaped_text)
                })
        }) {
            return None;
        }
        Some(old_shaped_text.clone())
    }

    fn shape_batch(&self) -> Option<Arc<ShapedText>> {
        let first = self.queue.first()?;

        let character_count = self.character_range.end - self.character_range.start;
        if let Some(old_shaping_result) = self.compatible_old_shaping_result(character_count) {
            return Some(old_shaping_result);
        };

        let mut options: ShapingOptions = (&first.info).into();
        options.script = self.resolved_script.unwrap_or(first.info.script);

        let font = &first.info.font_info.font;
        Some(font.shape_text(&self.text[self.byte_range.clone()], &options))
    }

    /// Flush this [`ShapingQueue`]. If any content had been collected up to this point,
    /// it will be shaped and the resulting [`ShapedTextSlice`]s will be assigned to their
    /// originating [`TextRun`]s.
    pub(crate) fn flush(&mut self) {
        let Some(shaped_text) = self.shape_batch() else {
            return;
        };

        let mut slicer = BatchSlicer {
            slicer: ShapedTextSlicer::new(shaped_text.clone()),
            text: self.text,
            line_breaker: &mut self.line_breaker,
            character_offset_origin: self.character_range.start,
        };

        for entry in self.queue.drain(..) {
            let mut text_run = entry.text_run.borrow_mut();
            let style = text_run.inline_styles().style.borrow().clone();
            let (runs, break_at_start) =
                slicer.slice_shaped_text_at_line_break_opportunities(&entry, &style);

            if let TextRunItem::TextSegment(text_segment) =
                &mut text_run.items[entry.index_in_text_run]
            {
                text_segment.shaped_text = Some(shaped_text.clone());
                text_segment.runs = runs;
                text_segment.break_at_start = break_at_start;
            }
        }
    }

    fn compatible_with_batch(&self, text: &ShapingQueueText) -> bool {
        // If the queue is empty, we can always add new text to the batch.
        let Some(last) = self.queue.last() else {
            return true;
        };

        // The new text is only compatible with the current batch if their character and
        // text byte boundaries are contiguous.
        if last.character_range.end != text.character_range.start ||
            last.byte_range.end != text.byte_range.start
        {
            return false;
        }

        // The `FontInfo`s of the batch and the new text need to match exactly to shape
        // together.
        if !Arc::ptr_eq(&last.info.font_info, &text.info.font_info) &&
            *last.info.font_info != *text.info.font_info
        {
            return false;
        }

        // Any resolved `Script` has to be compatible with any new specific `Script`.
        !script_is_specific(text.info.script) ||
            self.resolved_script
                .is_none_or(|resolved_script| resolved_script == text.info.script)
    }

    fn push_text(&mut self, text: ShapingQueueText) {
        if !self.compatible_with_batch(&text) {
            self.flush();
        }

        if self.queue.is_empty() {
            self.character_range = text.character_range.clone();
            self.byte_range = text.byte_range.clone();
            self.resolved_script = None;
        } else {
            self.character_range.end = text.character_range.end;
            self.byte_range.end = text.byte_range.end;
        }
        if self.resolved_script.is_none() && script_is_specific(text.info.script) {
            self.resolved_script = Some(text.info.script);
        }

        self.queue.push(text);
    }

    /// Push a new [`ShapingQueueEntry`] on to this [`ShapingQueue`], maybe flushing
    /// previously collected entries.
    pub(crate) fn push(&mut self, entry: ShapingQueueEntry) {
        match entry {
            ShapingQueueEntry::PreservedTabOrNewline => self.flush(),
            ShapingQueueEntry::Text(shaping_queue_text) => self.push_text(shaping_queue_text),
        }
    }
}
