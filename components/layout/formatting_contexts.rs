/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use app_units::Au;
use malloc_size_of_derive::MallocSizeOf;
use script::layout_dom::{ServoLayoutElement, ServoLayoutNode};
use servo_arc::Arc;
use style::context::SharedStyleContext;
use style::logical_geometry::Direction;
use style::properties::ComputedValues;
use style::selector_parser::PseudoElement;

use crate::context::LayoutContext;
use crate::dom_traversal::{Contents, NodeAndStyleInfo};
use crate::flexbox::FlexContainer;
use crate::flow::BlockFormattingContext;
use crate::fragment_tree::{BaseFragmentInfo, FragmentFlags};
use crate::geom::LazySize;
use crate::layout_box_base::{
    CacheableLayoutResult, CacheableLayoutResultAndInputs, LayoutBoxBase,
};
use crate::positioned::PositioningContext;
use crate::replaced::ReplacedContents;
use crate::sizing::{self, ComputeInlineContentSizes, ContentSizes, InlineContentSizesResult};
use crate::style_ext::{AspectRatio, DisplayInside, LayoutStyle};
use crate::table::Table;
use crate::taffy::TaffyContainer;
use crate::{
    ConstraintSpace, ContainingBlock, IndefiniteContainingBlock, LogicalVec2, PropagatedBoxTreeData,
};

/// <https://drafts.csswg.org/css-display/#independent-formatting-context>
#[derive(Debug, MallocSizeOf)]
pub(crate) struct IndependentFormattingContext {
    pub base: LayoutBoxBase,
    pub contents: IndependentFormattingContextContents,
}

#[derive(Debug, MallocSizeOf)]
pub(crate) enum IndependentFormattingContextContents {
    Replaced(ReplacedContents),
    Flow(BlockFormattingContext),
    Flex(FlexContainer),
    Grid(TaffyContainer),
    Table(Table),
    // Other layout modes go here
}

/// The baselines of a layout or a [`crate::fragment_tree::BoxFragment`]. Some layout
/// uses the first and some layout uses the last.
#[derive(Clone, Copy, Debug, Default, MallocSizeOf)]
pub(crate) struct Baselines {
    pub first: Option<Au>,
    pub last: Option<Au>,
}

impl Baselines {
    pub(crate) fn offset(&self, block_offset: Au) -> Baselines {
        Self {
            first: self.first.map(|first| first + block_offset),
            last: self.last.map(|last| last + block_offset),
        }
    }
}

impl IndependentFormattingContext {
    pub fn construct(
        context: &LayoutContext,
        node_and_style_info: &NodeAndStyleInfo,
        display_inside: DisplayInside,
        contents: Contents,
        propagated_data: PropagatedBoxTreeData,
    ) -> Self {
        let mut base_fragment_info: BaseFragmentInfo = node_and_style_info.into();

        match contents {
            Contents::NonReplaced(non_replaced_contents) => {
                let contents = match display_inside {
                    DisplayInside::Flow { is_list_item } |
                    DisplayInside::FlowRoot { is_list_item } => {
                        IndependentFormattingContextContents::Flow(
                            BlockFormattingContext::construct(
                                context,
                                node_and_style_info,
                                non_replaced_contents,
                                propagated_data,
                                is_list_item,
                            ),
                        )
                    },
                    DisplayInside::Grid => {
                        IndependentFormattingContextContents::Grid(TaffyContainer::construct(
                            context,
                            node_and_style_info,
                            non_replaced_contents,
                            propagated_data,
                        ))
                    },
                    DisplayInside::Flex => {
                        IndependentFormattingContextContents::Flex(FlexContainer::construct(
                            context,
                            node_and_style_info,
                            non_replaced_contents,
                            propagated_data,
                        ))
                    },
                    DisplayInside::Table => {
                        let table_grid_style = context
                            .style_context
                            .stylist
                            .style_for_anonymous::<ServoLayoutElement>(
                                &context.style_context.guards,
                                &PseudoElement::ServoTableGrid,
                                &node_and_style_info.style,
                            );
                        base_fragment_info.flags.insert(FragmentFlags::DO_NOT_PAINT);
                        IndependentFormattingContextContents::Table(Table::construct(
                            context,
                            node_and_style_info,
                            table_grid_style,
                            non_replaced_contents,
                            propagated_data,
                        ))
                    },
                };
                Self {
                    base: LayoutBoxBase::new(base_fragment_info, node_and_style_info.style.clone()),
                    contents,
                }
            },
            Contents::Replaced(contents) => {
                base_fragment_info.flags.insert(FragmentFlags::IS_REPLACED);
                Self {
                    base: LayoutBoxBase::new(base_fragment_info, node_and_style_info.style.clone()),
                    contents: IndependentFormattingContextContents::Replaced(contents),
                }
            },
        }
    }

