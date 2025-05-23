/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::Cow;
use std::convert::TryFrom;
use std::mem;

use atomic_refcell::AtomicRef;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use script::layout_dom::ServoLayoutNode;
use script_layout_interface::wrapper_traits::LayoutNode;
use servo_arc::Arc;
use style::properties::ComputedValues;
use style::properties::longhands::list_style_position::computed_value::T as ListStylePosition;
use style::selector_parser::{PseudoElement, RestyleDamage};
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

    pub(crate) fn repair(
        &mut self,
        context: &LayoutContext,
        info: &NodeAndStyleInfo<'_>,
        contents: NonReplacedContents,
        propagated_data: PropagatedBoxTreeData,
        is_list_item: bool,
    ) {
        self.contents
            .repair(context, info, contents, propagated_data, is_list_item);
        self.contains_floats = self.contents.contains_floats();
    }
}

enum BlockLevelJob<'dom> {
    Copy(ArcRefCell<BlockLevelBox>),
    Repair(BlockLevelRepairJob<'dom>),
    Create(BlockLevelCreateJob<'dom>),
}

struct BlockLevelRepairJob<'dom> {
    info: NodeAndStyleInfo<'dom>,
    repaired_box: ArcRefCell<BlockLevelBox>,
    propagated_data: PropagatedBoxTreeData,
    display_inside: DisplayInside,
    contents: Contents,
}

struct BlockLevelCreateJob<'dom> {
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

/// Some childern node of a given DOM node do not have `REBUILD_BOX` restyle damage.
/// Thus, their boxes do not need to be rebuilt and can be retrieved from the
/// previously built children boxes of the given DOM node's repaired block container.
///
/// This helper struct used to try to find the matched boxes in previously built
/// children boxes for a given children node. For block level node, this helper
/// is not very useful. It is more useful to find the inline formatting context
/// that the inline level node participates in, and the anonymous box that wraps
/// the inline formatting context.
struct PreviouslyBuiltChildernMatcher {
    block_level_boxes: Vec<ArcRefCell<BlockLevelBox>>,
    block_level_boxes_cursor: usize,
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

    /// The list of block-level boxes to be repaired or copied from previous
    /// block container building result, or newly built, for the final block
    /// container.
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

    /// The [`PreviouslyBuiltChildrenMatcher`] if we need to repair the current block
    /// container, otherwise None.
    previously_built_children_matcher: Option<PreviouslyBuiltChildernMatcher>,
}

impl BlockContainer {
    pub fn construct(
        context: &LayoutContext,
        info: &NodeAndStyleInfo<'_>,
        contents: NonReplacedContents,
        propagated_data: PropagatedBoxTreeData,
        is_list_item: bool,
    ) -> BlockContainer {
        let builder = BlockContainerBuilder::new(context, info, propagated_data);
        builder.build(contents, is_list_item)
    }

    pub fn repair(
        &mut self,
        context: &LayoutContext,
        info: &NodeAndStyleInfo<'_>,
        contents: NonReplacedContents,
        propagated_data: PropagatedBoxTreeData,
        is_list_item: bool,
    ) {
        let builder = BlockContainerBuilder::new_for_repair(context, info, propagated_data, self);
        let _ = mem::replace(self, builder.build(contents, is_list_item));
    }
}

impl PreviouslyBuiltChildernMatcher {
    fn new(block_container: &mut BlockContainer) -> Self {
        let previously_built_block_level_boxes = match block_container {
            BlockContainer::BlockLevelBoxes(boxes) => mem::take(boxes),
            BlockContainer::InlineFormattingContext(_) => vec![],
        };

        Self {
            block_level_boxes: previously_built_block_level_boxes,
            block_level_boxes_cursor: 0,
        }
    }

    fn match_and_advance(
        &mut self,
        info: &NodeAndStyleInfo<'_>,
    ) -> Option<ArcRefCell<BlockLevelBox>> {
        if info.is_anonymous() {
            return self.find_first_anonymous_box_and_advance();
        }

        let data = info.node.layout_data_mut();
        let layout_box = data.for_pseudo(info.pseudo_element_type);

        let previously_bound_box = match &*AtomicRef::filter_map(layout_box, Option::as_ref)? {
            LayoutBox::BlockLevel(block_level_box) => block_level_box.clone(),
            _ => return None,
        };

        // Skip the mismatch boxes, which were produced by the removed node or
        // the node has `REBUILD_BOX` restyle damage, to find the box produced by
        // current given node.
        loop {
            let curr = self.next()?;
            if ArcRefCell::ptr_eq(&curr, &previously_bound_box) {
                return Some(curr);
            }
        }
    }

