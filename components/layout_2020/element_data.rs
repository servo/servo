/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::cell::ArcRefCell;
use crate::flow::inline::InlineLevelBox;
use crate::flow::BlockLevelBox;

#[derive(Default)]
pub struct LayoutDataForElement {
    pub(super) self_box: ArcRefCell<Option<LayoutBox>>,
    pub(super) pseudo_elements: Option<Box<PseudoElementBoxes>>,
}

#[derive(Default)]
pub(super) struct PseudoElementBoxes {
    pub before: ArcRefCell<Option<LayoutBox>>,
    pub after: ArcRefCell<Option<LayoutBox>>,
}

pub(super) enum LayoutBox {
    DisplayContents,
    BlockLevel(ArcRefCell<BlockLevelBox>),
    InlineLevel(ArcRefCell<InlineLevelBox>),
}
