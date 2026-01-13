/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::mem;
use std::ops::Range;

use app_units::Au;
use base::text::is_bidi_control;
use fonts::{
    FontContext, FontRef, GlyphRun, LAST_RESORT_GLYPH_ADVANCE, ShapingFlags, ShapingOptions,
    TextByteRange,
};
use fonts_traits::ByteIndex;
use log::warn;
use malloc_size_of_derive::MallocSizeOf;
use servo_arc::Arc;
use style::computed_values::text_rendering::T as TextRendering;
use style::computed_values::white_space_collapse::T as WhiteSpaceCollapse;
use style::computed_values::word_break::T as WordBreak;
use style::properties::ComputedValues;
use style::str::char_is_whitespace;
use style::values::computed::OverflowWrap;
use unicode_bidi::{BidiInfo, Level};
use unicode_script::Script;
use xi_unicode::linebreak_property;

use super::line_breaker::LineBreaker;
use super::{InlineFormattingContextLayout, SharedInlineStyles};
use crate::context::LayoutContext;
use crate::fragment_tree::BaseFragmentInfo;

// These constants are the xi-unicode line breaking classes that are defined in
// `table.rs`. Unfortunately, they are only identified by number.
pub(crate) const XI_LINE_BREAKING_CLASS_CM: u8 = 9;
pub(crate) const XI_LINE_BREAKING_CLASS_GL: u8 = 12;
pub(crate) const XI_LINE_BREAKING_CLASS_ZW: u8 = 28;
pub(crate) const XI_LINE_BREAKING_CLASS_WJ: u8 = 30;
pub(crate) const XI_LINE_BREAKING_CLASS_ZWJ: u8 = 42;

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
    FollowLinebreaker,
}

#[derive(Debug, MallocSizeOf)]
pub(crate) struct TextRunSegment {
    /// The index of this font in the parent [`super::InlineFormattingContext`]'s collection of font
    /// information.
    pub font: FontRef,

    /// The [`Script`] of this segment.
    pub script: Script,

    /// The bidi Level of this segment.
    pub bidi_level: Level,

    /// The range of bytes in the parent [`super::InlineFormattingContext`]'s text content.
    pub range: Range<usize>,

    /// Whether or not the linebreaker said that we should allow a line break at the start of this
    /// segment.
    pub break_at_start: bool,

    /// The shaped runs within this segment.
    pub runs: Vec<GlyphRun>,
}

impl TextRunSegment {
    fn new(font: FontRef, script: Script, bidi_level: Level, start_offset: usize) -> Self {
        Self {
            font,
            script,
            bidi_level,
            range: start_offset..start_offset,
            runs: Vec::new(),
            break_at_start: false,
        }
    }

    /// Update this segment if the Font and Script are compatible. The update will only
    /// ever make the Script specific. Returns true if the new Font and Script are
    /// compatible with this segment or false otherwise.
    fn update_if_compatible(
        &mut self,
        layout_context: &LayoutContext,
        new_font: &FontRef,
        script: Script,
        bidi_level: Level,
    ) -> bool {
        fn is_specific(script: Script) -> bool {
            script != Script::Common && script != Script::Inherited
        }

        if bidi_level != self.bidi_level {
            return false;
        }

        let painter_id = layout_context.painter_id;
        let font_context = &layout_context.font_context;
        if new_font.key(painter_id, font_context) !=
            self.font
                .key(layout_context.painter_id, &layout_context.font_context) ||
            new_font.descriptor.pt_size != self.font.descriptor.pt_size
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
        ifc: &mut InlineFormattingContextLayout,
    ) {
        if self.break_at_start && soft_wrap_policy == SegmentStartSoftWrapPolicy::FollowLinebreaker
        {
            soft_wrap_policy = SegmentStartSoftWrapPolicy::Force;
        }

        let mut byte_processed = ByteIndex(0);
        for (run_index, run) in self.runs.iter().enumerate() {
            ifc.possibly_flush_deferred_forced_line_break();

            // If this whitespace forces a line break, queue up a hard line break the next time we
            // see any content. We don't line break immediately, because we'd like to finish processing
            // any ongoing inline boxes before ending the line.
            if run.is_single_preserved_newline() {
                byte_processed = byte_processed + run.range.len();
                ifc.defer_forced_line_break();
                continue;
            }
            // Break before each unbreakable run in this TextRun, except the first unless the
            // linebreaker was set to break before the first run.
            if run_index != 0 || soft_wrap_policy == SegmentStartSoftWrapPolicy::Force {
                ifc.process_soft_wrap_opportunity();
            }

            let range_start = byte_processed + ByteIndex(self.range.start as isize);
            ifc.push_glyph_store_to_unbreakable_segment(
                run.glyph_store.clone(),
                text_run,
                &self.font,
                self.bidi_level,
                TextByteRange::new(range_start, range_start + run.range.len()),
            );
            byte_processed = byte_processed + run.range.len();
        }
    }

