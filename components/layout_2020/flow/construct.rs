/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::Cow;
use std::convert::{TryFrom, TryInto};

use rayon::iter::{IntoParallelIterator, ParallelIterator};
use servo_arc::Arc;
use style::properties::longhands::list_style_position::computed_value::T as ListStylePosition;
use style::properties::ComputedValues;
use style::selector_parser::PseudoElement;
use style::str::char_is_whitespace;
use style::values::specified::text::TextDecorationLine;

use crate::cell::ArcRefCell;
use crate::context::LayoutContext;
use crate::dom::{BoxSlot, LayoutBox, NodeExt};
use crate::dom_traversal::{Contents, NodeAndStyleInfo, NonReplacedContents, TraversalHandler};
use crate::flow::float::FloatBox;
use crate::flow::inline::{InlineBox, InlineFormattingContext, InlineLevelBox};
use crate::flow::text_run::TextRun;
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

    pub fn construct_for_text_runs(
        runs: impl Iterator<Item = TextRun>,
        layout_context: &LayoutContext,
        text_decoration_line: TextDecorationLine,
    ) -> Self {
        let inline_level_boxes = runs
            .map(|run| ArcRefCell::new(InlineLevelBox::TextRun(run)))
            .collect();

        let ifc = InlineFormattingContext {
            inline_level_boxes,
            font_metrics: Vec::new(),
            text_decoration_line,
            has_first_formatted_line: true,
            contains_floats: false,
        };
        Self {
            contents: BlockContainer::construct_inline_formatting_context(layout_context, ifc),
            contains_floats: false,
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
        builder.finish()
    }

    pub(super) fn construct_inline_formatting_context(
        layout_context: &LayoutContext,
        mut ifc: InlineFormattingContext,
    ) -> Self {
        // TODO(mrobinson): Perhaps it would be better to iteratively break and shape the contents
        // of the IFC, and not wait until it is completely built.
        ifc.break_and_shape_text(layout_context);
        BlockContainer::InlineFormattingContext(ifc)
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
            ongoing_inline_formatting_context: InlineFormattingContext::new(
                text_decoration_line,
                /* has_first_formatted_line = */ true,
            ),
            ongoing_inline_boxes_stack: Vec::new(),
            anonymous_style: None,
            anonymous_table_content: Vec::new(),
        }
    }

    pub(crate) fn finish(mut self) -> BlockContainer {
        debug_assert!(self.ongoing_inline_boxes_stack.is_empty());

        self.finish_anonymous_table_if_needed();

        if !self.ongoing_inline_formatting_context.is_empty() {
            if self.block_level_boxes.is_empty() {
                return BlockContainer::construct_inline_formatting_context(
                    self.context,
                    self.ongoing_inline_formatting_context,
                );
            }
            self.end_ongoing_inline_formatting_context();
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

        // Text decorations are not propagated to atomic inline-level descendants.
        // From https://drafts.csswg.org/css2/#lining-striking-props:
        // >  Note that text decorations are not propagated to floating and absolutely
        // > positioned descendants, nor to the contents of atomic inline-level descendants
        // > such as inline blocks and inline tables.
        let inline_table = !self.ongoing_inline_boxes_stack.is_empty();
        let propagated_text_decoration_line = if inline_table {
            TextDecorationLine::NONE
        } else {
            self.ongoing_inline_formatting_context.text_decoration_line
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
            self.current_inline_level_boxes()
                .push(ArcRefCell::new(InlineLevelBox::Atomic(ifc)));
        } else {
            self.end_ongoing_inline_formatting_context();
            let anonymous_info = self.info.new_anonymous(ifc.style().clone());
            let table_block = ArcRefCell::new(BlockLevelBox::Independent(ifc));
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
                    DisplayOutside::Inline => box_slot.set(LayoutBox::InlineLevel(
                        self.handle_inline_level_element(info, inside, contents),
                    )),
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

    fn handle_text(&mut self, info: &NodeAndStyleInfo<Node>, input: Cow<'dom, str>) {
        if input.is_empty() {
            return;
        }

        // If we are building an anonymous table ie this text directly followed internal
        // table elements that did not have a `<table>` ancestor, then we forward all
        // whitespace to the table builder.
        if !self.anonymous_table_content.is_empty() && input.chars().all(char_is_whitespace) {
            self.anonymous_table_content
                .push(AnonymousTableContent::Text(info.clone(), input));
            return;
        } else {
            self.finish_anonymous_table_if_needed();
        }

        // TODO: We can do better here than `push_str` and wait until we are breaking and
        // shaping text to allocate space big enough for the final text. It would require
        // collecting all Cow strings into a vector and passing them along to text breaking
        // and shaping during final InlineFormattingContext construction.
        let inlines = self.current_inline_level_boxes();
        if let Some(mut last_box) = inlines.last_mut().map(|last| last.borrow_mut()) {
            if let InlineLevelBox::TextRun(ref mut text_run) = *last_box {
                text_run.text.push_str(&input);
                return;
            }
        }

        inlines.push(ArcRefCell::new(InlineLevelBox::TextRun(TextRun::new(
            info.into(),
            Arc::clone(&info.style),
            input.into(),
        ))));
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
                default_font_index: None,
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

            self.finish_anonymous_table_if_needed();

            let mut inline_box = self
                .ongoing_inline_boxes_stack
                .pop()
                .expect("no ongoing inline level box found");
            inline_box.is_last_fragment = true;
            ArcRefCell::new(InlineLevelBox::InlineBox(inline_box))
        } else {
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
                        default_font_index: None,
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
        );
        std::mem::swap(&mut self.ongoing_inline_formatting_context, &mut ifc);

        let info = self.info.new_anonymous(anonymous_style.clone());
        self.block_level_boxes.push(BlockLevelJob {
            info,
            // FIXME(nox): We should be storing this somewhere.
            box_slot: BoxSlot::dummy(),
            kind: BlockLevelCreator::SameFormattingContextBlock(
                IntermediateBlockContainer::InlineFormattingContext(
                    BlockContainer::construct_inline_formatting_context(self.context, ifc),
                ),
            ),
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
