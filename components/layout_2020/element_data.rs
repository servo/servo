/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::cell::ArcRefCell;
use crate::flow::inline::InlineLevelBox;
use crate::flow::BlockLevelBox;
use atomic_refcell::AtomicRefCell;
use servo_arc::Arc;

#[derive(Default)]
pub struct LayoutDataForElement {
    pub(super) self_box: Arc<AtomicRefCell<Option<LayoutBox>>>,
    pub(super) pseudo_elements: Option<Box<PseudoElementBoxes>>,
}

#[derive(Default)]
pub(super) struct PseudoElementBoxes {
    pub before: Arc<AtomicRefCell<Option<LayoutBox>>>,
    pub after: Arc<AtomicRefCell<Option<LayoutBox>>>,
}

pub(super) enum LayoutBox {
    DisplayContents,
    BlockLevel(ArcRefCell<BlockLevelBox>),
    InlineLevel(ArcRefCell<InlineLevelBox>),
}
