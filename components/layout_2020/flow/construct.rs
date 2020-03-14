/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::cell::ArcRefCell;
use crate::context::LayoutContext;
use crate::dom_traversal::{BoxSlot, Contents, NodeExt, NonReplacedContents, TraversalHandler};
use crate::element_data::LayoutBox;
use crate::flow::float::FloatBox;
use crate::flow::inline::{InlineBox, InlineFormattingContext, InlineLevelBox, TextRun};
use crate::flow::{BlockContainer, BlockFormattingContext, BlockLevelBox};
use crate::formatting_contexts::IndependentFormattingContext;
use crate::positioned::AbsolutelyPositionedBox;
use crate::sizing::{BoxContentSizes, ContentSizes, ContentSizesRequest};
use crate::style_ext::{ComputedValuesExt, DisplayGeneratingBox, DisplayInside, DisplayOutside};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use rayon_croissant::ParallelIteratorExt;
use servo_arc::Arc;
use std::convert::{TryFrom, TryInto};
use style::properties::ComputedValues;
use style::selector_parser::PseudoElement;

impl BlockFormattingContext {
    pub fn construct<'dom>(
        context: &LayoutContext,
        node: impl NodeExt<'dom>,
        style: &Arc<ComputedValues>,
        contents: NonReplacedContents,
        content_sizes: ContentSizesRequest,
    ) -> (Self, BoxContentSizes) {
        let (contents, contains_floats, inline_content_sizes) =
            BlockContainer::construct(context, node, style, contents, content_sizes);
        // FIXME: add contribution to `inline_content_sizes` of floats in this formatting context
        // https://dbaron.org/css/intrinsic/#intrinsic
        let bfc = Self {
            contents,
            contains_floats: contains_floats == ContainsFloats::Yes,
        };
        (bfc, inline_content_sizes)
    }
}

struct BlockLevelJob<'dom, Node> {
    node: Node,
    box_slot: BoxSlot<'dom>,
    style: Arc<ComputedValues>,
    kind: BlockLevelCreator,
}

enum BlockLevelCreator {
    SameFormattingContextBlock(IntermediateBlockContainer),
    Independent {
        display_inside: DisplayInside,
        contents: Contents,
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
    Deferred(NonReplacedContents),
}

/// A builder for a block container.
///
/// This builder starts from the first child of a given DOM node
/// and does a preorder traversal of all of its inclusive siblings.
struct BlockContainerBuilder<'dom, 'style, Node> {
    context: &'style LayoutContext<'style>,

    root: Node,

    block_container_style: &'style Arc<ComputedValues>,

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

    /// Whether the resulting block container contains any float box.
    contains_floats: ContainsFloats,
}

impl BlockContainer {
    pub fn construct<'dom>(
        context: &LayoutContext,
        root: impl NodeExt<'dom>,
        block_container_style: &Arc<ComputedValues>,
        contents: NonReplacedContents,
        content_sizes: ContentSizesRequest,
    ) -> (BlockContainer, ContainsFloats, BoxContentSizes) {
        let mut builder = BlockContainerBuilder {
            context,
            root,
            block_container_style,
            block_level_boxes: Vec::new(),
            ongoing_inline_formatting_context: InlineFormattingContext::default(),
            ongoing_inline_boxes_stack: Vec::new(),
            anonymous_style: None,
            contains_floats: ContainsFloats::No,
        };

        contents.traverse(context, root, block_container_style, &mut builder);

        debug_assert!(builder.ongoing_inline_boxes_stack.is_empty());

        if !builder
            .ongoing_inline_formatting_context
            .inline_level_boxes
            .is_empty()
        {
            if builder.block_level_boxes.is_empty() {
                let content_sizes = content_sizes.compute(|| {
                    builder
                        .ongoing_inline_formatting_context
                        .inline_content_sizes(context)
                });
                let container = BlockContainer::InlineFormattingContext(
                    builder.ongoing_inline_formatting_context,
                );
                return (container, builder.contains_floats, content_sizes);
            }
            builder.end_ongoing_inline_formatting_context();
        }

        struct Accumulator {
            contains_floats: ContainsFloats,
            outer_content_sizes_of_children: ContentSizes,
        }
        let mut acc = Accumulator {
            contains_floats: builder.contains_floats,
            outer_content_sizes_of_children: ContentSizes::zero(),
        };
        let mapfold = |acc: &mut Accumulator, creator: BlockLevelJob<'dom, _>| {
            let (block_level_box, box_contains_floats) = creator.finish(
                context,
                content_sizes.if_requests_inline(|| &mut acc.outer_content_sizes_of_children),
            );
            acc.contains_floats |= box_contains_floats;
            block_level_box
        };
        let block_level_boxes = if context.use_rayon {
            builder
                .block_level_boxes
                .into_par_iter()
                .mapfold_reduce_into(
                    &mut acc,
                    mapfold,
                    || Accumulator {
                        contains_floats: ContainsFloats::No,
                        outer_content_sizes_of_children: ContentSizes::zero(),
                    },
                    |left, right| {
                        left.contains_floats |= right.contains_floats;
                        if content_sizes.requests_inline() {
                            left.outer_content_sizes_of_children
                                .max_assign(&right.outer_content_sizes_of_children)
                        }
                    },
                )
                .collect()
        } else {
            builder
                .block_level_boxes
                .into_iter()
                .map(|x| mapfold(&mut acc, x))
                .collect()
        };
        let container = BlockContainer::BlockLevelBoxes(block_level_boxes);

