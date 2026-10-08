/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ops::Range;
use std::sync::Arc;

use fonts::{ShapedText, ShapedTextSlice, ShapedTextSlicer, ShapingOptions, TrailingWhiteSpace};
use icu_properties::props::{EnumeratedProperty, GeneralCategory, LineBreak};
use icu_segmenter::options::LineBreakOptions;
use servo_base::text::{AssumeUnder4GB, Utf8CodeUnits, Utf32CodeUnits};
use style::computed_values::text_wrap_mode::T as TextWrapMode;
use style::computed_values::white_space_collapse::T as WhiteSpaceCollapse;
use style::computed_values::word_break::T as WordBreak;
use style::properties::ComputedValues;
use style::properties::style_structs::InheritedText;
use unicode_script::Script;

use crate::ArcRefCell;
use crate::flow::inline::line_breaker::LineBreaker;
use crate::flow::inline::text_run::{FontAndScriptInfo, TextRun, TextRunItem, script_is_specific};

/// An entry on the shaping queue that represents text that needs to be shaped.
/// This contains a lot of duplicated data from `TextRunSegment` so that
/// it can outlive a mutable borrow on the owning `TextRun`.
pub(crate) struct ShapingQueueText {
    info: FontAndScriptInfo,
    byte_range: Range<Utf8CodeUnits>,
    character_range: Range<Utf32CodeUnits>,
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
    character_offset_origin: Utf32CodeUnits,
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
        let text_style = parent_style.get_inherited_text();
        let mut break_at_start =
            self.line_breaker.take_additional_break_at_start() == Some(range.start);
        let linebreaks = self
            .line_breaker
            .advance_to_linebreaks_in_range(segment.byte_range.clone());
        let linebreak_iter = linebreaks.iter().chain(std::iter::once(&range.end));

        let mut current_character_offset =
            segment.character_range.start - self.character_offset_origin;

        let mut slices = Vec::with_capacity(linebreaks.len());
        let mut maybe_push_slice_and_update_character_offset = |slice_text: &str| {
            current_character_offset += Utf32CodeUnits::length_of(AssumeUnder4GB, slice_text);
            let (trailing_white_space, all_white_space) =
                trailing_white_space_of(slice_text, parent_style);
            if let Some(slice) = self.slicer.slice_until_character_offset(
                current_character_offset,
                trailing_white_space,
                all_white_space,
            ) {
                slices.push(slice);
            }
        };

        let mut last_slice_end = segment.byte_range.start;
        for break_index in linebreak_iter {
            if *break_index == segment.byte_range.start &&
                !line_break_ignored_for_keep_all(self.text, text_style, *break_index)
            {
                break_at_start = true;
                continue;
            }

            let slice = last_slice_end..*break_index;

            // `keep-all` might suppress line breaks between certain characters and that check
            // is done here, but not in the case that the line break is the last one for this
            // segment (in order to push the rest of the text).
            if *break_index != segment.byte_range.end &&
                line_break_ignored_for_keep_all(self.text, text_style, *break_index)
            {
                continue;
            }

            last_slice_end = *break_index;
            if slice.is_empty() {
                continue;
            }

            let full_slice_text = &self.text[Utf8CodeUnits::to_usize_range(&slice)];
            if text_style.white_space_collapse != WhiteSpaceCollapse::BreakSpaces {
                maybe_push_slice_and_update_character_offset(full_slice_text)
            } else {
                for slice_text in full_slice_text.split_inclusive(breaks_for_break_spaces) {
                    maybe_push_slice_and_update_character_offset(slice_text)
                }
            }
        }

        if text_style.white_space_collapse == WhiteSpaceCollapse::BreakSpaces &&
            self.text[Utf8CodeUnits::to_usize_range(&segment.byte_range)]
                .chars()
                .last()
                .is_some_and(breaks_for_break_spaces)
        {
            self.line_breaker
                .set_additional_break_at_start(segment.byte_range.end);
        }

