/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::Cow;
use std::cell::LazyCell;
use std::ops::Range;
use std::sync::Arc;

use atomic_refcell::AtomicRefCell;
use fonts::TextByteRange;
use icu_properties::BidiClass;
use layout_api::{LayoutNode, ScriptSelection};
use servo_base::text::{RangeAny, Utf32CodeUnits};
use style::computed_values::direction::T as Direction;
use style::computed_values::white_space_collapse::T as WhiteSpaceCollapse;
use style::dom::NodeInfo;
use style::selector_parser::PseudoElement;
use unicode_bidi::Level;
use unicode_categories::UnicodeCategories;

use super::text_run::TextRun;
use super::{
    InlineBox, InlineBoxIdentifier, InlineBoxes, InlineFormattingContext, InlineItem,
    SharedInlineStyles,
};
use crate::cell::ArcRefCell;
use crate::context::LayoutContext;
use crate::dom::{LayoutBox, NodeExt};
use crate::dom_traversal::{BoxTreeString, NodeAndStyleInfo};
use crate::flow::BlockLevelBox;
use crate::flow::float::FloatBox;
use crate::flow::inline::text_transform::{OffsetMap, TextTransformationIterator};
use crate::formatting_contexts::IndependentFormattingContext;
use crate::positioned::AbsolutelyPositionedBox;
use crate::style_ext::ComputedValuesExt;

#[derive(Default)]
pub(crate) struct InlineFormattingContextBuilder {
    /// A stack of [`SharedInlineStyles`] including one for the root, one for each inline box on the
    /// inline box stack, and importantly, one for every `display: contents` element that we are
    /// currently processing. Normally `display: contents` elements don't affect the structure of
    /// the [`InlineFormattingContext`], but the styles they provide do style their children.
    pub shared_inline_styles_stack: Vec<SharedInlineStyles>,

    /// The collection of text strings that make up this [`InlineFormattingContext`] under
    /// construction.
    pub text_segments: Vec<String>,

    /// The current offset in the final text string of this [`InlineFormattingContext`],
    /// used to properly set the text range of new [`InlineItem::TextRun`]s.
    current_text_offset: usize,

    /// The current character offset in the final text string of this [`InlineFormattingContext`],
    /// used to properly set the text range of new [`InlineItem::TextRun`]s. Note that this is
    /// different from the UTF-8 code point offset.
    current_character_offset: usize,

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

    /// The current list of [`InlineItem`]s in this [`InlineFormattingContext`] under
    /// construction. This is stored in a flat list to make it easy to access the last
    /// item.
    pub inline_items: Vec<InlineItem>,

    /// The current [`InlineBox`] tree of this [`InlineFormattingContext`] under construction.
    pub inline_boxes: InlineBoxes,

    /// The ongoing stack of inline boxes stack of the builder.
    ///
    /// Contains all the currently ongoing inline boxes we entered so far.
    /// The traversal is at all times as deep in the tree as this stack is,
    /// which is why the code doesn't need to keep track of the actual
    /// container root (see `handle_inline_level_element`).
    ///
    /// When an inline box ends, it's removed from this stack.
    inline_box_stack: Vec<InlineBoxIdentifier>,

    /// Whether this [`InlineFormattingContextBuilder`] is empty for the purposes of ignoring
    /// during box tree construction. An IFC is empty if it only contains TextRuns with
    /// completely collapsible whitespace. When that happens it can be ignored completely.
    pub is_empty: bool,

    /// Whether or not the `::first-letter` pseudo-element of this inline formatting context
    /// has been processed yet.
    has_processed_first_letter: bool,

    /// Whether or not the inline formatting context under construction has any kind of
    /// right-to-left content such as a character with an RTL character class or a `dir`
    /// attribute specifying right-to-left content.
    pub has_right_to_left_content: bool,

    /// An [`OffsetMap`] used to map selections from their offset before inline formatting
    /// context text transformation to their offsets after transformation.
    pub offset_map: OffsetMap,
}

