/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use app_units::Au;
use servo_arc::Arc;
use style::properties::ComputedValues;
use style::selector_parser::PseudoElement;

use crate::context::LayoutContext;
use crate::dom::NodeExt;
use crate::dom_traversal::{Contents, NodeAndStyleInfo};
use crate::flexbox::FlexContainer;
use crate::flow::BlockFormattingContext;
use crate::fragment_tree::{
    BaseFragmentInfo, BoxFragment, Fragment, FragmentFlags, SpecificLayoutInfo,
};
use crate::geom::LogicalSides;
use crate::layout_box_base::LayoutBoxBase;
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
#[derive(Debug)]
pub(crate) struct IndependentFormattingContext {
    pub base: LayoutBoxBase,
    pub contents: IndependentFormattingContextContents,
}

#[derive(Debug)]
pub(crate) enum IndependentFormattingContextContents {
    NonReplaced(IndependentNonReplacedContents),
    Replaced(ReplacedContents),
}

// Private so that code outside of this module cannot match variants.
// It should got through methods instead.
#[derive(Debug)]
pub(crate) enum IndependentNonReplacedContents {
    Flow(BlockFormattingContext),
    Flex(FlexContainer),
    Grid(TaffyContainer),
    Table(Table),
    // Other layout modes go here
}

/// The baselines of a layout or a [`crate::fragment_tree::BoxFragment`]. Some layout
/// uses the first and some layout uses the last.
#[derive(Clone, Copy, Debug, Default)]
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

pub(crate) struct IndependentLayout {
    pub fragments: Vec<Fragment>,

    /// <https://drafts.csswg.org/css2/visudet.html#root-height>
    pub content_block_size: Au,

    /// If a table has collapsed columns, it can become smaller than what the parent
    /// formatting context decided. This is the resulting inline content size.
    /// This is None for non-table layouts and for tables without collapsed columns.
    pub content_inline_size_for_table: Option<Au>,

    /// The offset of the last inflow baseline of this layout in the content area, if
    /// there was one. This is used to propagate baselines to the ancestors of `display:
    /// inline-block`.
    pub baselines: Baselines,

    /// Whether or not this layout depends on the containing block size.
    pub depends_on_block_constraints: bool,

    /// Additional information of this layout that could be used by Javascripts and devtools.
    pub specific_layout_info: Option<SpecificLayoutInfo>,
}

pub(crate) struct IndependentLayoutResult {
    pub fragment: BoxFragment,
    pub baselines: Option<Baselines>,
    pub pbm_sums: LogicalSides<Au>,
}

impl IndependentFormattingContext {
    pub fn construct<'dom, Node: NodeExt<'dom>>(
        context: &LayoutContext,
        node_and_style_info: &NodeAndStyleInfo<Node>,
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
                            .style_for_anonymous::<Node::ConcreteElement>(
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
}

impl IndependentNonReplacedContents {
    pub fn layout(
        &self,
        layout_context: &LayoutContext,
        positioning_context: &mut PositioningContext,
        containing_block_for_children: &ContainingBlock,
        containing_block: &ContainingBlock,
    ) -> IndependentLayout {
        match self {
            IndependentNonReplacedContents::Flow(bfc) => bfc.layout(
                layout_context,
                positioning_context,
                containing_block_for_children,
            ),
            IndependentNonReplacedContents::Flex(fc) => fc.layout(
                layout_context,
                positioning_context,
                containing_block_for_children,
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
            ),
        }
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
