/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::{Borrow, BorrowMut};
use std::cell::Cell;

use app_units::Au;
use atomic_refcell::{AtomicRefCell, AtomicRefMut};
use style::properties::longhands::align_content::computed_value::T as AlignContent;
use style::properties::longhands::align_items::computed_value::T as AlignItems;
use style::properties::longhands::align_self::computed_value::T as AlignSelf;
use style::properties::longhands::box_sizing::computed_value::T as BoxSizing;
use style::properties::longhands::flex_direction::computed_value::T as FlexDirection;
use style::properties::longhands::flex_wrap::computed_value::T as FlexWrap;
use style::properties::longhands::justify_content::computed_value::T as JustifyContent;
use style::values::computed::length::Size;
use style::values::computed::Length;
use style::values::generics::flex::GenericFlexBasis as FlexBasis;
use style::values::generics::length::LengthPercentageOrAuto;
use style::values::CSSFloat;
use style::Zero;
use taffy::CoreStyle;

use super::geom::{
    FlexAxis, FlexRelativeRect, FlexRelativeSides, FlexRelativeVec2, MainStartCrossStart,
};
use super::{FlexContainer, FlexLevelBox, FlexLevelBoxInner};
use crate::cell::ArcRefCell;
use crate::context::LayoutContext;
use crate::formatting_contexts::{
    Baselines, IndependentFormattingContext, IndependentLayout, ReplacedFormattingContext,
};
use crate::fragment_tree::{BoxFragment, CollapsedBlockMargins, Fragment};
use crate::geom::{AuOrAuto, LengthOrAuto, LogicalRect, LogicalSides, LogicalVec2};
use crate::positioned::{AbsolutelyPositionedBox, PositioningContext, PositioningContextLength};
use crate::sizing::ContentSizes;
use crate::style_ext::{Clamp, ComputedValuesExt};
use crate::ContainingBlock;

use servo_arc::Arc;
use taffy_stylo::{TaffyStyloStyle, TaffyStyloStyleRef};

// FIMXE: “Flex items […] `z-index` values other than `auto` create a stacking context
// even if `position` is `static` (behaving exactly as if `position` were `relative`).”
// https://drafts.csswg.org/css-flexbox/#painting
// (likely in `display_list/stacking_context.rs`)

fn resolve_content_size(constraint: taffy::AvailableSpace, content_sizes: ContentSizes) -> f32 {
    match constraint {
        taffy::AvailableSpace::Definite(limit) => {
            let min = content_sizes.min_content.to_f32_px();
            let max = content_sizes.max_content.to_f32_px();
            limit.min(max).max(min)
        },
        taffy::AvailableSpace::MinContent => content_sizes.min_content.to_f32_px(),
        taffy::AvailableSpace::MaxContent => content_sizes.max_content.to_f32_px(),
    }
}

#[inline(always)]
fn with_independant_formatting_context<T>(
    item: &mut FlexLevelBoxInner,
    cb: impl FnOnce(&mut IndependentFormattingContext) -> T,
) -> T {
    match item {
        FlexLevelBoxInner::FlexItem(ref mut context) => cb(context),
        FlexLevelBoxInner::OutOfFlowAbsolutelyPositionedBox(ref abspos_box) => {
            let mut abspos_box = AtomicRefCell::borrow_mut(abspos_box);
            cb(&mut abspos_box.context)
        },
    }
}

fn measure_replace_box(replaced: &ReplacedFormattingContext, containing_block: &ContainingBlock) {}

/// Layout parameters and intermediate results about a flex container,
/// grouped to avoid passing around many parameters
struct FlexContext<'a> {
    source_child_nodes: &'a [ArcRefCell<FlexLevelBox>],
    layout_context: &'a LayoutContext<'a>,
    positioning_context: &'a mut PositioningContext,
    // For items. Style is on containing_block
    containing_block: &'a ContainingBlock<'a>,
    // container_is_single_line: bool,
    // container_min_cross_size: Length,
    // container_max_cross_size: Option<Length>,
    // flex_axis: FlexAxis,
    // flex_direction_is_reversed: bool,
    // flex_wrap_reverse: bool,
    // main_start_cross_start_sides_are: MainStartCrossStart,
    // container_definite_inner_size: FlexRelativeVec2<Option<Length>>,
    // align_content: AlignContent,
    // align_items: AlignItems,
    // justify_content: JustifyContent,
}

struct ChildIter(std::ops::Range<usize>);
impl Iterator for ChildIter {
    type Item = taffy::NodeId;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|idx| taffy::NodeId::from(idx))
    }
}

impl taffy::TraversePartialTree for FlexContext<'_> {
    type ChildIter<'a> = ChildIter where Self: 'a;

    fn child_ids(&self, _node_id: taffy::NodeId) -> Self::ChildIter<'_> {
        ChildIter(0..self.source_child_nodes.len())
    }

    fn child_count(&self, _node_id: taffy::NodeId) -> usize {
        self.source_child_nodes.len()
    }

    fn get_child_id(&self, _node_id: taffy::NodeId, index: usize) -> taffy::NodeId {
        taffy::NodeId::from(index)
    }
}

