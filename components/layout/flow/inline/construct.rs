/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::Cow;
use std::char::{ToLowercase, ToUppercase};

use icu_segmenter::WordSegmenter;
use itertools::izip;
use style::computed_values::white_space_collapse::T as WhiteSpaceCollapse;
use style::values::specified::text::TextTransformCase;
use unicode_bidi::Level;

use super::text_run::TextRun;
use super::{
    InlineBox, InlineBoxIdentifier, InlineBoxes, InlineFormattingContext, InlineItem,
    SharedInlineStyles,
};
use crate::cell::ArcRefCell;
use crate::context::LayoutContext;
use crate::dom::LayoutBox;
use crate::dom_traversal::NodeAndStyleInfo;
use crate::flow::float::FloatBox;
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
    pub inline_items: Vec<ArcRefCell<InlineItem>>,

    /// The current [`InlineBox`] tree of this [`InlineFormattingContext`] under construction.
    pub inline_boxes: InlineBoxes,

    /// The ongoing stack of inline boxes stack of the builder.
    ///
    /// Contains all the currently ongoing inline boxes we entered so far.
    /// The traversal is at all times as deep in the tree as this stack is,
    /// which is why the code doesn't need to keep track of the actual
    /// container root (see `handle_inline_level_element`).
    //_
    /// When an inline box ends, it's removed from this stack.
    inline_box_stack: Vec<InlineBoxIdentifier>,

    /// Normally, an inline box produces a single box tree [`InlineItem`]. When a block
    /// element causes an inline box [to be split], it can produce multiple
    /// [`InlineItem`]s, all inserted into different [`InlineFormattingContext`]s.
    /// [`Self::block_in_inline_splits`] is responsible for tracking all of these split
    /// inline box results, so that they can be inserted into the [`crate::dom::BoxSlot`]
    /// for the DOM element once it has been processed for BoxTree construction.
    ///
    /// [to be split]: https://www.w3.org/TR/CSS2/visuren.html#anonymous-block-level
    block_in_inline_splits: Vec<Vec<ArcRefCell<InlineItem>>>,

    /// If the [`InlineBox`] of an inline-level element is not damaged, it can be reused
    /// to support incremental layout. An [`InlineBox`] can be split by block elements
    /// into multiple [`InlineBox`]es, all inserted into different
    /// [`InlineFormattingContext`]s. Therefore, [`Self::old_block_in_inline_splits`] is
    /// used to hold all these split inline boxes from the previous box tree construction
    /// that are about to be reused, ensuring they can be sequentially inserted into each
    /// newly built [`InlineFormattingContext`].
    old_block_in_inline_splits: Vec<Vec<ArcRefCell<InlineBox>>>,

    /// Whether or not the inline formatting context under construction has any
    /// uncollapsible text content.
    pub has_uncollapsible_text_content: bool,
}

impl InlineFormattingContextBuilder {
    pub(crate) fn new(info: &NodeAndStyleInfo) -> Self {
        Self::new_for_shared_styles(vec![info.into()])
    }

    pub(crate) fn new_for_shared_styles(
        shared_inline_styles_stack: Vec<SharedInlineStyles>,
    ) -> Self {
        Self {
            // For the purposes of `text-transform: capitalize` the start of the IFC is a word boundary.
            on_word_boundary: true,
            shared_inline_styles_stack,
            ..Default::default()
        }
    }

    pub(crate) fn currently_processing_inline_box(&self) -> bool {
        !self.inline_box_stack.is_empty()
    }

    fn push_control_character_string(&mut self, string_to_push: &str) {
        self.text_segments.push(string_to_push.to_owned());
        self.current_text_offset += string_to_push.len();
    }

