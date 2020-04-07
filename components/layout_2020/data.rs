/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::element_data::LayoutDataForElement;
use atomic_refcell::AtomicRefCell;
use script_layout_interface::StyleData;

pub struct StyleAndLayoutData<'dom> {
    pub style_data: &'dom StyleData,
    pub(super) layout_data: &'dom AtomicRefCell<LayoutDataForElement>,
}
