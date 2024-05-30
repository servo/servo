/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::fmt;

use serde::Serialize;
use servo_arc::Arc;
use style::properties::ComputedValues;

use crate::cell::ArcRefCell;
use crate::formatting_contexts::IndependentFormattingContext;
use crate::fragment_tree::Fragment;
use crate::positioned::{AbsolutelyPositionedBox, PositioningContext};

mod construct;
mod layout;

#[derive(Debug, Serialize)]
pub(crate) struct TaffyContainer {
    children: Vec<ArcRefCell<TaffyItemBox>>,
}

#[derive(Serialize)]
pub(crate) struct TaffyItemBox {
    pub(crate) taffy_layout_cache: taffy::Cache,
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
            .field("taffy_layout_cache", &self.taffy_layout_cache)
            .field("taffy_layout", &self.taffy_layout)
            .field("child_fragments", &self.child_fragments.len())
            .field("style", &self.style)
            .field("flex_level_box", &self.taffy_level_box)
            .finish()
    }
}

impl TaffyItemBox {
    fn new(flex_level_box: TaffyItemBoxInner) -> Self {
        let style: Arc<ComputedValues> = match &flex_level_box {
            TaffyItemBoxInner::InFlowBox(item) => item.style().clone(),
            TaffyItemBoxInner::OutOfFlowAbsolutelyPositionedBox(absbox) => {
                (*absbox).borrow().context.style().clone()
            },
        };

        Self {
            taffy_layout_cache: Default::default(),
            taffy_layout: Default::default(),
            child_fragments: Vec::new(),
            positioning_context: PositioningContext::new_for_containing_block_for_all_descendants(),
            style,
            taffy_level_box: flex_level_box,
        }
    }
}
