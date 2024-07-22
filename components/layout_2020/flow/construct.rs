/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::Cow;
use std::convert::TryFrom;

use rayon::iter::{IntoParallelIterator, ParallelIterator};
use servo_arc::Arc;
use style::properties::longhands::list_style_position::computed_value::T as ListStylePosition;
use style::properties::ComputedValues;
use style::selector_parser::PseudoElement;
use style::str::char_is_whitespace;
use style::values::specified::text::TextDecorationLine;

use super::inline::construct::InlineFormattingContextBuilder;
use super::inline::inline_box::InlineBox;
use super::inline::InlineFormattingContext;
use super::OutsideMarker;
use crate::cell::ArcRefCell;
use crate::context::LayoutContext;
use crate::dom::{BoxSlot, LayoutBox, NodeExt};
use crate::dom_traversal::{
    Contents, NodeAndStyleInfo, NonReplacedContents, PseudoElementContentItem, TraversalHandler,
};
use crate::flow::float::FloatBox;
use crate::flow::{BlockContainer, BlockFormattingContext, BlockLevelBox};
use crate::formatting_contexts::IndependentFormattingContext;
use crate::positioned::AbsolutelyPositionedBox;
use crate::style_ext::{ComputedValuesExt, DisplayGeneratingBox, DisplayInside, DisplayOutside};
use crate::table::{AnonymousTableContent, Table};