    fn shared_inline_styles(&self) -> SharedInlineStyles {
        self.shared_inline_styles_stack
            .last()
            .expect("Should always have at least one SharedInlineStyles")
            .clone()
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

        fn inline_level_box_is_empty(inline_level_box: &InlineItem) -> bool {
            match inline_level_box {
                InlineItem::StartInlineBox(_) => false,
                InlineItem::EndInlineBox => false,
                // Text content is handled by `self.has_uncollapsible_text` content above in order
                // to avoid having to iterate through the character once again.
                InlineItem::TextRun(_) => true,
                InlineItem::OutOfFlowAbsolutelyPositionedBox(..) => false,
                InlineItem::OutOfFlowFloatBox(_) => false,
                InlineItem::Atomic(..) => false,
            }
        }

        self.inline_items
            .iter()
            .all(|inline_level_box| inline_level_box_is_empty(&inline_level_box.borrow()))
    }

    pub(crate) fn push_atomic(
        &mut self,
        independent_formatting_context_creator: impl FnOnce()
            -> ArcRefCell<IndependentFormattingContext>,
        old_layout_box: Option<LayoutBox>,
    ) -> ArcRefCell<InlineItem> {
        // If there is an existing undamaged layout box that's compatible, use that.
        let independent_formatting_context = old_layout_box
            .and_then(LayoutBox::unsplit_inline_level_layout_box)
            .and_then(|inline_item| match &*inline_item.borrow() {
                InlineItem::Atomic(atomic, ..) => Some(atomic.clone()),
                _ => None,
            })
            .unwrap_or_else(independent_formatting_context_creator);

        let inline_level_box = ArcRefCell::new(InlineItem::Atomic(
            independent_formatting_context,
            self.current_text_offset,
            Level::ltr(), /* This will be assigned later if necessary. */
        ));
        self.inline_items.push(inline_level_box.clone());

        // Push an object replacement character for this atomic, which will ensure that the line breaker
        // inserts a line breaking opportunity here.
        self.push_control_character_string("\u{fffc}");

        self.last_inline_box_ended_with_collapsible_white_space = false;
        self.on_word_boundary = true;

        inline_level_box
    }

    pub(crate) fn push_absolutely_positioned_box(
        &mut self,
        absolutely_positioned_box_creator: impl FnOnce() -> ArcRefCell<AbsolutelyPositionedBox>,
        old_layout_box: Option<LayoutBox>,
    ) -> ArcRefCell<InlineItem> {
        let absolutely_positioned_box = old_layout_box
            .and_then(LayoutBox::unsplit_inline_level_layout_box)
            .and_then(|inline_item| match &*inline_item.borrow() {
                InlineItem::OutOfFlowAbsolutelyPositionedBox(positioned_box, ..) => {
                    Some(positioned_box.clone())
                },
                _ => None,
            })
            .unwrap_or_else(absolutely_positioned_box_creator);

        // We cannot just reuse the old inline item, because the `current_text_offset` may have changed.
        let inline_level_box = ArcRefCell::new(InlineItem::OutOfFlowAbsolutelyPositionedBox(
            absolutely_positioned_box,
            self.current_text_offset,
        ));

        self.inline_items.push(inline_level_box.clone());
        inline_level_box
    }

    pub(crate) fn push_float_box(
        &mut self,
        float_box_creator: impl FnOnce() -> ArcRefCell<FloatBox>,
        old_layout_box: Option<LayoutBox>,
    ) -> ArcRefCell<InlineItem> {
        let inline_level_box = old_layout_box
            .and_then(LayoutBox::unsplit_inline_level_layout_box)
            .unwrap_or_else(|| ArcRefCell::new(InlineItem::OutOfFlowFloatBox(float_box_creator())));

        debug_assert!(
            matches!(
                &*inline_level_box.borrow(),
                InlineItem::OutOfFlowFloatBox(..),
            ),
            "Created float box with incompatible `old_layout_box`"
        );

        self.inline_items.push(inline_level_box.clone());
        self.contains_floats = true;
        inline_level_box
    }

