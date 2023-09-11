/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::convert::TryInto;

use serde::Serialize;
use servo_arc::Arc;
use style::logical_geometry::WritingMode;
use style::properties::ComputedValues;
use style::values::computed::Length;
use style::values::specified::text::TextDecorationLine;

use crate::context::LayoutContext;
use crate::dom::NodeExt;
use crate::dom_traversal::{Contents, NodeAndStyleInfo};
use crate::flexbox::FlexContainer;
use crate::flow::BlockFormattingContext;
use crate::fragment_tree::{BaseFragmentInfo, Fragment};
use crate::positioned::PositioningContext;
use crate::replaced::ReplacedContent;
use crate::sizing::{self, ContentSizes};
use crate::style_ext::DisplayInside;
use crate::ContainingBlock;

/// https://drafts.csswg.org/css-display/#independent-formatting-context
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
    // Other layout modes go here
}

pub(crate) struct IndependentLayout {
    pub fragments: Vec<Fragment>,

    /// https://drafts.csswg.org/css2/visudet.html#root-height
    pub content_block_size: Length,
}

impl IndependentFormattingContext {
    pub fn construct<'dom>(
        context: &LayoutContext,
        node_and_style_info: &NodeAndStyleInfo<impl NodeExt<'dom>>,
        display_inside: DisplayInside,
        contents: Contents,
        propagated_text_decoration_line: TextDecorationLine,
    ) -> Self {
        match contents.try_into() {
            Ok(non_replaced) => {
                let contents = match display_inside {
                    DisplayInside::Flow { is_list_item } |
                    DisplayInside::FlowRoot { is_list_item } => {
                        NonReplacedFormattingContextContents::Flow(
                            BlockFormattingContext::construct(
                                context,
                                node_and_style_info,
                                non_replaced,
                                propagated_text_decoration_line,
                                is_list_item,
                            ),
                        )
                    },
                    DisplayInside::Flex => {
                        NonReplacedFormattingContextContents::Flex(FlexContainer::construct(
                            context,
                            node_and_style_info,
                            non_replaced,
                            propagated_text_decoration_line,
                        ))
                    },
                };
                Self::NonReplaced(NonReplacedFormattingContext {
                    base_fragment_info: node_and_style_info.into(),
                    style: Arc::clone(&node_and_style_info.style),
                    content_sizes: None,
                    contents,
                })
            },
            Err(contents) => Self::Replaced(ReplacedFormattingContext {
                base_fragment_info: node_and_style_info.into(),
                style: Arc::clone(&node_and_style_info.style),
                contents,
            }),
        }
    }

    pub fn construct_for_text_runs<'dom>(
        node_and_style_info: &NodeAndStyleInfo<impl NodeExt<'dom>>,
        runs: impl Iterator<Item = crate::flow::inline::TextRun>,
        propagated_text_decoration_line: TextDecorationLine,
    ) -> Self {
        let bfc =
            BlockFormattingContext::construct_for_text_runs(runs, propagated_text_decoration_line);
        Self::NonReplaced(NonReplacedFormattingContext {
            base_fragment_info: node_and_style_info.into(),
            style: Arc::clone(&node_and_style_info.style),
            content_sizes: None,
            contents: NonReplacedFormattingContextContents::Flow(bfc),
        })
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

    pub fn inline_content_sizes(&self, layout_context: &LayoutContext) -> ContentSizes {
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
                let contents = &non_replaced.contents;
                sizing::outer_inline(&style, containing_block_writing_mode, || {
                    content_sizes
                        .get_or_insert_with(|| {
                            contents.inline_content_sizes(layout_context, style.writing_mode)
                        })
                        .clone()
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
        containing_block: &ContainingBlock,
    ) -> IndependentLayout {
        match &self.contents {
            NonReplacedFormattingContextContents::Flow(bfc) => {
                bfc.layout(layout_context, positioning_context, containing_block)
            },
            NonReplacedFormattingContextContents::Flex(fc) => {
                fc.layout(layout_context, positioning_context, containing_block)
            },
        }
    }

    pub fn inline_content_sizes(&mut self, layout_context: &LayoutContext) -> ContentSizes {
        let writing_mode = self.style.writing_mode;
        let contents = &self.contents;
        self.content_sizes
            .get_or_insert_with(|| contents.inline_content_sizes(layout_context, writing_mode))
            .clone()
    }
}

impl NonReplacedFormattingContextContents {
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
        }
    }
}
