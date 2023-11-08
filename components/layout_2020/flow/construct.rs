/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::Cow;
use std::convert::{TryFrom, TryInto};

use rayon::iter::{IntoParallelIterator, ParallelIterator};
use servo_arc::Arc;
use style::computed_values::white_space::T as WhiteSpace;
use style::properties::longhands::list_style_position::computed_value::T as ListStylePosition;
use style::properties::ComputedValues;
use style::selector_parser::PseudoElement;
use style::values::specified::text::TextDecorationLine;

use crate::cell::ArcRefCell;
use crate::context::LayoutContext;
use crate::dom::{BoxSlot, LayoutBox, NodeExt};
use crate::dom_traversal::{Contents, NodeAndStyleInfo, NonReplacedContents, TraversalHandler};
use crate::flow::float::FloatBox;
use crate::flow::inline::{InlineBox, InlineFormattingContext, InlineLevelBox, TextRun};
use crate::flow::{BlockContainer, BlockFormattingContext, BlockLevelBox};
use crate::formatting_contexts::IndependentFormattingContext;
use crate::positioned::AbsolutelyPositionedBox;
use crate::style_ext::{ComputedValuesExt, DisplayGeneratingBox, DisplayInside, DisplayOutside};

impl BlockFormattingContext {
    pub fn construct<'dom, Node>(
        context: &LayoutContext,
        info: &NodeAndStyleInfo<Node>,
        contents: NonReplacedContents,
        propagated_text_decoration_line: TextDecorationLine,
        is_list_item: bool,
    ) -> Self
    where
        Node: NodeExt<'dom>,
    {
        let contents = BlockContainer::construct(
            context,
            info,
            contents,
            propagated_text_decoration_line,
            is_list_item,
        );
        let contains_floats = contents.contains_floats();

        Self {
            contents,
            contains_floats,
        }
    }

    pub fn construct_for_text_runs<'dom>(
        runs: impl Iterator<Item = TextRun>,
        text_decoration_line: TextDecorationLine,
    ) -> Self {
        // FIXME: do white space collapsing
        let inline_level_boxes = runs
            .map(|run| ArcRefCell::new(InlineLevelBox::TextRun(run)))
            .collect();

        let ifc = InlineFormattingContext {
            inline_level_boxes,
            text_decoration_line,
            has_first_formatted_line: true,
            contains_floats: false,
            ends_with_whitespace: false,
        };
        let contents = BlockContainer::InlineFormattingContext(ifc);
        let bfc = Self {
            contents,
            contains_floats: false,
        };
        bfc
    }
}

struct BlockLevelJob<'dom, Node> {
    info: NodeAndStyleInfo<Node>,
    box_slot: BoxSlot<'dom>,
    kind: BlockLevelCreator,
}

enum BlockLevelCreator {
    SameFormattingContextBlock(IntermediateBlockContainer),
    Independent {
        display_inside: DisplayInside,
        contents: Contents,
        propagated_text_decoration_line: TextDecorationLine,
    },
    OutOfFlowAbsolutelyPositionedBox {
        display_inside: DisplayInside,
        contents: Contents,
    },
    OutOfFlowFloatBox {
        display_inside: DisplayInside,
        contents: Contents,
    },
}

/// A block container that may still have to be constructed.
///
/// Represents either the inline formatting context of an anonymous block
/// box or the yet-to-be-computed block container generated from the children
/// of a given element.
///
/// Deferring allows using rayon’s `into_par_iter`.
enum IntermediateBlockContainer {
    InlineFormattingContext(InlineFormattingContext),
    Deferred {
        contents: NonReplacedContents,
        propagated_text_decoration_line: TextDecorationLine,
        is_list_item: bool,
    },
}

/// A builder for a block container.
///
/// This builder starts from the first child of a given DOM node
/// and does a preorder traversal of all of its inclusive siblings.
struct BlockContainerBuilder<'dom, 'style, Node> {
    context: &'style LayoutContext<'style>,

