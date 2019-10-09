/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::XRInputSourceArrayBinding;
use crate::dom::bindings::codegen::Bindings::XRInputSourceArrayBinding::XRInputSourceArrayMethods;
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::globalscope::GlobalScope;
use crate::dom::xrinputsource::XRInputSource;
use crate::dom::xrsession::XRSession;
use dom_struct::dom_struct;
use webxr_api::InputId;

#[dom_struct]
pub struct XRInputSourceArray {
    reflector_: Reflector,
    input_sources: DomRefCell<Vec<Dom<XRInputSource>>>,
}

impl XRInputSourceArray {
    fn new_inherited() -> XRInputSourceArray {
        XRInputSourceArray {
            reflector_: Reflector::new(),
            input_sources: DomRefCell::new(vec![]),
        }
    }

    pub fn new(global: &GlobalScope) -> DomRoot<XRInputSourceArray> {
        reflect_dom_object(
            Box::new(XRInputSourceArray::new_inherited()),
            global,
            XRInputSourceArrayBinding::Wrap,
        )
    }

    pub fn set_initial_inputs(&self, session: &XRSession) {
        let mut input_sources = self.input_sources.borrow_mut();
        let global = self.global();
        session.with_session(|sess| {
            for info in sess.initial_inputs() {
                // XXXManishearth we should be able to listen for updates
                // to the input sources
                let input = XRInputSource::new(&global, &session, *info);
                input_sources.push(Dom::from_ref(&input));
            }
        });
    }

    pub fn find(&self, id: InputId) -> Option<DomRoot<XRInputSource>> {
        self.input_sources
            .borrow()
            .iter()
            .find(|x| x.id() == id)
            .map(|x| DomRoot::from_ref(&**x))
    }
}

impl XRInputSourceArrayMethods for XRInputSourceArray {
    /// https://immersive-web.github.io/webxr/#dom-xrinputsourcearray-length
    fn Length(&self) -> u32 {
        self.input_sources.borrow().len() as u32
    }

    /// https://immersive-web.github.io/webxr/#xrinputsourcearray
    fn IndexedGetter(&self, n: u32) -> Option<DomRoot<XRInputSource>> {
        self.input_sources
            .borrow()
            .get(n as usize)
            .map(|x| DomRoot::from_ref(&**x))
    }
}
