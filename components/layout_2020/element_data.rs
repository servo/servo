/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::cell::ArcRefCell;
use crate::flexbox::FlexLevelBox;
use crate::flow::inline::InlineLevelBox;
use crate::flow::BlockLevelBox;

#[derive(Default)]
pub struct LayoutDataForElement {
    pub(super) self_box: ArcRefCell<Option<LayoutBox>>,
    pub(super) pseudo_before_box: ArcRefCell<Option<LayoutBox>>,
    pub(super) pseudo_after_box: ArcRefCell<Option<LayoutBox>>,
}

pub(super) enum LayoutBox {
    DisplayContents,
    BlockLevel(ArcRefCell<BlockLevelBox>),
    InlineLevel(ArcRefCell<InlineLevelBox>),
    FlexLevel(ArcRefCell<FlexLevelBox>),
}