    pub(crate) fn start_inline_box(
        &mut self,
        inline_box_creator: impl FnOnce() -> ArcRefCell<InlineBox>,
        block_in_inline_splits: Option<Vec<ArcRefCell<InlineItem>>>,
        old_layout_box: Option<LayoutBox>,
    ) {
        // If there is an existing undamaged layout box that's compatible, use the `InlineBox` within it.
        if let Some(LayoutBox::InlineLevel(inline_level_box)) = old_layout_box {
            let old_block_in_inline_splits: Vec<ArcRefCell<InlineBox>> = inline_level_box
                .iter()
                .rev() // reverse to facilate the `Vec::pop` operation
                .filter_map(|inline_item| match &*inline_item.borrow() {
                    InlineItem::StartInlineBox(inline_box) => Some(inline_box.clone()),
                    _ => None,
                })
                .collect();

            debug_assert!(
                old_block_in_inline_splits.is_empty() ||
                    old_block_in_inline_splits.len() == inline_level_box.len(),
                "Create inline box with incompatible `old_layout_box`"
            );

            self.start_inline_box_internal(
                inline_box_creator,
                block_in_inline_splits,
                old_block_in_inline_splits,
            );
        } else {
            self.start_inline_box_internal(inline_box_creator, block_in_inline_splits, vec![]);
        }
    }

    pub fn start_inline_box_internal(
        &mut self,
        inline_box_creator: impl FnOnce() -> ArcRefCell<InlineBox>,
        block_in_inline_splits: Option<Vec<ArcRefCell<InlineItem>>>,
        mut old_block_in_inline_splits: Vec<ArcRefCell<InlineBox>>,
    ) {
        let inline_box = old_block_in_inline_splits
            .pop()
            .unwrap_or_else(inline_box_creator);

        let borrowed_inline_box = inline_box.borrow();
        self.push_control_character_string(borrowed_inline_box.base.style.bidi_control_chars().0);

        // Don't push a `SharedInlineStyles` if we are pushing this box when splitting
        // an IFC for a block-in-inline split. Shared styles are pushed as part of setting
        // up the second split of the IFC.
        if borrowed_inline_box.is_first_split {
            self.shared_inline_styles_stack
                .push(borrowed_inline_box.shared_inline_styles.clone());
        }
        std::mem::drop(borrowed_inline_box);

        let identifier = self.inline_boxes.start_inline_box(inline_box.clone());
        let inline_level_box = ArcRefCell::new(InlineItem::StartInlineBox(inline_box));
        self.inline_items.push(inline_level_box.clone());
        self.inline_box_stack.push(identifier);

        let mut block_in_inline_splits = block_in_inline_splits.unwrap_or_default();
        block_in_inline_splits.push(inline_level_box);
        self.block_in_inline_splits.push(block_in_inline_splits);

        self.old_block_in_inline_splits
            .push(old_block_in_inline_splits);
    }

    /// End the ongoing inline box in this [`InlineFormattingContextBuilder`], returning
    /// shared references to all of the box tree items that were created for it. More than
    /// a single box tree items may be produced for a single inline box when that inline
    /// box is split around a block-level element.
    pub(crate) fn end_inline_box(&mut self) -> Vec<ArcRefCell<InlineItem>> {
        self.shared_inline_styles_stack.pop();

        let (identifier, block_in_inline_splits) = self.end_inline_box_internal();
        let inline_level_box = self.inline_boxes.get(&identifier);
        {
            let mut inline_level_box = inline_level_box.borrow_mut();
            inline_level_box.is_last_split = true;
            self.push_control_character_string(inline_level_box.base.style.bidi_control_chars().1);
        }

        debug_assert!(
            self.old_block_in_inline_splits
                .last()
                .is_some_and(|inline_boxes| inline_boxes.is_empty()),
            "Reuse incompatible `old_block_in_inline_splits` for inline boxes",
        );
        let _ = self.old_block_in_inline_splits.pop();

        block_in_inline_splits.unwrap_or_default()
    }

