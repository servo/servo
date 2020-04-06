/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::context::LayoutContext;
use crate::dom_traversal::{Contents, NodeExt, NodeFlags};
use crate::flow::BlockFormattingContext;
use crate::fragments::Fragment;
use crate::positioned::PositioningContext;
use crate::replaced::ReplacedContent;
use crate::sizing::{BoxContentSizes, ContentSizesRequest};
use crate::style_ext::DisplayInside;
use crate::ContainingBlock;
use servo_arc::Arc;
use std::convert::TryInto;
use style::dom::OpaqueNode;
use style::properties::ComputedValues;
use style::values::computed::Length;
use style::values::specified::text::TextDecorationLine;

/// https://drafts.csswg.org/css-display/#independent-formatting-context
#[derive(Debug, Serialize)]
pub(crate) struct IndependentFormattingContext {
    pub tag: OpaqueNode,
    #[serde(skip_serializing)]
    pub style: Arc<ComputedValues>,

    /// If it was requested during construction
    pub content_sizes: BoxContentSizes,

    contents: IndependentFormattingContextContents,

    pub flags: NodeFlags,
}

pub(crate) struct IndependentLayout {
    pub fragments: Vec<Fragment>,

    /// https://drafts.csswg.org/css2/visudet.html#root-height
    pub content_block_size: Length,
}

// Private so that code outside of this module cannot match variants.
// It should got through methods instead.
#[derive(Debug, Serialize)]
enum IndependentFormattingContextContents {
    Flow(BlockFormattingContext),

    // Not called FC in specs, but behaves close enough
    Replaced(ReplacedContent),
    // Other layout modes go here
}

pub(crate) struct NonReplacedIFC<'a>(NonReplacedIFCKind<'a>);

enum NonReplacedIFCKind<'a> {
    Flow(&'a BlockFormattingContext),
}

impl IndependentFormattingContext {
    pub fn construct<'dom>(
        context: &LayoutContext,
        node: impl NodeExt<'dom>,
        style: Arc<ComputedValues>,
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
                        node,
                        &style,
                        non_replaced,
                        content_sizes,
                        propagated_text_decoration_line,
                    );
                    Self {
                        tag: node.as_opaque(),
                        style,
                        content_sizes,
                        contents: IndependentFormattingContextContents::Flow(bfc),
                        flags: node.get_node_flags(),
                    }
                },
            },
            Err(replaced) => {
                let content_sizes = content_sizes.compute(|| replaced.inline_content_sizes(&style));
                Self {
                    tag: node.as_opaque(),
                    style,
                    content_sizes,
                    contents: IndependentFormattingContextContents::Replaced(replaced),
                    flags: node.get_node_flags(),
                }
            },
        }
    }

    pub fn as_replaced(&self) -> Result<&ReplacedContent, NonReplacedIFC> {
        use self::IndependentFormattingContextContents as Contents;
        use self::NonReplacedIFC as NR;
        use self::NonReplacedIFCKind as Kind;
        match &self.contents {
            Contents::Replaced(r) => Ok(r),
            Contents::Flow(f) => Err(NR(Kind::Flow(f))),
        }
    }
}

impl NonReplacedIFC<'_> {
    pub fn layout(
        &self,
        layout_context: &LayoutContext,
        positioning_context: &mut PositioningContext,
        containing_block: &ContainingBlock,
        tree_rank: usize,
    ) -> IndependentLayout {
        match &self.0 {
            NonReplacedIFCKind::Flow(bfc) => bfc.layout(
                layout_context,
                positioning_context,
                containing_block,
                tree_rank,
            ),
        }
    }
}