    pub fn is_replaced(&self) -> bool {
        matches!(
            self.contents,
            IndependentFormattingContextContents::Replaced(_)
        )
    }

    #[inline]
    pub fn style(&self) -> &Arc<ComputedValues> {
        &self.base.style
    }

    #[inline]
    pub fn base_fragment_info(&self) -> BaseFragmentInfo {
        self.base.base_fragment_info
    }

    pub(crate) fn inline_content_sizes(
        &self,
        layout_context: &LayoutContext,
        constraint_space: &ConstraintSpace,
    ) -> InlineContentSizesResult {
        self.base
            .inline_content_sizes(layout_context, constraint_space, &self.contents)
    }

    /// Computes the tentative intrinsic block sizes that may be needed while computing
    /// the intrinsic inline sizes. Therefore, this ignores the values of the sizing
    /// properties in both axes.
    /// A return value of `None` indicates that there is no suitable tentative intrinsic
    /// block size, so intrinsic keywords in the block sizing properties will be ignored,
    /// possibly resulting in an indefinite [`SizeConstraint`] for computing the intrinsic
    /// inline sizes and laying out the contents.
    /// A return value of `Some` indicates that intrinsic keywords in the block sizing
    /// properties will be resolved as the contained value, guaranteeing a definite amount
    /// for computing the intrinsic inline sizes and laying out the contents.
    pub(crate) fn tentative_block_content_size(
        &self,
        preferred_aspect_ratio: Option<AspectRatio>,
    ) -> Option<ContentSizes> {
        // See <https://github.com/w3c/csswg-drafts/issues/12333> regarding the difference
        // in behavior for the replaced and non-replaced cases.
        match &self.contents {
            IndependentFormattingContextContents::Replaced(contents) => {
                // For replaced elements with no ratio, the returned value doesn't matter.
                let ratio = preferred_aspect_ratio?;
                let writing_mode = self.style().writing_mode;
                let inline_size = contents.fallback_inline_size(writing_mode);
                let block_size = ratio.compute_dependent_size(Direction::Block, inline_size);
                Some(block_size.into())
            },
            _ => None,
        }
    }

    pub(crate) fn outer_inline_content_sizes(
        &self,
        layout_context: &LayoutContext,
        containing_block: &IndefiniteContainingBlock,
        auto_minimum: &LogicalVec2<Au>,
        auto_block_size_stretches_to_containing_block: bool,
    ) -> InlineContentSizesResult {
        sizing::outer_inline(
            &self.layout_style(),
            containing_block,
            auto_minimum,
            auto_block_size_stretches_to_containing_block,
            self.is_replaced(),
            true, /* establishes_containing_block */
            |padding_border_sums| self.preferred_aspect_ratio(padding_border_sums),
            |constraint_space| self.inline_content_sizes(layout_context, constraint_space),
            |preferred_aspect_ratio| self.tentative_block_content_size(preferred_aspect_ratio),
        )
    }

    pub(crate) fn repair_style(
        &mut self,
        context: &SharedStyleContext,
        node: &ServoLayoutNode,
        new_style: &Arc<ComputedValues>,
    ) {
        self.base.repair_style(new_style);
        match &mut self.contents {
            IndependentFormattingContextContents::Replaced(..) => {},
            IndependentFormattingContextContents::Flow(block_formatting_context) => {
                block_formatting_context.repair_style(node, new_style);
            },
            IndependentFormattingContextContents::Flex(flex_container) => {
                flex_container.repair_style(new_style)
            },
            IndependentFormattingContextContents::Grid(taffy_container) => {
                taffy_container.repair_style(new_style)
            },
            IndependentFormattingContextContents::Table(table) => {
                table.repair_style(context, new_style)
            },
        }
    }