    fn shape_and_push_range(
        &mut self,
        range: &Range<usize>,
        formatting_context_text: &str,
        options: &ShapingOptions,
    ) {
        self.runs.push(GlyphRun {
            glyph_store: self
                .font
                .shape_text(&formatting_context_text[range.clone()], options),
            range: TextByteRange::new(
                ByteIndex(range.start as isize),
                ByteIndex(range.end as isize),
            ),
        });
    }

    /// Shape the text of this [`TextRunSegment`], first finding "words" for the shaper by processing
    /// the linebreaks found in the owning [`super::InlineFormattingContext`]. Linebreaks are filtered,
    /// based on the style of the parent inline box.
    fn shape_text(
        &mut self,
        parent_style: &ComputedValues,
        formatting_context_text: &str,
        linebreaker: &mut LineBreaker,
        shaping_options: &ShapingOptions,
    ) {
        // Gather the linebreaks that apply to this segment from the inline formatting context's collection
        // of line breaks. Also add a simulated break at the end of the segment in order to ensure the final
        // piece of text is processed.
        let range = self.range.clone();
        let linebreaks = linebreaker.advance_to_linebreaks_in_range(self.range.clone());
        let linebreak_iter = linebreaks.iter().chain(std::iter::once(&range.end));

        self.runs.clear();
        self.runs.reserve(linebreaks.len());
        self.break_at_start = false;

        let text_style = parent_style.get_inherited_text().clone();
        let can_break_anywhere = text_style.word_break == WordBreak::BreakAll ||
            text_style.overflow_wrap == OverflowWrap::Anywhere ||
            text_style.overflow_wrap == OverflowWrap::BreakWord;

        let mut last_slice = self.range.start..self.range.start;
        for break_index in linebreak_iter {
            if *break_index == self.range.start {
                self.break_at_start = true;
                continue;
            }

            let mut options = *shaping_options;

            // Extend the slice to the next UAX#14 line break opportunity.
            let mut slice = last_slice.end..*break_index;
            let word = &formatting_context_text[slice.clone()];

            // Split off any trailing whitespace into a separate glyph run.
            let mut whitespace = slice.end..slice.end;
            let mut rev_char_indices = word.char_indices().rev().peekable();

            let mut ends_with_whitespace = false;
            let ends_with_newline = rev_char_indices
                .peek()
                .is_some_and(|&(_, character)| character == '\n');
            if let Some((first_white_space_index, first_white_space_character)) = rev_char_indices
                .take_while(|&(_, character)| char_is_whitespace(character))
                .last()
            {
                ends_with_whitespace = true;
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
                    options
                        .flags
                        .insert(ShapingFlags::ENDS_WITH_WHITESPACE_SHAPING_FLAG);
                }

                slice.end = whitespace.start;
            }

            // If there's no whitespace and `word-break` is set to `keep-all`, try increasing the slice.
            // TODO: This should only happen for CJK text.
            if !ends_with_whitespace &&
                *break_index != self.range.end &&
                text_style.word_break == WordBreak::KeepAll &&
                !can_break_anywhere
            {
                continue;
            }

            // Only advance the last slice if we are not going to try to expand the slice.
            last_slice = slice.start..*break_index;

            // Push the non-whitespace part of the range.
            if !slice.is_empty() {
                self.shape_and_push_range(&slice, formatting_context_text, &options);
            }

            if whitespace.is_empty() {
                continue;
            }

            options.flags.insert(
                ShapingFlags::IS_WHITESPACE_SHAPING_FLAG |
                    ShapingFlags::ENDS_WITH_WHITESPACE_SHAPING_FLAG,
            );

            // If `white-space-collapse: break-spaces` is active, insert a line breaking opportunity
            // between each white space character in the white space that we trimmed off.
            if text_style.white_space_collapse == WhiteSpaceCollapse::BreakSpaces {
                let start_index = whitespace.start;
                for (index, character) in formatting_context_text[whitespace].char_indices() {
                    let index = start_index + index;
                    self.shape_and_push_range(
                        &(index..index + character.len_utf8()),
                        formatting_context_text,
                        &options,
                    );
                }
                continue;
            }

            // The breaker breaks after every newline, so either there is none,
            // or there is exactly one at the very end. In the latter case,
            // split it into a different run. That's because shaping considers
            // a newline to have the same advance as a space, but during layout
            // we want to treat the newline as having no advance.
            if ends_with_newline && whitespace.len() > 1 {
                self.shape_and_push_range(
                    &(whitespace.start..whitespace.end - 1),
                    formatting_context_text,
                    &options,
                );
                self.shape_and_push_range(
                    &(whitespace.end - 1..whitespace.end),
                    formatting_context_text,
                    &options,
                );
            } else {
                self.shape_and_push_range(&whitespace, formatting_context_text, &options);
            }
        }
    }
}