        (slices, break_at_start)
    }
}

fn line_break_ignored_for_keep_all(
    text: &str,
    text_style: &InheritedText,
    break_index: Utf8CodeUnits,
) -> bool {
    if text_style.word_break != WordBreak::KeepAll {
        return false;
    }

    let break_index = break_index.0 as usize;
    let text_before = &text[..break_index];
    let Some(character_before) = text_before.chars().rev().find(|character| {
        !matches!(
            LineBreak::for_char(*character),
            LineBreak::CombiningMark | LineBreak::ZWJ,
        )
    }) else {
        return false;
    };

    if !suppresses_line_break_for_keep_all(character_before) {
        return false;
    }

    text[break_index..]
        .chars()
        .next()
        .is_some_and(suppresses_line_break_for_keep_all)
}

/// From <https://drafts.csswg.org/css-text-4/#valdef-word-break-keep-all>:
/// > Breaking is forbidden within “words”: implicit soft wrap opportunities between
/// > typographic letter units (or other typographic character units belonging to the NU,
/// > AL, AI, or ID Unicode line breaking classes [UAX14]) are suppressed, i.e. breaks are
/// > prohibited between pairs of such characters (regardless of line-break settings other
/// > than anywhere) except where opportunities exist due to § 6.1.1.1 Lexical Word
/// > Breaking. Otherwise this option is equivalent to normal. In this style, sequences of
/// > CJK characters do not break.
///
/// From <https://drafts.csswg.org/css-text-4/#typographic-letter-unit>:
/// > A typographic letter unit (or letter for the purpose of this specification) is a
/// > typographic character unit belonging to one of the Letter or Number general
/// > categories. See Appendix E: Characters and Properties for how to determine the Unicode
/// > properties of a typographic character unit.
fn suppresses_line_break_for_keep_all(character: char) -> bool {
    let line_break_class = LineBreak::for_char(character);
    (matches!(
        GeneralCategory::for_char(character),
        GeneralCategory::UppercaseLetter |
            GeneralCategory::LowercaseLetter |
            GeneralCategory::TitlecaseLetter |
            GeneralCategory::ModifierLetter |
            GeneralCategory::OtherLetter |
            GeneralCategory::DecimalNumber |
            GeneralCategory::LetterNumber |
            GeneralCategory::OtherNumber
    ) || matches!(
        line_break_class,
        LineBreak::Numeric | LineBreak::Alphabetic | LineBreak::Ambiguous | LineBreak::Ideographic
    )) &&
    // From <https://drafts.csswg.org/css-text-4/#lexical-breaking>:
    // > To provide the expected normal behavior for Southeast Asian languages, typographic
    // > character units with line breaking class SA in [UAX14] must be treated as if they
    // > had class AL. However, the user agent must additionally analyze the content of a
    // > run of such characters to detect word boundaries and treat each boundary as a soft
    // > wrap opportunities.
    //
    // `ComplexContext` is the SA class here. `break-all` must not suppress dictionary-based
    // word breaks inside SA text.
    !matches!(line_break_class, LineBreak::ComplexContext)
}

fn breaks_for_break_spaces(character: char) -> bool {
    match CssTextType::from(character) {
        CssTextType::NonWhiteSpace => false,
        CssTextType::DocumentWhiteSpace => true,
        // From <https://www.unicode.org/reports/tr14/tr14-57.html#GL>:
        // > Non-breaking characters prohibit breaks on either side, but that prohibition
        // > can be overridden by SP or ZW.
        //
        // The specification also marks this class of characters as non-tailorable.
        CssTextType::OtherSpaceSeparator => LineBreak::for_char(character) != LineBreak::Glue,
    }
}

