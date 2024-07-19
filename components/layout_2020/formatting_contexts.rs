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

use crate::cell::ArcRefCell;
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
pub(crate) struct IndependentFormattingContext {
    pub base_fragment_info: BaseFragmentInfo,
    #[serde(skip_serializing)]
    pub style: Arc<ComputedValues>,
    /// If it was requested during construction
    ///
    /// TODO(valadaptive): try to stop using ArcRefCell here. We also don't need
    /// the Arc part, but AtomicRefCell doesn't have the serde feature enabled
    /// and I don't want to mess with that if further refactorings fix this
    content_sizes: ArcRefCell<Option<ContentSizes>>,
    pub contents: IndependentFormattingContextContents,
}

#[derive(Debug, Serialize)]
pub(crate) enum IndependentFormattingContextContents {
    NonReplaced(NonReplacedFormattingContextContents),
    Replaced(ReplacedContent),
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
    pub fn new_non_replaced(
        base_fragment_info: BaseFragmentInfo,
        style: Arc<ComputedValues>,
        contents: NonReplacedFormattingContextContents,
    ) -> Self {
        Self {
            base_fragment_info,
            style,
            content_sizes: ArcRefCell::new(None),
            contents: IndependentFormattingContextContents::NonReplaced(contents),
        }
    }

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
                Self {
                    style: Arc::clone(&node_and_style_info.style),
                    base_fragment_info,
                    content_sizes: ArcRefCell::new(None),
                    contents: IndependentFormattingContextContents::NonReplaced(contents),
                }
            },
            Contents::Replaced(contents) => {
                let mut base_fragment_info: BaseFragmentInfo = node_and_style_info.into();
                base_fragment_info.flags.insert(FragmentFlags::IS_REPLACED);
                Self {
                    base_fragment_info,
                    style: Arc::clone(&node_and_style_info.style),
                    content_sizes: ArcRefCell::new(None),
                    contents: IndependentFormattingContextContents::Replaced(contents),
                }
            },
        }
    }

    // TODO(valadaptive): remove this since we no longer have to retrieve this from the "inner" struct?
    pub fn style(&self) -> &Arc<ComputedValues> {
        &self.style
    }

    // TODO(valadaptive): remove this since we no longer have to retrieve this from the "inner" struct?
    pub fn base_fragment_info(&self) -> BaseFragmentInfo {
        self.base_fragment_info
    }

    pub fn inline_content_sizes(&self, layout_context: &LayoutContext) -> ContentSizes {
        match &self.contents {
            IndependentFormattingContextContents::NonReplaced(inner) => {
                *self.content_sizes.borrow_mut().get_or_insert_with(|| {
                    inner.inline_content_sizes(layout_context, self.style.writing_mode)
                })
            },
            IndependentFormattingContextContents::Replaced(inner) => {
                // TODO(valadaptive): Can we cache replaced content sizes too?
                inner.inline_content_sizes(&self.style)
            },
        }
    }

    pub fn outer_inline_content_sizes(
        &self,
        layout_context: &LayoutContext,
        containing_block_writing_mode: WritingMode,
    ) -> ContentSizes {
        sizing::outer_inline(&self.style, containing_block_writing_mode, || {
            self.inline_content_sizes(layout_context)
        })
    }
}

impl NonReplacedFormattingContextContents {
    pub fn layout(
        &self,
        layout_context: &LayoutContext,
        positioning_context: &mut PositioningContext,
        containing_block_for_children: &ContainingBlock,
        containing_block: &ContainingBlock,
    ) -> IndependentLayout {
        match &self {
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

    pub fn inline_content_sizes(
        &self,
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
