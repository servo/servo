/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::mem;
use std::ops::Range;

use app_units::Au;
use gfx::font::{FontRef, ShapingFlags, ShapingOptions};
use gfx::font_cache_thread::FontCacheThread;
use gfx::font_context::FontContext;
use gfx::text::glyph::GlyphRun;
use gfx_traits::ByteIndex;
use log::warn;
use range::Range as ServoRange;
use serde::Serialize;
use servo_arc::Arc;
use style::computed_values::text_rendering::T as TextRendering;
use style::computed_values::white_space_collapse::T as WhiteSpaceCollapse;
use style::computed_values::word_break::T as WordBreak;
use style::properties::style_structs::InheritedText;
use style::properties::ComputedValues;
use style::str::char_is_whitespace;
use style::values::computed::OverflowWrap;
use unicode_script::Script;
use xi_unicode::{linebreak_property, LineBreakLeafIter};

use super::{FontKeyAndMetrics, InlineFormattingContextState};
use crate::fragment_tree::BaseFragmentInfo;

// These constants are the xi-unicode line breaking classes that are defined in
// `table.rs`. Unfortunately, they are only identified by number.
const XI_LINE_BREAKING_CLASS_CM: u8 = 9;
const XI_LINE_BREAKING_CLASS_GL: u8 = 12;
const XI_LINE_BREAKING_CLASS_ZW: u8 = 28;
const XI_LINE_BREAKING_CLASS_WJ: u8 = 30;
const XI_LINE_BREAKING_CLASS_ZWJ: u8 = 40;

/// <https://www.w3.org/TR/css-display-3/#css-text-run>
#[derive(Debug, Serialize)]
pub(crate) struct TextRun {
    pub base_fragment_info: BaseFragmentInfo,
    #[serde(skip_serializing)]
    pub parent_style: Arc<ComputedValues>,
    pub text_range: Range<usize>,

    /// The text of this [`TextRun`] with a font selected, broken into unbreakable
    /// segments, and shaped.
    pub shaped_text: Vec<TextRunSegment>,

    /// Whether or not to prevent a soft wrap opportunity at the start of this [`TextRun`].
    /// This depends on the whether the first character in the run prevents a soft wrap
    /// opportunity.
    prevent_soft_wrap_opportunity_at_start: bool,

    /// Whether or not to prevent a soft wrap opportunity at the end of this [`TextRun`].
    /// This depends on the whether the last character in the run prevents a soft wrap
    /// opportunity.
    prevent_soft_wrap_opportunity_at_end: bool,
}

// There are two reasons why we might want to break at the start:
//
//  1. The line breaker told us that a break was necessary between two separate
//     instances of sending text to it.
//  2. We are following replaced content ie `have_deferred_soft_wrap_opportunity`.
//
// In both cases, we don't want to do this if the first character prevents a
// soft wrap opportunity.
#[derive(PartialEq)]
enum SegmentStartSoftWrapPolicy {
    Force,
    Prevent,
    FollowLinebreaker,
}

#[derive(Debug, Serialize)]
pub(crate) struct TextRunSegment {
    /// The index of this font in the parent [`super::InlineFormattingContext`]'s collection of font
    /// information.
    pub font_index: usize,

    /// The [`Script`] of this segment.
    #[serde(skip_serializing)]
    pub script: Script,

    /// The range of bytes in the [`TextRun`]'s text that this segment covers.
    pub range: ServoRange<ByteIndex>,

    /// Whether or not the linebreaker said that we should allow a line break at the start of this
    /// segment.
    pub break_at_start: bool,

    /// The shaped runs within this segment.
    pub runs: Vec<GlyphRun>,
}

impl TextRunSegment {
    fn new(font_index: usize, script: Script, byte_index: ByteIndex) -> Self {
        Self {
            script,
            font_index,
            range: ServoRange::new(byte_index, ByteIndex(0)),
            runs: Vec::new(),
            break_at_start: false,
        }
    }

