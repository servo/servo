/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use app_units::Au;
use malloc_size_of_derive::MallocSizeOf;
use script::layout_dom::{ServoLayoutElement, ServoLayoutNode};
use servo_arc::Arc;
use style::context::SharedStyleContext;
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
use crate::sizing::{self, ComputeInlineContentSizes, InlineContentSizesResult};
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
    NonReplaced(IndependentNonReplacedContents),
    Replaced(ReplacedContents),
}

// Private so that code outside of this module cannot match variants.
// It should got through methods instead.
#[derive(Debug, MallocSizeOf)]
pub(crate) enum IndependentNonReplacedContents {
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
                        IndependentNonReplacedContents::Flow(BlockFormattingContext::construct(
                            context,
                            node_and_style_info,
                            non_replaced_contents,
                            propagated_data,
                            is_list_item,
                        ))
                    },
                    DisplayInside::Grid => {
                        IndependentNonReplacedContents::Grid(TaffyContainer::construct(
                            context,
                            node_and_style_info,
                            non_replaced_contents,
                            propagated_data,
                        ))
                    },
                    DisplayInside::Flex => {
                        IndependentNonReplacedContents::Flex(FlexContainer::construct(
                            context,
                            node_and_style_info,
                            non_replaced_contents,
                            propagated_data,
                        ))
                    },
                    DisplayInside::Table => {
                        let table_grid_style = context
                            .shared_context()
                            .stylist
                            .style_for_anonymous::<ServoLayoutElement>(
                                &context.shared_context().guards,
                                &PseudoElement::ServoTableGrid,
                                &node_and_style_info.style,
                            );
                        base_fragment_info.flags.insert(FragmentFlags::DO_NOT_PAINT);
                        IndependentNonReplacedContents::Table(Table::construct(
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
                    contents: IndependentFormattingContextContents::NonReplaced(contents),
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
        match &self.contents {
            IndependentFormattingContextContents::NonReplaced(contents) => self
                .base
                .inline_content_sizes(layout_context, constraint_space, contents),
            IndependentFormattingContextContents::Replaced(contents) => self
                .base
                .inline_content_sizes(layout_context, constraint_space, contents),
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
        )
    }

    pub(crate) fn preferred_aspect_ratio(
        &self,
        padding_border_sums: &LogicalVec2<Au>,
    ) -> Option<AspectRatio> {
        match &self.contents {
            IndependentFormattingContextContents::NonReplaced(content) => {
                content.preferred_aspect_ratio()
            },
            IndependentFormattingContextContents::Replaced(content) => {
                content.preferred_aspect_ratio(self.style(), padding_border_sums)
            },
        }
    }

    #[inline]
    pub(crate) fn layout_style(&self) -> LayoutStyle {
        match &self.contents {
            IndependentFormattingContextContents::NonReplaced(content) => {
                content.layout_style(&self.base)
            },
            IndependentFormattingContextContents::Replaced(content) => {
                content.layout_style(&self.base)
            },
        }
    }

    pub(crate) fn repair_style(
        &mut self,
        context: &SharedStyleContext,
        node: &ServoLayoutNode,
        new_style: &Arc<ComputedValues>,
    ) {
        self.base.repair_style(new_style);
        match &mut self.contents {
            IndependentFormattingContextContents::NonReplaced(content) => {
                content.repair_style(context, node, new_style);
            },
            IndependentFormattingContextContents::Replaced(..) => {},
        }
    }
}

impl IndependentNonReplacedContents {
    pub(crate) fn layout_without_caching(
        &self,
        layout_context: &LayoutContext,
        positioning_context: &mut PositioningContext,
        containing_block_for_children: &ContainingBlock,
        containing_block: &ContainingBlock,
        depends_on_block_constraints: bool,
        lazy_block_size: &LazySize,
    ) -> CacheableLayoutResult {
        match self {
            IndependentNonReplacedContents::Flow(bfc) => bfc.layout(
                layout_context,
                positioning_context,
                containing_block_for_children,
                depends_on_block_constraints,
            ),
            IndependentNonReplacedContents::Flex(fc) => fc.layout(
                layout_context,
                positioning_context,
                containing_block_for_children,
                depends_on_block_constraints,
                lazy_block_size,
            ),
            IndependentNonReplacedContents::Grid(fc) => fc.layout(
                layout_context,
                positioning_context,
                containing_block_for_children,
                containing_block,
            ),
            IndependentNonReplacedContents::Table(table) => table.layout(
                layout_context,
                positioning_context,
                containing_block_for_children,
                containing_block,
                depends_on_block_constraints,
            ),
        }
    }

    #[servo_tracing::instrument(
        name = "IndependentNonReplacedContents::layout_with_caching",
        skip_all
    )]
    #[allow(clippy::too_many_arguments)]
    pub fn layout(
        &self,
        layout_context: &LayoutContext,
        positioning_context: &mut PositioningContext,
        containing_block_for_children: &ContainingBlock,
        containing_block: &ContainingBlock,
        base: &LayoutBoxBase,
        depends_on_block_constraints: bool,
        lazy_block_size: &LazySize,
    ) -> CacheableLayoutResult {
        if let Some(cache) = base.cached_layout_result.borrow().as_ref() {
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
            depends_on_block_constraints,
            lazy_block_size,
        );

        *base.cached_layout_result.borrow_mut() = Some(Box::new(CacheableLayoutResultAndInputs {
            result: result.clone(),
            positioning_context: child_positioning_context.clone(),
            containing_block_for_children_size: containing_block_for_children.size.clone(),
        }));
        positioning_context.append(child_positioning_context);

        result
    }

    #[inline]
    pub(crate) fn layout_style<'a>(&'a self, base: &'a LayoutBoxBase) -> LayoutStyle<'a> {
        match self {
            IndependentNonReplacedContents::Flow(fc) => fc.layout_style(base),
            IndependentNonReplacedContents::Flex(fc) => fc.layout_style(),
            IndependentNonReplacedContents::Grid(fc) => fc.layout_style(),
            IndependentNonReplacedContents::Table(fc) => fc.layout_style(None),
        }
    }

    #[inline]
    pub(crate) fn preferred_aspect_ratio(&self) -> Option<AspectRatio> {
        // TODO: support preferred aspect ratios on non-replaced boxes.
        None
    }

    #[inline]
    pub(crate) fn is_table(&self) -> bool {
        matches!(self, Self::Table(_))
    }

    fn repair_style(
        &mut self,
        context: &SharedStyleContext,
        node: &ServoLayoutNode,
        new_style: &Arc<ComputedValues>,
    ) {
        match self {
            IndependentNonReplacedContents::Flow(block_formatting_context) => {
                block_formatting_context.repair_style(node, new_style);
            },
            IndependentNonReplacedContents::Flex(flex_container) => {
                flex_container.repair_style(new_style)
            },
            IndependentNonReplacedContents::Grid(taffy_container) => {
                taffy_container.repair_style(new_style)
            },
            IndependentNonReplacedContents::Table(table) => table.repair_style(context, new_style),
        }
    }
}

impl ComputeInlineContentSizes for IndependentNonReplacedContents {
    fn compute_inline_content_sizes(
        &self,
        layout_context: &LayoutContext,
        constraint_space: &ConstraintSpace,
    ) -> InlineContentSizesResult {
        match self {
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