    /// This NodeAndStyleInfo contains the root node, the corresponding pseudo
    /// content designator, and the block container style.
    info: &'style NodeAndStyleInfo<Node>,

    /// The list of block-level boxes to be built for the final block container.
    ///
    /// Contains all the block-level jobs we found traversing the tree
    /// so far, if this is empty at the end of the traversal and the ongoing
    /// inline formatting context is not empty, the block container establishes
    /// an inline formatting context (see end of `build`).
    ///
    /// DOM nodes which represent block-level boxes are immediately pushed
    /// to this list with their style without ever being traversed at this
    /// point, instead we just move to their next sibling. If the DOM node
    /// doesn't have a next sibling, we either reached the end of the container
    /// root or there are ongoing inline-level boxes
    /// (see `handle_block_level_element`).
    block_level_boxes: Vec<BlockLevelJob<'dom, Node>>,

    /// The ongoing inline formatting context of the builder.
    ///
    /// Contains all the complete inline level boxes we found traversing the
    /// tree so far. If a block-level box is found during traversal,
    /// this inline formatting context is pushed as a block level box to
    /// the list of block-level boxes of the builder
    /// (see `end_ongoing_inline_formatting_context`).
    ongoing_inline_formatting_context: InlineFormattingContext,

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
    ongoing_inline_boxes_stack: Vec<InlineBox>,

    /// The style of the anonymous block boxes pushed to the list of block-level
    /// boxes, if any (see `end_ongoing_inline_formatting_context`).
    anonymous_style: Option<Arc<ComputedValues>>,
}

impl BlockContainer {
    pub fn construct<'dom, Node>(
        context: &LayoutContext,
        info: &NodeAndStyleInfo<Node>,
        contents: NonReplacedContents,
        propagated_text_decoration_line: TextDecorationLine,
        is_list_item: bool,
    ) -> BlockContainer
    where
        Node: NodeExt<'dom>,
    {
        let text_decoration_line =
            propagated_text_decoration_line | info.style.clone_text_decoration_line();
        let mut builder = BlockContainerBuilder {
            context,
            info,
            block_level_boxes: Vec::new(),
            ongoing_inline_formatting_context: InlineFormattingContext::new(
                text_decoration_line,
                /* has_first_formatted_line = */ true,
                /* ends_with_whitespace */ false,
            ),
            ongoing_inline_boxes_stack: Vec::new(),
            anonymous_style: None,
        };

        if is_list_item {
            if let Some(marker_contents) = crate::lists::make_marker(context, info) {
                let _position = info.style.clone_list_style_position();
                // FIXME: implement support for `outside` and remove this:
                let position = ListStylePosition::Inside;
                match position {
                    ListStylePosition::Inside => {
                        builder.handle_list_item_marker_inside(info, marker_contents)
                    },
                    ListStylePosition::Outside => {
                        // FIXME: implement layout for this case
                        // https://github.com/servo/servo/issues/27383
                        // and enable `list-style-position` and the `list-style` shorthand in Stylo.
                    },
                }
            }
        }

        contents.traverse(context, info, &mut builder);

        debug_assert!(builder.ongoing_inline_boxes_stack.is_empty());

        if !builder.ongoing_inline_formatting_context.is_empty() {
            if builder.block_level_boxes.is_empty() {
                return BlockContainer::InlineFormattingContext(
                    builder.ongoing_inline_formatting_context,
                );
            }
            builder.end_ongoing_inline_formatting_context();
        }

        let block_level_boxes = if context.use_rayon {
            builder
                .block_level_boxes
                .into_par_iter()
                .map(|block_level_job| block_level_job.finish(context))
                .collect()
        } else {
            builder
                .block_level_boxes
                .into_iter()
                .map(|block_level_job| block_level_job.finish(context))
                .collect()
        };

        BlockContainer::BlockLevelBoxes(block_level_boxes)
    }
}

