/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::Cow;

use rayon::iter::{IntoParallelIterator, ParallelIterator};
use servo_arc::Arc;
use style::properties::ComputedValues;
use style::properties::longhands::list_style_position::computed_value::T as ListStylePosition;
use style::selector_parser::PseudoElement;
use style::str::char_is_whitespace;

use super::OutsideMarker;
use super::inline::construct::InlineFormattingContextBuilder;
use super::inline::inline_box::InlineBox;
use super::inline::{InlineFormattingContext, SharedInlineStyles};
use crate::PropagatedBoxTreeData;
use crate::cell::ArcRefCell;
use crate::context::LayoutContext;
use crate::dom::{BoxSlot, LayoutBox, NodeExt};
use crate::dom_traversal::{
    Contents, NodeAndStyleInfo, NonReplacedContents, PseudoElementContentItem, TraversalHandler,
};
use crate::flow::float::FloatBox;
use crate::flow::{BlockContainer, BlockFormattingContext, BlockLevelBox};
use crate::formatting_contexts::IndependentFormattingContext;
use crate::fragment_tree::FragmentFlags;
use crate::layout_box_base::LayoutBoxBase;
use crate::positioned::AbsolutelyPositionedBox;
use crate::style_ext::{ComputedValuesExt, DisplayGeneratingBox, DisplayInside, DisplayOutside};
use crate::table::{AnonymousTableContent, Table};

