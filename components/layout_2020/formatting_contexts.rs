/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::context::LayoutContext;
use crate::dom_traversal::{Contents, NodeExt};
use crate::flow::BlockFormattingContext;
use crate::fragments::Fragment;
use crate::geom::flow_relative::Vec2;
use crate::positioned::AbsolutelyPositionedFragment;
use crate::replaced::ReplacedContent;
use crate::style_ext::{ComputedValuesExt, Direction, DisplayInside, Position, WritingMode};
use crate::ContainingBlock;
use servo_arc::Arc;
use std::convert::TryInto;
use style::context::SharedStyleContext;
use style::properties::ComputedValues;
use style::values::computed::Length;

/// https://drafts.csswg.org/css-display/#independent-formatting-context
#[derive(Debug)]
pub(crate) struct IndependentFormattingContext {
    pub style: Arc<ComputedValues>,
    contents: IndependentFormattingContextContents,
}

pub(crate) struct IndependentLayout {
    pub fragments: Vec<Fragment>,
    pub content_block_size: Length,
}

// Private so that code outside of this module cannot match variants.
// It should got through methods instead.
#[derive(Debug)]
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
    pub fn construct<'dom, 'style>(
        context: &SharedStyleContext<'style>,
        style: Arc<ComputedValues>,
        display_inside: DisplayInside,
        contents: Contents<impl NodeExt<'dom>>,
    ) -> Self {
        use self::IndependentFormattingContextContents as Contents;
        let contents = match contents.try_into() {
            Ok(non_replaced) => match display_inside {
                DisplayInside::Flow | DisplayInside::FlowRoot => Contents::Flow(
                    BlockFormattingContext::construct(context, &style, non_replaced),
                ),
            },
            Err(replaced) => Contents::Replaced(replaced),
        };
        Self { style, contents }
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

    pub fn layout<'a>(
        &'a self,
        layout_context: &LayoutContext,
        containing_block: &ContainingBlock,
        tree_rank: usize,
        absolutely_positioned_fragments: &mut Vec<AbsolutelyPositionedFragment<'a>>,
    ) -> IndependentLayout {
        match self.as_replaced() {
            Ok(replaced) => match *replaced {},
            Err(ifc) => ifc.layout(
                layout_context,
                containing_block,
                tree_rank,
                absolutely_positioned_fragments,
            ),
        }
    }
}

impl<'a> NonReplacedIFC<'a> {
    pub fn layout(
        &self,
        layout_context: &LayoutContext,
        containing_block: &ContainingBlock,
        tree_rank: usize,
        absolutely_positioned_fragments: &mut Vec<AbsolutelyPositionedFragment<'a>>,
    ) -> IndependentLayout {
        match &self.0 {
            NonReplacedIFCKind::Flow(bfc) => bfc.layout(
                layout_context,
                containing_block,
                tree_rank,
                absolutely_positioned_fragments,
            ),
        }
    }
}