impl InlineFormattingContextBuilder {
    /// <https://drafts.csswg.org/css-text/#white-space>:
    /// > Except where specified otherwise, white space processing in CSS affects only the document
    /// > white space characters: spaces (U+0020), tabs (U+0009), and segment breaks.
    ///
    /// From <https://github.com/w3c/csswg-drafts/issues/5147#issuecomment-637816669>:
    /// > HTML clearly treats CR, LF, and CRLF as segment breaks.
    ///
    /// Other browsers also consider the form feed character (0x0c) to be document white space, it
    /// seems.
    ///
    /// Taken all together, this is equivalent to the WhatWG Infra Standard's definition of ASCII
    /// white space.
    pub(crate) fn is_document_white_space(character: char) -> bool {
        character.is_ascii_whitespace()
    }

    pub(crate) fn new(info: &NodeAndStyleInfo, context: &LayoutContext) -> Self {
        let has_right_to_left_content = info.style.get_inherited_box().direction == Direction::Rtl;
        Self {
            // For the purposes of `text-transform: capitalize` the start of the IFC is a word boundary.
            on_word_boundary: true,
            is_empty: true,
            shared_inline_styles_stack: vec![SharedInlineStyles::from_info_and_context(
                info, context,
            )],
            has_right_to_left_content,
            ..Default::default()
        }
    }

    pub(crate) fn currently_processing_inline_box(&self) -> bool {
        !self.inline_box_stack.is_empty()
    }

    fn push_control_character_string(&mut self, string_to_push: &str) {
        self.text_segments.push(string_to_push.to_owned());
        self.current_text_offset += string_to_push.len();

        let new_characters = Utf32CodeUnits::length_of(string_to_push);
        self.current_character_offset += new_characters.0;
        self.offset_map.push_range(new_characters, new_characters);
    }

    fn shared_inline_styles(&self) -> SharedInlineStyles {
        self.shared_inline_styles_stack
            .last()
            .expect("Should always have at least one SharedInlineStyles")
            .clone()
    }

    pub(crate) fn push_atomic(
        &mut self,
        independent_formatting_context_creator: impl FnOnce()
            -> ArcRefCell<IndependentFormattingContext>,
        old_layout_box: Option<LayoutBox>,
    ) -> InlineItem {
        // If there is an existing undamaged layout box that's compatible, use that.
        let independent_formatting_context = old_layout_box
            .and_then(|layout_box| match layout_box {
                LayoutBox::InlineLevel(InlineItem::Atomic(atomic, ..)) => Some(atomic),
                _ => None,
            })
            .unwrap_or_else(independent_formatting_context_creator);

        let inline_level_box = InlineItem::Atomic(
            independent_formatting_context,
            self.current_text_offset,
            Level::ltr(), /* This will be assigned later if necessary. */
        );
        self.inline_items.push(inline_level_box.clone());
        self.is_empty = false;

        // Push an object replacement character for this atomic, which will ensure that the line breaker
        // inserts a line breaking opportunity here.
        self.push_control_character_string("\u{fffc}");

        self.last_inline_box_ended_with_collapsible_white_space = false;
        self.on_word_boundary = true;

        // Atomics such as images should prevent any following text as being interpreted as the first letter.
        self.has_processed_first_letter = true;

        inline_level_box
    }

    pub(crate) fn push_absolutely_positioned_box(
        &mut self,
        absolutely_positioned_box_creator: impl FnOnce() -> ArcRefCell<AbsolutelyPositionedBox>,
        old_layout_box: Option<LayoutBox>,
    ) -> InlineItem {
        let absolutely_positioned_box = old_layout_box
            .and_then(|layout_box| match layout_box {
                LayoutBox::InlineLevel(InlineItem::OutOfFlowAbsolutelyPositionedBox(
                    positioned_box,
                    ..,
                )) => Some(positioned_box),
                _ => None,
            })
            .unwrap_or_else(absolutely_positioned_box_creator);

        // We cannot just reuse the old inline item, because the `current_text_offset` may have changed.
        let inline_level_box = InlineItem::OutOfFlowAbsolutelyPositionedBox(
            absolutely_positioned_box,
            self.current_text_offset,
        );

        self.inline_items.push(inline_level_box.clone());
        self.is_empty = false;
        inline_level_box
    }