impl<'dom, Node> TraversalHandler<'dom, Node> for BlockContainerBuilder<'dom, '_, Node>
where
    Node: NodeExt<'dom>,
{
    fn handle_element(
        &mut self,
        info: &NodeAndStyleInfo<Node>,
        display: DisplayGeneratingBox,
        contents: Contents,
        box_slot: BoxSlot<'dom>,
    ) {
        match display {
            DisplayGeneratingBox::OutsideInside { outside, inside } => match outside {
                DisplayOutside::Inline => box_slot.set(LayoutBox::InlineLevel(
                    self.handle_inline_level_element(info, inside, contents),
                )),
                DisplayOutside::Block => {
                    let box_style = info.style.get_box();
                    // Floats and abspos cause blockification, so they only happen in this case.
                    // https://drafts.csswg.org/css2/visuren.html#dis-pos-flo
                    if box_style.position.is_absolutely_positioned() {
                        self.handle_absolutely_positioned_element(info, inside, contents, box_slot)
                    } else if box_style.float.is_floating() {
                        self.handle_float_element(info, inside, contents, box_slot)
                    } else {
                        self.handle_block_level_element(info, inside, contents, box_slot)
                    }
                },
            },
        }
    }

    fn handle_text(&mut self, info: &NodeAndStyleInfo<Node>, input: Cow<'dom, str>) {
        if input.is_empty() {
            return;
        }

        let (output, has_uncollapsible_content) = collapse_and_transform_whitespace(
            &input,
            info.style.get_inherited_text().white_space,
            self.ongoing_inline_formatting_context.ends_with_whitespace,
        );
        if output.is_empty() {
            return;
        }

        self.ongoing_inline_formatting_context.ends_with_whitespace =
            output.chars().last().unwrap().is_ascii_whitespace();

        let inlines = self.current_inline_level_boxes();
        match inlines.last_mut().map(|last| last.borrow_mut()) {
            Some(mut last_box) => match *last_box {
                InlineLevelBox::TextRun(ref mut text_run) => {
                    text_run.text.push_str(&output);
                    text_run.has_uncollapsible_content |= has_uncollapsible_content;
                    return;
                },
                _ => {},
            },
            _ => {},
        }

        inlines.push(ArcRefCell::new(InlineLevelBox::TextRun(TextRun {
            base_fragment_info: info.into(),
            parent_style: Arc::clone(&info.style),
            text: output,
            has_uncollapsible_content,
        })));
    }
}

fn preserve_segment_break() -> bool {
    true
}