impl BlockFormattingContext {
    pub(crate) fn construct<'dom, Node>(
        context: &LayoutContext,
        info: &NodeAndStyleInfo<Node>,
        contents: NonReplacedContents,
        propagated_text_decoration_line: TextDecorationLine,
        is_list_item: bool,
    ) -> Self
    where
        Node: NodeExt<'dom>,
    {
        Self::from_block_container(BlockContainer::construct(
            context,
            info,
            contents,
            propagated_text_decoration_line,
            is_list_item,
        ))
    }

    pub(crate) fn from_block_container(contents: BlockContainer) -> Self {
        let contains_floats = contents.contains_floats();
        Self {
            contents,
            contains_floats,
        }
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
    OutsideMarker {
        contents: Vec<PseudoElementContentItem>,
    },
    AnonymousTable {
        table_block: ArcRefCell<BlockLevelBox>,
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
    InlineFormattingContext(BlockContainer),
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
pub(crate) struct BlockContainerBuilder<'dom, 'style, Node> {
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

    /// Whether or not this builder has yet produced a block which would be
    /// be considered the first line for the purposes of `text-indent`.
    have_already_seen_first_line_for_text_indent: bool,

    /// The propagated [`TextDecorationLine`].
    text_decoration_line: TextDecorationLine,

    inline_formatting_context_builder: InlineFormattingContextBuilder,

    /// The style of the anonymous block boxes pushed to the list of block-level
    /// boxes, if any (see `end_ongoing_inline_formatting_context`).
    anonymous_style: Option<Arc<ComputedValues>>,

    /// A collection of content that is being added to an anonymous table. This is
    /// composed of any sequence of internal table elements or table captions that
    /// are found outside of a table.
    anonymous_table_content: Vec<AnonymousTableContent<'dom, Node>>,
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
        let mut builder =
            BlockContainerBuilder::new(context, info, propagated_text_decoration_line);

        if is_list_item {
            if let Some(marker_contents) = crate::lists::make_marker(context, info) {
                match info.style.clone_list_style_position() {
                    ListStylePosition::Inside => {
                        builder.handle_list_item_marker_inside(info, marker_contents)
                    },
                    ListStylePosition::Outside => {
                        builder.handle_list_item_marker_outside(info, marker_contents)
                    },
                }
            }
        }

        contents.traverse(context, info, &mut builder);
        builder.finish()
    }
}

impl<'dom, 'style, Node> BlockContainerBuilder<'dom, 'style, Node>
where
    Node: NodeExt<'dom>,
{
    pub(crate) fn new(
        context: &'style LayoutContext,
        info: &'style NodeAndStyleInfo<Node>,
        propagated_text_decoration_line: TextDecorationLine,
    ) -> Self {
        let text_decoration_line =
            propagated_text_decoration_line | info.style.clone_text_decoration_line();

        BlockContainerBuilder {
            context,
            info,
            block_level_boxes: Vec::new(),
            text_decoration_line,
            have_already_seen_first_line_for_text_indent: false,
            anonymous_style: None,
            anonymous_table_content: Vec::new(),
            inline_formatting_context_builder: InlineFormattingContextBuilder::new(),
        }
    }

    pub(crate) fn finish(mut self) -> BlockContainer {
        debug_assert!(!self
            .inline_formatting_context_builder
            .currently_processing_inline_box());

        self.finish_anonymous_table_if_needed();

        if let Some(inline_formatting_context) = self.inline_formatting_context_builder.finish(
            self.context,
            self.text_decoration_line,
            !self.have_already_seen_first_line_for_text_indent,
            self.info.is_single_line_text_input(),
        ) {
            // There are two options here. This block was composed of both one or more inline formatting contexts
            // and child blocks OR this block was a single inline formatting context. In the latter case, we
            // just return the inline formatting context as the block itself.
            if self.block_level_boxes.is_empty() {
                return BlockContainer::InlineFormattingContext(inline_formatting_context);
            }
            self.push_block_level_job_for_inline_formatting_context(inline_formatting_context);
        }

        let context = self.context;
        let block_level_boxes = if self.context.use_rayon {
            self.block_level_boxes
                .into_par_iter()
                .map(|block_level_job| block_level_job.finish(context))
                .collect()
        } else {
            self.block_level_boxes
                .into_iter()
                .map(|block_level_job| block_level_job.finish(context))
                .collect()
        };

        BlockContainer::BlockLevelBoxes(block_level_boxes)
    }

    fn finish_anonymous_table_if_needed(&mut self) {
        if self.anonymous_table_content.is_empty() {
            return;
        }

        // From https://drafts.csswg.org/css-tables/#fixup-algorithm:
        //  > If the box’s parent is an inline, run-in, or ruby box (or any box that would perform
        //  > inlinification of its children), then an inline-table box must be generated; otherwise
        //  > it must be a table box.
        //
        // Note that text content in the inline formatting context isn't enough to force the
        // creation of an inline table. It requires the parent to be an inline box.
        let inline_table = self
            .inline_formatting_context_builder
            .currently_processing_inline_box();

        // Text decorations are not propagated to atomic inline-level descendants.
        // From https://drafts.csswg.org/css2/#lining-striking-props:
        // >  Note that text decorations are not propagated to floating and absolutely
        // > positioned descendants, nor to the contents of atomic inline-level descendants
        // > such as inline blocks and inline tables.
        let propagated_text_decoration_line = if inline_table {
            TextDecorationLine::NONE
        } else {
            self.text_decoration_line
        };

        let contents: Vec<AnonymousTableContent<'dom, Node>> =
            self.anonymous_table_content.drain(..).collect();
        let last_text = match contents.last() {
            Some(AnonymousTableContent::Text(info, text)) => Some((info.clone(), text.clone())),
            _ => None,
        };

        let ifc = Table::construct_anonymous(
            self.context,
            self.info,
            contents,
            propagated_text_decoration_line,
        );

        if inline_table {
            self.inline_formatting_context_builder.push_atomic(ifc);
        } else {
            let anonymous_info = self.info.new_anonymous(ifc.style().clone());
            let table_block = ArcRefCell::new(BlockLevelBox::Independent(ifc));
            self.end_ongoing_inline_formatting_context();
            self.block_level_boxes.push(BlockLevelJob {
                info: anonymous_info,
                box_slot: BoxSlot::dummy(),
                kind: BlockLevelCreator::AnonymousTable { table_block },
            });
        }

        // If the last element in the anonymous table content is whitespace, that
        // whitespace doesn't actually belong to the table. It should be processed outside
        // ie become a space between the anonymous table and the rest of the block
        // content. Anonymous tables are really only constructed around internal table
        // elements and the whitespace between them, so this trailing whitespace should
        // not be included.
        //
        // See https://drafts.csswg.org/css-tables/#fixup-algorithm sections "Remove
        // irrelevant boxes" and "Generate missing parents."
        if let Some((info, text)) = last_text {
            self.handle_text(&info, text);
        }
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
            DisplayGeneratingBox::OutsideInside { outside, inside } => {
                self.finish_anonymous_table_if_needed();

                match outside {
                    DisplayOutside::Inline => {
                        self.handle_inline_level_element(info, inside, contents, box_slot)
                    },
                    DisplayOutside::Block => {
                        let box_style = info.style.get_box();
                        // Floats and abspos cause blockification, so they only happen in this case.
                        // https://drafts.csswg.org/css2/visuren.html#dis-pos-flo
                        if box_style.position.is_absolutely_positioned() {
                            self.handle_absolutely_positioned_element(
                                info, inside, contents, box_slot,
                            )
                        } else if box_style.float.is_floating() {
                            self.handle_float_element(info, inside, contents, box_slot)
                        } else {
                            self.handle_block_level_element(info, inside, contents, box_slot)
                        }
                    },
                };
            },
            DisplayGeneratingBox::LayoutInternal(_) => {
                self.anonymous_table_content
                    .push(AnonymousTableContent::Element {
                        info: info.clone(),
                        display,
                        contents,
                        box_slot,
                    });
            },
        }
    }

    fn handle_text(&mut self, info: &NodeAndStyleInfo<Node>, text: Cow<'dom, str>) {
        if text.is_empty() {
            return;
        }

        // If we are building an anonymous table ie this text directly followed internal
        // table elements that did not have a `<table>` ancestor, then we forward all
        // whitespace to the table builder.
        if !self.anonymous_table_content.is_empty() && text.chars().all(char_is_whitespace) {
            self.anonymous_table_content
                .push(AnonymousTableContent::Text(info.clone(), text));
            return;
        } else {
            self.finish_anonymous_table_if_needed();
        }

        self.inline_formatting_context_builder.push_text(text, info);
    }
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
            NonReplacedContents::OfPseudoElement(contents).into(),
            BoxSlot::dummy(),
        );
    }

    fn handle_list_item_marker_outside(
        &mut self,
        info: &NodeAndStyleInfo<Node>,
        contents: Vec<crate::dom_traversal::PseudoElementContentItem>,
    ) {
        self.block_level_boxes.push(BlockLevelJob {
            info: info.clone(),
            box_slot: BoxSlot::dummy(),
            kind: BlockLevelCreator::OutsideMarker { contents },
        });
    }

    fn handle_inline_level_element(
        &mut self,
        info: &NodeAndStyleInfo<Node>,
        display_inside: DisplayInside,
        contents: Contents,
        box_slot: BoxSlot<'dom>,
    ) {
        let (DisplayInside::Flow { is_list_item }, false) =
            (display_inside, contents.is_replaced())
        else {
            // If this inline element is an atomic, handle it and return.
            let atomic = self.inline_formatting_context_builder.push_atomic(
                IndependentFormattingContext::construct(
                    self.context,
                    info,
                    display_inside,
                    contents,
                    // Text decorations are not propagated to atomic inline-level descendants.
                    TextDecorationLine::NONE,
                ),
            );
            box_slot.set(LayoutBox::InlineLevel(atomic));
            return;
        };

        // Otherwise, this is just a normal inline box. Whatever happened before, all we need to do
        // before recurring is to remember this ongoing inline level box.
        self.inline_formatting_context_builder
            .start_inline_box(InlineBox::new(info));

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

        self.finish_anonymous_table_if_needed();

        box_slot.set(LayoutBox::InlineBox(
            self.inline_formatting_context_builder.end_inline_box(),
        ));
    }

    fn handle_block_level_element(
        &mut self,
        info: &NodeAndStyleInfo<Node>,
        display_inside: DisplayInside,
        contents: Contents,
        box_slot: BoxSlot<'dom>,
    ) {
        // We just found a block level element, all ongoing inline level boxes
        // need to be split around it.
        //
        // After calling `split_around_block_and_finish`,
        // `self.inline_formatting_context_builder` is set up with the state
        // that we want to have after we push the block below.
        if let Some(inline_formatting_context) = self
            .inline_formatting_context_builder
            .split_around_block_and_finish(
                self.context,
                self.text_decoration_line,
                !self.have_already_seen_first_line_for_text_indent,
            )
        {
            self.push_block_level_job_for_inline_formatting_context(inline_formatting_context);
        }

        let propagated_text_decoration_line = self.text_decoration_line;
        let kind = match contents {
            Contents::NonReplaced(contents) => match display_inside {
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
            Contents::Replaced(contents) => {
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

        // Any block also counts as the first line for the purposes of text indent. Even if
        // they don't actually indent.
        self.have_already_seen_first_line_for_text_indent = true;
    }

    fn handle_absolutely_positioned_element(
        &mut self,
        info: &NodeAndStyleInfo<Node>,
        display_inside: DisplayInside,
        contents: Contents,
        box_slot: BoxSlot<'dom>,
    ) {
        if !self.inline_formatting_context_builder.is_empty() {
            let inline_level_box = self
                .inline_formatting_context_builder
                .push_absolutely_positioned_box(AbsolutelyPositionedBox::construct(
                    self.context,
                    info,
                    display_inside,
                    contents,
                ));
            box_slot.set(LayoutBox::InlineLevel(inline_level_box));
            return;
        }

        let kind = BlockLevelCreator::OutOfFlowAbsolutelyPositionedBox {
            contents,
            display_inside,
        };
        self.block_level_boxes.push(BlockLevelJob {
            info: info.clone(),
            box_slot,
            kind,
        });
    }

    fn handle_float_element(
        &mut self,
        info: &NodeAndStyleInfo<Node>,
        display_inside: DisplayInside,
        contents: Contents,
        box_slot: BoxSlot<'dom>,
    ) {
        if !self.inline_formatting_context_builder.is_empty() {
            let inline_level_box =
                self.inline_formatting_context_builder
                    .push_float_box(FloatBox::construct(
                        self.context,
                        info,
                        display_inside,
                        contents,
                    ));
            box_slot.set(LayoutBox::InlineLevel(inline_level_box));
            return;
        }

        let kind = BlockLevelCreator::OutOfFlowFloatBox {
            contents,
            display_inside,
        };
        self.block_level_boxes.push(BlockLevelJob {
            info: info.clone(),
            box_slot,
            kind,
        });
    }

    fn end_ongoing_inline_formatting_context(&mut self) {
        if let Some(inline_formatting_context) = self.inline_formatting_context_builder.finish(
            self.context,
            self.text_decoration_line,
            !self.have_already_seen_first_line_for_text_indent,
            self.info.is_single_line_text_input(),
        ) {
            self.push_block_level_job_for_inline_formatting_context(inline_formatting_context);
        }
    }

    fn push_block_level_job_for_inline_formatting_context(
        &mut self,
        inline_formatting_context: InlineFormattingContext,
    ) {
        let block_container_style = &self.info.style;
        let layout_context = self.context;
        let anonymous_style = self.anonymous_style.get_or_insert_with(|| {
            layout_context
                .shared_context()
                .stylist
                .style_for_anonymous::<Node::ConcreteElement>(
                    &layout_context.shared_context().guards,
                    &PseudoElement::ServoAnonymousBox,
                    block_container_style,
                )
        });

        let info = self.info.new_anonymous(anonymous_style.clone());
        self.block_level_boxes.push(BlockLevelJob {
            info,
            // FIXME(nox): We should be storing this somewhere.
            box_slot: BoxSlot::dummy(),
            kind: BlockLevelCreator::SameFormattingContextBlock(
                IntermediateBlockContainer::InlineFormattingContext(
                    BlockContainer::InlineFormattingContext(inline_formatting_context),
                ),
            ),
        });

        self.have_already_seen_first_line_for_text_indent = true;
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
            BlockLevelCreator::OutsideMarker { contents } => {
                let marker_style = context
                    .shared_context()
                    .stylist
                    .style_for_anonymous::<Node::ConcreteElement>(
                        &context.shared_context().guards,
                        &PseudoElement::ServoLegacyText, // FIMXE: use `PseudoElement::Marker` when we add it
                        &info.style,
                    );
                let contents = NonReplacedContents::OfPseudoElement(contents);
                let block_container = BlockContainer::construct(
                    context,
                    &info.new_replacing_style(marker_style.clone()),
                    contents,
                    TextDecorationLine::empty(),
                    false, /* is_list_item */
                );
                ArcRefCell::new(BlockLevelBox::OutsideMarker(OutsideMarker {
                    marker_style,
                    list_item_style: info.style.clone(),
                    block_container,
                }))
            },
            BlockLevelCreator::AnonymousTable { table_block } => table_block,
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
            IntermediateBlockContainer::InlineFormattingContext(block_container) => block_container,
        }
    }
}