    pub(crate) fn push_float_box(
        &mut self,
        float_box_creator: impl FnOnce() -> ArcRefCell<FloatBox>,
        old_layout_box: Option<LayoutBox>,
    ) -> InlineItem {
        let inline_level_box = old_layout_box
            .and_then(|layout_box| match layout_box {
                LayoutBox::InlineLevel(inline_item) => Some(inline_item),
                _ => None,
            })
            .unwrap_or_else(|| InlineItem::OutOfFlowFloatBox(float_box_creator()));

        debug_assert!(
            matches!(inline_level_box, InlineItem::OutOfFlowFloatBox(..),),
            "Created float box with incompatible `old_layout_box`"
        );

        self.inline_items.push(inline_level_box.clone());
        self.is_empty = false;
        self.contains_floats = true;
        inline_level_box
    }

    pub(crate) fn push_block_level_box(&mut self, block_level: ArcRefCell<BlockLevelBox>) {
        assert!(self.currently_processing_inline_box());
        self.contains_floats = self.contains_floats || block_level.borrow().contains_floats();
        self.inline_items.push(InlineItem::BlockLevel(block_level));
    }

    pub(crate) fn start_inline_box(
        &mut self,
        inline_box_creator: impl FnOnce() -> ArcRefCell<InlineBox>,
        old_layout_box: Option<LayoutBox>,
    ) -> InlineItem {
        // If there is an existing undamaged layout box that's compatible, use the `InlineBox` within it.
        let inline_box = old_layout_box
            .and_then(|layout_box| match layout_box {
                LayoutBox::InlineLevel(InlineItem::StartInlineBox(inline_box)) => Some(inline_box),
                _ => None,
            })
            .unwrap_or_else(inline_box_creator);

        let borrowed_inline_box = inline_box.borrow();

        let style = &borrowed_inline_box.base.style;
        self.push_control_character_string(style.bidi_control_chars().0);
        self.has_right_to_left_content =
            self.has_right_to_left_content || style.get_inherited_box().direction == Direction::Rtl;

        self.shared_inline_styles_stack
            .push(borrowed_inline_box.shared_inline_styles.clone());
        std::mem::drop(borrowed_inline_box);

        let identifier = self.inline_boxes.start_inline_box(inline_box.clone());
        let inline_item = InlineItem::StartInlineBox(inline_box);
        self.inline_items.push(inline_item.clone());
        self.inline_box_stack.push(identifier);
        self.is_empty = false;
        inline_item
    }

    /// End the ongoing inline box in this [`InlineFormattingContextBuilder`], returning
    /// shared references to all of the box tree items that were created for it. More than
    /// a single box tree items may be produced for a single inline box when that inline
    /// box is split around a block-level element.
    pub(crate) fn end_inline_box(&mut self) {
        let identifier = self
            .inline_box_stack
            .pop()
            .expect("Ended non-existent inline box");
        let inline_level_box = self.inline_boxes.get(&identifier);

        self.shared_inline_styles_stack.pop();
        self.inline_items
            .push(InlineItem::EndInlineBox(inline_level_box.clone()));
        self.inline_boxes.end_inline_box(identifier);
        let bidi_control_chars = inline_level_box.borrow().base.style.bidi_control_chars();
        self.push_control_character_string(bidi_control_chars.1);
    }

