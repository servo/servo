/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::element_data::LayoutDataForElement;
use atomic_refcell::AtomicRefCell;
use script_layout_interface::StyleData;

#[repr(C)]
pub struct StyleAndLayoutData {
    pub style_data: StyleData,
    pub(super) layout_data: AtomicRefCell<LayoutDataForElement>,
}

impl StyleAndLayoutData {
    pub fn new() -> Self {
        Self {
            style_data: StyleData::new(),
            layout_data: Default::default(),
        }
    }
}
