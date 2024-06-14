/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::Cow;
use std::char::{ToLowercase, ToUppercase};

use style::computed_values::white_space_collapse::T as WhiteSpaceCollapse;
use style::values::computed::{TextDecorationLine, TextTransform};
use style::values::specified::text::TextTransformCase;
use unicode_segmentation::UnicodeSegmentation;

use super::text_run::TextRun;
use super::{InlineBox, InlineFormattingContext, InlineLevelBox};
use crate::cell::ArcRefCell;
use crate::context::LayoutContext;
use crate::dom::NodeExt;
use crate::dom_traversal::NodeAndStyleInfo;
use crate::flow::float::FloatBox;
use crate::formatting_contexts::IndependentFormattingContext;
use crate::positioned::AbsolutelyPositionedBox;

#[derive(Default)]
pub(crate) struct InlineFormattingContextBuilder {
    pub text_segments: Vec<String>,
    current_text_offset: usize,

    /// Whether the last processed node ended with whitespace. This is used to
    /// implement rule 4 of <https://www.w3.org/TR/css-text-3/#collapse>:
    ///
    /// > Any collapsible space immediately following another collapsible space—even one
    /// > outside the boundary of the inline containing that space, provided both spaces are
    /// > within the same inline formatting context—is collapsed to have zero advance width.
    /// > (It is invisible, but retains its soft wrap opportunity, if any.)
    last_inline_box_ended_with_collapsible_white_space: bool,

    /// Whether or not the current state of the inline formatting context is on a word boundary
    /// for the purposes of `text-transform: capitalize`.
    on_word_boundary: bool,

    /// Whether or not this inline formatting context will contain floats.
    pub contains_floats: bool,

    /// Inline elements are direct descendants of the element that establishes
    /// the inline formatting context that this builder builds.
    pub root_inline_boxes: Vec<ArcRefCell<InlineLevelBox>>,

    /// Whether or not the inline formatting context under construction has any
    /// uncollapsible text content.
    pub has_uncollapsible_text_content: bool,

    /// The ongoing stack of inline boxes stack of the builder.
    ///
    /// Contains all the currently ongoing inline boxes we entered so far.
    /// The traversal is at all times as deep in the tree as this stack is,
    /// which is why the code doesn't need to keep track of the actual
    /// container root (see `handle_inline_level_element`).
    ///
    /// Whenever the end of a DOM element that represents an inline box is
    /// reached, the inline box at the top of this stack is complete and ready
    /// to be pushed to the children of the next last ongoing inline box
    /// the ongoing inline formatting context if the stack is now empty,
    /// which means we reached the end of a child of the actual
    /// container root (see `move_to_next_sibling`).
    ///
    /// When an inline box ends, it's removed from this stack and added to
    /// [`Self::root_inline_boxes`].
    inline_box_stack: Vec<InlineBox>,
}

impl InlineFormattingContextBuilder {
    pub(crate) fn new() -> Self {
        // For the purposes of `text-transform: capitalize` the start of the IFC is a word boundary.
        Self {
            on_word_boundary: true,
            ..Default::default()
        }
    }

    pub(crate) fn currently_processing_inline_box(&self) -> bool {
        !self.inline_box_stack.is_empty()
    }

    /// Return true if this [`InlineFormattingContextBuilder`] is empty for the purposes of ignoring
    /// during box tree construction. An IFC is empty if it only contains TextRuns with
    /// completely collapsible whitespace. When that happens it can be ignored completely.
    pub(crate) fn is_empty(&self) -> bool {
        if self.has_uncollapsible_text_content {
            return false;
        }

        if !self.inline_box_stack.is_empty() {
            return false;
        }

        fn inline_level_boxes_are_empty(boxes: &[ArcRefCell<InlineLevelBox>]) -> bool {
            boxes
                .iter()
                .all(|inline_level_box| inline_level_box_is_empty(&inline_level_box.borrow()))
        }

        fn inline_level_box_is_empty(inline_level_box: &InlineLevelBox) -> bool {
            match inline_level_box {
                InlineLevelBox::InlineBox(_) => false,
                // Text content is handled by `self.has_uncollapsible_text` content above in order
                // to avoid having to iterate through the character once again.
                InlineLevelBox::TextRun(_) => true,
                InlineLevelBox::OutOfFlowAbsolutelyPositionedBox(_) => false,
                InlineLevelBox::OutOfFlowFloatBox(_) => false,
                InlineLevelBox::Atomic(_) => false,
            }
        }

        inline_level_boxes_are_empty(&self.root_inline_boxes)
    }