    /// This is like [`Self::push_text`], except that it might possibly add an anonymous box if
    ///
    ///  - This inline formatting context has a `::first-letter` style.
    ///  - No anonymous box for `::first-letter` has been added yet.
    ///  - First letter content is detected in this text.
    ///
    /// Note that this should only be used when processing text in block containers.
    pub(crate) fn push_text_with_possible_first_letter<'dom>(
        &mut self,
        text: BoxTreeString<'dom>,
        info: &NodeAndStyleInfo<'dom>,
        container_info: &NodeAndStyleInfo<'dom>,
        layout_context: &LayoutContext,
    ) -> bool {
        let document_selection = info.node.document_selection_in_text_node();
        if self.has_processed_first_letter || !container_info.pseudo_element_chain().is_empty() {
            self.push_text(text, info, document_selection);
            return false;
        }

        let Some(first_letter_info) =
            container_info.with_pseudo_element(layout_context, PseudoElement::FirstLetter)
        else {
            self.push_text(text, info, document_selection);
            return false;
        };

        let first_letter_range = first_letter_range(&text[..]);
        if first_letter_range.is_empty() {
            return false;
        }

        // Push any leading white space first.
        let first_letter_range_u32 = LazyCell::new(|| {
            Utf32CodeUnits::length_of(&text[..first_letter_range.start])..
                Utf32CodeUnits::length_of(&text[..first_letter_range.end])
        });
        if first_letter_range.start != 0 {
            let leading_whitespace_range = 0..first_letter_range.start;
            let leading_whitespace_selection_range =
                document_selection.and_then(|document_selection| {
                    let leading_whitespace_range_u32 = RangeAny {
                        start: None,
                        end: Some(first_letter_range_u32.start),
                    };
                    document_selection.intersect(leading_whitespace_range_u32)
                });

            self.push_text(
                Cow::Borrowed(&text[leading_whitespace_range]).into(),
                info,
                leading_whitespace_selection_range,
            );
        }

        // Push the first-letter text into an anonymous box with the `::first-letter` style.
        let box_slot = first_letter_info.node.box_slot();
        let inline_item = self.start_inline_box(
            || ArcRefCell::new(InlineBox::new(&first_letter_info, layout_context)),
            None,
        );
        box_slot.set(LayoutBox::InlineLevel(inline_item));

        let first_letter_text = Cow::Borrowed(&text[first_letter_range.clone()]);
        let first_letter_selection_range = document_selection.and_then(|document_selection| {
            document_selection
                .intersect((*first_letter_range_u32).clone().into())
                .map(|range| range.map(|offset| offset - first_letter_range_u32.start))
        });
        self.push_text(
            first_letter_text.into(),
            &first_letter_info,
            first_letter_selection_range,
        );
        self.end_inline_box();
        self.has_processed_first_letter = true;

        // Now push the non-first-letter text.
        let remaining_selection_range = document_selection.and_then(|document_selection| {
            let remaining_text_range_u32 = RangeAny {
                start: Some(first_letter_range_u32.end),
                end: document_selection.end,
            };
            document_selection
                .intersect(remaining_text_range_u32)
                .map(|range| range.map(|offset| offset - first_letter_range_u32.end))
        });
        self.push_text(
            Cow::Borrowed(&text[first_letter_range.end..]).into(),
            info,
            remaining_selection_range,
        );

        true
    }

    pub(crate) fn push_text<'dom>(
        &mut self,
        text: BoxTreeString<'dom>,
        info: &NodeAndStyleInfo<'dom>,
        document_selection: Option<RangeAny<Utf32CodeUnits>>,
    ) {
        let original_size_before = self.offset_map.total_original_size();
        let final_size_before = self.offset_map.total_final_size();

        let bidi_class_map = icu_properties::maps::bidi_class();
        let white_space_collapse = info.style.clone_white_space_collapse();
        let mut character_count = 0;
        let mut new_text = String::with_capacity(text.len());
        for iteration in TextTransformationIterator::new(
            &text,
            &info.style,
            self.last_inline_box_ended_with_collapsible_white_space,
            self.on_word_boundary,
        ) {
            self.offset_map.push_iteration(&iteration);
            for &character in iteration.characters() {
                character_count += 1;

                // If this character has a strong right-to-left class the new inline formatting context will
                // need to be BiDi-aware. This match is derived from the list of strong right-to-left classes
                // at https://www.unicode.org/reports/tr44/#Bidi_Class_Values.
                self.has_right_to_left_content = self.has_right_to_left_content ||
                    matches!(
                        bidi_class_map.get(character),
                        BidiClass::RightToLeft |
                            BidiClass::ArabicLetter |
                            BidiClass::RightToLeftEmbedding |
                            BidiClass::RightToLeftIsolate |
                            BidiClass::RightToLeftOverride
                    );

                self.is_empty = self.is_empty &&
                    match white_space_collapse {
                        WhiteSpaceCollapse::Collapse => Self::is_document_white_space(character),
                        WhiteSpaceCollapse::PreserveBreaks => {
                            Self::is_document_white_space(character) && character != '\n'
                        },
                        WhiteSpaceCollapse::Preserve | WhiteSpaceCollapse::BreakSpaces => false,
                    };

                new_text.push(character)
            }
        }

        if new_text.is_empty() {
            return;
        }

        let selection = info.node.form_control_selection_in_text_node().or_else(|| {
            let mapped_range = document_selection?.map(|offset| {
                self.offset_map.map(original_size_before + offset) - final_size_before
            });

            // Range unbounded at the start: the concrete start is offset zero.
            let start = mapped_range.start.unwrap_or(Utf32CodeUnits(0));
            // Range unbounded at the end: the concrete end is the full length.
            let end = mapped_range.end.unwrap_or(Utf32CodeUnits(character_count));

            if start == end {
                return None;
            }
            debug_assert!(end > start);

            Some(Arc::new(AtomicRefCell::new(ScriptSelection {
                range: TextByteRange::default(),
                character_range: start.0..end.0,
                enabled: true,
            })))
        });

        if let Some(last_character) = new_text.chars().next_back() {
            self.on_word_boundary = last_character.is_whitespace();
            self.last_inline_box_ended_with_collapsible_white_space =
                self.on_word_boundary && white_space_collapse != WhiteSpaceCollapse::Preserve;
        }

        let new_utf8_range = self.current_text_offset..self.current_text_offset + new_text.len();
        self.current_text_offset = new_utf8_range.end;

        let new_character_range =
            self.current_character_offset..self.current_character_offset + character_count;
        self.current_character_offset = new_character_range.end;

        self.text_segments.push(new_text);

        let current_inline_styles = self.shared_inline_styles();
        let box_slot = info.node.is_text_node().then(|| info.node.box_slot());
        let text_run = ArcRefCell::new(TextRun::new(
            info.into(),
            current_inline_styles,
            new_utf8_range,
            new_character_range,
            selection,
            box_slot
                .as_ref()
                .and_then(|box_slot| box_slot.take_layout_box_as_text_run()),
        ));
        self.inline_items
            .push(InlineItem::TextRun(text_run.clone()));

        if let Some(box_slot) = box_slot {
            box_slot.set(LayoutBox::Text(text_run));
        }
    }

    pub(crate) fn enter_display_contents(&mut self, shared_inline_styles: SharedInlineStyles) {
        self.shared_inline_styles_stack.push(shared_inline_styles);
    }

    pub(crate) fn leave_display_contents(&mut self) {
        self.shared_inline_styles_stack.pop();
    }

    /// Finish the current inline formatting context, returning [`None`] if the context was empty.
    pub(crate) fn finish(
        self,
        layout_context: &LayoutContext,
        has_first_formatted_line: bool,
        is_single_line_text_input: bool,
        default_bidi_level: Level,
    ) -> Option<InlineFormattingContext> {
        if self.is_empty {
            return None;
        }

        assert!(self.inline_box_stack.is_empty());
        assert_eq!(
            self.offset_map.total_final_size().0,
            self.current_character_offset
        );

        Some(InlineFormattingContext::new_with_builder(
            self,
            layout_context,
            has_first_formatted_line,
            is_single_line_text_input,
            default_bidi_level,
        ))
    }
}

