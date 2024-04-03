/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use webxr_api::{InputId, InputSource};

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::XRInputSourceArrayBinding::XRInputSourceArrayMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::event::Event;
use crate::dom::globalscope::GlobalScope;
use crate::dom::xrinputsource::XRInputSource;
use crate::dom::xrinputsourceschangeevent::XRInputSourcesChangeEvent;
use crate::dom::xrsession::XRSession;

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
        reflect_dom_object(Box::new(XRInputSourceArray::new_inherited()), global)
    }

    pub fn add_input_sources(&self, session: &XRSession, inputs: &[InputSource]) {
        let mut input_sources = self.input_sources.borrow_mut();
        let global = self.global();

        let mut added = vec![];
        for info in inputs {
            // This is quadratic, but won't be a problem for the only case
            // where we add multiple input sources (the initial input sources case)
            debug_assert!(
                !input_sources.iter().any(|i| i.id() == info.id),
                "Should never add a duplicate input id!"
            );
            let input = XRInputSource::new(&global, session, info.clone());
            input_sources.push(Dom::from_ref(&input));
            added.push(input);
        }

        let event = XRInputSourcesChangeEvent::new(
            &global,
            atom!("inputsourceschange"),
            false,
            true,
            session,
            &added,
            &[],
        );
        // Release the refcell guard
        drop(input_sources);
        event.upcast::<Event>().fire(session.upcast());
    }

    pub fn remove_input_source(&self, session: &XRSession, id: InputId) {
        let mut input_sources = self.input_sources.borrow_mut();
        let global = self.global();
        let removed = if let Some(i) = input_sources.iter().find(|i| i.id() == id) {
            [DomRoot::from_ref(&**i)]
        } else {
            return;
        };

        let event = XRInputSourcesChangeEvent::new(
            &global,
            atom!("inputsourceschange"),
            false,
            true,
            session,
            &[],
            &removed,
        );
        input_sources.retain(|i| i.id() != id);
        // release the refcell guard
        drop(input_sources);
        event.upcast::<Event>().fire(session.upcast());
    }

    pub fn add_remove_input_source(&self, session: &XRSession, id: InputId, info: InputSource) {
        let mut input_sources = self.input_sources.borrow_mut();
        let global = self.global();
        let root;
        let removed = if let Some(i) = input_sources.iter().find(|i| i.id() == id) {
            root = [DomRoot::from_ref(&**i)];
            &root as &[_]
        } else {
            warn!("Could not find removed input source with id {:?}", id);
            &[]
        };
        input_sources.retain(|i| i.id() != id);
        let input = XRInputSource::new(&global, session, info);
        input_sources.push(Dom::from_ref(&input));

        let added = [input];

        let event = XRInputSourcesChangeEvent::new(
            &global,
            atom!("inputsourceschange"),
            false,
            true,
            session,
            &added,
            removed,
        );
        // release the refcell guard
        drop(input_sources);
        event.upcast::<Event>().fire(session.upcast());
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
    /// <https://immersive-web.github.io/webxr/#dom-xrinputsourcearray-length>
    fn Length(&self) -> u32 {
        self.input_sources.borrow().len() as u32
    }

    /// <https://immersive-web.github.io/webxr/#xrinputsourcearray>
    fn IndexedGetter(&self, n: u32) -> Option<DomRoot<XRInputSource>> {
        self.input_sources
            .borrow()
            .get(n as usize)
            .map(|x| DomRoot::from_ref(&**x))
    }
}