    // Retrieves the mutable reference of inline boxes either from the last
    // element of a stack or directly from the formatting context, depending on the situation.
    fn current_inline_level_boxes(&mut self) -> &mut Vec<ArcRefCell<InlineLevelBox>> {
        match self.inline_box_stack.last_mut() {
            Some(last) => &mut last.children,
            None => &mut self.root_inline_boxes,
        }
    }

    pub(crate) fn push_atomic(
        &mut self,
        independent_formatting_context: IndependentFormattingContext,
    ) -> ArcRefCell<InlineLevelBox> {
        let inline_level_box =
            ArcRefCell::new(InlineLevelBox::Atomic(independent_formatting_context));
        self.current_inline_level_boxes()
            .push(inline_level_box.clone());

        // Push an object replacement character for this atomic, which will ensure that the line breaker
        // inserts a line breaking opportunity here.
        let string_to_push = "\u{fffc}";
        self.text_segments.push(string_to_push.to_owned());
        self.current_text_offset += string_to_push.len();

        self.last_inline_box_ended_with_collapsible_white_space = false;
        self.on_word_boundary = true;

        inline_level_box
    }

    pub(crate) fn push_absolutely_positioned_box(
        &mut self,
        absolutely_positioned_box: AbsolutelyPositionedBox,
    ) -> ArcRefCell<InlineLevelBox> {
        let absolutely_positioned_box = ArcRefCell::new(absolutely_positioned_box);
        let inline_level_box = ArcRefCell::new(InlineLevelBox::OutOfFlowAbsolutelyPositionedBox(
            absolutely_positioned_box,
        ));
        self.current_inline_level_boxes()
            .push(inline_level_box.clone());
        inline_level_box
    }

    pub(crate) fn push_float_box(&mut self, float_box: FloatBox) -> ArcRefCell<InlineLevelBox> {
        let inline_level_box = ArcRefCell::new(InlineLevelBox::OutOfFlowFloatBox(float_box));
        self.current_inline_level_boxes()
            .push(inline_level_box.clone());
        self.contains_floats = true;
        inline_level_box
    }

    pub(crate) fn start_inline_box<'dom, Node: NodeExt<'dom>>(
        &mut self,
        info: &NodeAndStyleInfo<Node>,
    ) {
        self.inline_box_stack.push(InlineBox::new(info))
    }

    pub(crate) fn end_inline_box(&mut self) -> ArcRefCell<InlineLevelBox> {
        self.end_inline_box_internal(true)
    }

    fn end_inline_box_internal(&mut self, is_last_fragment: bool) -> ArcRefCell<InlineLevelBox> {
        let mut inline_box = self
            .inline_box_stack
            .pop()
            .expect("no ongoing inline level box found");

        if is_last_fragment {
            inline_box.is_last_fragment = true;
        }

        let inline_box = ArcRefCell::new(InlineLevelBox::InlineBox(inline_box));
        self.current_inline_level_boxes().push(inline_box.clone());

        inline_box
    }