/// Computes the range of the first letter.
///
/// The range includes any preceding punctuation and white space, and any trailing punctuation. Any
/// non-punctuation following the letter/number/symbol of first-letter ends the range. Intervening
/// spaces within trailing punctuation are not supported yet.
///
/// If the resulting range is empty, no compatible first-letter text was found.
///
/// <https://drafts.csswg.org/css-pseudo/#first-letter-pattern>
fn first_letter_range(text: &str) -> Range<usize> {
    enum State {
        /// All characters that precede the `PrecedingWhitespaceAndPunctuation` state.
        Start,
        /// All preceding punctuation and intervening whitepace that precedes the `Lns` state.
        PrecedingPunctuation,
        /// Unicode general category L: letter, N: number and S: symbol
        Lns,
        /// All punctuation (but no whitespace or other characters), that
        /// come after the `Lns` state.
        TrailingPunctuation,
    }

    let mut start = 0;
    let mut state = State::Start;
    for (index, character) in text.char_indices() {
        match &mut state {
            State::Start => {
                if character.is_letter() || character.is_number() || character.is_symbol() {
                    start = index;
                    state = State::Lns;
                } else if character.is_punctuation() {
                    start = index;
                    state = State::PrecedingPunctuation
                }
            },
            State::PrecedingPunctuation => {
                if character.is_letter() || character.is_number() || character.is_symbol() {
                    state = State::Lns;
                } else if !character.is_separator_space() && !character.is_punctuation() {
                    return 0..0;
                }
            },
            State::Lns => {
                // TODO: Implement support for intervening spaces
                // <https://drafts.csswg.org/css-pseudo/#first-letter-pattern>
                if character.is_punctuation() &&
                    !character.is_punctuation_open() &&
                    !character.is_punctuation_dash()
                {
                    state = State::TrailingPunctuation;
                } else {
                    return start..index;
                }
            },
            State::TrailingPunctuation => {
                // TODO: Implement support for intervening spaces
                // <https://drafts.csswg.org/css-pseudo/#first-letter-pattern>
                if character.is_punctuation() &&
                    !character.is_punctuation_open() &&
                    !character.is_punctuation_dash()
                {
                    continue;
                } else {
                    return start..index;
                }
            },
        }
    }

    match state {
        State::Start | State::PrecedingPunctuation => 0..0,
        State::Lns | State::TrailingPunctuation => start..text.len(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_first_letter_eq(text: &str, expected: &str) {
        let range = first_letter_range(text);
        assert_eq!(&text[range], expected);
    }

    #[test]
    fn test_first_letter_range() {
        // All spaces
        assert_first_letter_eq("", "");
        assert_first_letter_eq("  ", "");

        // Spaces and punctuation only
        assert_first_letter_eq("(", "");
        assert_first_letter_eq(" (", "");
        assert_first_letter_eq("( ", "");
        assert_first_letter_eq("()", "");

        // Invalid chars
        assert_first_letter_eq("\u{0903}", "");

        // First letter only
        assert_first_letter_eq("A", "A");
        assert_first_letter_eq(" A", "A");
        assert_first_letter_eq("A ", "A");
        assert_first_letter_eq(" A ", "A");

        // Word
        assert_first_letter_eq("App", "A");
        assert_first_letter_eq(" App", "A");
        assert_first_letter_eq("App ", "A");

        // Preceding punctuation(s), intervening spaces and first letter
        assert_first_letter_eq(r#""A"#, r#""A"#);
        assert_first_letter_eq(r#" "A"#, r#""A"#);
        assert_first_letter_eq(r#""A "#, r#""A"#);
        assert_first_letter_eq(r#"" A"#, r#"" A"#);
        assert_first_letter_eq(r#" "A "#, r#""A"#);
        assert_first_letter_eq(r#"("A"#, r#"("A"#);
        assert_first_letter_eq(r#" ("A"#, r#"("A"#);
        assert_first_letter_eq(r#"( "A"#, r#"( "A"#);
        assert_first_letter_eq(r#"[ ( "A"#, r#"[ ( "A"#);

        // First letter and succeeding punctuation(s)
        // TODO: modify test cases when intervening spaces in succeeding puntuations is supported
        assert_first_letter_eq(r#"A""#, r#"A""#);
        assert_first_letter_eq(r#"A" "#, r#"A""#);
        assert_first_letter_eq(r#"A)]"#, r#"A)]"#);
        assert_first_letter_eq(r#"A" )]"#, r#"A""#);
        assert_first_letter_eq(r#"A)] >"#, r#"A)]"#);

        // All
        assert_first_letter_eq(r#" ("A" )]"#, r#"("A""#);
        assert_first_letter_eq(r#" ("A")] >"#, r#"("A")]"#);

        // Non ASCII chars
        assert_first_letter_eq("一", "一");
        assert_first_letter_eq(" 一 ", "一");
        assert_first_letter_eq("一二三", "一");
        assert_first_letter_eq(" 一二三 ", "一");
        assert_first_letter_eq("（一二三）", "（一");
        assert_first_letter_eq(" （一二三） ", "（一");
        assert_first_letter_eq("（（一", "（（一");
        assert_first_letter_eq(" （ （一", "（ （一");
        assert_first_letter_eq("一）", "一）");
        assert_first_letter_eq("一））", "一））");
        assert_first_letter_eq("一） ）", "一）");
    }
}