impl taffy::LayoutPartialTree for FlexContext<'_> {
    type CoreContainerStyle<'a> = TaffyStyloStyleRef<'a> where Self: 'a;
    type CacheMut<'b> = AtomicRefMut<'b, taffy::Cache> where Self: 'b;

    fn get_core_container_style(&self, node_id: taffy::NodeId) -> Self::CoreContainerStyle<'_> {
        TaffyStyloStyleRef(self.containing_block.style)
    }

    fn set_unrounded_layout(&mut self, node_id: taffy::NodeId, layout: &taffy::Layout) {
        let id = usize::from(node_id);
        (*self.source_child_nodes[id]).borrow_mut().taffy_layout = *layout;
    }

    fn get_cache_mut(&mut self, node_id: taffy::NodeId) -> AtomicRefMut<'_, taffy::Cache> {
        let id = usize::from(node_id);
        let mut_ref: AtomicRefMut<'_, _> = (*self.source_child_nodes[id]).borrow_mut();
        AtomicRefMut::map(mut_ref, |node| &mut node.taffy_layout_cache)
    }

    fn compute_child_layout(
        &mut self,
        node_id: taffy::NodeId,
        inputs: taffy::LayoutInput,
    ) -> taffy::LayoutOutput {
        // compute_cached_layout(self, node_id, inputs, |parent, node_id, inputs| {
        let mut child = (*self.source_child_nodes[usize::from(node_id)]).borrow_mut();

        with_independant_formatting_context(
            &mut child.flex_level_box,
            |independent_context| -> taffy::LayoutOutput {
                let logical_size = match independent_context {
                    IndependentFormattingContext::Replaced(replaced) => {
                        replaced.contents.used_size_as_if_inline_element(
                            &self.containing_block,
                            &replaced.style,
                            None, //box_size,
                            &replaced.style.padding_border_margin(&self.containing_block),
                        )
                    },

                    // TODO: better handling of flexbox items (which can't precompute inline sizes)
                    IndependentFormattingContext::NonReplaced(non_replaced) => {
                        // Compute inline size
                        let inline_sizes = non_replaced.inline_content_sizes(&self.layout_context);
                        let inline_size =
                            resolve_content_size(inputs.available_space.width, inline_sizes);

                        let block_size = match inputs.axis {
                            // Height will be ignored for `RequestedAxis::Horizontal` so we simply return 0.0 as a placeholder value
                            taffy::RequestedAxis::Horizontal => 0.0,
                            // If we actually need a height then we run layout to compute it
                            taffy::RequestedAxis::Vertical | taffy::RequestedAxis::Both => {
                                let containing_block_for_children = ContainingBlock {
                                    inline_size: Au::from_f32_px(inline_size),
                                    block_size: LengthPercentageOrAuto::Auto,
                                    style: &self.containing_block.style,
                                };
                                let mut positioning_context = PositioningContext::new_for_subtree(
                                    self.positioning_context
                                        .collects_for_nearest_positioned_ancestor(),
                                );
                                let layout = non_replaced.layout(
                                    &self.layout_context,
                                    &mut positioning_context,
                                    &containing_block_for_children,
                                    &self.containing_block,
                                );

                                layout.content_block_size.to_f32_px()
                            },
                        };

                        LogicalVec2 {
                            inline: Au::from_f32_px(inline_size),
                            block: Au::from_f32_px(block_size),
                        }
                    },
                };

                let size = inputs.known_dimensions.unwrap_or(taffy::Size {
                    width: logical_size.inline.to_f32_px(),
                    height: logical_size.block.to_f32_px(),
                });

                taffy::LayoutOutput {
                    size,
                    content_size: size,
                    ..taffy::LayoutOutput::DEFAULT
                }
            },
        )

        // })
    }
}

impl taffy::LayoutFlexboxContainer for FlexContext<'_> {
    type ContainerStyle<'a> = TaffyStyloStyleRef<'a>
    where
        Self: 'a;

    type ItemStyle<'a> = TaffyStyloStyle
    where
        Self: 'a;

    fn get_flexbox_container_style(
        &self,
        node_id: taffy::prelude::NodeId,
    ) -> Self::ContainerStyle<'_> {
        TaffyStyloStyleRef(self.containing_block.style)
    }

    // TODO: Make a RefCell variant of TaffyStyloStyle to avoid the Arc clone here
    fn get_flexbox_child_style(
        &self,
        child_node_id: taffy::prelude::NodeId,
    ) -> Self::ItemStyle<'_> {
        let id = usize::from(child_node_id);
        let child = (*self.source_child_nodes[id]).borrow();
        let style = child.style.clone();
        TaffyStyloStyle(style)
    }
}

