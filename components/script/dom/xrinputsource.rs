/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::XRInputSourceBinding;
use crate::dom::bindings::reflector::{Reflector, reflect_dom_object};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::globalscope::GlobalScope;
use crate::dom::xrsession::XRSession;
use dom_struct::dom_struct;

#[dom_struct]
pub struct XRInputSource {
    reflector: Reflector,
    session: Dom<XRSession>,
}

impl XRInputSource {
    pub fn new_inherited(session: &XRSession) -> XRInputSource {
        XRInputSource {
            reflector: Reflector::new(),
            session: Dom::from_ref(session),
        }
    }

    pub fn new(global: &GlobalScope, session: &XRSession) -> DomRoot<XRInputSource> {
        reflect_dom_object(
            Box::new(XRInputSource::new_inherited(session)),
            global,
            XRInputSourceBinding::Wrap,
        )
    }
}