        let Accumulator {
            contains_floats,
            outer_content_sizes_of_children,
        } = acc;
        let content_sizes = content_sizes.compute(|| outer_content_sizes_of_children);
        (container, contains_floats, content_sizes)
    }
}

impl<'dom, Node> TraversalHandler<'dom, Node> for BlockContainerBuilder<'dom, '_, Node>
where
    Node: NodeExt<'dom>,
{
    fn handle_element(
        &mut self,
        node: Node,
        style: &Arc<ComputedValues>,
        display: DisplayGeneratingBox,
        contents: Contents,
        box_slot: BoxSlot<'dom>,
    ) {
        match display {
            DisplayGeneratingBox::OutsideInside { outside, inside } => match outside {
                DisplayOutside::Inline => box_slot.set(LayoutBox::InlineLevel(
                    self.handle_inline_level_element(node, style, inside, contents),
                )),
                DisplayOutside::Block => {
                    let box_style = style.get_box();
                    // Floats and abspos cause blockification, so they only happen in this case.
                    // https://drafts.csswg.org/css2/visuren.html#dis-pos-flo
                    if box_style.position.is_absolutely_positioned() {
                        self.handle_absolutely_positioned_element(
                            node,
                            style.clone(),
                            inside,
                            contents,
                            box_slot,
                        )
                    } else if box_style.float.is_floating() {
                        self.handle_float_element(node, style.clone(), inside, contents, box_slot)
                    } else {
                        self.handle_block_level_element(
                            node,
                            style.clone(),
                            inside,
                            contents,
                            box_slot,
                        )
                    }
                },
            },
        }
    }

    fn handle_text(&mut self, node: Node, input: String, parent_style: &Arc<ComputedValues>) {
        let (leading_whitespace, mut input) = self.handle_leading_whitespace(&input);
        if leading_whitespace || !input.is_empty() {
            // This text node should be pushed either to the next ongoing
            // inline level box with the parent style of that inline level box
            // that will be ended, or directly to the ongoing inline formatting
            // context with the parent style of that builder.
            let inlines = self.current_inline_level_boxes();

            let mut new_text_run_contents;
            let output;

            {
                let mut last_box = inlines.last_mut().map(|last| last.borrow_mut());
                let last_text = last_box.as_mut().and_then(|last| match &mut **last {
                    InlineLevelBox::TextRun(last) => Some(&mut last.text),
                    _ => None,
                });

                if let Some(text) = last_text {
                    // Append to the existing text run
                    new_text_run_contents = None;
                    output = text;
                } else {
                    new_text_run_contents = Some(String::new());
                    output = new_text_run_contents.as_mut().unwrap();
                }

                if leading_whitespace {
                    output.push(' ')
                }
                loop {
                    if let Some(i) = input.bytes().position(|b| b.is_ascii_whitespace()) {
                        let (non_whitespace, rest) = input.split_at(i);
                        output.push_str(non_whitespace);
                        output.push(' ');
                        if let Some(i) = rest.bytes().position(|b| !b.is_ascii_whitespace()) {
                            input = &rest[i..];
                        } else {
                            break;
                        }
                    } else {
                        output.push_str(input);
                        break;
                    }
                }
            }

            if let Some(text) = new_text_run_contents {
                let parent_style = parent_style.clone();
                inlines.push(ArcRefCell::new(InlineLevelBox::TextRun(TextRun {
                    tag: node.as_opaque(),
                    parent_style,
                    text,
                })))
            }
        }
    }
}