    /// Update this segment if the Font and Script are compatible. The update will only
    /// ever make the Script specific. Returns true if the new Font and Script are
    /// compatible with this segment or false otherwise.
    fn update_if_compatible(
        &mut self,
        new_font: &FontRef,
        script: Script,
        fonts: &[FontKeyAndMetrics],
    ) -> bool {
        fn is_specific(script: Script) -> bool {
            script != Script::Common && script != Script::Inherited
        }

        let current_font_key_and_metrics = &fonts[self.font_index];
        if new_font.font_key != current_font_key_and_metrics.key ||
            new_font.descriptor.pt_size != current_font_key_and_metrics.pt_size
        {
            return false;
        }

        if !is_specific(self.script) && is_specific(script) {
            self.script = script;
        }
        script == self.script || !is_specific(script)
    }

    fn layout_into_line_items(
        &self,
        text_run: &TextRun,
        mut soft_wrap_policy: SegmentStartSoftWrapPolicy,
        ifc: &mut InlineFormattingContextState,
    ) {
        if self.break_at_start && soft_wrap_policy == SegmentStartSoftWrapPolicy::FollowLinebreaker
        {
            soft_wrap_policy = SegmentStartSoftWrapPolicy::Force;
        }

        for (run_index, run) in self.runs.iter().enumerate() {
            ifc.possibly_flush_deferred_forced_line_break();

            // If this whitespace forces a line break, queue up a hard line break the next time we
            // see any content. We don't line break immediately, because we'd like to finish processing
            // any ongoing inline boxes before ending the line.
            if run.is_single_preserved_newline() {
                ifc.defer_forced_line_break();
                continue;
            }

            // Break before each unbreakable run in this TextRun, except the first unless the
            // linebreaker was set to break before the first run.
            if run_index != 0 || soft_wrap_policy == SegmentStartSoftWrapPolicy::Force {
                ifc.process_soft_wrap_opportunity();
            }

            ifc.push_glyph_store_to_unbreakable_segment(
                run.glyph_store.clone(),
                text_run,
                self.font_index,
            );
        }
    }
}

impl TextRun {
    pub(crate) fn new(
        base_fragment_info: BaseFragmentInfo,
        parent_style: Arc<ComputedValues>,
        text_range: Range<usize>,
    ) -> Self {
        Self {
            base_fragment_info,
            parent_style,
            text_range,
            shaped_text: Vec::new(),
            prevent_soft_wrap_opportunity_at_start: false,
            prevent_soft_wrap_opportunity_at_end: false,
        }
    }

    pub(super) fn break_and_shape(
        &mut self,
        text_content: &str,
        font_context: &FontContext<FontCacheThread>,
        linebreaker: &mut Option<LineBreakLeafIter>,
        font_cache: &mut Vec<FontKeyAndMetrics>,
    ) {
        let segment_results = self.segment_text(text_content, font_context, font_cache);
        let inherited_text_style = self.parent_style.get_inherited_text().clone();
        let letter_spacing = if inherited_text_style.letter_spacing.0.px() != 0. {
            Some(app_units::Au::from(inherited_text_style.letter_spacing.0))
        } else {
            None
        };

        let mut flags = ShapingFlags::empty();
        if letter_spacing.is_some() {
            flags.insert(ShapingFlags::IGNORE_LIGATURES_SHAPING_FLAG);
        }
        if inherited_text_style.text_rendering == TextRendering::Optimizespeed {
            flags.insert(ShapingFlags::IGNORE_LIGATURES_SHAPING_FLAG);
            flags.insert(ShapingFlags::DISABLE_KERNING_SHAPING_FLAG)
        }
        if inherited_text_style.word_break == WordBreak::KeepAll {
            flags.insert(ShapingFlags::KEEP_ALL_FLAG);
        }

        let specified_word_spacing = &inherited_text_style.word_spacing;
        let style_word_spacing: Option<Au> = specified_word_spacing.to_length().map(|l| l.into());
        let segments = segment_results
            .into_iter()
            .map(|(mut segment, font)| {
                let word_spacing = style_word_spacing.unwrap_or_else(|| {
                    let space_width = font
                        .glyph_index(' ')
                        .map(|glyph_id| font.glyph_h_advance(glyph_id))
                        .unwrap_or(gfx::font::LAST_RESORT_GLYPH_ADVANCE);
                    specified_word_spacing.to_used_value(Au::from_f64_px(space_width))
                });
                let shaping_options = ShapingOptions {
                    letter_spacing,
                    word_spacing,
                    script: segment.script,
                    flags,
                };
                (segment.runs, segment.break_at_start) = break_and_shape(
                    font,
                    &text_content[segment.range.begin().0 as usize..segment.range.end().0 as usize],
                    &inherited_text_style,
                    &shaping_options,
                    linebreaker,
                );

                segment
            })
            .collect();

        let _ = std::mem::replace(&mut self.shaped_text, segments);
    }

