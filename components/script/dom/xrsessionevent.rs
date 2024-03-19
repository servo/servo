/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::rust::HandleObject;
use servo_atoms::Atom;

use crate::dom::bindings::codegen::Bindings::EventBinding::Event_Binding::EventMethods;
use crate::dom::bindings::codegen::Bindings::XRSessionEventBinding::{self, XRSessionEventMethods};
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::{reflect_dom_object_with_proto, DomObject};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::event::Event;
use crate::dom::globalscope::GlobalScope;
use crate::dom::window::Window;
use crate::dom::xrsession::XRSession;

#[dom_struct]
pub struct XRSessionEvent {
    event: Event,
    session: Dom<XRSession>,
}

impl XRSessionEvent {
    #[allow(crown::unrooted_must_root)]
    fn new_inherited(session: &XRSession) -> XRSessionEvent {
        XRSessionEvent {
            event: Event::new_inherited(),
            session: Dom::from_ref(session),
        }
    }

    pub fn new(
        global: &GlobalScope,
        type_: Atom,
        bubbles: bool,
        cancelable: bool,
        session: &XRSession,
    ) -> DomRoot<XRSessionEvent> {
        Self::new_with_proto(global, None, type_, bubbles, cancelable, session)
    }

    fn new_with_proto(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        type_: Atom,
        bubbles: bool,
        cancelable: bool,
        session: &XRSession,
    ) -> DomRoot<XRSessionEvent> {
        let trackevent = reflect_dom_object_with_proto(
            Box::new(XRSessionEvent::new_inherited(session)),
            global,
            proto,
        );
        {
            let event = trackevent.upcast::<Event>();
            event.init_event(type_, bubbles, cancelable);
        }
        trackevent
    }

    #[allow(non_snake_case)]
    pub fn Constructor(
        window: &Window,
        proto: Option<HandleObject>,
        type_: DOMString,
        init: &XRSessionEventBinding::XRSessionEventInit,
    ) -> Fallible<DomRoot<XRSessionEvent>> {
        Ok(XRSessionEvent::new_with_proto(
            &window.global(),
            proto,
            Atom::from(type_),
            init.parent.bubbles,
            init.parent.cancelable,
            &init.session,
        ))
    }
}

impl XRSessionEventMethods for XRSessionEvent {
    // https://immersive-web.github.io/webxr/#dom-xrsessioneventinit-session
    fn Session(&self) -> DomRoot<XRSession> {
        DomRoot::from_ref(&*self.session)
    }

    // https://dom.spec.whatwg.org/#dom-event-istrusted
    fn IsTrusted(&self) -> bool {
        self.event.IsTrusted()
    }
}