/// Collapse and transform whitespace in the given input according to the rules in
/// <https://drafts.csswg.org/css-text-3/#white-space-phase-1>. This method doesn't
/// follow the steps exactly since they are defined in a multi-pass appraoach, but it
/// tries to be effectively the same transformation.
///
/// Returns the transformed text as a [String] and also whether or not the input had
/// any uncollapsible content.
fn collapse_and_transform_whitespace<'text>(
    input: &'text str,
    white_space: WhiteSpace,
    trim_beginning_white_space: bool,
) -> (String, bool) {
    // Point 4.1.1 first bullet:
    // > If white-space is set to normal, nowrap, or pre-line, whitespace
    // > characters are considered collapsible
    // If whitespace is not considered collapsible, it is preserved entirely, which
    // means that we can simply return the input string exactly.
    if white_space.preserve_spaces() {
        return (input.to_owned(), true);
    }

    let mut output = String::with_capacity(input.len());
    let mut has_uncollapsible_content = false;
    let mut had_whitespace = false;
    let mut following_newline = false;
    let mut in_whitespace_at_beginning = true;

    let is_leading_trimmed_whitespace =
        |in_whitespace_at_beginning: bool| in_whitespace_at_beginning && trim_beginning_white_space;

    // Point 4.1.1:
    // > 2. Any sequence of collapsible spaces and tabs immediately preceding or
    // >    following a segment break is removed.
    // > 3. Every collapsible tab is converted to a collapsible space (U+0020).
    // > 4. Any collapsible space immediately following another collapsible space—even
    // >    one outside the boundary of the inline containing that space, provided both
    // >    spaces are within the same inline formatting context—is collapsed to have zero
    // >    advance width.
    let push_pending_whitespace_if_needed =
        |output: &mut String,
         had_whitespace: bool,
         following_newline: bool,
         in_whitespace_at_beginning: bool| {
            if had_whitespace &&
                !following_newline &&
                !is_leading_trimmed_whitespace(in_whitespace_at_beginning)
            {
                output.push(' ');
            }
        };

    for character in input.chars() {
        // Don't push non-newline whitespace immediately. Instead wait to push it until we
        // know that it isn't followed by a newline. See `push_pending_whitespace_if_needed`
        //  above.
        if character.is_ascii_whitespace() && character != '\n' {
            had_whitespace = true;
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
            // > collapsible and are instead transformed  into a preserved line feed"
            if white_space == WhiteSpace::PreLine {
                has_uncollapsible_content = true;
                had_whitespace = false;
                output.push('\n');

            // Point 4.1.3:
            // > 1. First, any collapsible segment break immediately following another
            // >    collapsible segment break is removed.
            // > 2. Then any remaining segment break is either transformed into a space (U+0020)
            // >    or removed depending on the context before and after the break.
            } else if !following_newline &&
                preserve_segment_break() &&
                !is_leading_trimmed_whitespace(in_whitespace_at_beginning)
            {
                had_whitespace = false;
                output.push(' ');
            }
            following_newline = true;
            continue;
        }

        push_pending_whitespace_if_needed(
            &mut output,
            had_whitespace,
            following_newline,
            in_whitespace_at_beginning,
        );

        has_uncollapsible_content = true;
        had_whitespace = false;
        in_whitespace_at_beginning = false;
        following_newline = false;
        output.push(character);
    }

    push_pending_whitespace_if_needed(
        &mut output,
        had_whitespace,
        following_newline,
        in_whitespace_at_beginning,
    );

    (output, has_uncollapsible_content)
}

