/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
mod layout;
mod stylo_taffy;
use std::fmt;

use serde::Serialize;
use servo_arc::Arc;
use style::properties::ComputedValues;
use style::values::computed::TextDecorationLine;
use stylo_taffy::TaffyStyloStyle;

use crate::cell::ArcRefCell;
use crate::construct_modern::{ModernContainerBuilder, ModernItemKind};
use crate::context::LayoutContext;
use crate::dom::{LayoutBox, NodeExt};
use crate::dom_traversal::{NodeAndStyleInfo, NonReplacedContents};
use crate::formatting_contexts::IndependentFormattingContext;
use crate::fragment_tree::Fragment;
use crate::positioned::{AbsolutelyPositionedBox, PositioningContext};

#[derive(Debug, Serialize)]
pub(crate) struct TaffyContainer {
    children: Vec<ArcRefCell<TaffyItemBox>>,
    #[serde(skip_serializing)]
    style: Arc<ComputedValues>,
}

impl TaffyContainer {
    pub fn construct<'dom>(
        context: &LayoutContext,
        info: &NodeAndStyleInfo<impl NodeExt<'dom>>,
        contents: NonReplacedContents,
        propagated_text_decoration_line: TextDecorationLine,
    ) -> Self {
        let text_decoration_line =
            propagated_text_decoration_line | info.style.clone_text_decoration_line();
        let mut builder = ModernContainerBuilder::new(context, info, text_decoration_line);
        contents.traverse(context, info, &mut builder);
        let items = builder.finish();

        let children = items
            .into_iter()
            .map(|item| {
                let box_ = match item.kind {
                    ModernItemKind::InFlow => ArcRefCell::new(TaffyItemBox::new(
                        TaffyItemBoxInner::InFlowBox(item.formatting_context),
                    )),
                    ModernItemKind::OutOfFlow => {
                        let abs_pos_box =
                            ArcRefCell::new(AbsolutelyPositionedBox::new(item.formatting_context));
                        ArcRefCell::new(TaffyItemBox::new(
                            TaffyItemBoxInner::OutOfFlowAbsolutelyPositionedBox(abs_pos_box),
                        ))
                    },
                };

                if let Some(box_slot) = item.box_slot {
                    box_slot.set(LayoutBox::TaffyItemBox(box_.clone()));
                }

                box_
            })
            .collect();

        Self {
            children,
            style: info.style.clone(),
        }
    }
}

#[derive(Serialize)]
pub(crate) struct TaffyItemBox {
    pub(crate) taffy_layout: taffy::Layout,
    pub(crate) child_fragments: Vec<Fragment>,
    #[serde(skip_serializing)]
    pub(crate) positioning_context: PositioningContext,
    #[serde(skip_serializing)]
    pub(crate) style: Arc<ComputedValues>,
    pub(crate) taffy_level_box: TaffyItemBoxInner,
}

#[derive(Debug, Serialize)]
pub(crate) enum TaffyItemBoxInner {
    InFlowBox(IndependentFormattingContext),
    OutOfFlowAbsolutelyPositionedBox(ArcRefCell<AbsolutelyPositionedBox>),
}

impl fmt::Debug for TaffyItemBox {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TaffyItemBox")
            .field("taffy_layout", &self.taffy_layout)
            .field("child_fragments", &self.child_fragments.len())
            .field("style", &self.style)
            .field("taffy_level_box", &self.taffy_level_box)
            .finish()
    }
}

impl TaffyItemBox {
    fn new(inner: TaffyItemBoxInner) -> Self {
        let style: Arc<ComputedValues> = match &inner {
            TaffyItemBoxInner::InFlowBox(item) => item.style().clone(),
            TaffyItemBoxInner::OutOfFlowAbsolutelyPositionedBox(absbox) => {
                (*absbox).borrow().context.style().clone()
            },
        };

        Self {
            taffy_layout: Default::default(),
            child_fragments: Vec::new(),
            positioning_context: PositioningContext::new_for_containing_block_for_all_descendants(),
            style,
            taffy_level_box: inner,
        }
    }
}
