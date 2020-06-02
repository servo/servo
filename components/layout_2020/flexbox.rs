/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::cell::ArcRefCell;
use crate::context::LayoutContext;
use crate::dom_traversal::{BoxSlot, Contents, NodeExt, NonReplacedContents, TraversalHandler};
use crate::element_data::LayoutBox;
use crate::formatting_contexts::{IndependentFormattingContext, IndependentLayout};
use crate::positioned::{AbsolutelyPositionedBox, PositioningContext};
use crate::sizing::{BoxContentSizes, ContentSizes, ContentSizesRequest};
use crate::style_ext::DisplayGeneratingBox;
use crate::ContainingBlock;
use servo_arc::Arc;
use std::borrow::Cow;
use style::properties::ComputedValues;
use style::values::computed::Length;
use style::values::specified::text::TextDecorationLine;
use style::Zero;

// FIXME: `min-width: auto` is not zero: https://drafts.csswg.org/css-flexbox/#min-size-auto

#[derive(Debug, Serialize)]
pub(crate) struct FlexContainer {
    children: Vec<ArcRefCell<FlexLevelBox>>,
}

#[derive(Debug, Serialize)]
pub(crate) enum FlexLevelBox {
    FlexItem(IndependentFormattingContext),
    OutOfFlowAbsolutelyPositionedBox(Arc<AbsolutelyPositionedBox>),
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
        let text_decoration_line =
            propagated_text_decoration_line | style.clone_text_decoration_line();
        let mut builder = FlexContainerBuilder {
            context,
            text_decoration_line,
            flex_container: Self {
                children: Vec::new(),
            },
        };
        contents.traverse(context, node, style, &mut builder);
        let content_sizes = content_sizes.compute(|| {
            // FIXME
            ContentSizes::zero()
        });
        (builder.flex_container, content_sizes)
    }
}

struct FlexContainerBuilder<'context> {
    context: &'context LayoutContext<'context>,
    text_decoration_line: TextDecorationLine,
    flex_container: FlexContainer,
}

impl<'context, 'dom, Node: 'dom> TraversalHandler<'dom, Node> for FlexContainerBuilder<'context>
where
    Node: NodeExt<'dom>,
{
    fn handle_text(
        &mut self,
        node: Node,
        text: Cow<'dom, str>,
        parent_style: &Arc<ComputedValues>,
    ) {
        // FIXME
        let _ = node;
        let _ = text;
        let _ = parent_style;
    }

    /// Or pseudo-element
    fn handle_element(
        &mut self,
        node: Node,
        style: &Arc<ComputedValues>,
        display: DisplayGeneratingBox,
        contents: Contents,
        box_slot: BoxSlot<'dom>,
    ) {
        let display_inside = match display {
            DisplayGeneratingBox::OutsideInside { inside, .. } => inside,
        };
        let box_ = if style.get_box().position.is_absolutely_positioned() {
            // https://drafts.csswg.org/css-flexbox/#abspos-items
            ArcRefCell::new(FlexLevelBox::OutOfFlowAbsolutelyPositionedBox(Arc::new(
                AbsolutelyPositionedBox::construct(
                    self.context,
                    node,
                    style.clone(),
                    display_inside,
                    contents,
                ),
            )))
        } else {
            ArcRefCell::new(FlexLevelBox::FlexItem(IndependentFormattingContext::construct(
                self.context,
                node,
                style.clone(),
                display_inside,
                contents,
                ContentSizesRequest::None, // FIXME: request sizes when we start using them
                self.text_decoration_line,
            )))
        };
        self.flex_container.children.push(box_.clone());
        box_slot.set(LayoutBox::FlexLevel(box_))
    }
}

impl FlexContainer {
    pub(crate) fn layout(
        &self,
        layout_context: &LayoutContext,
        positioning_context: &mut PositioningContext,
        containing_block: &ContainingBlock,
        tree_rank: usize,
    ) -> IndependentLayout {
        // FIXME
        let _ = layout_context;
        let _ = positioning_context;
        let _ = containing_block;
        let _ = tree_rank;
        IndependentLayout {
            fragments: Vec::new(),
            content_block_size: Length::zero(),
        }
    }
}