    pub(crate) fn push_text<'dom, Node: NodeExt<'dom>>(
        &mut self,
        text: Cow<'dom, str>,
        info: &NodeAndStyleInfo<Node>,
    ) {
        let white_space_collapse = info.style.clone_white_space_collapse();
        let collapsed = WhitespaceCollapse::new(
            text.chars(),
            white_space_collapse,
            self.last_inline_box_ended_with_collapsible_white_space,
        );

        let text_transform = info.style.clone_text_transform();
        let capitalized_text: String;
        let char_iterator: Box<dyn Iterator<Item = char>> =
            if text_transform.case_ == TextTransformCase::Capitalize {
                // `TextTransformation` doesn't support capitalization, so we must capitalize the whole
                // string at once and make a copy. Here `on_word_boundary` indicates whether or not the
                // inline formatting context as a whole is on a word boundary. This is different from
                // `last_inline_box_ended_with_collapsible_white_space` because the word boundaries are
                // between atomic inlines and at the start of the IFC, and because preserved spaces
                // are a word boundary.
                let collapsed_string: String = collapsed.collect();
                capitalized_text = capitalize_string(&collapsed_string, self.on_word_boundary);
                Box::new(capitalized_text.chars())
            } else if !text_transform.is_none() {
                // If `text-transform` is active, wrap the `WhitespaceCollapse` iterator in
                // a `TextTransformation` iterator.
                Box::new(TextTransformation::new(collapsed, text_transform))
            } else {
                Box::new(collapsed)
            };

        let white_space_collapse = info.style.clone_white_space_collapse();
        let new_text: String = char_iterator
            .map(|character| {
                self.has_uncollapsible_text_content |= matches!(
                    white_space_collapse,
                    WhiteSpaceCollapse::Preserve | WhiteSpaceCollapse::BreakSpaces
                ) || !character.is_ascii_whitespace() ||
                    (character == '\n' && white_space_collapse != WhiteSpaceCollapse::Collapse);
                character
            })
            .collect();

        if new_text.is_empty() {
            return;
        }

        if let Some(last_character) = new_text.chars().next_back() {
            self.on_word_boundary = last_character.is_whitespace();
            self.last_inline_box_ended_with_collapsible_white_space =
                self.on_word_boundary && white_space_collapse != WhiteSpaceCollapse::Preserve;
        }

        let new_range = self.current_text_offset..self.current_text_offset + new_text.len();
        self.current_text_offset = new_range.end;
        self.text_segments.push(new_text);

        let inlines = self.current_inline_level_boxes();
        if let Some(mut last_box) = inlines.last_mut().map(|last| last.borrow_mut()) {
            if let InlineLevelBox::TextRun(ref mut text_run) = *last_box {
                text_run.text_range.end = new_range.end;
                return;
            }
        }

        inlines.push(ArcRefCell::new(InlineLevelBox::TextRun(TextRun::new(
            info.into(),
            info.style.clone(),
            new_range,
        ))));
    }

    pub(crate) fn split_around_block_and_finish(
        &mut self,
        layout_context: &LayoutContext,
        text_decoration_line: TextDecorationLine,
        has_first_formatted_line: bool,
    ) -> Option<InlineFormattingContext> {
        if self.is_empty() {
            return None;
        }

        // Create a new inline builder which will be active after the block splits this inline formatting
        // context. It has the same inline box structure as this builder, except the boxes are
        // marked as not being the first fragment. No inline content is carried over to this new
        // builder.
        let mut inline_buidler_from_before_split = std::mem::replace(
            self,
            InlineFormattingContextBuilder {
                on_word_boundary: true,
                inline_box_stack: self
                    .inline_box_stack
                    .iter()
                    .map(|inline_box| inline_box.split_around_block())
                    .collect(),
                ..Default::default()
            },
        );

        // End all ongoing inline boxes in the first builder, but ensure that they are not
        // marked as the final fragments, so that they do not get inline end margin, borders,
        // and padding.
        while !inline_buidler_from_before_split.inline_box_stack.is_empty() {
            inline_buidler_from_before_split.end_inline_box_internal(false);
        }

        inline_buidler_from_before_split.finish(
            layout_context,
            text_decoration_line,
            has_first_formatted_line,
        )
    }

    /// Finish the current inline formatting context, returning [`None`] if the context was empty.
    pub(crate) fn finish(
        &mut self,
        layout_context: &LayoutContext,
        text_decoration_line: TextDecorationLine,
        has_first_formatted_line: bool,
    ) -> Option<InlineFormattingContext> {
        if self.is_empty() {
            return None;
        }

        let old_builder = std::mem::replace(self, InlineFormattingContextBuilder::new());
        assert!(old_builder.inline_box_stack.is_empty());

        Some(InlineFormattingContext::new_with_builder(
            old_builder,
            layout_context,
            text_decoration_line,
            has_first_formatted_line,
        ))
    }
}

fn preserve_segment_break() -> bool {
    true
}

pub struct WhitespaceCollapse<InputIterator> {
    char_iterator: InputIterator,
    white_space_collapse: WhiteSpaceCollapse,

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
        white_space_collapse: WhiteSpaceCollapse,
        trim_beginning_white_space: bool,
    ) -> Self {
        Self {
            char_iterator,
            white_space_collapse,
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
        if self.white_space_collapse == WhiteSpaceCollapse::Preserve ||
            self.white_space_collapse == WhiteSpaceCollapse::BreakSpaces
        {
            // From <https://drafts.csswg.org/css-text-3/#white-space-processing>:
            // > Carriage returns (U+000D) are treated identically to spaces (U+0020) in all respects.
            //
            // In the non-preserved case these are converted to space below.
            return match self.char_iterator.next() {
                Some('\r') => Some(' '),
                next => next,
            };
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
                if self.white_space_collapse != WhiteSpaceCollapse::Collapse {
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