impl BlockFormattingContext {
    pub(crate) fn construct(
        context: &LayoutContext,
        info: &NodeAndStyleInfo<'_>,
        contents: NonReplacedContents,
        propagated_data: PropagatedBoxTreeData,
        is_list_item: bool,
    ) -> Self {
        Self::from_block_container(BlockContainer::construct(
            context,
            info,
            contents,
            propagated_data,
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

struct BlockLevelJob<'dom> {
    info: NodeAndStyleInfo<'dom>,
    box_slot: BoxSlot<'dom>,
    propagated_data: PropagatedBoxTreeData,
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
    OutsideMarker {
        list_item_style: Arc<ComputedValues>,
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
        propagated_data: PropagatedBoxTreeData,
        is_list_item: bool,
    },
}

/// A builder for a block container.
///
/// This builder starts from the first child of a given DOM node
/// and does a preorder traversal of all of its inclusive siblings.
pub(crate) struct BlockContainerBuilder<'dom, 'style> {
    context: &'style LayoutContext<'style>,

    /// This NodeAndStyleInfo contains the root node, the corresponding pseudo
    /// content designator, and the block container style.
    info: &'style NodeAndStyleInfo<'dom>,

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
    block_level_boxes: Vec<BlockLevelJob<'dom>>,

    /// Whether or not this builder has yet produced a block which would be
    /// be considered the first line for the purposes of `text-indent`.
    have_already_seen_first_line_for_text_indent: bool,

    /// The propagated data to use for BoxTree construction.
    propagated_data: PropagatedBoxTreeData,

    /// The [`InlineFormattingContextBuilder`] if we have encountered any inline items,
    /// otherwise None.
    ///
    /// TODO: This can be `OnceCell` once `OnceCell::get_mut_or_init` is stabilized.
    inline_formatting_context_builder: Option<InlineFormattingContextBuilder>,

    /// The [`NodeAndStyleInfo`] to use for anonymous block boxes pushed to the list of
    /// block-level boxes, lazily initialized.
    anonymous_box_info: Option<NodeAndStyleInfo<'dom>>,

    /// A collection of content that is being added to an anonymous table. This is
    /// composed of any sequence of internal table elements or table captions that
    /// are found outside of a table.
    anonymous_table_content: Vec<AnonymousTableContent<'dom>>,

    /// Any [`InlineFormattingContexts`] created need to know about the ongoing `display: contents`
    /// ancestors that have been processed. This `Vec` allows passing those into new
    /// [`InlineFormattingContext`]s that we create.
    display_contents_shared_styles: Vec<SharedInlineStyles>,
}

impl BlockContainer {
    pub fn construct(
        context: &LayoutContext,
        info: &NodeAndStyleInfo<'_>,
        contents: NonReplacedContents,
        propagated_data: PropagatedBoxTreeData,
        is_list_item: bool,
    ) -> BlockContainer {
        let mut builder = BlockContainerBuilder::new(context, info, propagated_data);

        if is_list_item {
            if let Some((marker_info, marker_contents)) = crate::lists::make_marker(context, info) {
                match marker_info.style.clone_list_style_position() {
                    ListStylePosition::Inside => {
                        builder.handle_list_item_marker_inside(&marker_info, info, marker_contents)
                    },
                    ListStylePosition::Outside => builder.handle_list_item_marker_outside(
                        &marker_info,
                        info,
                        marker_contents,
                        info.style.clone(),
                    ),
                }
            }
        }

        contents.traverse(context, info, &mut builder);
        builder.finish()
    }
}

impl<'dom, 'style> BlockContainerBuilder<'dom, 'style> {
    pub(crate) fn new(
        context: &'style LayoutContext,
        info: &'style NodeAndStyleInfo<'dom>,
        propagated_data: PropagatedBoxTreeData,
    ) -> Self {
        BlockContainerBuilder {
            context,
            info,
            block_level_boxes: Vec::new(),
            propagated_data,
            have_already_seen_first_line_for_text_indent: false,
            anonymous_box_info: None,
            anonymous_table_content: Vec::new(),
            inline_formatting_context_builder: None,
            display_contents_shared_styles: Vec::new(),
        }
    }

    fn currently_processing_inline_box(&self) -> bool {
        self.inline_formatting_context_builder
            .as_ref()
            .is_some_and(InlineFormattingContextBuilder::currently_processing_inline_box)
    }

    fn ensure_inline_formatting_context_builder(&mut self) -> &mut InlineFormattingContextBuilder {
        self.inline_formatting_context_builder
            .get_or_insert_with(|| {
                let mut builder = InlineFormattingContextBuilder::new(self.info);
                for shared_inline_styles in self.display_contents_shared_styles.iter() {
                    builder.enter_display_contents(shared_inline_styles.clone());
                }
                builder
            })
    }

    fn finish_ongoing_inline_formatting_context(&mut self) -> Option<InlineFormattingContext> {
        self.inline_formatting_context_builder.take()?.finish(
            self.context,
            !self.have_already_seen_first_line_for_text_indent,
            self.info.is_single_line_text_input(),
            self.info.style.to_bidi_level(),
        )
    }

    pub(crate) fn finish(mut self) -> BlockContainer {
        debug_assert!(!self.currently_processing_inline_box());

        self.finish_anonymous_table_if_needed();

        if let Some(inline_formatting_context) = self.finish_ongoing_inline_formatting_context() {
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
        let inline_table = self.currently_processing_inline_box();

        let contents: Vec<AnonymousTableContent<'dom>> =
            self.anonymous_table_content.drain(..).collect();
        let last_text = match contents.last() {
            Some(AnonymousTableContent::Text(info, text)) => Some((info.clone(), text.clone())),
            _ => None,
        };

        let (table_info, ifc) =
            Table::construct_anonymous(self.context, self.info, contents, self.propagated_data);

        if inline_table {
            self.ensure_inline_formatting_context_builder()
                .push_atomic(|| ArcRefCell::new(ifc), None);
        } else {
            let table_block = ArcRefCell::new(BlockLevelBox::Independent(ifc));

            if let Some(inline_formatting_context) = self.finish_ongoing_inline_formatting_context()
            {
                self.push_block_level_job_for_inline_formatting_context(inline_formatting_context);
            }

            let box_slot = table_info
                .node
                .pseudo_element_box_slot(PseudoElement::ServoAnonymousTable);
            self.block_level_boxes.push(BlockLevelJob {
                info: table_info,
                box_slot,
                kind: BlockLevelCreator::AnonymousTable { table_block },
                propagated_data: self.propagated_data,
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

impl<'dom> TraversalHandler<'dom> for BlockContainerBuilder<'dom, '_> {
    fn handle_element(
        &mut self,
        info: &NodeAndStyleInfo<'dom>,
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

    fn handle_text(&mut self, info: &NodeAndStyleInfo<'dom>, text: Cow<'dom, str>) {
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

        self.ensure_inline_formatting_context_builder()
            .push_text(text, info);
    }

    fn enter_display_contents(&mut self, styles: SharedInlineStyles) {
        self.display_contents_shared_styles.push(styles.clone());
        if let Some(builder) = self.inline_formatting_context_builder.as_mut() {
            builder.enter_display_contents(styles);
        }
    }

    fn leave_display_contents(&mut self) {
        self.display_contents_shared_styles.pop();
        if let Some(builder) = self.inline_formatting_context_builder.as_mut() {
            builder.leave_display_contents();
        }
    }
}

impl<'dom> BlockContainerBuilder<'dom, '_> {
    fn handle_list_item_marker_inside(
        &mut self,
        marker_info: &NodeAndStyleInfo<'dom>,
        container_info: &NodeAndStyleInfo<'dom>,
        contents: Vec<crate::dom_traversal::PseudoElementContentItem>,
    ) {
        // TODO: We do not currently support saving box slots for ::marker pseudo-elements
        // that are part nested in ::before and ::after pseudo elements. For now, just
        // forget about them once they are built.
        let box_slot = match container_info.pseudo_element_type {
            Some(_) => BoxSlot::dummy(),
            None => marker_info
                .node
                .pseudo_element_box_slot(PseudoElement::Marker),
        };

        self.handle_inline_level_element(
            marker_info,
            DisplayInside::Flow {
                is_list_item: false,
            },
            NonReplacedContents::OfPseudoElement(contents).into(),
            box_slot,
        );
    }

    fn handle_list_item_marker_outside(
        &mut self,
        marker_info: &NodeAndStyleInfo<'dom>,
        container_info: &NodeAndStyleInfo<'dom>,
        contents: Vec<crate::dom_traversal::PseudoElementContentItem>,
        list_item_style: Arc<ComputedValues>,
    ) {
        // TODO: We do not currently support saving box slots for ::marker pseudo-elements
        // that are part nested in ::before and ::after pseudo elements. For now, just
        // forget about them once they are built.
        let box_slot = match container_info.pseudo_element_type {
            Some(_) => BoxSlot::dummy(),
            None => marker_info
                .node
                .pseudo_element_box_slot(PseudoElement::Marker),
        };

        self.block_level_boxes.push(BlockLevelJob {
            info: marker_info.clone(),
            box_slot,
            kind: BlockLevelCreator::OutsideMarker {
                contents,
                list_item_style,
            },
            propagated_data: self.propagated_data,
        });
    }

    fn handle_inline_level_element(
        &mut self,
        info: &NodeAndStyleInfo<'dom>,
        display_inside: DisplayInside,
        contents: Contents,
        box_slot: BoxSlot<'dom>,
    ) {
        let old_layout_box = box_slot.take_layout_box_if_undamaged(info.damage);
        let (is_list_item, non_replaced_contents) = match (display_inside, contents) {
            (
                DisplayInside::Flow { is_list_item },
                Contents::NonReplaced(non_replaced_contents),
            ) => (is_list_item, non_replaced_contents),
            (_, contents) => {
                // If this inline element is an atomic, handle it and return.
                let context = self.context;
                let propagated_data = self.propagated_data;

                let construction_callback = || {
                    ArcRefCell::new(IndependentFormattingContext::construct(
                        context,
                        info,
                        display_inside,
                        contents,
                        propagated_data,
                    ))
                };

                let atomic = self
                    .ensure_inline_formatting_context_builder()
                    .push_atomic(construction_callback, old_layout_box);
                box_slot.set(LayoutBox::InlineLevel(vec![atomic]));
                return;
            },
        };

        // Otherwise, this is just a normal inline box. Whatever happened before, all we need to do
        // before recurring is to remember this ongoing inline level box.
        self.ensure_inline_formatting_context_builder()
            .start_inline_box(
                || ArcRefCell::new(InlineBox::new(info)),
                None,
                old_layout_box,
            );

        if is_list_item {
            if let Some((marker_info, marker_contents)) =
                crate::lists::make_marker(self.context, info)
            {
                // Ignore `list-style-position` here:
                // “If the list item is an inline box: this value is equivalent to `inside`.”
                // https://drafts.csswg.org/css-lists/#list-style-position-outside
                self.handle_list_item_marker_inside(&marker_info, info, marker_contents)
            }
        }

        // `unwrap` doesn’t panic here because `is_replaced` returned `false`.
        non_replaced_contents.traverse(self.context, info, self);

        self.finish_anonymous_table_if_needed();

        // As we are ending this inline box, during the course of the `traverse()` above, the ongoing
        // inline formatting context may have been split around block-level elements. In that case,
        // more than a single inline box tree item may have been produced for this inline-level box.
        // `InlineFormattingContextBuilder::end_inline_box()` is returning all of those box tree
        // items.
        box_slot.set(LayoutBox::InlineLevel(
            self.inline_formatting_context_builder
                .as_mut()
                .expect("Should be building an InlineFormattingContext")
                .end_inline_box(),
        ));
    }

    fn handle_block_level_element(
        &mut self,
        info: &NodeAndStyleInfo<'dom>,
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
            .as_mut()
            .and_then(|builder| {
                builder.split_around_block_and_finish(
                    self.context,
                    !self.have_already_seen_first_line_for_text_indent,
                    self.info.style.to_bidi_level(),
                )
            })
        {
            self.push_block_level_job_for_inline_formatting_context(inline_formatting_context);
        }

        let propagated_data = self.propagated_data;
        let kind = match contents {
            Contents::NonReplaced(contents) => match display_inside {
                DisplayInside::Flow { is_list_item }
                    // Fragment flags are just used to indicate that the element is not replaced, so empty
                    // flags are okay here.
                    if !info.style.establishes_block_formatting_context(
                        FragmentFlags::empty()
                    ) =>
                {
                    BlockLevelCreator::SameFormattingContextBlock(
                        IntermediateBlockContainer::Deferred {
                            contents,
                            propagated_data,
                            is_list_item,
                        },
                    )
                },
                _ => BlockLevelCreator::Independent {
                    display_inside,
                    contents: contents.into(),
                },
            },
            Contents::Replaced(contents) => {
                let contents = Contents::Replaced(contents);
                BlockLevelCreator::Independent {
                    display_inside,
                    contents,
                }
            },
        };
        self.block_level_boxes.push(BlockLevelJob {
            info: info.clone(),
            box_slot,
            kind,
            propagated_data,
        });

        // Any block also counts as the first line for the purposes of text indent. Even if
        // they don't actually indent.
        self.have_already_seen_first_line_for_text_indent = true;
    }

    fn handle_absolutely_positioned_element(
        &mut self,
        info: &NodeAndStyleInfo<'dom>,
        display_inside: DisplayInside,
        contents: Contents,
        box_slot: BoxSlot<'dom>,
    ) {
        if let Some(builder) = self.inline_formatting_context_builder.as_mut() {
            if !builder.is_empty() {
                let constructor = || {
                    ArcRefCell::new(AbsolutelyPositionedBox::construct(
                        self.context,
                        info,
                        display_inside,
                        contents,
                    ))
                };
                let old_layout_box = box_slot.take_layout_box_if_undamaged(info.damage);
                let inline_level_box =
                    builder.push_absolutely_positioned_box(constructor, old_layout_box);
                box_slot.set(LayoutBox::InlineLevel(vec![inline_level_box]));
                return;
            }
        }

        let kind = BlockLevelCreator::OutOfFlowAbsolutelyPositionedBox {
            contents,
            display_inside,
        };
        self.block_level_boxes.push(BlockLevelJob {
            info: info.clone(),
            box_slot,
            kind,
            propagated_data: self.propagated_data,
        });
    }

    fn handle_float_element(
        &mut self,
        info: &NodeAndStyleInfo<'dom>,
        display_inside: DisplayInside,
        contents: Contents,
        box_slot: BoxSlot<'dom>,
    ) {
        if let Some(builder) = self.inline_formatting_context_builder.as_mut() {
            if !builder.is_empty() {
                let constructor = || {
                    ArcRefCell::new(FloatBox::construct(
                        self.context,
                        info,
                        display_inside,
                        contents,
                        self.propagated_data,
                    ))
                };
                let old_layout_box = box_slot.take_layout_box_if_undamaged(info.damage);
                let inline_level_box = builder.push_float_box(constructor, old_layout_box);
                box_slot.set(LayoutBox::InlineLevel(vec![inline_level_box]));
                return;
            }
        }

        let kind = BlockLevelCreator::OutOfFlowFloatBox {
            contents,
            display_inside,
        };
        self.block_level_boxes.push(BlockLevelJob {
            info: info.clone(),
            box_slot,
            kind,
            propagated_data: self.propagated_data,
        });
    }

    fn push_block_level_job_for_inline_formatting_context(
        &mut self,
        inline_formatting_context: InlineFormattingContext,
    ) {
        let layout_context = self.context;
        let info = self
            .anonymous_box_info
            .get_or_insert_with(|| {
                self.info
                    .pseudo(layout_context, PseudoElement::ServoAnonymousBox)
                    .expect("Should never fail to create anonymous box")
            })
            .clone();

        let box_slot = self
            .info
            .node
            .pseudo_element_box_slot(PseudoElement::ServoAnonymousBox);
        self.block_level_boxes.push(BlockLevelJob {
            info,
            box_slot,
            kind: BlockLevelCreator::SameFormattingContextBlock(
                IntermediateBlockContainer::InlineFormattingContext(
                    BlockContainer::InlineFormattingContext(inline_formatting_context),
                ),
            ),
            propagated_data: self.propagated_data,
        });

        self.have_already_seen_first_line_for_text_indent = true;
    }
}

impl BlockLevelJob<'_> {
    fn finish(self, context: &LayoutContext) -> ArcRefCell<BlockLevelBox> {
        let info = &self.info;

        // If this `BlockLevelBox` is undamaged and it has been laid out before, reuse
        // the old one, while being sure to clear the layout cache.
        if !info.damage.has_box_damage() {
            if let Some(block_level_box) = match self.box_slot.slot.as_ref() {
                Some(box_slot) => match &*box_slot.borrow() {
                    Some(LayoutBox::BlockLevel(block_level_box)) => Some(block_level_box.clone()),
                    _ => None,
                },
                None => None,
            } {
                return block_level_box;
            }
        }

        let block_level_box = match self.kind {
            BlockLevelCreator::SameFormattingContextBlock(intermediate_block_container) => {
                let contents = intermediate_block_container.finish(context, info);
                let contains_floats = contents.contains_floats();
                ArcRefCell::new(BlockLevelBox::SameFormattingContextBlock {
                    base: LayoutBoxBase::new(info.into(), info.style.clone()),
                    contents,
                    contains_floats,
                })
            },
            BlockLevelCreator::Independent {
                display_inside,
                contents,
            } => {
                let context = IndependentFormattingContext::construct(
                    context,
                    info,
                    display_inside,
                    contents,
                    self.propagated_data,
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
                self.propagated_data,
            ))),
            BlockLevelCreator::OutsideMarker {
                contents,
                list_item_style,
            } => {
                let contents = NonReplacedContents::OfPseudoElement(contents);
                let block_container = BlockContainer::construct(
                    context,
                    info,
                    contents,
                    self.propagated_data,
                    false, /* is_list_item */
                );
                // An outside ::marker must establish a BFC, and can't contain floats.
                let block_formatting_context = BlockFormattingContext {
                    contents: block_container,
                    contains_floats: false,
                };
                ArcRefCell::new(BlockLevelBox::OutsideMarker(OutsideMarker {
                    base: LayoutBoxBase::new(info.into(), info.style.clone()),
                    block_formatting_context,
                    list_item_style,
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
    fn finish(self, context: &LayoutContext, info: &NodeAndStyleInfo<'_>) -> BlockContainer {
        match self {
            IntermediateBlockContainer::Deferred {
                contents,
                propagated_data,
                is_list_item,
            } => BlockContainer::construct(context, info, contents, propagated_data, is_list_item),
            IntermediateBlockContainer::InlineFormattingContext(block_container) => block_container,
        }
    }
}
