/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::char::{ToLowercase, ToUppercase};
use std::mem;

use app_units::Au;
use gfx::font::{FontRef, ShapingFlags, ShapingOptions};
use gfx::font_cache_thread::FontCacheThread;
use gfx::font_context::FontContext;
use gfx::text::text_run::GlyphRun;
use gfx_traits::ByteIndex;
use log::warn;
use range::Range;
use serde::Serialize;
use servo_arc::Arc;
use style::computed_values::text_rendering::T as TextRendering;
use style::computed_values::white_space::T as WhiteSpace;
use style::computed_values::word_break::T as WordBreak;
use style::properties::ComputedValues;
use style::values::specified::text::TextTransformCase;
use style::values::specified::TextTransform;
use unicode_script::Script;
use unicode_segmentation::UnicodeSegmentation;
use xi_unicode::{linebreak_property, LineBreakLeafIter};

use super::inline::{FontKeyAndMetrics, InlineFormattingContextState};
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
    pub text: String,

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
    pub range: Range<ByteIndex>,

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
            range: Range::new(byte_index, ByteIndex(0)),
            runs: Vec::new(),
            break_at_start: false,
        }
    }

    /// Update this segment if the Font and Script are compatible. The update will only
    /// ever make the Script specific. Returns true if the new Font and Script are
    /// compatible with this segment or false otherwise.
    fn update_if_compatible(
        &mut self,
        font: &FontRef,
        script: Script,
        fonts: &[FontKeyAndMetrics],
    ) -> bool {
        fn is_specific(script: Script) -> bool {
            script != Script::Common && script != Script::Inherited
        }

        let current_font_key_and_metrics = &fonts[self.font_index];
        let new_font = font.borrow();
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
            if text_run.glyph_run_is_preserved_newline(run) {
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
        text: String,
    ) -> Self {
        Self {
            base_fragment_info,
            parent_style,
            text,
            shaped_text: Vec::new(),
            prevent_soft_wrap_opportunity_at_start: false,
            prevent_soft_wrap_opportunity_at_end: false,
        }
    }

    /// Whether or not this [`TextRun`] has uncollapsible content. This is used
    /// to determine if an [`super::InlineFormattingContext`] is considered empty or not.
    pub(super) fn has_uncollapsible_content(&self) -> bool {
        let white_space = self.parent_style.clone_white_space();
        if white_space.preserve_spaces() && !self.text.is_empty() {
            return true;
        }

        for character in self.text.chars() {
            if !character.is_ascii_whitespace() {
                return true;
            }
            if character == '\n' && white_space.preserve_newlines() {
                return true;
            }
        }

        false
    }

    pub(super) fn break_and_shape(
        &mut self,
        font_context: &mut FontContext<FontCacheThread>,
        linebreaker: &mut Option<LineBreakLeafIter>,
        font_cache: &mut Vec<FontKeyAndMetrics>,
        last_inline_box_ended_with_white_space: &mut bool,
        on_word_boundary: &mut bool,
    ) {
        let segment_results = self.segment_text(
            font_context,
            font_cache,
            last_inline_box_ended_with_white_space,
            on_word_boundary,
        );
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
                let mut font = font.borrow_mut();
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
                (segment.runs, segment.break_at_start) =
                    gfx::text::text_run::TextRun::break_and_shape(
                        &mut font,
                        &self.text
                            [segment.range.begin().0 as usize..segment.range.end().0 as usize],
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
        font_context: &mut FontContext<FontCacheThread>,
        font_cache: &mut Vec<FontKeyAndMetrics>,
        last_inline_box_ended_with_white_space: &mut bool,
        on_word_boundary: &mut bool,
    ) -> Vec<(TextRunSegment, FontRef)> {
        let font_group = font_context.font_group(self.parent_style.clone_font());
        let mut current: Option<(TextRunSegment, FontRef)> = None;
        let mut results = Vec::new();

        // TODO: Eventually the text should come directly from the Cow strings of the DOM nodes.
        let text = std::mem::take(&mut self.text);
        let collapsed = WhitespaceCollapse::new(
            text.as_str().chars(),
            self.parent_style.clone_white_space(),
            *last_inline_box_ended_with_white_space,
        );

        let text_transform = self.parent_style.clone_text_transform();
        let collected_text: String;
        let char_iterator: Box<dyn Iterator<Item = char>> =
            if text_transform.case_ == TextTransformCase::Capitalize {
                // `TextTransformation` doesn't support capitalization, so we must capitalize the whole
                // string at once and make a copy. Here `on_word_boundary` indicates whether or not the
                // inline formatting context as a whole is on a word boundary. This is different from
                // `last_inline_box_ended_with_white_space` because the word boundaries are between
                // atomic inlines and at the start of the IFC.
                let collapsed_string: String = collapsed.collect();
                collected_text = capitalize_string(&collapsed_string, *on_word_boundary);
                Box::new(collected_text.chars())
            } else if !text_transform.is_none() {
                // If `text-transform` is active, wrap the `WhitespaceCollapse` iterator in
                // a `TextTransformation` iterator.
                Box::new(TextTransformation::new(collapsed, text_transform))
            } else {
                Box::new(collapsed)
            };

        let mut next_byte_index = 0;
        let text = char_iterator
            .map(|character| {
                let current_byte_index = next_byte_index;
                next_byte_index += character.len_utf8();

                *last_inline_box_ended_with_white_space = character.is_whitespace();
                *on_word_boundary = *last_inline_box_ended_with_white_space;

                let prevents_soft_wrap_opportunity =
                    char_prevents_soft_wrap_opportunity_when_before_or_after_atomic(character);
                if current_byte_index == 0 && prevents_soft_wrap_opportunity {
                    self.prevent_soft_wrap_opportunity_at_start = true;
                }
                self.prevent_soft_wrap_opportunity_at_end = prevents_soft_wrap_opportunity;

                if char_does_not_change_font(character) {
                    return character;
                }

                let font = match font_group
                    .borrow_mut()
                    .find_by_codepoint(font_context, character)
                {
                    Some(font) => font,
                    None => return character,
                };

                // If the existing segment is compatible with the character, keep going.
                let script = Script::from(character);
                if let Some(current) = current.as_mut() {
                    if current.0.update_if_compatible(&font, script, font_cache) {
                        return character;
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

                character
            })
            .collect();

        let _ = std::mem::replace(&mut self.text, text);

        // Either we have a current segment or we only had control character and whitespace. In both
        // of those cases, just use the first font.
        if current.is_none() {
            current = font_group.borrow_mut().first(font_context).map(|font| {
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
                .extend_to(ByteIndex(self.text.len() as isize));
            results.push(last_segment);
        }

        results
    }

    pub(super) fn layout_into_line_items(&self, ifc: &mut InlineFormattingContextState) {
        if self.text.is_empty() {
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

    pub(super) fn glyph_run_is_preserved_newline(&self, run: &GlyphRun) -> bool {
        if !run.glyph_store.is_whitespace() || run.range.length() != ByteIndex(1) {
            return false;
        }
        if !self
            .parent_style
            .get_inherited_text()
            .white_space
            .preserve_newlines()
        {
            return false;
        }

        let byte = self.text.as_bytes().get(run.range.begin().to_usize());
        byte == Some(&b'\n')
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
    if character.is_whitespace() || character.is_control() {
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
    let font = font.borrow();
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
    font_context: &mut FontContext<FontCacheThread>,
) -> Option<FontRef> {
    let font = font_context
        .font_group(style.clone_font())
        .borrow_mut()
        .first(font_context);
    if font.is_none() {
        warn!("Could not find font for style: {:?}", style.clone_font());
    }
    font
}

fn preserve_segment_break() -> bool {
    true
}

pub struct WhitespaceCollapse<InputIterator> {
    char_iterator: InputIterator,
    white_space: WhiteSpace,

    /// Whether or not we should collapse white space completely at the start of the string.
    /// This is true when the last character handled in our owning [`super::InlineFormattingContext`]
    /// was collapsible white space.
    remove_collapsible_white_space_at_start: bool,

    /// Whether or not the last character produced was newline. There is special behavior
    /// we do after each newline.
    following_newline: bool,

    /// Whether or not we have seen any non-white space characters, indicating that we are not
    /// in a collapsible white space section at the beginning of the string.
    have_seen_non_white_space_characters: bool,

    /// Whether the last character that we processed was a non-newline white space character. When
    /// collapsing white space we need to wait until the next non-white space character or the end
    /// of the string to push a single white space.
    inside_white_space: bool,

    /// When we enter a collapsible white space region, we may need to wait to produce a single
    /// white space character as soon as we encounter a non-white space character. When that
    /// happens we queue up the non-white space character for the next iterator call.
    character_pending_to_return: Option<char>,
}

impl<InputIterator> WhitespaceCollapse<InputIterator> {
    pub fn new(
        char_iterator: InputIterator,
        white_space: WhiteSpace,
        trim_beginning_white_space: bool,
    ) -> Self {
        Self {
            char_iterator,
            white_space,
            remove_collapsible_white_space_at_start: trim_beginning_white_space,
            inside_white_space: false,
            following_newline: false,
            have_seen_non_white_space_characters: false,
            character_pending_to_return: None,
        }
    }

    fn is_leading_trimmed_white_space(&self) -> bool {
        !self.have_seen_non_white_space_characters && self.remove_collapsible_white_space_at_start
    }

    /// Whether or not we need to produce a space character if the next character is not a newline
    /// and not white space. This happens when we are exiting a section of white space and we
    /// waited to produce a single space character for the entire section of white space (but
    /// not following or preceding a newline).
    fn need_to_produce_space_character_after_white_space(&self) -> bool {
        self.inside_white_space && !self.following_newline && !self.is_leading_trimmed_white_space()
    }
}

impl<InputIterator> Iterator for WhitespaceCollapse<InputIterator>
where
    InputIterator: Iterator<Item = char>,
{
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        // Point 4.1.1 first bullet:
        // > If white-space is set to normal, nowrap, or pre-line, whitespace
        // > characters are considered collapsible
        // If whitespace is not considered collapsible, it is preserved entirely, which
        // means that we can simply return the input string exactly.
        if self.white_space.preserve_spaces() {
            return self.char_iterator.next();
        }

        if let Some(character) = self.character_pending_to_return.take() {
            self.inside_white_space = false;
            self.have_seen_non_white_space_characters = true;
            self.following_newline = false;
            return Some(character);
        }

        while let Some(character) = self.char_iterator.next() {
            // Don't push non-newline whitespace immediately. Instead wait to push it until we
            // know that it isn't followed by a newline. See `push_pending_whitespace_if_needed`
            // above.
            if character.is_ascii_whitespace() && character != '\n' {
                self.inside_white_space = true;
                continue;
            }

            // Point 4.1.1:
            // > 2. Collapsible segment breaks are transformed for rendering according to the
            // >    segment break transformation rules.
            if character == '\n' {
                // From <https://drafts.csswg.org/css-text-3/#line-break-transform>
                // (4.1.3 -- the segment break transformation rules):
                //
                // > When white-space is pre, pre-wrap, or pre-line, segment breaks are not
                // > collapsible and are instead transformed into a preserved line feed"
                if self.white_space == WhiteSpace::PreLine {
                    self.inside_white_space = false;
                    self.following_newline = true;
                    return Some(character);

                // Point 4.1.3:
                // > 1. First, any collapsible segment break immediately following another
                // >    collapsible segment break is removed.
                // > 2. Then any remaining segment break is either transformed into a space (U+0020)
                // >    or removed depending on the context before and after the break.
                } else if !self.following_newline &&
                    preserve_segment_break() &&
                    !self.is_leading_trimmed_white_space()
                {
                    self.inside_white_space = false;
                    self.following_newline = true;
                    return Some(' ');
                } else {
                    self.following_newline = true;
                    continue;
                }
            }

            // Point 4.1.1:
            // > 2. Any sequence of collapsible spaces and tabs immediately preceding or
            // >    following a segment break is removed.
            // > 3. Every collapsible tab is converted to a collapsible space (U+0020).
            // > 4. Any collapsible space immediately following another collapsible space—even
            // >    one outside the boundary of the inline containing that space, provided both
            // >    spaces are within the same inline formatting context—is collapsed to have zero
            // >    advance width.
            if self.need_to_produce_space_character_after_white_space() {
                self.inside_white_space = false;
                self.character_pending_to_return = Some(character);
                return Some(' ');
            }

            self.inside_white_space = false;
            self.have_seen_non_white_space_characters = true;
            self.following_newline = false;
            return Some(character);
        }

        if self.need_to_produce_space_character_after_white_space() {
            self.inside_white_space = false;
            return Some(' ');
        }

        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.char_iterator.size_hint()
    }

    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.char_iterator.count()
    }
}

enum PendingCaseConversionResult {
    Uppercase(ToUppercase),
    Lowercase(ToLowercase),
}

impl PendingCaseConversionResult {
    fn next(&mut self) -> Option<char> {
        match self {
            PendingCaseConversionResult::Uppercase(to_uppercase) => to_uppercase.next(),
            PendingCaseConversionResult::Lowercase(to_lowercase) => to_lowercase.next(),
        }
    }
}

/// This is an interator that consumes a char iterator and produces character transformed
/// by the given CSS `text-transform` value. It currently does not support
/// `text-transform: capitalize` because Unicode segmentation libraries do not support
/// streaming input one character at a time.
pub struct TextTransformation<InputIterator> {
    /// The input character iterator.
    char_iterator: InputIterator,
    /// The `text-transform` value to use.
    text_transform: TextTransform,
    /// If an uppercasing or lowercasing produces more than one character, this
    /// caches them so that they can be returned in subsequent iterator calls.
    pending_case_conversion_result: Option<PendingCaseConversionResult>,
}

impl<InputIterator> TextTransformation<InputIterator> {
    pub fn new(char_iterator: InputIterator, text_transform: TextTransform) -> Self {
        Self {
            char_iterator,
            text_transform,
            pending_case_conversion_result: None,
        }
    }
}

impl<InputIterator> Iterator for TextTransformation<InputIterator>
where
    InputIterator: Iterator<Item = char>,
{
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(character) = self
            .pending_case_conversion_result
            .as_mut()
            .and_then(|result| result.next())
        {
            return Some(character);
        }
        self.pending_case_conversion_result = None;

        for character in self.char_iterator.by_ref() {
            match self.text_transform.case_ {
                TextTransformCase::None => return Some(character),
                TextTransformCase::Uppercase => {
                    let mut pending_result =
                        PendingCaseConversionResult::Uppercase(character.to_uppercase());
                    if let Some(character) = pending_result.next() {
                        self.pending_case_conversion_result = Some(pending_result);
                        return Some(character);
                    }
                },
                TextTransformCase::Lowercase => {
                    let mut pending_result =
                        PendingCaseConversionResult::Lowercase(character.to_lowercase());
                    if let Some(character) = pending_result.next() {
                        self.pending_case_conversion_result = Some(pending_result);
                        return Some(character);
                    }
                },
                // `text-transform: capitalize` currently cannot work on a per-character basis,
                // so must be handled outside of this iterator.
                // TODO: Add support for `full-width` and `full-size-kana`.
                _ => return Some(character),
            }
        }
        None
    }
}

/// Given a string and whether the start of the string represents a word boundary, create a copy of
/// the string with letters after word boundaries capitalized.
fn capitalize_string(string: &str, allow_word_at_start: bool) -> String {
    let mut output_string = String::new();
    output_string.reserve(string.len());

    let mut bounds = string.unicode_word_indices().peekable();
    let mut byte_index = 0;
    for character in string.chars() {
        let current_byte_index = byte_index;
        byte_index += character.len_utf8();

        if let Some((next_index, _)) = bounds.peek() {
            if *next_index == current_byte_index {
                bounds.next();

                if current_byte_index != 0 || allow_word_at_start {
                    output_string.extend(character.to_uppercase());
                    continue;
                }
            }
        }

        output_string.push(character);
    }

    output_string
}