    /// Take the [`TextRun`]'s text and turn it into [`TextRunSegment`]s. Each segment has a matched
    /// font and script. Fonts may differ when glyphs are found in fallback fonts. Fonts are stored
    /// in the `font_cache` which is a cache of all font keys and metrics used in this
    /// [`super::InlineFormattingContext`].
    fn segment_text(
        &mut self,
        text_content: &str,
        font_context: &FontContext<FontCacheThread>,
        font_cache: &mut Vec<FontKeyAndMetrics>,
    ) -> Vec<(TextRunSegment, FontRef)> {
        let font_group = font_context.font_group(self.parent_style.clone_font());
        let mut current: Option<(TextRunSegment, FontRef)> = None;
        let mut results = Vec::new();

        let char_iterator = TwoCharsAtATimeIterator::new(text_content.chars());
        let mut next_byte_index = 0;
        for (character, next_character) in char_iterator {
            let current_byte_index = next_byte_index;
            next_byte_index += character.len_utf8();

            let prevents_soft_wrap_opportunity =
                char_prevents_soft_wrap_opportunity_when_before_or_after_atomic(character);
            if current_byte_index == 0 && prevents_soft_wrap_opportunity {
                self.prevent_soft_wrap_opportunity_at_start = true;
            }
            self.prevent_soft_wrap_opportunity_at_end = prevents_soft_wrap_opportunity;

            if char_does_not_change_font(character) {
                continue;
            }

            let Some(font) =
                font_group
                    .write()
                    .find_by_codepoint(font_context, character, next_character)
            else {
                continue;
            };

            // If the existing segment is compatible with the character, keep going.
            let script = Script::from(character);
            if let Some(current) = current.as_mut() {
                if current.0.update_if_compatible(&font, script, font_cache) {
                    continue;
                }
            }

            let font_index = add_or_get_font(&font, font_cache);

            // Add the new segment and finish the existing one, if we had one. If the first
            // characters in the run were control characters we may be creating the first
            // segment in the middle of the run (ie the start should be 0).
            let start_byte_index = match current {
                Some(_) => ByteIndex(current_byte_index as isize),
                None => ByteIndex(0_isize),
            };
            let new = (
                TextRunSegment::new(font_index, script, start_byte_index),
                font,
            );
            if let Some(mut finished) = current.replace(new) {
                finished.0.range.extend_to(start_byte_index);
                results.push(finished);
            }
        }

        // Either we have a current segment or we only had control character and whitespace. In both
        // of those cases, just use the first font.
        if current.is_none() {
            current = font_group.write().first(font_context).map(|font| {
                let font_index = add_or_get_font(&font, font_cache);
                (
                    TextRunSegment::new(font_index, Script::Common, ByteIndex(0)),
                    font,
                )
            })
        }

        // Extend the last segment to the end of the string and add it to the results.
        if let Some(mut last_segment) = current.take() {
            last_segment
                .0
                .range
                .extend_to(ByteIndex(text_content.len() as isize));
            results.push(last_segment);
        }

        results
    }

    pub(super) fn layout_into_line_items(&self, ifc: &mut InlineFormattingContextState) {
        if self.text_range.is_empty() {
            return;
        }

        // If we are following replaced content, we should have a soft wrap opportunity, unless the
        // first character of this `TextRun` prevents that soft wrap opportunity. If we see such a
        // character it should also override the LineBreaker's indication to break at the start.
        let have_deferred_soft_wrap_opportunity =
            mem::replace(&mut ifc.have_deferred_soft_wrap_opportunity, false);
        let mut soft_wrap_policy = match self.prevent_soft_wrap_opportunity_at_start {
            true => SegmentStartSoftWrapPolicy::Prevent,
            false if have_deferred_soft_wrap_opportunity => SegmentStartSoftWrapPolicy::Force,
            false => SegmentStartSoftWrapPolicy::FollowLinebreaker,
        };

        for segment in self.shaped_text.iter() {
            segment.layout_into_line_items(self, soft_wrap_policy, ifc);
            soft_wrap_policy = SegmentStartSoftWrapPolicy::FollowLinebreaker;
        }

        ifc.prevent_soft_wrap_opportunity_before_next_atomic =
            self.prevent_soft_wrap_opportunity_at_end;
    }
}

