/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Layout construction code that is shared between modern layout modes (Flexbox and CSS Grid)

use std::borrow::Cow;
use std::sync::LazyLock;

use rayon::iter::{IntoParallelIterator, ParallelIterator};
use style::selector_parser::PseudoElement;

use crate::PropagatedBoxTreeData;
use crate::context::LayoutContext;
use crate::dom::BoxSlot;
use crate::dom_traversal::{Contents, NodeAndStyleInfo, TraversalHandler};
use crate::flow::inline::construct::InlineFormattingContextBuilder;
use crate::flow::{BlockContainer, BlockFormattingContext};
use crate::formatting_contexts::{
    IndependentFormattingContext, IndependentFormattingContextContents,
    IndependentNonReplacedContents,
};
use crate::layout_box_base::LayoutBoxBase;
use crate::style_ext::DisplayGeneratingBox;

/// A builder used for both flex and grid containers.
pub(crate) struct ModernContainerBuilder<'a, 'dom> {
    context: &'a LayoutContext<'a>,
    info: &'a NodeAndStyleInfo<'dom>,
    propagated_data: PropagatedBoxTreeData,
    contiguous_text_runs: Vec<ModernContainerTextRun<'dom>>,
    /// To be run in parallel with rayon in `finish`
    jobs: Vec<ModernContainerJob<'dom>>,
    has_text_runs: bool,
}

enum ModernContainerJob<'dom> {
    ElementOrPseudoElement {
        info: NodeAndStyleInfo<'dom>,
        display: DisplayGeneratingBox,
        contents: Contents,
        box_slot: BoxSlot<'dom>,
    },
    TextRuns(Vec<ModernContainerTextRun<'dom>>),
}

struct ModernContainerTextRun<'dom> {
    info: NodeAndStyleInfo<'dom>,
    text: Cow<'dom, str>,
}

impl ModernContainerTextRun<'_> {
    /// <https://drafts.csswg.org/css-text/#white-space>
    fn is_only_document_white_space(&self) -> bool {
        // FIXME: is this the right definition? See
        // https://github.com/w3c/csswg-drafts/issues/5146
        // https://github.com/w3c/csswg-drafts/issues/5147
        self.text
            .bytes()
            .all(|byte| matches!(byte, b' ' | b'\n' | b'\t'))
    }
}

pub(crate) enum ModernItemKind {
    InFlow,
    OutOfFlow,
}

pub(crate) struct ModernItem<'dom> {
    pub kind: ModernItemKind,
    pub order: i32,
    pub box_slot: Option<BoxSlot<'dom>>,
    pub formatting_context: IndependentFormattingContext,
}

impl<'dom> TraversalHandler<'dom> for ModernContainerBuilder<'_, 'dom> {
    fn handle_text(&mut self, info: &NodeAndStyleInfo<'dom>, text: Cow<'dom, str>) {
        self.contiguous_text_runs.push(ModernContainerTextRun {
            info: info.clone(),
            text,
        })
    }

    /// Or pseudo-element
    fn handle_element(
        &mut self,
        info: &NodeAndStyleInfo<'dom>,
        display: DisplayGeneratingBox,
        contents: Contents,
        box_slot: BoxSlot<'dom>,
    ) {
        self.wrap_any_text_in_anonymous_block_container();

        self.jobs.push(ModernContainerJob::ElementOrPseudoElement {
            info: info.clone(),
            display,
            contents,
            box_slot,
        })
    }
}

impl<'a, 'dom> ModernContainerBuilder<'a, 'dom> {
    pub fn new(
        context: &'a LayoutContext<'a>,
        info: &'a NodeAndStyleInfo<'dom>,
        propagated_data: PropagatedBoxTreeData,
    ) -> Self {
        ModernContainerBuilder {
            context,
            info,
            propagated_data: propagated_data.disallowing_percentage_table_columns(),
            contiguous_text_runs: Vec::new(),
            jobs: Vec::new(),
            has_text_runs: false,
        }
    }

    fn wrap_any_text_in_anonymous_block_container(&mut self) {
        let runs = std::mem::take(&mut self.contiguous_text_runs);
        if runs
            .iter()
            .all(ModernContainerTextRun::is_only_document_white_space)
        {
            // There is no text run, or they all only contain document white space characters
        } else {
            self.jobs.push(ModernContainerJob::TextRuns(runs));
            self.has_text_runs = true;
        }
    }

    pub(crate) fn finish(mut self) -> Vec<ModernItem<'dom>> {
        self.wrap_any_text_in_anonymous_block_container();

        let anonymous_info = LazyLock::new(|| {
            self.info
                .pseudo(self.context, PseudoElement::ServoAnonymousBox)
                .expect("Should always be able to construct info for anonymous boxes.")
        });
        let mut children: Vec<ModernItem> = std::mem::take(&mut self.jobs)
            .into_par_iter()
            .filter_map(|job| match job {
                ModernContainerJob::TextRuns(runs) => {
                    let mut inline_formatting_context_builder =
                        InlineFormattingContextBuilder::new();
                    for flex_text_run in runs.into_iter() {
                        inline_formatting_context_builder
                            .push_text(flex_text_run.text, &flex_text_run.info);
                    }

                    let inline_formatting_context = inline_formatting_context_builder.finish(
                        self.context,
                        self.propagated_data,
                        true,  /* has_first_formatted_line */
                        false, /* is_single_line_text_box */
                        self.info.style.writing_mode.to_bidi_level(),
                    )?;

                    let block_formatting_context = BlockFormattingContext::from_block_container(
                        BlockContainer::InlineFormattingContext(inline_formatting_context),
                    );
                    let info: &NodeAndStyleInfo = &anonymous_info;
                    let formatting_context = IndependentFormattingContext {
                        base: LayoutBoxBase::new(info.into(), info.style.clone()),
                        contents: IndependentFormattingContextContents::NonReplaced(
                            IndependentNonReplacedContents::Flow(block_formatting_context),
                        ),
                    };

                    Some(ModernItem {
                        kind: ModernItemKind::InFlow,
                        order: 0,
                        box_slot: None,
                        formatting_context,
                    })
                },
                ModernContainerJob::ElementOrPseudoElement {
                    info,
                    display,
                    contents,
                    box_slot,
                } => {
                    let is_abspos = info.style.get_box().position.is_absolutely_positioned();

                    // Text decorations are not propagated to any out-of-flow descendants. In addition,
                    // absolutes don't affect the size of ancestors so it is fine to allow descendent
                    // tables to resolve percentage columns.
                    let propagated_data = match is_abspos {
                        false => self.propagated_data,
                        true => PropagatedBoxTreeData::default(),
                    };

                    let formatting_context = IndependentFormattingContext::construct(
                        self.context,
                        &info,
                        display.display_inside(),
                        contents,
                        propagated_data,
                    );

                    if is_abspos {
                        Some(ModernItem {
                            kind: ModernItemKind::OutOfFlow,
                            order: 0,
                            box_slot: Some(box_slot),
                            formatting_context,
                        })
                    } else {
                        Some(ModernItem {
                            kind: ModernItemKind::InFlow,
                            order: info.style.clone_order(),
                            box_slot: Some(box_slot),
                            formatting_context,
                        })
                    }
                },
            })
            .collect::<Vec<_>>();

        // https://drafts.csswg.org/css-flexbox/#order-modified-document-order
        children.sort_by_key(|child| child.order);

        children
    }
}
