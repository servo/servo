/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use app_units::Au;
use serde::Serialize;
use servo_arc::Arc;
use style::logical_geometry::WritingMode;
use style::properties::ComputedValues;
use style::selector_parser::PseudoElement;
use style::values::specified::text::TextDecorationLine;

use crate::context::LayoutContext;
use crate::dom::NodeExt;
use crate::dom_traversal::{Contents, NodeAndStyleInfo};
use crate::flexbox::FlexContainer;
use crate::flow::BlockFormattingContext;
use crate::fragment_tree::{BaseFragmentInfo, Fragment, FragmentFlags};
use crate::positioned::PositioningContext;
use crate::replaced::ReplacedContent;
use crate::sizing::{self, ContentSizes};
use crate::style_ext::DisplayInside;
use crate::table::Table;
use crate::ContainingBlock;

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
    pub content_sizes: Option<ContentSizes>,
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
                    content_sizes: None,
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

    pub fn inline_content_sizes(&mut self, layout_context: &LayoutContext) -> ContentSizes {
        match self {
            Self::NonReplaced(inner) => inner
                .contents
                .inline_content_sizes(layout_context, inner.style.writing_mode),
            Self::Replaced(inner) => inner.contents.inline_content_sizes(&inner.style),
        }
    }

    pub fn outer_inline_content_sizes(
        &mut self,
        layout_context: &LayoutContext,
        containing_block_writing_mode: WritingMode,
    ) -> ContentSizes {
        match self {
            Self::NonReplaced(non_replaced) => {
                let style = &non_replaced.style;
                let content_sizes = &mut non_replaced.content_sizes;
                let contents = &mut non_replaced.contents;
                sizing::outer_inline(style, containing_block_writing_mode, || {
                    *content_sizes.get_or_insert_with(|| {
                        contents.inline_content_sizes(layout_context, style.writing_mode)
                    })
                })
            },
            Self::Replaced(replaced) => {
                sizing::outer_inline(&replaced.style, containing_block_writing_mode, || {
                    replaced.contents.inline_content_sizes(&replaced.style)
                })
            },
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
            NonReplacedFormattingContextContents::Table(table) => table.layout(
                layout_context,
                positioning_context,
                containing_block_for_children,
                containing_block,
            ),
        }
    }

    pub fn inline_content_sizes(&mut self, layout_context: &LayoutContext) -> ContentSizes {
        let writing_mode = self.style.writing_mode;
        let contents = &mut self.contents;
        *self
            .content_sizes
            .get_or_insert_with(|| contents.inline_content_sizes(layout_context, writing_mode))
    }
}

impl NonReplacedFormattingContextContents {
    pub fn inline_content_sizes(
        &mut self,
        layout_context: &LayoutContext,
        writing_mode: WritingMode,
    ) -> ContentSizes {
        match self {
            Self::Flow(inner) => inner
                .contents
                .inline_content_sizes(layout_context, writing_mode),
            Self::Flex(inner) => inner.inline_content_sizes(),
            Self::Table(table) => table.inline_content_sizes(layout_context, writing_mode),
        }
    }
}
