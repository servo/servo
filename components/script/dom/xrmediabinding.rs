/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::reflector::Reflector;
use crate::dom::bindings::root::Dom;
use crate::dom::bindings::root::DomRoot;
use crate::dom::window::Window;
use crate::dom::xrsession::XRSession;
use dom_struct::dom_struct;

#[dom_struct]
pub struct XRMediaBinding {
    reflector: Reflector,
    session: Dom<XRSession>,
}

impl XRMediaBinding {
    pub fn new_inherited(session: &XRSession) -> XRMediaBinding {
        XRMediaBinding {
            reflector: Reflector::new(),
            session: Dom::from_ref(session),
        }
    }

    pub fn new(global: &Window, session: &XRSession) -> DomRoot<XRMediaBinding> {
        reflect_dom_object(Box::new(XRMediaBinding::new_inherited(session)), global)
    }

    #[allow(non_snake_case)]
    pub fn Constructor(global: &Window, session: &XRSession) -> DomRoot<XRMediaBinding> {
        XRMediaBinding::new(global, session)
    }
}
