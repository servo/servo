/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use super::{FlexContainer, FlexLevelBox};
use crate::cell::ArcRefCell;
use crate::context::LayoutContext;
use crate::dom_traversal::{
    BoxSlot, Contents, NodeAndStyleInfo, NodeExt, NonReplacedContents, TraversalHandler,
};
use crate::element_data::LayoutBox;
use crate::formatting_contexts::IndependentFormattingContext;
use crate::fragments::Tag;
use crate::positioned::AbsolutelyPositionedBox;
use crate::style_ext::DisplayGeneratingBox;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::borrow::Cow;
use style::values::specified::text::TextDecorationLine;

impl FlexContainer {
    pub fn construct<'dom>(
        context: &LayoutContext,
        info: &NodeAndStyleInfo<impl NodeExt<'dom>>,
        contents: NonReplacedContents,
        propagated_text_decoration_line: TextDecorationLine,
    ) -> Self {
        let text_decoration_line =
            propagated_text_decoration_line | info.style.clone_text_decoration_line();
        let mut builder = FlexContainerBuilder {
            context,
            info,
            text_decoration_line,
            contiguous_text_runs: Vec::new(),
            jobs: Vec::new(),
            has_text_runs: false,
        };
        contents.traverse(context, info, &mut builder);
        builder.finish()
    }
}

/// https://drafts.csswg.org/css-flexbox/#flex-items
struct FlexContainerBuilder<'a, 'dom, Node> {
    context: &'a LayoutContext<'a>,
    info: &'a NodeAndStyleInfo<Node>,
    text_decoration_line: TextDecorationLine,
    contiguous_text_runs: Vec<TextRun<'dom, Node>>,
    /// To be run in parallel with rayon in `finish`
    jobs: Vec<FlexLevelJob<'dom, Node>>,
    has_text_runs: bool,
}

enum FlexLevelJob<'dom, Node> {
    /// Or pseudo-element
    Element {
        info: NodeAndStyleInfo<Node>,
        display: DisplayGeneratingBox,
        contents: Contents,
        box_slot: BoxSlot<'dom>,
    },
    TextRuns(Vec<TextRun<'dom, Node>>),
}

struct TextRun<'dom, Node> {
    info: NodeAndStyleInfo<Node>,
    text: Cow<'dom, str>,
}

impl<'a, 'dom, Node: 'dom> TraversalHandler<'dom, Node> for FlexContainerBuilder<'a, 'dom, Node>
where
    Node: NodeExt<'dom>,
{
    fn handle_text(&mut self, info: &NodeAndStyleInfo<Node>, text: Cow<'dom, str>) {
        self.contiguous_text_runs.push(TextRun {
            info: info.clone(),
            text,
        })
    }

    /// Or pseudo-element
    fn handle_element(
        &mut self,
        info: &NodeAndStyleInfo<Node>,
        display: DisplayGeneratingBox,
        contents: Contents,
        box_slot: BoxSlot<'dom>,
    ) {
        // FIXME: are text runs considered "contiguous" if they are only separated
        // by an out-of-flow abspos element?
        // (That is, are they wrapped in the same anonymous flex item, or each its own?)
        self.wrap_any_text_in_anonymous_block_container();

        self.jobs.push(FlexLevelJob::Element {
            info: info.clone(),
            display,
            contents,
            box_slot,
        })
    }
}

/// https://drafts.csswg.org/css-text/#white-space
fn is_only_document_white_space<Node>(run: &TextRun<'_, Node>) -> bool {
    // FIXME: is this the right definition? See
    // https://github.com/w3c/csswg-drafts/issues/5146
    // https://github.com/w3c/csswg-drafts/issues/5147
    run.text
        .bytes()
        .all(|byte| matches!(byte, b' ' | b'\n' | b'\t'))
}

impl<'a, 'dom, Node: 'dom> FlexContainerBuilder<'a, 'dom, Node>
where
    Node: NodeExt<'dom>,
{
    fn wrap_any_text_in_anonymous_block_container(&mut self) {
        let runs = std::mem::take(&mut self.contiguous_text_runs);
        if runs.iter().all(is_only_document_white_space) {
            // There is no text run, or they all only contain document white space characters
        } else {
            self.jobs.push(FlexLevelJob::TextRuns(runs));
            self.has_text_runs = true;
        }
    }

    fn finish(mut self) -> FlexContainer {
        self.wrap_any_text_in_anonymous_block_container();

        let anonymous_style = if self.has_text_runs {
            Some(
                self.context
                    .shared_context()
                    .stylist
                    .style_for_anonymous::<Node::ConcreteElement>(
                        &self.context.shared_context().guards,
                        &style::selector_parser::PseudoElement::ServoText,
                        &self.info.style,
                    ),
            )
        } else {
            None
        };

        let mut children = std::mem::take(&mut self.jobs)
            .into_par_iter()
            .map(|job| match job {
                FlexLevelJob::TextRuns(runs) => ArcRefCell::new(FlexLevelBox::FlexItem(
                    IndependentFormattingContext::construct_for_text_runs(
                        &self
                            .info
                            .new_replacing_style(anonymous_style.clone().unwrap()),
                        runs.into_iter().map(|run| crate::flow::inline::TextRun {
                            tag: Tag::from_node_and_style_info(&run.info),
                            text: run.text.into(),
                            parent_style: run.info.style,
                        }),
                        self.text_decoration_line,
                    ),
                )),
                FlexLevelJob::Element {
                    info,
                    display,
                    contents,
                    box_slot,
                } => {
                    let display_inside = match display {
                        DisplayGeneratingBox::OutsideInside { inside, .. } => inside,
                    };
                    let box_ = if info.style.get_box().position.is_absolutely_positioned() {
                        // https://drafts.csswg.org/css-flexbox/#abspos-items
                        ArcRefCell::new(FlexLevelBox::OutOfFlowAbsolutelyPositionedBox(
                            ArcRefCell::new(AbsolutelyPositionedBox::construct(
                                self.context,
                                &info,
                                display_inside,
                                contents,
                            )),
                        ))
                    } else {
                        ArcRefCell::new(FlexLevelBox::FlexItem(
                            IndependentFormattingContext::construct(
                                self.context,
                                &info,
                                display_inside,
                                contents,
                                self.text_decoration_line,
                            ),
                        ))
                    };
                    box_slot.set(LayoutBox::FlexLevel(box_.clone()));
                    box_
                },
            })
            .collect::<Vec<_>>();

        // https://drafts.csswg.org/css-flexbox/#order-modified-document-order
        children.sort_by_key(|child| match &*child.borrow() {
            FlexLevelBox::FlexItem(item) => item.style().clone_order(),

            // “Absolutely-positioned children of a flex container are treated
            //  as having order: 0 for the purpose of determining their painting order
            //  relative to flex items.”
            FlexLevelBox::OutOfFlowAbsolutelyPositionedBox(_) => 0,
        });

        FlexContainer { children }
    }
}
