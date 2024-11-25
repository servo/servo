/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use app_units::Au;
use atomic_refcell::AtomicRefCell;
use serde::Serialize;
use servo_arc::Arc;
use style::properties::ComputedValues;
use style::selector_parser::PseudoElement;
use style::values::specified::text::TextDecorationLine;

use crate::context::LayoutContext;
use crate::dom::NodeExt;
use crate::dom_traversal::{Contents, NodeAndStyleInfo};
use crate::flexbox::FlexContainer;
use crate::flow::BlockFormattingContext;
use crate::fragment_tree::{BaseFragmentInfo, BoxFragment, Fragment, FragmentFlags};
use crate::geom::LogicalSides;
use crate::positioned::PositioningContext;
use crate::replaced::ReplacedContent;
use crate::sizing::{self, InlineContentSizesResult};
use crate::style_ext::{AspectRatio, DisplayInside};
use crate::table::Table;
use crate::taffy::TaffyContainer;
use crate::{
    ConstraintSpace, ContainingBlock, IndefiniteContainingBlock, LogicalVec2, SizeConstraint,
};

/// <https://drafts.csswg.org/css-display/#independent-formatting-context>
#[derive(Debug, Serialize)]
pub(crate) enum IndependentFormattingContext {
    NonReplaced(NonReplacedFormattingContext),
    Replaced(ReplacedFormattingContext),
}

#[derive(Debug, Serialize)]
pub(crate) struct NonReplacedFormattingContext {
    pub base_fragment_info: BaseFragmentInfo,
    #[serde(skip_serializing)]
    pub style: Arc<ComputedValues>,
    /// If it was requested during construction
    #[serde(skip_serializing)]
    pub content_sizes_result: AtomicRefCell<Option<(SizeConstraint, InlineContentSizesResult)>>,
    pub contents: NonReplacedFormattingContextContents,
}

#[derive(Debug, Serialize)]
pub(crate) struct ReplacedFormattingContext {
    pub base_fragment_info: BaseFragmentInfo,
    #[serde(skip_serializing)]
    pub style: Arc<ComputedValues>,
    pub contents: ReplacedContent,
}

// Private so that code outside of this module cannot match variants.
// It should got through methods instead.
#[derive(Debug, Serialize)]
pub(crate) enum NonReplacedFormattingContextContents {
    Flow(BlockFormattingContext),
    Flex(FlexContainer),
    Grid(TaffyContainer),
    Table(Table),
    // Other layout modes go here
}