impl<'dom, Node> BlockContainerBuilder<'dom, '_, Node>
where
    Node: NodeExt<'dom>,
{
    /// Returns:
    ///
    /// * Whether this text run has preserved (non-collapsible) leading whitespace
    /// * The contents starting at the first non-whitespace character (or the empty string)
    fn handle_leading_whitespace<'text>(&mut self, text: &'text str) -> (bool, &'text str) {
        // FIXME: this is only an approximation of
        // https://drafts.csswg.org/css2/text.html#white-space-model
        if !text.starts_with(|c: char| c.is_ascii_whitespace()) {
            return (false, text);
        }

        let preserved = match whitespace_is_preserved(self.current_inline_level_boxes()) {
            WhitespacePreservedResult::Unknown => {
                // Paragraph start.
                false
            },
            WhitespacePreservedResult::NotPreserved => false,
            WhitespacePreservedResult::Preserved => true,
        };

        let text = text.trim_start_matches(|c: char| c.is_ascii_whitespace());
        return (preserved, text);

        fn whitespace_is_preserved(
            inline_level_boxes: &[ArcRefCell<InlineLevelBox>],
        ) -> WhitespacePreservedResult {
            for inline_level_box in inline_level_boxes.iter().rev() {
                match *inline_level_box.borrow() {
                    InlineLevelBox::TextRun(ref r) => {
                        if r.text.ends_with(' ') {
                            return WhitespacePreservedResult::NotPreserved;
                        }
                        return WhitespacePreservedResult::Preserved;
                    },
                    InlineLevelBox::Atomic { .. } => {
                        return WhitespacePreservedResult::NotPreserved;
                    },
                    InlineLevelBox::OutOfFlowAbsolutelyPositionedBox(_) |
                    InlineLevelBox::OutOfFlowFloatBox(_) => {},
                    InlineLevelBox::InlineBox(ref b) => {
                        match whitespace_is_preserved(&b.children) {
                            WhitespacePreservedResult::Unknown => {},
                            result => return result,
                        }
                    },
                }
            }

            WhitespacePreservedResult::Unknown
        }

        #[derive(Clone, Copy, PartialEq)]
        enum WhitespacePreservedResult {
            Preserved,
            NotPreserved,
            Unknown,
        }
    }

    fn handle_inline_level_element(
        &mut self,
        node: Node,
        style: &Arc<ComputedValues>,
        display_inside: DisplayInside,
        contents: Contents,
    ) -> ArcRefCell<InlineLevelBox> {
        let box_ = if display_inside == DisplayInside::Flow && !contents.is_replaced() {
            // We found un inline box.
            // Whatever happened before, all we need to do before recurring
            // is to remember this ongoing inline level box.
            self.ongoing_inline_boxes_stack.push(InlineBox {
                tag: node.as_opaque(),
                style: style.clone(),
                first_fragment: true,
                last_fragment: false,
                children: vec![],
            });

            // `unwrap` doesn’t panic here because `is_replaced` returned `false`.
            NonReplacedContents::try_from(contents).unwrap().traverse(
                self.context,
                node,
                &style,
                self,
            );

            let mut inline_box = self
                .ongoing_inline_boxes_stack
                .pop()
                .expect("no ongoing inline level box found");
            inline_box.last_fragment = true;
            ArcRefCell::new(InlineLevelBox::InlineBox(inline_box))
        } else {
            ArcRefCell::new(InlineLevelBox::Atomic(
                IndependentFormattingContext::construct(
                    self.context,
                    node,
                    style.clone(),
                    display_inside,
                    contents,
                    ContentSizesRequest::inline_if(!style.inline_size_is_length()),
                ),
            ))
        };
        self.current_inline_level_boxes().push(box_.clone());
        box_
    }

    fn handle_block_level_element(
        &mut self,
        node: Node,
        style: Arc<ComputedValues>,
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
                        tag: ongoing.tag,
                        style: ongoing.style.clone(),
                        first_fragment: ongoing.first_fragment,
                        // The fragmented boxes before the block level element
                        // are obviously not the last fragment.
                        last_fragment: false,
                        children: std::mem::take(&mut ongoing.children),
                    };
                    ongoing.first_fragment = false;
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

        // We found a block level element, so the ongoing inline formatting
        // context needs to be ended.
        self.end_ongoing_inline_formatting_context();

        let kind = match contents.try_into() {
            Ok(contents) => match display_inside {
                DisplayInside::Flow => BlockLevelCreator::SameFormattingContextBlock(
                    IntermediateBlockContainer::Deferred(contents),
                ),
                _ => BlockLevelCreator::Independent {
                    display_inside,
                    contents: contents.into(),
                },
            },
            Err(contents) => {
                let contents = Contents::Replaced(contents);
                BlockLevelCreator::Independent {
                    display_inside,
                    contents,
                }
            },
        };
        self.block_level_boxes.push(BlockLevelJob {
            node,
            box_slot,
            style,
            kind,
        });
    }

    fn handle_absolutely_positioned_element(
        &mut self,
        node: Node,
        style: Arc<ComputedValues>,
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
                node,
                box_slot,
                style,
                kind,
            });
        } else {
            let box_ = ArcRefCell::new(InlineLevelBox::OutOfFlowAbsolutelyPositionedBox(Arc::new(
                AbsolutelyPositionedBox::construct(
                    self.context,
                    node,
                    style,
                    display_inside,
                    contents,
                ),
            )));
            self.current_inline_level_boxes().push(box_.clone());
            box_slot.set(LayoutBox::InlineLevel(box_))
        }
    }

    fn handle_float_element(
        &mut self,
        node: Node,
        style: Arc<ComputedValues>,
        display_inside: DisplayInside,
        contents: Contents,
        box_slot: BoxSlot<'dom>,
    ) {
        self.contains_floats = ContainsFloats::Yes;

        if !self.has_ongoing_inline_formatting_context() {
            let kind = BlockLevelCreator::OutOfFlowFloatBox {
                contents,
                display_inside,
            };
            self.block_level_boxes.push(BlockLevelJob {
                node,
                box_slot,
                style,
                kind,
            });
        } else {
            let box_ = ArcRefCell::new(InlineLevelBox::OutOfFlowFloatBox(FloatBox::construct(
                self.context,
                node,
                style,
                display_inside,
                contents,
            )));
            self.current_inline_level_boxes().push(box_.clone());
            box_slot.set(LayoutBox::InlineLevel(box_))
        }
    }

    fn end_ongoing_inline_formatting_context(&mut self) {
        if self
            .ongoing_inline_formatting_context
            .inline_level_boxes
            .is_empty()
        {
            // There should never be an empty inline formatting context.
            return;
        }

        let context = self.context;
        let block_container_style = self.block_container_style;
        let anonymous_style = self.anonymous_style.get_or_insert_with(|| {
            context
                .shared_context()
                .stylist
                .style_for_anonymous::<Node::ConcreteElement>(
                    &context.shared_context().guards,
                    &PseudoElement::ServoText,
                    &block_container_style,
                )
        });

        let kind = BlockLevelCreator::SameFormattingContextBlock(
            IntermediateBlockContainer::InlineFormattingContext(std::mem::take(
                &mut self.ongoing_inline_formatting_context,
            )),
        );
        self.block_level_boxes.push(BlockLevelJob {
            node: self.root,
            // FIXME(nox): We should be storing this somewhere.
            box_slot: BoxSlot::dummy(),
            style: anonymous_style.clone(),
            kind,
        });
    }

    fn current_inline_level_boxes(&mut self) -> &mut Vec<ArcRefCell<InlineLevelBox>> {
        match self.ongoing_inline_boxes_stack.last_mut() {
            Some(last) => &mut last.children,
            None => &mut self.ongoing_inline_formatting_context.inline_level_boxes,
        }
    }

    fn has_ongoing_inline_formatting_context(&self) -> bool {
        !self
            .ongoing_inline_formatting_context
            .inline_level_boxes
            .is_empty() ||
            !self.ongoing_inline_boxes_stack.is_empty()
    }
}