    fn find_first_anonymous_box_and_advance(&mut self) -> Option<ArcRefCell<BlockLevelBox>> {
        unreachable!("Unexpected situations, and unimplemented");
    }

    fn next(&mut self) -> Option<ArcRefCell<BlockLevelBox>> {
        if self.block_level_boxes_cursor < self.block_level_boxes.len() {
            self.block_level_boxes_cursor += 1;
            return Some(self.block_level_boxes[self.block_level_boxes_cursor - 1].clone());
        }

        None
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
            previously_built_children_matcher: None,
        }
    }

    fn new_for_repair(
        context: &'style LayoutContext,
        info: &'style NodeAndStyleInfo<'dom>,
        propagated_data: PropagatedBoxTreeData,
        repaired_block_container: &mut BlockContainer,
    ) -> Self {
        let mut builder = Self::new(context, info, propagated_data);
        builder.previously_built_children_matcher = Some(PreviouslyBuiltChildernMatcher::new(
            repaired_block_container,
        ));
        builder
    }

    fn build(mut self, contents: NonReplacedContents, is_list_item: bool) -> BlockContainer {
        if is_list_item {
            if let Some((marker_info, marker_contents)) =
                crate::lists::make_marker(self.context, self.info)
            {
                match marker_info.style.clone_list_style_position() {
                    ListStylePosition::Inside => self.handle_list_item_marker_inside(
                        &marker_info,
                        self.info,
                        marker_contents,
                    ),
                    ListStylePosition::Outside => self.handle_list_item_marker_outside(
                        &marker_info,
                        self.info,
                        marker_contents,
                        self.info.style.clone(),
                    ),
                }
            }
        }

        contents.traverse(self.context, self.info, &mut self);
        self.finish()
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
            self.info.style.writing_mode.to_bidi_level(),
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
                .push_atomic(ifc);
        } else {
            let table_block = ArcRefCell::new(BlockLevelBox::Independent(ifc));

            if let Some(inline_formatting_context) = self.finish_ongoing_inline_formatting_context()
            {
                self.push_block_level_job_for_inline_formatting_context(inline_formatting_context);
            }

            self.block_level_boxes
                .push(BlockLevelJob::Create(BlockLevelCreateJob {
                    info: table_info,
                    box_slot: BoxSlot::dummy(),
                    kind: BlockLevelCreator::AnonymousTable { table_block },
                    propagated_data: self.propagated_data,
                }));
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

    fn need_clear_pseudo_element_box(&self, node: &ServoLayoutNode<'dom>) -> bool {
        let damage = node.style_data().unwrap().element_data.borrow().damage;
        if damage.contains(RestyleDamage::REBUILD_BOX) {
            return true;
        }

        false
    }
}

