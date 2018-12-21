/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::XRViewBinding;
use crate::dom::bindings::codegen::Bindings::XRViewBinding::{XREye, XRViewMethods};
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::globalscope::GlobalScope;
use crate::dom::xrsession::XRSession;
use dom_struct::dom_struct;

#[dom_struct]
pub struct XRView {
    reflector_: Reflector,
    session: Dom<XRSession>,
    eye: XREye,
}

impl XRView {
    fn new_inherited(session: &XRSession, eye: XREye) -> XRView {
        XRView {
            reflector_: Reflector::new(),
            session: Dom::from_ref(session),
            eye
        }
    }

    pub fn new(global: &GlobalScope, session: &XRSession, eye: XREye) -> DomRoot<XRView> {
        reflect_dom_object(
            Box::new(XRView::new_inherited(session, eye)),
            global,
            XRViewBinding::Wrap,
        )
    }

    pub fn session(&self) -> &XRSession {
        &self.session
    }
}

impl XRViewMethods for XRView {
    /// https://immersive-web.github.io/webxr/#dom-xrview-eye
    fn Eye(&self) -> XREye {
        self.eye
    }
}