impl<'dom, Node> BlockLevelJob<'dom, Node>
where
    Node: NodeExt<'dom>,
{
    fn finish(
        self,
        context: &LayoutContext,
        max_assign_in_flow_outer_content_sizes_to: Option<&mut ContentSizes>,
    ) -> (ArcRefCell<BlockLevelBox>, ContainsFloats) {
        let node = self.node;
        let style = self.style;
        let (block_level_box, contains_floats) = match self.kind {
            BlockLevelCreator::SameFormattingContextBlock(contents) => {
                let (contents, contains_floats, box_content_sizes) = contents.finish(
                    context,
                    node,
                    &style,
                    ContentSizesRequest::inline_if(
                        max_assign_in_flow_outer_content_sizes_to.is_some() &&
                            !style.inline_size_is_length(),
                    ),
                );
                if let Some(to) = max_assign_in_flow_outer_content_sizes_to {
                    to.max_assign(&box_content_sizes.outer_inline(&style))
                }
                let block_level_box = ArcRefCell::new(BlockLevelBox::SameFormattingContextBlock {
                    tag: node.as_opaque(),
                    contents,
                    style,
                });
                (block_level_box, contains_floats)
            },
            BlockLevelCreator::Independent {
                display_inside,
                contents,
            } => {
                let content_sizes = ContentSizesRequest::inline_if(
                    max_assign_in_flow_outer_content_sizes_to.is_some() &&
                        !style.inline_size_is_length(),
                );
                let contents = IndependentFormattingContext::construct(
                    context,
                    node,
                    style,
                    display_inside,
                    contents,
                    content_sizes,
                );
                if let Some(to) = max_assign_in_flow_outer_content_sizes_to {
                    to.max_assign(&contents.content_sizes.outer_inline(&contents.style))
                }
                (
                    ArcRefCell::new(BlockLevelBox::Independent(contents)),
                    ContainsFloats::No,
                )
            },
            BlockLevelCreator::OutOfFlowAbsolutelyPositionedBox {
                display_inside,
                contents,
            } => {
                let block_level_box =
                    ArcRefCell::new(BlockLevelBox::OutOfFlowAbsolutelyPositionedBox(Arc::new(
                        AbsolutelyPositionedBox::construct(
                            context,
                            node,
                            style,
                            display_inside,
                            contents,
                        ),
                    )));
                (block_level_box, ContainsFloats::No)
            },
            BlockLevelCreator::OutOfFlowFloatBox {
                display_inside,
                contents,
            } => {
                let block_level_box = ArcRefCell::new(BlockLevelBox::OutOfFlowFloatBox(
                    FloatBox::construct(context, node, style, display_inside, contents),
                ));
                (block_level_box, ContainsFloats::Yes)
            },
        };
        self.box_slot
            .set(LayoutBox::BlockLevel(block_level_box.clone()));
        (block_level_box, contains_floats)
    }
}

impl IntermediateBlockContainer {
    fn finish<'dom>(
        self,
        context: &LayoutContext,
        node: impl NodeExt<'dom>,
        style: &Arc<ComputedValues>,
        content_sizes: ContentSizesRequest,
    ) -> (BlockContainer, ContainsFloats, BoxContentSizes) {
        match self {
            IntermediateBlockContainer::Deferred(contents) => {
                BlockContainer::construct(context, node, style, contents, content_sizes)
            },
            IntermediateBlockContainer::InlineFormattingContext(ifc) => {
                let content_sizes = content_sizes.compute(|| ifc.inline_content_sizes(context));
                // If that inline formatting context contained any float, those
                // were already taken into account during the first phase of
                // box construction.
                (
                    BlockContainer::InlineFormattingContext(ifc),
                    ContainsFloats::No,
                    content_sizes,
                )
            },
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ContainsFloats {
    No,
    Yes,
}

impl std::ops::BitOrAssign for ContainsFloats {
    fn bitor_assign(&mut self, other: Self) {
        if other == ContainsFloats::Yes {
            *self = ContainsFloats::Yes;
        }
    }
}
