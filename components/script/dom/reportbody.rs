/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use dom_struct::dom_struct;

use crate::dom::bindings::codegen::Bindings::ReportBodyBinding::ReportBodyMethods;
use crate::dom::bindings::reflector::Reflector;

#[dom_struct]
pub(crate) struct ReportBody {
    reflector_: Reflector,

    body: String,
}

impl ReportBody {
    pub(crate) fn new_inherited(body: String) -> Self {
        Self {
            reflector_: Reflector::new(),
            body,
        }
    }
}

impl ReportBodyMethods<crate::DomTypeHolder> for ReportBody {}