impl<'dom, Node> BlockContainerBuilder<'dom, '_, Node>
where
    Node: NodeExt<'dom>,
{
    fn handle_list_item_marker_inside(
        &mut self,
        info: &NodeAndStyleInfo<Node>,
        contents: Vec<crate::dom_traversal::PseudoElementContentItem>,
    ) {
        let marker_style = self
            .context
            .shared_context()
            .stylist
            .style_for_anonymous::<Node::ConcreteElement>(
                &self.context.shared_context().guards,
                &PseudoElement::ServoLegacyText, // FIMXE: use `PseudoElement::Marker` when we add it
                &info.style,
            );
        self.handle_inline_level_element(
            &info.new_replacing_style(marker_style),
            DisplayInside::Flow {
                is_list_item: false,
            },
            Contents::OfPseudoElement(contents),
        );
    }

    fn handle_inline_level_element(
        &mut self,
        info: &NodeAndStyleInfo<Node>,
        display_inside: DisplayInside,
        contents: Contents,
    ) -> ArcRefCell<InlineLevelBox> {
        let box_ = if let (DisplayInside::Flow { is_list_item }, false) =
            (display_inside, contents.is_replaced())
        {
            // We found un inline box.
            // Whatever happened before, all we need to do before recurring
            // is to remember this ongoing inline level box.
            self.ongoing_inline_boxes_stack.push(InlineBox {
                base_fragment_info: info.into(),
                style: info.style.clone(),
                is_first_fragment: true,
                is_last_fragment: false,
                children: vec![],
            });

            if is_list_item {
                if let Some(marker_contents) = crate::lists::make_marker(self.context, info) {
                    // Ignore `list-style-position` here:
                    // “If the list item is an inline box: this value is equivalent to `inside`.”
                    // https://drafts.csswg.org/css-lists/#list-style-position-outside
                    self.handle_list_item_marker_inside(info, marker_contents)
                }
            }

            // `unwrap` doesn’t panic here because `is_replaced` returned `false`.
            NonReplacedContents::try_from(contents)
                .unwrap()
                .traverse(self.context, info, self);

            let mut inline_box = self
                .ongoing_inline_boxes_stack
                .pop()
                .expect("no ongoing inline level box found");
            inline_box.is_last_fragment = true;
            ArcRefCell::new(InlineLevelBox::InlineBox(inline_box))
        } else {
            self.ongoing_inline_formatting_context.ends_with_whitespace = false;
            ArcRefCell::new(InlineLevelBox::Atomic(
                IndependentFormattingContext::construct(
                    self.context,
                    info,
                    display_inside,
                    contents,
                    // Text decorations are not propagated to atomic inline-level descendants.
                    TextDecorationLine::NONE,
                ),
            ))
        };
        self.current_inline_level_boxes().push(box_.clone());
        box_
    }

    fn handle_block_level_element(
        &mut self,
        info: &NodeAndStyleInfo<Node>,
        display_inside: DisplayInside,
        contents: Contents,
        box_slot: BoxSlot<'dom>,
    ) {
        // We just found a block level element, all ongoing inline level boxes
        // need to be split around it. We iterate on the fragmented inline
        // level box stack to take their contents and set their first_fragment
        // field to false, for the fragmented inline level boxes that will
        // come after the block level element.
        let mut fragmented_inline_boxes =
            self.ongoing_inline_boxes_stack
                .iter_mut()
                .rev()
                .map(|ongoing| {
                    let fragmented = InlineBox {
                        base_fragment_info: ongoing.base_fragment_info,
                        style: ongoing.style.clone(),
                        is_first_fragment: ongoing.is_first_fragment,
                        // The fragmented boxes before the block level element
                        // are obviously not the last fragment.
                        is_last_fragment: false,
                        children: std::mem::take(&mut ongoing.children),
                    };
                    ongoing.is_first_fragment = false;
                    fragmented
                });

        if let Some(last) = fragmented_inline_boxes.next() {
            // There were indeed some ongoing inline level boxes before
            // the block, we accumulate them as a single inline level box
            // to be pushed to the ongoing inline formatting context.
            let mut fragmented_inline = InlineLevelBox::InlineBox(last);
            for mut fragmented_parent_inline_box in fragmented_inline_boxes {
                fragmented_parent_inline_box
                    .children
                    .push(ArcRefCell::new(fragmented_inline));
                fragmented_inline = InlineLevelBox::InlineBox(fragmented_parent_inline_box);
            }

            self.ongoing_inline_formatting_context
                .inline_level_boxes
                .push(ArcRefCell::new(fragmented_inline));
        }

        let propagated_text_decoration_line =
            self.ongoing_inline_formatting_context.text_decoration_line;

        // We found a block level element, so the ongoing inline formatting
        // context needs to be ended.
        self.end_ongoing_inline_formatting_context();

        let kind = match contents.try_into() {
            Ok(contents) => match display_inside {
                DisplayInside::Flow { is_list_item }
                    if !info.style.establishes_block_formatting_context() =>
                {
                    BlockLevelCreator::SameFormattingContextBlock(
                        IntermediateBlockContainer::Deferred {
                            contents,
                            propagated_text_decoration_line,
                            is_list_item,
                        },
                    )
                },
                _ => BlockLevelCreator::Independent {
                    display_inside,
                    contents: contents.into(),
                    propagated_text_decoration_line,
                },
            },
            Err(contents) => {
                let contents = Contents::Replaced(contents);
                BlockLevelCreator::Independent {
                    display_inside,
                    contents,
                    propagated_text_decoration_line,
                }
            },
        };
        self.block_level_boxes.push(BlockLevelJob {
            info: info.clone(),
            box_slot,
            kind,
        });
    }

    fn handle_absolutely_positioned_element(
        &mut self,
        info: &NodeAndStyleInfo<Node>,
        display_inside: DisplayInside,
        contents: Contents,
        box_slot: BoxSlot<'dom>,
    ) {
        if !self.has_ongoing_inline_formatting_context() {
            let kind = BlockLevelCreator::OutOfFlowAbsolutelyPositionedBox {
                contents,
                display_inside,
            };
            self.block_level_boxes.push(BlockLevelJob {
                info: info.clone(),
                box_slot,
                kind,
            });
        } else {
            let box_ = ArcRefCell::new(InlineLevelBox::OutOfFlowAbsolutelyPositionedBox(
                ArcRefCell::new(AbsolutelyPositionedBox::construct(
                    self.context,
                    info,
                    display_inside,
                    contents,
                )),
            ));
            self.current_inline_level_boxes().push(box_.clone());
            box_slot.set(LayoutBox::InlineLevel(box_))
        }
    }

    fn handle_float_element(
        &mut self,
        info: &NodeAndStyleInfo<Node>,
        display_inside: DisplayInside,
        contents: Contents,
        box_slot: BoxSlot<'dom>,
    ) {
        if !self.has_ongoing_inline_formatting_context() {
            let kind = BlockLevelCreator::OutOfFlowFloatBox {
                contents,
                display_inside,
            };
            self.block_level_boxes.push(BlockLevelJob {
                info: info.clone(),
                box_slot,
                kind,
            });
        } else {
            let box_ = ArcRefCell::new(InlineLevelBox::OutOfFlowFloatBox(FloatBox::construct(
                self.context,
                info,
                display_inside,
                contents,
            )));
            self.ongoing_inline_formatting_context.contains_floats = true;
            self.current_inline_level_boxes().push(box_.clone());
            box_slot.set(LayoutBox::InlineLevel(box_))
        }
    }

    fn end_ongoing_inline_formatting_context(&mut self) {
        if self.ongoing_inline_formatting_context.is_empty() {
            // There should never be an empty inline formatting context.
            self.ongoing_inline_formatting_context
                .has_first_formatted_line = false;
            self.ongoing_inline_formatting_context.ends_with_whitespace = false;
            return;
        }

        let context = self.context;
        let block_container_style = &self.info.style;
        let anonymous_style = self.anonymous_style.get_or_insert_with(|| {
            context
                .shared_context()
                .stylist
                .style_for_anonymous::<Node::ConcreteElement>(
                    &context.shared_context().guards,
                    &PseudoElement::ServoAnonymousBox,
                    block_container_style,
                )
        });

        let mut ifc = InlineFormattingContext::new(
            self.ongoing_inline_formatting_context.text_decoration_line,
            /* has_first_formatted_line = */ false,
            /* ends_with_whitespace */ false,
        );
        std::mem::swap(&mut self.ongoing_inline_formatting_context, &mut ifc);
        let kind = BlockLevelCreator::SameFormattingContextBlock(
            IntermediateBlockContainer::InlineFormattingContext(ifc),
        );
        let info = self.info.new_replacing_style(anonymous_style.clone());
        self.block_level_boxes.push(BlockLevelJob {
            info,
            // FIXME(nox): We should be storing this somewhere.
            box_slot: BoxSlot::dummy(),
            kind,
        });
    }

    // Retrieves the mutable reference of inline boxes either from the last
    // element of a stack or directly from the formatting context, depending on the situation.
    fn current_inline_level_boxes(&mut self) -> &mut Vec<ArcRefCell<InlineLevelBox>> {
        match self.ongoing_inline_boxes_stack.last_mut() {
            Some(last) => &mut last.children,
            None => &mut self.ongoing_inline_formatting_context.inline_level_boxes,
        }
    }

    fn has_ongoing_inline_formatting_context(&self) -> bool {
        !self.ongoing_inline_formatting_context.is_empty() ||
            !self.ongoing_inline_boxes_stack.is_empty()
    }
}

