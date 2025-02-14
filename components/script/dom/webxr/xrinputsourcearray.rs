/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use webxr_api::{InputId, InputSource};

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::XRInputSourceArrayBinding::XRInputSourceArrayMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::{reflect_dom_object, DomGlobal, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::event::Event;
use crate::dom::globalscope::GlobalScope;
use crate::dom::xrinputsource::XRInputSource;
use crate::dom::xrinputsourceschangeevent::XRInputSourcesChangeEvent;
use crate::dom::xrsession::XRSession;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct XRInputSourceArray {
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

    pub(crate) fn new(global: &GlobalScope) -> DomRoot<XRInputSourceArray> {
        reflect_dom_object(
            Box::new(XRInputSourceArray::new_inherited()),
            global,
            CanGc::note(),
        )
    }

    pub(crate) fn add_input_sources(
        &self,
        session: &XRSession,
        inputs: &[InputSource],
        can_gc: CanGc,
    ) {
        let global = self.global();

        let mut added = vec![];
        for info in inputs {
            // This is quadratic, but won't be a problem for the only case
            // where we add multiple input sources (the initial input sources case)
            debug_assert!(
                !self
                    .input_sources
                    .borrow()
                    .iter()
                    .any(|i| i.id() == info.id),
                "Should never add a duplicate input id!"
            );
            let input = XRInputSource::new(&global, session, info.clone(), can_gc);
            self.input_sources.borrow_mut().push(Dom::from_ref(&input));
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
            can_gc,
        );
        event.upcast::<Event>().fire(session.upcast(), can_gc);
    }

    pub(crate) fn remove_input_source(&self, session: &XRSession, id: InputId, can_gc: CanGc) {
        let global = self.global();
        let removed = if let Some(i) = self.input_sources.borrow().iter().find(|i| i.id() == id) {
            i.gamepad().update_connected(false, false, can_gc);
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
            can_gc,
        );
        self.input_sources.borrow_mut().retain(|i| i.id() != id);
        event.upcast::<Event>().fire(session.upcast(), can_gc);
    }

    pub(crate) fn add_remove_input_source(
        &self,
        session: &XRSession,
        id: InputId,
        info: InputSource,
        can_gc: CanGc,
    ) {
        let global = self.global();
        let root;
        let removed = if let Some(i) = self.input_sources.borrow().iter().find(|i| i.id() == id) {
            i.gamepad().update_connected(false, false, can_gc);
            root = [DomRoot::from_ref(&**i)];
            &root as &[_]
        } else {
            warn!("Could not find removed input source with id {:?}", id);
            &[]
        };
        self.input_sources.borrow_mut().retain(|i| i.id() != id);
        let input = XRInputSource::new(&global, session, info, can_gc);
        self.input_sources.borrow_mut().push(Dom::from_ref(&input));

        let added = [input];

        let event = XRInputSourcesChangeEvent::new(
            &global,
            atom!("inputsourceschange"),
            false,
            true,
            session,
            &added,
            removed,
            can_gc,
        );
        event.upcast::<Event>().fire(session.upcast(), can_gc);
    }

    pub(crate) fn find(&self, id: InputId) -> Option<DomRoot<XRInputSource>> {
        self.input_sources
            .borrow()
            .iter()
            .find(|x| x.id() == id)
            .map(|x| DomRoot::from_ref(&**x))
    }
}

impl XRInputSourceArrayMethods<crate::DomTypeHolder> for XRInputSourceArray {
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
