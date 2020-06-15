/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::context::LayoutContext;
use crate::dom_traversal::{Contents, NodeAndStyleInfo, NodeExt};
use crate::flexbox::FlexContainer;
use crate::flow::BlockFormattingContext;
use crate::fragments::{Fragment, Tag};
use crate::positioned::PositioningContext;
use crate::replaced::ReplacedContent;
use crate::sizing::{BoxContentSizes, ContentSizes, ContentSizesRequest};
use crate::style_ext::DisplayInside;
use crate::ContainingBlock;
use servo_arc::Arc;
use std::convert::TryInto;
use style::properties::ComputedValues;
use style::values::computed::Length;
use style::values::specified::text::TextDecorationLine;

/// https://drafts.csswg.org/css-display/#independent-formatting-context
#[derive(Debug, Serialize)]
pub(crate) enum IndependentFormattingContext {
    NonReplaced(NonReplacedFormattingContext),
    Replaced(ReplacedFormattingContext),
}

#[derive(Debug, Serialize)]
pub(crate) struct NonReplacedFormattingContext {
    pub tag: Tag,
    #[serde(skip_serializing)]
    pub style: Arc<ComputedValues>,
    /// If it was requested during construction
    pub content_sizes: BoxContentSizes,
    pub contents: NonReplacedFormattingContextContents,
}

#[derive(Debug, Serialize)]
pub(crate) struct ReplacedFormattingContext {
    pub tag: Tag,
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
        info: &NodeAndStyleInfo<impl NodeExt<'dom>>,
        display_inside: DisplayInside,
        contents: Contents,
        content_sizes: ContentSizesRequest,
        propagated_text_decoration_line: TextDecorationLine,
    ) -> Self {
        match contents.try_into() {
            Ok(non_replaced) => match display_inside {
                DisplayInside::Flow | DisplayInside::FlowRoot => {
                    let (bfc, content_sizes) = BlockFormattingContext::construct(
                        context,
                        info,
                        non_replaced,
                        content_sizes,
                        propagated_text_decoration_line,
                    );
                    Self::NonReplaced(NonReplacedFormattingContext {
                        tag: Tag::from_node_and_style_info(info),
                        style: Arc::clone(&info.style),
                        content_sizes,
                        contents: NonReplacedFormattingContextContents::Flow(bfc),
                    })
                },
                DisplayInside::Flex => {
                    let (fc, content_sizes) = FlexContainer::construct(
                        context,
                        info,
                        non_replaced,
                        content_sizes,
                        propagated_text_decoration_line,
                    );
                    Self::NonReplaced(NonReplacedFormattingContext {
                        tag: Tag::from_node_and_style_info(info),
                        style: Arc::clone(&info.style),
                        content_sizes,
                        contents: NonReplacedFormattingContextContents::Flex(fc),
                    })
                },
            },
            Err(contents) => Self::Replaced(ReplacedFormattingContext {
                tag: Tag::from_node_and_style_info(info),
                style: Arc::clone(&info.style),
                contents,
            }),
        }
    }

    pub fn construct_for_text_runs<'dom>(
        context: &LayoutContext,
        info: &NodeAndStyleInfo<impl NodeExt<'dom>>,
        runs: impl Iterator<Item = crate::flow::inline::TextRun>,
        content_sizes: ContentSizesRequest,
        propagated_text_decoration_line: TextDecorationLine,
    ) -> Self {
        let (bfc, content_sizes) = BlockFormattingContext::construct_for_text_runs(
            context,
            runs,
            content_sizes,
            propagated_text_decoration_line,
        );
        Self::NonReplaced(NonReplacedFormattingContext {
            tag: Tag::from_node_and_style_info(info),
            style: Arc::clone(&info.style),
            content_sizes,
            contents: NonReplacedFormattingContextContents::Flow(bfc),
        })
    }

    pub fn style(&self) -> &Arc<ComputedValues> {
        match self {
            Self::NonReplaced(inner) => &inner.style,
            Self::Replaced(inner) => &inner.style,
        }
    }

    pub fn tag(&self) -> Tag {
        match self {
            Self::NonReplaced(inner) => inner.tag,
            Self::Replaced(inner) => inner.tag,
        }
    }

    pub fn content_sizes(&self) -> ContentSizes {
        match self {
            Self::NonReplaced(inner) => inner.content_sizes.expect_inline().clone(),
            Self::Replaced(inner) => inner.contents.inline_content_sizes(&inner.style),
        }
    }
}

impl NonReplacedFormattingContext {
    pub fn layout(
        &self,
        layout_context: &LayoutContext,
        positioning_context: &mut PositioningContext,
        containing_block: &ContainingBlock,
        tree_rank: usize,
    ) -> IndependentLayout {
        match &self.contents {
            NonReplacedFormattingContextContents::Flow(bfc) => bfc.layout(
                layout_context,
                positioning_context,
                containing_block,
                tree_rank,
            ),
            NonReplacedFormattingContextContents::Flex(fc) => fc.layout(
                layout_context,
                positioning_context,
                containing_block,
                tree_rank,
            ),
        }
    }
}