impl<'dom, Node> BlockLevelJob<'dom, Node>
where
    Node: NodeExt<'dom>,
{
    fn finish(self, context: &LayoutContext) -> ArcRefCell<BlockLevelBox> {
        let info = &self.info;
        let block_level_box = match self.kind {
            BlockLevelCreator::SameFormattingContextBlock(intermediate_block_container) => {
                let contents = intermediate_block_container.finish(context, info);
                let contains_floats = contents.contains_floats();
                ArcRefCell::new(BlockLevelBox::SameFormattingContextBlock {
                    base_fragment_info: info.into(),
                    contents,
                    style: Arc::clone(&info.style),
                    contains_floats,
                })
            },
            BlockLevelCreator::Independent {
                display_inside,
                contents,
                propagated_text_decoration_line,
            } => {
                let context = IndependentFormattingContext::construct(
                    context,
                    info,
                    display_inside,
                    contents,
                    propagated_text_decoration_line,
                );
                ArcRefCell::new(BlockLevelBox::Independent(context))
            },
            BlockLevelCreator::OutOfFlowAbsolutelyPositionedBox {
                display_inside,
                contents,
            } => ArcRefCell::new(BlockLevelBox::OutOfFlowAbsolutelyPositionedBox(
                ArcRefCell::new(AbsolutelyPositionedBox::construct(
                    context,
                    info,
                    display_inside,
                    contents,
                )),
            )),
            BlockLevelCreator::OutOfFlowFloatBox {
                display_inside,
                contents,
            } => ArcRefCell::new(BlockLevelBox::OutOfFlowFloatBox(FloatBox::construct(
                context,
                info,
                display_inside,
                contents,
            ))),
        };
        self.box_slot
            .set(LayoutBox::BlockLevel(block_level_box.clone()));
        block_level_box
    }
}