/// A single [`TextRun`] for the box tree. These are all descendants of
/// [`super::InlineBox`] or the root of the [`super::InlineFormattingContext`].  During
/// box tree construction, text is split into [`TextRun`]s based on their font, script,
/// etc. When these are created text is already shaped.
///
/// <https://www.w3.org/TR/css-display-3/#css-text-run>
#[derive(Debug, MallocSizeOf)]
pub(crate) struct TextRun {
    /// The [`BaseFragmentInfo`] for this [`TextRun`]. Usually this comes from the
    /// original text node in the DOM for the text.
    pub base_fragment_info: BaseFragmentInfo,

    /// The [`crate::SharedStyle`] from this [`TextRun`]s parent element. This is
    /// shared so that incremental layout can simply update the parent element and
    /// this [`TextRun`] will be updated automatically.
    pub inline_styles: SharedInlineStyles,

    /// The range of text in [`super::InlineFormattingContext::text_content`] of the
    /// [`super::InlineFormattingContext`] that owns this [`TextRun`]. These are UTF-8 offsets.
    pub text_range: Range<usize>,

    /// The text of this [`TextRun`] with a font selected, broken into unbreakable
    /// segments, and shaped.
    pub shaped_text: Vec<TextRunSegment>,

    /// The selection range for the DOM text node that originated this [`TextRun`]. This
    /// comes directly from the DOM.
    pub selection_range: Option<TextByteRange>,
}

impl TextRun {
    pub(crate) fn new(
        base_fragment_info: BaseFragmentInfo,
        inline_styles: SharedInlineStyles,
        text_range: Range<usize>,
        selection_range: Option<TextByteRange>,
    ) -> Self {
        Self {
            base_fragment_info,
            inline_styles,
            text_range,
            shaped_text: Vec::new(),
            selection_range,
        }
    }