/// Returns a tuple containing the [`TrailingWhiteSpace`] values for the text and boolean
/// indicating whether all of the content was white space.
fn trailing_white_space_of(
    text: &str,
    style: &ComputedValues,
) -> (TrailingWhiteSpace<Utf32CodeUnits>, bool) {
    let (anything_hangable, anything_removable) =
        match (style.get_white_space_collapse(), style.get_text_wrap_mode()) {
            (WhiteSpaceCollapse::BreakSpaces, _) |
            (WhiteSpaceCollapse::Preserve, TextWrapMode::Nowrap) => (false, false),
            (WhiteSpaceCollapse::Preserve, TextWrapMode::Wrap) => (true, false),
            _ => (true, true),
        };

    let mut removable = 0;
    let mut hangable = 0;
    let mut all_white_space = true;
    for character in text.chars().rev() {
        match CssTextType::from(character) {
            CssTextType::NonWhiteSpace => {
                all_white_space = false;
                break;
            },
            CssTextType::DocumentWhiteSpace if hangable == 0 && anything_removable => {
                removable += 1;
            },
            CssTextType::DocumentWhiteSpace | CssTextType::OtherSpaceSeparator
                if anything_hangable =>
            {
                hangable += 1;
            },
            _ => {},
        }
    }

    (
        TrailingWhiteSpace {
            hangable: Utf32CodeUnits(hangable),
            removable: Utf32CodeUnits(removable),
        },
        all_white_space,
    )
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
    byte_range: Range<Utf8CodeUnits>,
    /// The character range of the text to shape in [`Self::text`] for the current batch.
    /// Only contiguous ranges can be shaped together.
    character_range: Range<Utf32CodeUnits>,
    /// The resolved script for the current batch. This is used to gradually turn non-specific
    /// scripts into a resolved value for shaping.
    resolved_script: Option<Script>,
}

impl<'a> ShapingQueue<'a> {
    pub(crate) fn new(text: &'a str, line_break_options: LineBreakOptions<'_>) -> Self {
        Self {
            queue: Default::default(),
            text,
            line_breaker: LineBreaker::new(text, line_break_options),
            byte_range: Default::default(),
            character_range: Default::default(),
            resolved_script: None,
        }
    }

    fn compatible_old_shaping_result(
        &self,
        character_count: Utf32CodeUnits,
    ) -> Option<Arc<ShapedText>> {
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
        Some(font.shape_text(
            &self.text[Utf8CodeUnits::to_usize_range(&self.byte_range)],
            &options,
        ))
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CssTextType {
    /// A [`char`] that is non-space content.
    NonWhiteSpace,
    /// <https://drafts.csswg.org/css-text-3/#white-space>.
    DocumentWhiteSpace,
    /// <https://drafts.csswg.org/css-text-3/#other-space-separators>.
    OtherSpaceSeparator,
}

impl From<char> for CssTextType {
    fn from(character: char) -> Self {
        match character {
            // See <https://drafts.csswg.org/css-text-3/#white-space>:
            // > Except where specified otherwise, white space processing in CSS affects only the
            // > document white space characters: spaces (U+0020), tabs (U+0009), and segment breaks.
            ' ' | '\t' | '\n' | '\r' => Self::DocumentWhiteSpace,
            // This is a fast path to avoid having to do Unicode category classification for ASCII
            // characters.
            _ if character.is_ascii() => Self::NonWhiteSpace,
            // See <https://drafts.csswg.org/css-text-3/#other-space-separators>:
            // > Besides space (U+0020) and no-break space (U+00A0), Unicode defines a number of
            // > additional space separator characters. [UNICODE] In this specification all characters
            // > in the Unicode general category Zs except space (U+0020) and no-break space (U+00A0)
            // > are collectively referred to as other space separators.
            //
            // Note: ' ' (space) is handled above.
            _ if GeneralCategory::for_char(character) == GeneralCategory::SpaceSeparator &&
                character != '\u{00a0}' =>
            {
                Self::OtherSpaceSeparator
            },
            _ => Self::NonWhiteSpace,
        }
    }
}
