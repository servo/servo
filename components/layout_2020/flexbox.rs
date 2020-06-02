/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::context::LayoutContext;
use crate::dom_traversal::{NodeExt, NonReplacedContents};
use crate::formatting_contexts::IndependentLayout;
use crate::positioned::PositioningContext;
use crate::sizing::{BoxContentSizes, ContentSizesRequest};
use crate::ContainingBlock;
use servo_arc::Arc;
use style::properties::ComputedValues;
use style::values::specified::text::TextDecorationLine;

// FIXME: `min-width: auto` is not zero: https://drafts.csswg.org/css-flexbox/#min-size-auto

#[derive(Debug, Serialize)]
pub(crate) struct FlexContainer {
    unimplemented_fallback: crate::flow::BlockFormattingContext,
}

impl FlexContainer {
    pub fn construct<'dom>(
        context: &LayoutContext,
        node: impl NodeExt<'dom>,
        style: &Arc<ComputedValues>,
        contents: NonReplacedContents,
        content_sizes: ContentSizesRequest,
        propagated_text_decoration_line: TextDecorationLine,
    ) -> (Self, BoxContentSizes) {
        let (unimplemented_fallback, content_sizes) =
            crate::flow::BlockFormattingContext::construct(
                context,
                node,
                style,
                contents,
                content_sizes,
                propagated_text_decoration_line,
            );
        (
            Self {
                unimplemented_fallback,
            },
            content_sizes,
        )
    }

    pub(crate) fn layout(
        &self,
        layout_context: &LayoutContext,
        positioning_context: &mut PositioningContext,
        containing_block: &ContainingBlock,
        tree_rank: usize,
    ) -> IndependentLayout {
        self.unimplemented_fallback.layout(
            layout_context,
            positioning_context,
            containing_block,
            tree_rank,
        )
    }
}