/// The baselines of a layout or a [`crate::fragment_tree::BoxFragment`]. Some layout
/// uses the first and some layout uses the last.
#[derive(Clone, Copy, Debug, Default, Serialize)]
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

    /// The contents of a table may force it to become wider than what we would expect
    /// from 'width' and 'min-width'. This is the resulting inline content size,
    /// or None for non-table layouts.
    pub content_inline_size_for_table: Option<Au>,

    /// The offset of the last inflow baseline of this layout in the content area, if
    /// there was one. This is used to propagate baselines to the ancestors of `display:
    /// inline-block`.
    pub baselines: Baselines,

    /// Whether or not this layout depends on the containing block size.
    pub depends_on_block_constraints: bool,
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
        propagated_text_decoration_line: TextDecorationLine,
    ) -> Self {
        match contents {
            Contents::NonReplaced(non_replaced_contents) => {
                let mut base_fragment_info: BaseFragmentInfo = node_and_style_info.into();
                let contents = match display_inside {
                    DisplayInside::Flow { is_list_item } |
                    DisplayInside::FlowRoot { is_list_item } => {
                        NonReplacedFormattingContextContents::Flow(
                            BlockFormattingContext::construct(
                                context,
                                node_and_style_info,
                                non_replaced_contents,
                                propagated_text_decoration_line,
                                is_list_item,
                            ),
                        )
                    },
                    DisplayInside::Grid => {
                        NonReplacedFormattingContextContents::Grid(TaffyContainer::construct(
                            context,
                            node_and_style_info,
                            non_replaced_contents,
                            propagated_text_decoration_line,
                        ))
                    },
                    DisplayInside::Flex => {
                        NonReplacedFormattingContextContents::Flex(FlexContainer::construct(
                            context,
                            node_and_style_info,
                            non_replaced_contents,
                            propagated_text_decoration_line,
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
                        NonReplacedFormattingContextContents::Table(Table::construct(
                            context,
                            node_and_style_info,
                            table_grid_style,
                            non_replaced_contents,
                            propagated_text_decoration_line,
                        ))
                    },
                };
                Self::NonReplaced(NonReplacedFormattingContext {
                    style: Arc::clone(&node_and_style_info.style),
                    base_fragment_info,
                    content_sizes_result: AtomicRefCell::default(),
                    contents,
                })
            },
            Contents::Replaced(contents) => {
                let mut base_fragment_info: BaseFragmentInfo = node_and_style_info.into();
                base_fragment_info.flags.insert(FragmentFlags::IS_REPLACED);
                Self::Replaced(ReplacedFormattingContext {
                    base_fragment_info,
                    style: Arc::clone(&node_and_style_info.style),
                    contents,
                })
            },
        }
    }

    pub fn style(&self) -> &Arc<ComputedValues> {
        match self {
            Self::NonReplaced(inner) => &inner.style,
            Self::Replaced(inner) => &inner.style,
        }
    }

    pub fn base_fragment_info(&self) -> BaseFragmentInfo {
        match self {
            Self::NonReplaced(inner) => inner.base_fragment_info,
            Self::Replaced(inner) => inner.base_fragment_info,
        }
    }

    pub(crate) fn inline_content_sizes(
        &self,
        layout_context: &LayoutContext,
        constraint_space: &ConstraintSpace,
        containing_block: &IndefiniteContainingBlock,
    ) -> InlineContentSizesResult {
        match self {
            Self::NonReplaced(inner) => {
                inner.inline_content_sizes(layout_context, constraint_space)
            },
            Self::Replaced(inner) => inner.contents.inline_content_sizes(
                layout_context,
                constraint_space,
                inner.preferred_aspect_ratio(containing_block),
            ),
        }
    }

    pub(crate) fn outer_inline_content_sizes(
        &self,
        layout_context: &LayoutContext,
        containing_block: &IndefiniteContainingBlock,
        auto_minimum: &LogicalVec2<Au>,
        auto_block_size_stretches_to_containing_block: bool,
    ) -> InlineContentSizesResult {
        match self {
            Self::NonReplaced(non_replaced) => non_replaced.outer_inline_content_sizes(
                layout_context,
                containing_block,
                auto_minimum,
                auto_block_size_stretches_to_containing_block,
            ),
            Self::Replaced(replaced) => sizing::outer_inline(
                &replaced.style,
                containing_block,
                auto_minimum,
                auto_block_size_stretches_to_containing_block,
                |constraint_space| {
                    replaced.contents.inline_content_sizes(
                        layout_context,
                        constraint_space,
                        replaced.preferred_aspect_ratio(containing_block),
                    )
                },
            ),
        }
    }

    pub(crate) fn preferred_aspect_ratio(
        &self,
        containing_block: &IndefiniteContainingBlock,
    ) -> Option<AspectRatio> {
        match self {
            Self::NonReplaced(_) => None,
            Self::Replaced(replaced) => replaced.preferred_aspect_ratio(containing_block),
        }
    }
}

impl NonReplacedFormattingContext {
    pub fn layout(
        &self,
        layout_context: &LayoutContext,
        positioning_context: &mut PositioningContext,
        containing_block_for_children: &ContainingBlock,
        containing_block: &ContainingBlock,
    ) -> IndependentLayout {
        match &self.contents {
            NonReplacedFormattingContextContents::Flow(bfc) => bfc.layout(
                layout_context,
                positioning_context,
                containing_block_for_children,
            ),
            NonReplacedFormattingContextContents::Flex(fc) => fc.layout(
                layout_context,
                positioning_context,
                containing_block_for_children,
                containing_block,
            ),
            NonReplacedFormattingContextContents::Grid(fc) => fc.layout(
                layout_context,
                positioning_context,
                containing_block_for_children,
                containing_block,
            ),
            NonReplacedFormattingContextContents::Table(table) => table.layout(
                layout_context,
                positioning_context,
                containing_block_for_children,
                containing_block,
            ),
        }
    }

    pub(crate) fn inline_content_sizes(
        &self,
        layout_context: &LayoutContext,
        constraint_space: &ConstraintSpace,
    ) -> InlineContentSizesResult {
        let mut cache = self.content_sizes_result.borrow_mut();
        if let Some((previous_cb_block_size, result)) = *cache {
            if !result.depends_on_block_constraints ||
                previous_cb_block_size == constraint_space.block_size
            {
                return result;
            }
            // TODO: Should we keep multiple caches for various block sizes?
        }

        let result = self
            .contents
            .inline_content_sizes(layout_context, constraint_space);
        *cache = Some((constraint_space.block_size, result));
        result
    }

    pub(crate) fn outer_inline_content_sizes(
        &self,
        layout_context: &LayoutContext,
        containing_block: &IndefiniteContainingBlock,
        auto_minimum: &LogicalVec2<Au>,
        auto_block_size_stretches_to_containing_block: bool,
    ) -> InlineContentSizesResult {
        sizing::outer_inline(
            &self.style,
            containing_block,
            auto_minimum,
            auto_block_size_stretches_to_containing_block,
            |constraint_space| self.inline_content_sizes(layout_context, constraint_space),
        )
    }
}

impl NonReplacedFormattingContextContents {
    pub(crate) fn inline_content_sizes(
        &self,
        layout_context: &LayoutContext,
        constraint_space: &ConstraintSpace,
    ) -> InlineContentSizesResult {
        match self {
            Self::Flow(inner) => inner
                .contents
                .inline_content_sizes(layout_context, constraint_space),
            Self::Flex(inner) => inner.inline_content_sizes(layout_context, constraint_space),
            Self::Grid(inner) => inner.inline_content_sizes(layout_context, constraint_space),
            Self::Table(table) => table.inline_content_sizes(layout_context, constraint_space),
        }
    }
}

impl ReplacedFormattingContext {
    pub(crate) fn preferred_aspect_ratio(
        &self,
        containing_block: &IndefiniteContainingBlock,
    ) -> Option<AspectRatio> {
        self.contents
            .preferred_aspect_ratio(containing_block, &self.style)
    }
}
