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
use crate::positioned::AbsolutelyPositionedBox;

mod construct;
#[path = "layout_taffy.rs"]
mod layout;

// #[path = "layout.rs"]
// mod old_layout;

#[derive(Debug, Serialize)]
pub(crate) struct FlexContainer {
    children: Vec<ArcRefCell<FlexLevelBox>>,
}

#[derive(Serialize)]
pub(crate) struct FlexLevelBox {
    pub(crate) taffy_layout_cache: taffy::Cache,
    pub(crate) taffy_layout: taffy::Layout,
    pub(crate) child_fragments: Vec<Fragment>,
    #[serde(skip_serializing)]
    pub(crate) style: Arc<ComputedValues>,
    pub(crate) flex_level_box: FlexLevelBoxInner,
}

impl fmt::Debug for FlexLevelBox {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FlexLevelBox")
            .field("taffy_layout_cache", &self.taffy_layout_cache)
            .field("taffy_layout", &self.taffy_layout)
            .field("child_fragments", &self.child_fragments.len())
            .field("style", &self.style)
            .field("flex_level_box", &self.flex_level_box)
            .finish()
    }
}

impl FlexLevelBox {
    fn new(flex_level_box: FlexLevelBoxInner) -> Self {
        let style: Arc<ComputedValues> = match &flex_level_box {
            FlexLevelBoxInner::FlexItem(item) => item.style().clone(),
            FlexLevelBoxInner::OutOfFlowAbsolutelyPositionedBox(absbox) => {
                (*absbox).borrow().context.style().clone()
            },
        };

        Self {
            taffy_layout_cache: Default::default(),
            taffy_layout: Default::default(),
            child_fragments: Vec::new(),
            style,
            flex_level_box,
        }
    }
}

#[derive(Debug, Serialize)]
pub(crate) enum FlexLevelBoxInner {
    FlexItem(IndependentFormattingContext),
    OutOfFlowAbsolutelyPositionedBox(ArcRefCell<AbsolutelyPositionedBox>),
}