/// Whether or not this character will rpevent a soft wrap opportunity when it
/// comes before or after an atomic inline element.
///
/// From <https://www.w3.org/TR/css-text-3/#line-break-details>:
///
/// > For Web-compatibility there is a soft wrap opportunity before and after each
/// > replaced element or other atomic inline, even when adjacent to a character that
/// > would normally suppress them, including U+00A0 NO-BREAK SPACE. However, with
/// > the exception of U+00A0 NO-BREAK SPACE, there must be no soft wrap opportunity
/// > between atomic inlines and adjacent characters belonging to the Unicode GL, WJ,
/// > or ZWJ line breaking classes.
fn char_prevents_soft_wrap_opportunity_when_before_or_after_atomic(character: char) -> bool {
    if character == '\u{00A0}' {
        return false;
    }
    let class = linebreak_property(character);
    class == XI_LINE_BREAKING_CLASS_GL ||
        class == XI_LINE_BREAKING_CLASS_WJ ||
        class == XI_LINE_BREAKING_CLASS_ZWJ
}

/// Whether or not this character should be able to change the font during segmentation.  Certain
/// character are not rendered at all, so it doesn't matter what font we use to render them. They
/// should just be added to the current segment.
fn char_does_not_change_font(character: char) -> bool {
    if character.is_control() {
        return true;
    }
    if character == '\u{00A0}' {
        return true;
    }
    let class = linebreak_property(character);
    class == XI_LINE_BREAKING_CLASS_CM ||
        class == XI_LINE_BREAKING_CLASS_GL ||
        class == XI_LINE_BREAKING_CLASS_ZW ||
        class == XI_LINE_BREAKING_CLASS_WJ ||
        class == XI_LINE_BREAKING_CLASS_ZWJ
}

pub(super) fn add_or_get_font(font: &FontRef, ifc_fonts: &mut Vec<FontKeyAndMetrics>) -> usize {
    for (index, ifc_font_info) in ifc_fonts.iter().enumerate() {
        if ifc_font_info.key == font.font_key && ifc_font_info.pt_size == font.descriptor.pt_size {
            return index;
        }
    }
    ifc_fonts.push(FontKeyAndMetrics {
        metrics: font.metrics.clone(),
        key: font.font_key,
        pt_size: font.descriptor.pt_size,
    });
    ifc_fonts.len() - 1
}

pub(super) fn get_font_for_first_font_for_style(
    style: &ComputedValues,
    font_context: &FontContext<FontCacheThread>,
) -> Option<FontRef> {
    let font = font_context
        .font_group(style.clone_font())
        .write()
        .first(font_context);
    if font.is_none() {
        warn!("Could not find font for style: {:?}", style.clone_font());
    }
    font
}
pub(crate) struct TwoCharsAtATimeIterator<InputIterator> {
    /// The input character iterator.
    iterator: InputIterator,
    /// The first character to produce in the next run of the iterator.
    next_character: Option<char>,
}

impl<InputIterator> TwoCharsAtATimeIterator<InputIterator> {
    fn new(iterator: InputIterator) -> Self {
        Self {
            iterator,
            next_character: None,
        }
    }
}

impl<InputIterator> Iterator for TwoCharsAtATimeIterator<InputIterator>
where
    InputIterator: Iterator<Item = char>,
{
    type Item = (char, Option<char>);

    fn next(&mut self) -> Option<Self::Item> {
        // If the iterator isn't initialized do that now.
        if self.next_character.is_none() {
            self.next_character = self.iterator.next();
        }

        let Some(character) = self.next_character else {
            return None;
        };

        self.next_character = self.iterator.next();
        Some((character, self.next_character))
    }
}