    fn end_inline_box_internal(
        &mut self,
    ) -> (InlineBoxIdentifier, Option<Vec<ArcRefCell<InlineItem>>>) {
        let identifier = self
            .inline_box_stack
            .pop()
            .expect("Ended non-existent inline box");
        self.inline_items
            .push(ArcRefCell::new(InlineItem::EndInlineBox));

        self.inline_boxes.end_inline_box(identifier);

        // This might be `None` if this builder has already drained its block-in-inline-splits
        // into the new builder on the other side of a new block-in-inline split.
        let block_in_inline_splits = self.block_in_inline_splits.pop();

        (identifier, block_in_inline_splits)
    }

    pub(crate) fn push_text<'dom>(&mut self, text: Cow<'dom, str>, info: &NodeAndStyleInfo<'dom>) {
        let white_space_collapse = info.style.clone_white_space_collapse();
        let collapsed = WhitespaceCollapse::new(
            text.chars(),
            white_space_collapse,
            self.last_inline_box_ended_with_collapsible_white_space,
        );

        // TODO: Not all text transforms are about case, this logic should stop ignoring
        // TextTransform::FULL_WIDTH and TextTransform::FULL_SIZE_KANA.
        let text_transform = info.style.clone_text_transform().case();
        let capitalized_text: String;
        let char_iterator: Box<dyn Iterator<Item = char>> = match text_transform {
            TextTransformCase::None => Box::new(collapsed),
            TextTransformCase::Capitalize => {
                // `TextTransformation` doesn't support capitalization, so we must capitalize the whole
                // string at once and make a copy. Here `on_word_boundary` indicates whether or not the
                // inline formatting context as a whole is on a word boundary. This is different from
                // `last_inline_box_ended_with_collapsible_white_space` because the word boundaries are
                // between atomic inlines and at the start of the IFC, and because preserved spaces
                // are a word boundary.
                let collapsed_string: String = collapsed.collect();
                capitalized_text = capitalize_string(&collapsed_string, self.on_word_boundary);
                Box::new(capitalized_text.chars())
            },
            _ => {
                // If `text-transform` is active, wrap the `WhitespaceCollapse` iterator in
                // a `TextTransformation` iterator.
                Box::new(TextTransformation::new(collapsed, text_transform))
            },
        };

        let white_space_collapse = info.style.clone_white_space_collapse();
        let new_text: String = char_iterator
            .inspect(|&character| {
                self.has_uncollapsible_text_content |= matches!(
                    white_space_collapse,
                    WhiteSpaceCollapse::Preserve | WhiteSpaceCollapse::BreakSpaces
                ) || !character.is_ascii_whitespace() ||
                    (character == '\n' && white_space_collapse != WhiteSpaceCollapse::Collapse);
            })
            .collect();

        if new_text.is_empty() {
            return;
        }

        let selection_range = info.get_selection_range();
        if let Some(last_character) = new_text.chars().next_back() {
            self.on_word_boundary = last_character.is_whitespace();
            self.last_inline_box_ended_with_collapsible_white_space =
                self.on_word_boundary && white_space_collapse != WhiteSpaceCollapse::Preserve;
        }

        let new_range = self.current_text_offset..self.current_text_offset + new_text.len();
        self.current_text_offset = new_range.end;
        self.text_segments.push(new_text);

        if let Some(inline_item) = self.inline_items.last() {
            if let InlineItem::TextRun(text_run) = &mut *inline_item.borrow_mut() {
                text_run.borrow_mut().text_range.end = new_range.end;
                return;
            }
        }