impl IntermediateBlockContainer {
    fn finish<'dom, Node>(
        self,
        context: &LayoutContext,
        info: &NodeAndStyleInfo<Node>,
    ) -> BlockContainer
    where
        Node: NodeExt<'dom>,
    {
        match self {
            IntermediateBlockContainer::Deferred {
                contents,
                propagated_text_decoration_line,
                is_list_item,
            } => BlockContainer::construct(
                context,
                info,
                contents,
                propagated_text_decoration_line,
                is_list_item,
            ),
            IntermediateBlockContainer::InlineFormattingContext(ifc) => {
                BlockContainer::InlineFormattingContext(ifc)
            },
        }
    }
}

#[test]
fn test_collapase_and_transform_whitespace() {
    let output = collapse_and_transform_whitespace("H ", WhiteSpace::Normal, false);
    assert_eq!(output.0, "H ");
    assert!(output.1);

    let output = collapse_and_transform_whitespace(" W", WhiteSpace::Normal, true);
    assert_eq!(output.0, "W");
    assert!(output.1);

    let output = collapse_and_transform_whitespace(" W", WhiteSpace::Normal, false);
    assert_eq!(output.0, " W");
    assert!(output.1);

    let output = collapse_and_transform_whitespace(" H  W", WhiteSpace::Normal, false);
    assert_eq!(output.0, " H W");
    assert!(output.1);

    let output = collapse_and_transform_whitespace("\n   H  \n \t  W", WhiteSpace::Normal, false);
    assert_eq!(output.0, " H W");

    let output = collapse_and_transform_whitespace("\n   H  \n \t  W   \n", WhiteSpace::Pre, false);
    assert_eq!(output.0, "\n   H  \n \t  W   \n");
    assert!(output.1);

    let output =
        collapse_and_transform_whitespace("\n   H  \n \t  W   \n ", WhiteSpace::PreLine, false);
    assert_eq!(output.0, "\nH\nW\n");
    assert!(output.1);

    let output = collapse_and_transform_whitespace(" ", WhiteSpace::Normal, true);
    assert_eq!(output.0, "");
    assert!(!output.1);

    let output = collapse_and_transform_whitespace(" ", WhiteSpace::Normal, false);
    assert_eq!(output.0, " ");
    assert!(!output.1);

    let output = collapse_and_transform_whitespace("\n        ", WhiteSpace::Normal, true);
    assert_eq!(output.0, "");
    assert!(!output.1);

    let output = collapse_and_transform_whitespace("\n        ", WhiteSpace::Normal, false);
    assert_eq!(output.0, " ");
    assert!(!output.1);
}
