/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::JSContext;
use script_bindings::cell::DomRefCell;
use script_bindings::reflector::{Reflector, reflect_dom_object};
use webxr_api::{InputId, InputSource};

use crate::dom::bindings::codegen::Bindings::XRInputSourceArrayBinding::XRInputSourceArrayMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::event::Event;
use crate::dom::window::Window;
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

    pub(crate) fn new(window: &Window, can_gc: CanGc) -> DomRoot<XRInputSourceArray> {
        reflect_dom_object(
            Box::new(XRInputSourceArray::new_inherited()),
            window,
            can_gc,
        )
    }

    pub(crate) fn add_input_sources(
        &self,
        cx: &mut JSContext,
        session: &XRSession,
        inputs: &[InputSource],
    ) {
        let global = self.global();
        let window = global.as_window();

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
            let input = XRInputSource::new(cx, window, session, info.clone());
            self.input_sources.borrow_mut().push(Dom::from_ref(&input));
            added.push(input);
        }

        let event = XRInputSourcesChangeEvent::new(
            window,
            atom!("inputsourceschange"),
            false,
            true,
            session,
            &added,
            &[],
            CanGc::from_cx(cx),
        );
        event.upcast::<Event>().fire(cx, session.upcast());
    }

    pub(crate) fn remove_input_source(&self, cx: &mut JSContext, session: &XRSession, id: InputId) {
        let global = self.global();
        let window = global.as_window();
        let removed = if let Some(i) = self.input_sources.borrow().iter().find(|i| i.id() == id) {
            i.gamepad().update_connected(cx, false, false);
            [DomRoot::from_ref(&**i)]
        } else {
            return;
        };

        let event = XRInputSourcesChangeEvent::new(
            window,
            atom!("inputsourceschange"),
            false,
            true,
            session,
            &[],
            &removed,
            CanGc::from_cx(cx),
        );
        self.input_sources.borrow_mut().retain(|i| i.id() != id);
        event.upcast::<Event>().fire(cx, session.upcast());
    }

    pub(crate) fn add_remove_input_source(
        &self,
        cx: &mut JSContext,
        session: &XRSession,
        id: InputId,
        info: InputSource,
    ) {
        let global = self.global();
        let window = global.as_window();
        let root;
        let removed = if let Some(i) = self.input_sources.borrow().iter().find(|i| i.id() == id) {
            i.gamepad().update_connected(cx, false, false);
            root = [DomRoot::from_ref(&**i)];
            &root as &[_]
        } else {
            warn!("Could not find removed input source with id {:?}", id);
            &[]
        };
        self.input_sources.borrow_mut().retain(|i| i.id() != id);
        let input = XRInputSource::new(cx, window, session, info);
        self.input_sources.borrow_mut().push(Dom::from_ref(&input));

        let added = [input];

        let event = XRInputSourcesChangeEvent::new(
            window,
            atom!("inputsourceschange"),
            false,
            true,
            session,
            &added,
            removed,
            CanGc::from_cx(cx),
        );
        event.upcast::<Event>().fire(cx, session.upcast());
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
