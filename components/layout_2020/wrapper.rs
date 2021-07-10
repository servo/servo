/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![allow(unsafe_code)]

use crate::data::StyleAndLayoutData;
use script_layout_interface::wrapper_traits::GetStyleAndOpaqueLayoutData;

pub trait GetStyleAndLayoutData<'dom> {
    fn get_style_and_layout_data(self) -> Option<StyleAndLayoutData<'dom>>;
}

impl<'dom, T> GetStyleAndLayoutData<'dom> for T
where
    T: GetStyleAndOpaqueLayoutData<'dom>,
{
    fn get_style_and_layout_data(self) -> Option<StyleAndLayoutData<'dom>> {
        self.get_style_and_opaque_layout_data()
            .map(|data| StyleAndLayoutData {
                style_data: &data.style_data,
                layout_data: data.generic_data.downcast_ref().unwrap(),
            })
    }
}