pub fn break_and_shape(
    font: FontRef,
    text: &str,
    text_style: &InheritedText,
    shaping_options: &ShapingOptions,
    breaker: &mut Option<LineBreakLeafIter>,
) -> (Vec<GlyphRun>, bool) {
    let mut glyphs = vec![];

    if breaker.is_none() {
        if text.is_empty() {
            return (glyphs, true);
        }
        *breaker = Some(LineBreakLeafIter::new(text, 0));
    }

    let breaker = breaker.as_mut().unwrap();

    let mut push_range = |range: &Range<usize>, options: &ShapingOptions| {
        glyphs.push(GlyphRun {
            glyph_store: font.shape_text(&text[range.clone()], options),
            range: ServoRange::new(
                ByteIndex(range.start as isize),
                ByteIndex(range.len() as isize),
            ),
        });
    };

    let can_break_anywhere = text_style.word_break == WordBreak::BreakAll ||
        text_style.overflow_wrap == OverflowWrap::Anywhere ||
        text_style.overflow_wrap == OverflowWrap::BreakWord;

    let mut break_at_zero = false;
    let mut last_slice_end = 0;
    while last_slice_end != text.len() {
        let (break_index, _is_hard_break) = breaker.next(text);
        if break_index == 0 {
            break_at_zero = true;
        }

        // Extend the slice to the next UAX#14 line break opportunity.
        let mut slice = last_slice_end..break_index;
        let word = &text[slice.clone()];

        // Split off any trailing whitespace into a separate glyph run.
        let mut whitespace = slice.end..slice.end;
        let mut rev_char_indices = word.char_indices().rev().peekable();
        let ends_with_newline = rev_char_indices.peek().map_or(false, |&(_, c)| c == '\n');
        if let Some((first_white_space_index, first_white_space_character)) = rev_char_indices
            .take_while(|&(_, c)| char_is_whitespace(c))
            .last()
        {
            whitespace.start = slice.start + first_white_space_index;

            // If line breaking for a piece of text that has `white-space-collapse: break-spaces` there
            // is a line break opportunity *after* every preserved space, but not before. This means
            // that we should not split off the first whitespace, unless that white-space is a preserved
            // newline.
            //
            // An exception to this is if the style tells us that we can break in the middle of words.
            if text_style.white_space_collapse == WhiteSpaceCollapse::BreakSpaces &&
                first_white_space_character != '\n' &&
                !can_break_anywhere
            {
                whitespace.start += first_white_space_character.len_utf8();
            }

            slice.end = whitespace.start;
        }

        // If there's no whitespace and `word-break` is set to `keep-all`, try increasing the slice.
        // TODO: This should only happen for CJK text.
        let can_break_anywhere = text_style.word_break == WordBreak::BreakAll ||
            text_style.overflow_wrap == OverflowWrap::Anywhere ||
            text_style.overflow_wrap == OverflowWrap::BreakWord;
        if whitespace.is_empty() &&
            break_index != text.len() &&
            text_style.word_break == WordBreak::KeepAll &&
            !can_break_anywhere
        {
            continue;
        }

        // Only advance the last_slice_end if we are not going to try to expand the slice.
        last_slice_end = break_index;

        // Push the non-whitespace part of the range.
        if !slice.is_empty() {
            push_range(&slice, shaping_options);
        }

        if whitespace.is_empty() {
            continue;
        }

        let mut options = *shaping_options;
        options
            .flags
            .insert(ShapingFlags::IS_WHITESPACE_SHAPING_FLAG);

        // If `white-space-collapse: break-spaces` is active, insert a line breaking opportunity
        // between each white space character in the white space that we trimmed off.
        if text_style.white_space_collapse == WhiteSpaceCollapse::BreakSpaces {
            let start_index = whitespace.start;
            for (index, character) in text[whitespace].char_indices() {
                let index = start_index + index;
                push_range(&(index..index + character.len_utf8()), &options);
            }
            continue;
        }

        // The breaker breaks after every newline, so either there is none,
        // or there is exactly one at the very end. In the latter case,
        // split it into a different run. That's because shaping considers
        // a newline to have the same advance as a space, but during layout
        // we want to treat the newline as having no advance.
        if ends_with_newline && whitespace.len() > 1 {
            push_range(&(whitespace.start..whitespace.end - 1), &options);
            push_range(&(whitespace.end - 1..whitespace.end), &options);
        } else {
            push_range(&whitespace, &options);
        }
    }
    (glyphs, break_at_zero)
}