/// Child of a FlexContainer. Can either be absolutely positioned, or not. If not,
/// a placeholder is used and flex content is stored outside of this enum.
enum FlexContent {
    AbsolutelyPositionedBox(ArcRefCell<AbsolutelyPositionedBox>),
    FlexItemPlaceholder,
}

impl FlexContainer {
    pub fn inline_content_sizes(&self) -> ContentSizes {
        // FIXME: implement this. The spec for it is the same as for "normal" layout:
        // https://drafts.csswg.org/css-flexbox/#layout-algorithm
        // … except that the parts that say “the flex container is being sized
        // under a min or max-content constraint” apply.
        ContentSizes::zero() // Return an incorrect result rather than panic
    }

    /// <https://drafts.csswg.org/css-flexbox/#layout-algorithm>
    pub(crate) fn layout(
        &self,
        layout_context: &LayoutContext,
        positioning_context: &mut PositioningContext,
        containing_block: &ContainingBlock,
    ) -> IndependentLayout {
        // Actual length may be less, but we guess that usually not by a lot
        let mut flex_items = Vec::with_capacity(self.children.len());

        // Absolutely-positioned children of the flex container may be interleaved
        // with flex items. We need to preserve their relative order for correct painting order,
        // which is the order of `Fragment`s in this function’s return value.
        //
        // Example:
        // absolutely_positioned_items_with_original_order = [Some(item), Some(item), None, Some(item), None]
        // flex_items                                      =                         [item,             item]
        let absolutely_positioned_items_with_original_order = self
            .children
            .iter()
            .map(|arcrefcell| {
                let borrowed = (**arcrefcell).borrow_mut();
                match borrowed.flex_level_box {
                    FlexLevelBoxInner::OutOfFlowAbsolutelyPositionedBox(absolutely_positioned) => {
                        FlexContent::AbsolutelyPositionedBox(absolutely_positioned.clone())
                    },
                    FlexLevelBoxInner::FlexItem(_) => {
                        let item =
                            AtomicRefMut::map(borrowed, |child| match child.flex_level_box {
                                FlexLevelBoxInner::FlexItem(ref mut item) => item,
                                _ => unreachable!(),
                            });
                        flex_items.push(item);
                        FlexContent::FlexItemPlaceholder
                    },
                }
            })
            .collect::<Vec<_>>();

        let style = containing_block.style;

        let mut flex_context = FlexContext {
            layout_context,
            positioning_context,
            containing_block,
            source_child_nodes: &self.children,
            // container_min_cross_size,
            // container_max_cross_size,
            // container_is_single_line,
            // flex_axis,
            // flex_direction_is_reversed,
            // flex_wrap_reverse,
            // align_content,
            // align_items,
            // justify_content,
            // main_start_cross_start_sides_are: MainStartCrossStart::from(
            //     flex_direction,
            //     flex_wrap_reverse,
            // ),
            // // https://drafts.csswg.org/css-flexbox/#definite-sizes
            // container_definite_inner_size: flex_axis.vec2_to_flex_relative(LogicalVec2 {
            //     inline: Some(containing_block.inline_size.into()),
            //     block: containing_block.block_size.non_auto().map(|t| t.into()),
            // }),
        };

        taffy::compute_flexbox_layout(
            &mut flex_context,
            taffy::NodeId::from(usize::MAX),
            taffy::LayoutInput {
                run_mode: taffy::RunMode::PerformLayout,
                sizing_mode: taffy::SizingMode::InherentSize,
                axis: todo!(),
                known_dimensions: todo!(),
                parent_size: todo!(),
                available_space: todo!(),
                vertical_margins_are_collapsible: todo!(),
            },
        );

        let fragments = absolutely_positioned_items_with_original_order
            .into_iter()
            .map(|child_as_abspos| match child_as_abspos {
                FlexContent::AbsolutelyPositionedBox(absolutely_positioned) => {
                    let hoisted_box = AbsolutelyPositionedBox::to_hoisted(
                        absolutely_positioned,
                        LogicalVec2::zero(),
                        containing_block,
                    );
                    let hoisted_fragment = hoisted_box.fragment.clone();
                    positioning_context.push(hoisted_box);
                    Fragment::AbsoluteOrFixedPositioned(hoisted_fragment)
                },
                FlexContent::FlexItemPlaceholder => {
                    // The `flex_item_fragments` iterator yields one fragment
                    // per flex item, in the original order.
                    let (box_fragment, mut child_positioning_context) =
                        flex_item_fragments.next().unwrap();
                    let fragment = Fragment::Box(box_fragment);
                    child_positioning_context.adjust_static_position_of_hoisted_fragments(
                        &fragment,
                        PositioningContextLength::zero(),
                    );
                    positioning_context.append(child_positioning_context);
                    fragment
                },
            })
            .collect::<Vec<_>>();

        IndependentLayout {
            fragments,
            content_block_size: content_block_size.into(),
            content_inline_size_for_table: None,
            baselines: Baselines::default(),
        }
    }
}