    pub(super) fn segment_and_shape(
        &mut self,
        formatting_context_text: &str,
        layout_context: &LayoutContext,
        linebreaker: &mut LineBreaker,
        bidi_info: &BidiInfo,
    ) {
        let parent_style = self.inline_styles.style.borrow().clone();
        let inherited_text_style = parent_style.get_inherited_text().clone();
        let letter_spacing = inherited_text_style
            .letter_spacing
            .0
            .resolve(parent_style.clone_font().font_size.computed_size());
        let letter_spacing = if letter_spacing.px() != 0. {
            Some(app_units::Au::from(letter_spacing))
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

        let specified_word_spacing = &inherited_text_style.word_spacing;
        let style_word_spacing: Option<Au> = specified_word_spacing.to_length().map(|l| l.into());

        let segments = self
            .segment_text_by_font(
                layout_context,
                formatting_context_text,
                bidi_info,
                &parent_style,
            )
            .into_iter()
            .map(|mut segment| {
                let word_spacing = style_word_spacing.unwrap_or_else(|| {
                    let space_width = segment
                        .font
                        .glyph_index(' ')
                        .map(|glyph_id| segment.font.glyph_h_advance(glyph_id))
                        .unwrap_or(LAST_RESORT_GLYPH_ADVANCE);
                    specified_word_spacing.to_used_value(Au::from_f64_px(space_width))
                });

                let mut flags = flags;
                if segment.bidi_level.is_rtl() {
                    flags.insert(ShapingFlags::RTL_FLAG);
                }
                let shaping_options = ShapingOptions {
                    letter_spacing,
                    word_spacing,
                    script: segment.script,
                    flags,
                };

                segment.shape_text(
                    &parent_style,
                    formatting_context_text,
                    linebreaker,
                    &shaping_options,
                );

                segment
            })
            .collect();

        let _ = std::mem::replace(&mut self.shaped_text, segments);
    }

    /// Take the [`TextRun`]'s text and turn it into [`TextRunSegment`]s. Each segment has a matched
    /// font and script. Fonts may differ when glyphs are found in fallback fonts.
    /// [`super::InlineFormattingContext`].
    fn segment_text_by_font(
        &mut self,
        layout_context: &LayoutContext,
        formatting_context_text: &str,
        bidi_info: &BidiInfo,
        parent_style: &Arc<ComputedValues>,
    ) -> Vec<TextRunSegment> {
        let font_group = layout_context
            .font_context
            .font_group(parent_style.clone_font());
        let mut current: Option<TextRunSegment> = None;
        let mut results = Vec::new();

        let text_run_text = &formatting_context_text[self.text_range.clone()];
        let char_iterator = TwoCharsAtATimeIterator::new(text_run_text.chars());
        let mut next_byte_index = self.text_range.start;
        for (character, next_character) in char_iterator {
            let current_byte_index = next_byte_index;
            next_byte_index += character.len_utf8();

            if char_does_not_change_font(character) {
                continue;
            }

            // If the script and BiDi level do not change, use the current font as the first fallback. This
            // can potentially speed up fallback on long font lists or with uncommon scripts which might be
            // at the bottom of the list.
            let script = Script::from(character);
            let bidi_level = bidi_info.levels[current_byte_index];
            let current_font = current.as_ref().and_then(|text_run_segment| {
                if text_run_segment.bidi_level == bidi_level && text_run_segment.script == script {
                    Some(text_run_segment.font.clone())
                } else {
                    None
                }
            });

            let lang = parent_style.get_font()._x_lang.clone();

            let Some(font) = font_group.find_by_codepoint(
                &layout_context.font_context,
                character,
                next_character,
                current_font,
                Some(lang.0.as_ref().to_string()),
            ) else {
                continue;
            };

            // If the existing segment is compatible with the character, keep going.
            if let Some(current) = current.as_mut() {
                if current.update_if_compatible(layout_context, &font, script, bidi_level) {
                    continue;
                }
            }

            // Add the new segment and finish the existing one, if we had one. If the first
            // characters in the run were control characters we may be creating the first
            // segment in the middle of the run (ie the start should be the start of this
            // text run's text).
            let start_byte_index = match current {
                Some(_) => current_byte_index,
                None => self.text_range.start,
            };
            let new = TextRunSegment::new(font, script, bidi_level, start_byte_index);
            if let Some(mut finished) = current.replace(new) {
                // The end of the previous segment is the start of the next one.
                finished.range.end = current_byte_index;
                results.push(finished);
            }
        }

        // Either we have a current segment or we only had control character and whitespace. In both
        // of those cases, just use the first font.
        if current.is_none() {
            current = font_group.first(&layout_context.font_context).map(|font| {
                TextRunSegment::new(font, Script::Common, Level::ltr(), self.text_range.start)
            })
        }

        // Extend the last segment to the end of the string and add it to the results.
        if let Some(mut last_segment) = current.take() {
            last_segment.range.end = self.text_range.end;
            results.push(last_segment);
        }

        results
    }

    pub(super) fn layout_into_line_items(&self, ifc: &mut InlineFormattingContextLayout) {
        if self.text_range.is_empty() {
            return;
        }

        // If we are following replaced content, we should have a soft wrap opportunity, unless the
        // first character of this `TextRun` prevents that soft wrap opportunity. If we see such a
        // character it should also override the LineBreaker's indication to break at the start.
        let have_deferred_soft_wrap_opportunity =
            mem::replace(&mut ifc.have_deferred_soft_wrap_opportunity, false);
        let mut soft_wrap_policy = match have_deferred_soft_wrap_opportunity {
            true => SegmentStartSoftWrapPolicy::Force,
            false => SegmentStartSoftWrapPolicy::FollowLinebreaker,
        };

        for segment in self.shaped_text.iter() {
            segment.layout_into_line_items(self, soft_wrap_policy, ifc);
            soft_wrap_policy = SegmentStartSoftWrapPolicy::FollowLinebreaker;
        }
    }
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
    if is_bidi_control(character) {
        return false;
    }

    let class = linebreak_property(character);
    class == XI_LINE_BREAKING_CLASS_CM ||
        class == XI_LINE_BREAKING_CLASS_GL ||
        class == XI_LINE_BREAKING_CLASS_ZW ||
        class == XI_LINE_BREAKING_CLASS_WJ ||
        class == XI_LINE_BREAKING_CLASS_ZWJ
}

pub(super) fn get_font_for_first_font_for_style(
    style: &ComputedValues,
    font_context: &FontContext,
) -> Option<FontRef> {
    let font = font_context
        .font_group(style.clone_font())
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
        let character = self.next_character?;
        self.next_character = self.iterator.next();
        Some((character, self.next_character))
    }
}