    #[inline]
    pub(crate) fn is_table(&self) -> bool {
        matches!(
            &self.contents,
            IndependentFormattingContextContents::Table(_)
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn layout_without_caching(
        &self,
        layout_context: &LayoutContext,
        positioning_context: &mut PositioningContext,
        containing_block_for_children: &ContainingBlock,
        containing_block: &ContainingBlock,
        preferred_aspect_ratio: Option<AspectRatio>,
        depends_on_block_constraints: bool,
        lazy_block_size: &LazySize,
    ) -> CacheableLayoutResult {
        match &self.contents {
            IndependentFormattingContextContents::Replaced(replaced) => replaced.layout(
                layout_context,
                containing_block_for_children,
                preferred_aspect_ratio,
                &self.base,
                depends_on_block_constraints,
                lazy_block_size,
            ),
            IndependentFormattingContextContents::Flow(bfc) => bfc.layout(
                layout_context,
                positioning_context,
                containing_block_for_children,
                depends_on_block_constraints,
            ),
            IndependentFormattingContextContents::Flex(fc) => fc.layout(
                layout_context,
                positioning_context,
                containing_block_for_children,
                depends_on_block_constraints,
                lazy_block_size,
            ),
            IndependentFormattingContextContents::Grid(fc) => fc.layout(
                layout_context,
                positioning_context,
                containing_block_for_children,
                containing_block,
            ),
            IndependentFormattingContextContents::Table(table) => table.layout(
                layout_context,
                positioning_context,
                containing_block_for_children,
                containing_block,
                depends_on_block_constraints,
            ),
        }
    }

    #[servo_tracing::instrument(
        name = "IndependentFormattingContext::layout_with_caching",
        skip_all
    )]
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn layout(
        &self,
        layout_context: &LayoutContext,
        positioning_context: &mut PositioningContext,
        containing_block_for_children: &ContainingBlock,
        containing_block: &ContainingBlock,
        preferred_aspect_ratio: Option<AspectRatio>,
        depends_on_block_constraints: bool,
        lazy_block_size: &LazySize,
    ) -> CacheableLayoutResult {
        if let Some(cache) = self.base.cached_layout_result.borrow().as_ref() {
            let cache = &**cache;
            if cache.containing_block_for_children_size.inline ==
                containing_block_for_children.size.inline &&
                (cache.containing_block_for_children_size.block ==
                    containing_block_for_children.size.block ||
                    !(cache.result.depends_on_block_constraints ||
                        depends_on_block_constraints))
            {
                positioning_context.append(cache.positioning_context.clone());
                return cache.result.clone();
            }
            #[cfg(feature = "tracing")]
            tracing::debug!(
                name: "NonReplaced cache miss",
                cached = ?cache.containing_block_for_children_size,
                required = ?containing_block_for_children.size,
            );
        }

        let mut child_positioning_context = PositioningContext::default();
        let result = self.layout_without_caching(
            layout_context,
            &mut child_positioning_context,
            containing_block_for_children,
            containing_block,
            preferred_aspect_ratio,
            depends_on_block_constraints,
            lazy_block_size,
        );

        *self.base.cached_layout_result.borrow_mut() =
            Some(Box::new(CacheableLayoutResultAndInputs {
                result: result.clone(),
                positioning_context: child_positioning_context.clone(),
                containing_block_for_children_size: containing_block_for_children.size.clone(),
            }));
        positioning_context.append(child_positioning_context);

        result
    }

    #[inline]
    pub(crate) fn layout_style(&self) -> LayoutStyle {
        match &self.contents {
            IndependentFormattingContextContents::Replaced(fc) => fc.layout_style(&self.base),
            IndependentFormattingContextContents::Flow(fc) => fc.layout_style(&self.base),
            IndependentFormattingContextContents::Flex(fc) => fc.layout_style(),
            IndependentFormattingContextContents::Grid(fc) => fc.layout_style(),
            IndependentFormattingContextContents::Table(fc) => fc.layout_style(None),
        }
    }

    #[inline]
    pub(crate) fn preferred_aspect_ratio(
        &self,
        padding_border_sums: &LogicalVec2<Au>,
    ) -> Option<AspectRatio> {
        match &self.contents {
            IndependentFormattingContextContents::Replaced(replaced) => {
                replaced.preferred_aspect_ratio(self.style(), padding_border_sums)
            },
            // TODO: support preferred aspect ratios on non-replaced boxes.
            _ => None,
        }
    }
}

impl ComputeInlineContentSizes for IndependentFormattingContextContents {
    fn compute_inline_content_sizes(
        &self,
        layout_context: &LayoutContext,
        constraint_space: &ConstraintSpace,
    ) -> InlineContentSizesResult {
        match self {
            Self::Replaced(inner) => {
                inner.compute_inline_content_sizes(layout_context, constraint_space)
            },
            Self::Flow(inner) => inner
                .contents
                .compute_inline_content_sizes(layout_context, constraint_space),
            Self::Flex(inner) => {
                inner.compute_inline_content_sizes(layout_context, constraint_space)
            },
            Self::Grid(inner) => {
                inner.compute_inline_content_sizes(layout_context, constraint_space)
            },
            Self::Table(inner) => {
                inner.compute_inline_content_sizes(layout_context, constraint_space)
            },
        }
    }
}
