/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::html::form_controls::input_type::SpecificInputType;

#[derive(Default, JSTraceable, MallocSizeOf, PartialEq)]
pub(crate) struct HiddenInputType();

impl SpecificInputType for HiddenInputType {}
