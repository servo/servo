/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::cell::ArcRefCell;
use crate::context::LayoutContext;
use crate::dom_traversal::{BoxSlot, Contents, NodeExt, NonReplacedContents, TraversalHandler};
use crate::element_data::LayoutBox;
use crate::flow::inline::TextRun;
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
            node,
            style,
            anonymous_style: None,
            text_decoration_line,
            contiguous_text_runs: Vec::new(),
            children: Vec::new(),
        };
        contents.traverse(context, node, style, &mut builder);
        let content_sizes = content_sizes.compute(|| {
            // FIXME
            ContentSizes::zero()
        });
        (builder.finish(), content_sizes)
    }
}

/// https://drafts.csswg.org/css-flexbox/#flex-items
struct FlexContainerBuilder<'a, Node> {
    context: &'a LayoutContext<'a>,
    node: Node,
    style: &'a Arc<ComputedValues>,
    anonymous_style: Option<Arc<ComputedValues>>,
    text_decoration_line: TextDecorationLine,
    contiguous_text_runs: Vec<TextRun>,
    children: Vec<ArcRefCell<FlexLevelBox>>,
}

impl<'a, 'dom, Node: 'dom> TraversalHandler<'dom, Node> for FlexContainerBuilder<'a, Node>
where
    Node: NodeExt<'dom>,
{
    fn handle_text(
        &mut self,
        node: Node,
        text: Cow<'dom, str>,
        parent_style: &Arc<ComputedValues>,
    ) {
        self.contiguous_text_runs.push(TextRun {
            tag: node.as_opaque(),
            parent_style: parent_style.clone(),
            text: text.into(),
        })
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
        // FIXME: are text runs considered "contiguous" if they are only separated
        // by an out-of-flow abspos element?
        // (That is, are they wrapped in the same anonymous flex item, or each its own?)
        self.wrap_any_text_in_anonymous_block_container();

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
            ArcRefCell::new(FlexLevelBox::FlexItem(
                IndependentFormattingContext::construct(
                    self.context,
                    node,
                    style.clone(),
                    display_inside,
                    contents,
                    ContentSizesRequest::None, // FIXME: request sizes when we start using them
                    self.text_decoration_line,
                ),
            ))
        };
        self.children.push(box_.clone());
        box_slot.set(LayoutBox::FlexLevel(box_))
    }
}

/// https://drafts.csswg.org/css-text/#white-space
fn is_only_document_white_space(string: &str) -> bool {
    // FIXME: is this the right definition? See
    // https://github.com/w3c/csswg-drafts/issues/5146
    // https://github.com/w3c/csswg-drafts/issues/5147
    string
        .bytes()
        .all(|byte| matches!(byte, b' ' | b'\n' | b'\t'))
}

impl<'a, 'dom, Node: 'dom> FlexContainerBuilder<'a, Node>
where
    Node: NodeExt<'dom>,
{
    fn wrap_any_text_in_anonymous_block_container(&mut self) {
        if self
            .contiguous_text_runs
            .iter()
            .all(|run| is_only_document_white_space(&run.text))
        {
            // There is no text run, or they all only contain document white space characters
            self.contiguous_text_runs.clear();
            return;
        }
        let context = self.context;
        let style = self.style;
        let anonymous_style = self.anonymous_style.get_or_insert_with(|| {
            context
                .shared_context()
                .stylist
                .style_for_anonymous::<Node::ConcreteElement>(
                    &context.shared_context().guards,
                    &style::selector_parser::PseudoElement::ServoText,
                    style,
                )
        });
        self.children.push(ArcRefCell::new(FlexLevelBox::FlexItem(
            IndependentFormattingContext::construct_for_text_runs(
                self.context,
                self.node,
                anonymous_style.clone(),
                self.contiguous_text_runs.drain(..),
                ContentSizesRequest::None, // FIXME: request sizes when we start using them
                self.text_decoration_line,
            ),
        )))
    }

    fn finish(mut self) -> FlexContainer {
        self.wrap_any_text_in_anonymous_block_container();
        FlexContainer {
            children: self.children,
        }
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