impl<'dom> BlockContainerBuilder<'dom, '_> {
    fn try_reuse_block_level_box(
        &mut self,
        info: &NodeAndStyleInfo<'dom>,
    ) -> Option<ArcRefCell<BlockLevelBox>> {
        let previously_built_children_matcher = self.previously_built_children_matcher.as_mut()?;

        if info
            .get_restyle_damage()
            .contains(RestyleDamage::REBUILD_BOX)
        {
            return None;
        }

        previously_built_children_matcher.match_and_advance(info)
    }

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
        // always create new box for it.
        if container_info.pseudo_element_type.is_some() {
            self.create_list_item_marker_outside(
                marker_info,
                container_info,
                contents,
                list_item_style,
            );
            return;
        }

        let Some(reused_block_level_box) = self.try_reuse_block_level_box(marker_info) else {
            self.create_list_item_marker_outside(
                marker_info,
                container_info,
                contents,
                list_item_style,
            );
            return;
        };

        if marker_info
            .get_restyle_damage()
            .contains(RestyleDamage::REPAIR_BOX)
        {
            self.block_level_boxes
                .push(BlockLevelJob::Repair(BlockLevelRepairJob {
                    info: marker_info.clone(),
                    repaired_box: reused_block_level_box,
                    propagated_data: self.propagated_data,
                    contents: NonReplacedContents::OfPseudoElement(contents).into(),
                    display_inside: DisplayInside::Flow {
                        is_list_item: false,
                    },
                }));
            return;
        }

        self.block_level_boxes
            .push(BlockLevelJob::Copy(reused_block_level_box));
    }

    fn create_list_item_marker_outside(
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

        self.block_level_boxes
            .push(BlockLevelJob::Create(BlockLevelCreateJob {
                info: marker_info.clone(),
                box_slot,
                kind: BlockLevelCreator::OutsideMarker {
                    contents,
                    list_item_style,
                },
                propagated_data: self.propagated_data,
            }));
    }

    fn handle_inline_level_element(
        &mut self,
        info: &NodeAndStyleInfo<'dom>,
        display_inside: DisplayInside,
        contents: Contents,
        box_slot: BoxSlot<'dom>,
    ) {
        let propagated_data = self.propagated_data;
        let (DisplayInside::Flow { is_list_item }, false) =
            (display_inside, contents.is_replaced())
        else {
            // If this inline element is an atomic, handle it and return.
            let context = self.context;
            let atomic = self.ensure_inline_formatting_context_builder().push_atomic(
                IndependentFormattingContext::construct(
                    context,
                    info,
                    display_inside,
                    contents,
                    propagated_data,
                ),
            );
            box_slot.set(LayoutBox::InlineLevel(vec![atomic]));
            return;
        };

        // Otherwise, this is just a normal inline box. Whatever happened before, all we need to do
        // before recurring is to remember this ongoing inline level box.
        self.ensure_inline_formatting_context_builder()
            .start_inline_box(InlineBox::new(info), None);

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
        NonReplacedContents::try_from(contents)
            .unwrap()
            .traverse(self.context, info, self);

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
                    self.info.style.writing_mode.to_bidi_level(),
                )
            })
        {
            self.push_block_level_job_for_inline_formatting_context(inline_formatting_context);
        }

        if let Some(reused_block_level_box) = self.try_reuse_block_level_box(info) {
            if info
                .get_restyle_damage()
                .contains(RestyleDamage::REPAIR_BOX)
            {
                self.block_level_boxes
                    .push(BlockLevelJob::Repair(BlockLevelRepairJob {
                        info: info.clone(),
                        repaired_box: reused_block_level_box,
                        propagated_data: self.propagated_data,
                        contents,
                        display_inside,
                    }));
            } else {
                self.block_level_boxes
                    .push(BlockLevelJob::Copy(reused_block_level_box));
            }
        } else {
            self.create_block_level_box(info, display_inside, contents, box_slot);
        }

        // Any block also counts as the first line for the purposes of text indent. Even if
        // they don't actually indent.
        self.have_already_seen_first_line_for_text_indent = true;
    }

    fn create_block_level_box(
        &mut self,
        info: &NodeAndStyleInfo<'dom>,
        display_inside: DisplayInside,
        contents: Contents,
        box_slot: BoxSlot<'dom>,
    ) {
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
        self.block_level_boxes
            .push(BlockLevelJob::Create(BlockLevelCreateJob {
                info: info.clone(),
                box_slot,
                kind,
                propagated_data,
            }));
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
                let inline_level_box =
                    builder.push_absolutely_positioned_box(AbsolutelyPositionedBox::construct(
                        self.context,
                        info,
                        display_inside,
                        contents,
                    ));
                box_slot.set(LayoutBox::InlineLevel(vec![inline_level_box]));
                return;
            }
        }

        if let Some(reused_block_level_box) = self.try_reuse_block_level_box(info) {
            if info
                .get_restyle_damage()
                .contains(RestyleDamage::REPAIR_BOX)
            {
                self.block_level_boxes
                    .push(BlockLevelJob::Repair(BlockLevelRepairJob {
                        info: info.clone(),
                        repaired_box: reused_block_level_box,
                        propagated_data: self.propagated_data,
                        contents,
                        display_inside,
                    }));
            } else {
                self.block_level_boxes
                    .push(BlockLevelJob::Copy(reused_block_level_box));
            }
            return;
        }

        let kind = BlockLevelCreator::OutOfFlowAbsolutelyPositionedBox {
            contents,
            display_inside,
        };
        self.block_level_boxes
            .push(BlockLevelJob::Create(BlockLevelCreateJob {
                info: info.clone(),
                box_slot,
                kind,
                propagated_data: self.propagated_data,
            }));
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
                let inline_level_box = builder.push_float_box(FloatBox::construct(
                    self.context,
                    info,
                    display_inside,
                    contents,
                    self.propagated_data,
                ));
                box_slot.set(LayoutBox::InlineLevel(vec![inline_level_box]));
                return;
            }
        }

        if let Some(reused_block_level_box) = self.try_reuse_block_level_box(info) {
            if info
                .get_restyle_damage()
                .contains(RestyleDamage::REPAIR_BOX)
            {
                self.block_level_boxes
                    .push(BlockLevelJob::Repair(BlockLevelRepairJob {
                        info: info.clone(),
                        repaired_box: reused_block_level_box,
                        propagated_data: self.propagated_data,
                        contents,
                        display_inside,
                    }));
            } else {
                self.block_level_boxes
                    .push(BlockLevelJob::Copy(reused_block_level_box));
            }
            return;
        }

        let kind = BlockLevelCreator::OutOfFlowFloatBox {
            contents,
            display_inside,
        };
        self.block_level_boxes
            .push(BlockLevelJob::Create(BlockLevelCreateJob {
                info: info.clone(),
                box_slot,
                kind,
                propagated_data: self.propagated_data,
            }));
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

        self.block_level_boxes
            .push(BlockLevelJob::Create(BlockLevelCreateJob {
                info,
                // FIXME(nox): We should be storing this somewhere.
                box_slot: BoxSlot::dummy(),
                kind: BlockLevelCreator::SameFormattingContextBlock(
                    IntermediateBlockContainer::InlineFormattingContext(
                        BlockContainer::InlineFormattingContext(inline_formatting_context),
                    ),
                ),
                propagated_data: self.propagated_data,
            }));

        self.have_already_seen_first_line_for_text_indent = true;
    }
}