        self.inline_items
            .push(ArcRefCell::new(InlineItem::TextRun(ArcRefCell::new(
                TextRun::new(
                    info.into(),
                    self.shared_inline_styles(),
                    new_range,
                    selection_range,
                ),
            ))));
    }

    pub(crate) fn enter_display_contents(&mut self, shared_inline_styles: SharedInlineStyles) {
        self.shared_inline_styles_stack.push(shared_inline_styles);
    }

    pub(crate) fn leave_display_contents(&mut self) {
        self.shared_inline_styles_stack.pop();
    }

    pub(crate) fn split_around_block_and_finish(
        &mut self,
        layout_context: &LayoutContext,
        has_first_formatted_line: bool,
        default_bidi_level: Level,
    ) -> Option<InlineFormattingContext> {
        if self.is_empty() {
            return None;
        }

        // Create a new inline builder which will be active after the block splits this inline formatting
        // context. It has the same inline box structure as this builder, except the boxes are
        // marked as not being the first fragment. No inline content is carried over to this new
        // builder.
        let mut new_builder = Self::new_for_shared_styles(self.shared_inline_styles_stack.clone());

        let block_in_inline_splits = std::mem::take(&mut self.block_in_inline_splits);
        let old_block_in_inline_splits = std::mem::take(&mut self.old_block_in_inline_splits);
        for (identifier, already_collected_inline_boxes, being_recollected_inline_boxes) in izip!(
            self.inline_box_stack.iter(),
            block_in_inline_splits,
            old_block_in_inline_splits
        ) {
            // Start a new inline box for every ongoing inline box in this
            // InlineFormattingContext once we are done processing this block element,
            // being sure to give the block-in-inline-split to the new
            // InlineFormattingContext. These will finally be inserted into the DOM's
            // BoxSlot once the inline box has been fully processed. Meanwhile, being
            // sure to give the old-block-in-inline-split to new InlineFormattingContext,
            // so that them will be inserted into each following InlineFormattingContext.
            let split_inline_box_callback = || {
                ArcRefCell::new(
                    self.inline_boxes
                        .get(identifier)
                        .borrow()
                        .split_around_block(),
                )
            };
            new_builder.start_inline_box_internal(
                split_inline_box_callback,
                Some(already_collected_inline_boxes),
                being_recollected_inline_boxes,
            );
        }
        let mut inline_builder_from_before_split = std::mem::replace(self, new_builder);

        // End all ongoing inline boxes in the first builder, but ensure that they are not
        // marked as the final fragments, so that they do not get inline end margin, borders,
        // and padding.
        while !inline_builder_from_before_split.inline_box_stack.is_empty() {
            inline_builder_from_before_split.end_inline_box_internal();
        }

        inline_builder_from_before_split.finish(
            layout_context,
            has_first_formatted_line,
            /* is_single_line_text_input = */ false,
            default_bidi_level,
        )
    }

    /// Finish the current inline formatting context, returning [`None`] if the context was empty.
    pub(crate) fn finish(
        self,
        layout_context: &LayoutContext,
        has_first_formatted_line: bool,
        is_single_line_text_input: bool,
        default_bidi_level: Level,
    ) -> Option<InlineFormattingContext> {
        if self.is_empty() {
            return None;
        }

        assert!(self.inline_box_stack.is_empty());
        debug_assert!(self.old_block_in_inline_splits.is_empty());
        Some(InlineFormattingContext::new_with_builder(
            self,
            layout_context,
            has_first_formatted_line,
            is_single_line_text_input,
            default_bidi_level,
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
    text_transform: TextTransformCase,
    /// If an uppercasing or lowercasing produces more than one character, this
    /// caches them so that they can be returned in subsequent iterator calls.
    pending_case_conversion_result: Option<PendingCaseConversionResult>,
}

impl<InputIterator> TextTransformation<InputIterator> {
    pub fn new(char_iterator: InputIterator, text_transform: TextTransformCase) -> Self {
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
            match self.text_transform {
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
                TextTransformCase::Capitalize => return Some(character),
            }
        }
        None
    }
}

/// Given a string and whether the start of the string represents a word boundary, create a copy of
/// the string with letters after word boundaries capitalized.
pub(crate) fn capitalize_string(string: &str, allow_word_at_start: bool) -> String {
    let mut output_string = String::new();
    output_string.reserve(string.len());

    let word_segmenter = WordSegmenter::new_auto();
    let mut bounds = word_segmenter.segment_str(string).peekable();
    let mut byte_index = 0;
    for character in string.chars() {
        let current_byte_index = byte_index;
        byte_index += character.len_utf8();

        if let Some(next_index) = bounds.peek() {
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
