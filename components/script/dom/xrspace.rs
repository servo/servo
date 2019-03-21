/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::XRSpaceBinding;
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::xrsession::XRSession;
use dom_struct::dom_struct;

#[dom_struct]
pub struct XRSpace {
    eventtarget: EventTarget,
    session: Dom<XRSession>,
}

impl XRSpace {
    pub fn new_inherited(session: &XRSession) -> XRSpace {
        XRSpace {
            eventtarget: EventTarget::new_inherited(),
            session: Dom::from_ref(session),
        }
    }

    #[allow(unused)]
    pub fn new(global: &GlobalScope, session: &XRSession) -> DomRoot<XRSpace> {
        reflect_dom_object(
            Box::new(XRSpace::new_inherited(session)),
            global,
            XRSpaceBinding::Wrap,
        )
    }
}