impl BlockLevelBox {
    pub(crate) fn repair(
        &mut self,
        context: &LayoutContext,
        info: &NodeAndStyleInfo<'_>,
        repaired_contents: Contents,
        display_inside: DisplayInside,
        propagated_data: PropagatedBoxTreeData,
    ) {
        match self {
            BlockLevelBox::SameFormattingContextBlock {
                base,
                contents,
                contains_floats,
            } => match display_inside {
                DisplayInside::Flow { is_list_item } | DisplayInside::FlowRoot { is_list_item } => {
                    contents.repair(
                        context,
                        info,
                        repaired_contents
                            .try_into()
                            .expect("Expect NonReplacedContents, but got ReplacedContents!"),
                        propagated_data,
                        is_list_item,
                    );
                    // Currently, the incremental layout is not fully ready. When a box is preserved
                    // during incremental box tree update, its cached layout result is also preserved.
                    // So, we have to invalidate all caches here to ensure that the layout is recomputed
                    // correctly.
                    base.invalidate_all_caches();
                    base.repair_style(&info.style);
                    *contains_floats = contents.contains_floats();
                },
                _ => unreachable!("Expect flow display inside"),
            },
            BlockLevelBox::Independent(independent_formatting_context) => {
                independent_formatting_context.repair(
                    context,
                    info,
                    repaired_contents,
                    display_inside,
                    propagated_data,
                )
            },
            BlockLevelBox::OutOfFlowAbsolutelyPositionedBox(positioned_box) => {
                positioned_box.borrow_mut().context.repair(
                    context,
                    info,
                    repaired_contents,
                    display_inside,
                    PropagatedBoxTreeData::default(),
                )
            },
            BlockLevelBox::OutOfFlowFloatBox(float_box) => float_box.contents.repair(
                context,
                info,
                repaired_contents,
                display_inside,
                propagated_data,
            ),
            BlockLevelBox::OutsideMarker(marker) => {
                marker.block_formatting_context.repair(
                    context,
                    info,
                    repaired_contents
                        .try_into()
                        .expect("Expect NonReplacedContents, but got ReplacedContents!"),
                    propagated_data,
                    false,
                );
                marker.repair_style(context.shared_context(), &info.node, &info.style);
                // Currently, the incremental layout is not fully ready. When a box is preserved
                // during incremental box tree update, its cached layout result is also preserved.
                // So, we have to invalidate all caches here to ensure that the layout is recomputed
                // correctly.
                marker.invalidate_all_caches();
            },
        };
    }
}

impl BlockLevelRepairJob<'_> {
    fn finish(self, context: &LayoutContext) -> ArcRefCell<BlockLevelBox> {
        self.repaired_box.borrow_mut().repair(
            context,
            &self.info,
            self.contents,
            self.display_inside,
            self.propagated_data,
        );
        self.repaired_box
    }
}

impl BlockLevelCreateJob<'_> {
    fn finish(self, context: &LayoutContext) -> ArcRefCell<BlockLevelBox> {
        let info = &self.info;
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

impl BlockLevelJob<'_> {
    fn finish(self, context: &LayoutContext) -> ArcRefCell<BlockLevelBox> {
        match self {
            BlockLevelJob::Copy(copied_box) => {
                copied_box.borrow().invalidate_subtree_caches();
                copied_box
            },
            BlockLevelJob::Repair(repair_job) => repair_job.finish(context),
            BlockLevelJob::Create(create_job) => create_job.finish(context),
        }
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
