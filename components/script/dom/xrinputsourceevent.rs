/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::rust::HandleObject;
use servo_atoms::Atom;

use crate::dom::bindings::codegen::Bindings::EventBinding::Event_Binding::EventMethods;
use crate::dom::bindings::codegen::Bindings::XRInputSourceEventBinding::{
    self, XRInputSourceEventMethods,
};
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::{reflect_dom_object_with_proto, DomObject};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::event::Event;
use crate::dom::globalscope::GlobalScope;
use crate::dom::window::Window;
use crate::dom::xrframe::XRFrame;
use crate::dom::xrinputsource::XRInputSource;

#[dom_struct]
pub struct XRInputSourceEvent {
    event: Event,
    frame: Dom<XRFrame>,
    source: Dom<XRInputSource>,
}

impl XRInputSourceEvent {
    #[allow(crown::unrooted_must_root)]
    fn new_inherited(frame: &XRFrame, source: &XRInputSource) -> XRInputSourceEvent {
        XRInputSourceEvent {
            event: Event::new_inherited(),
            frame: Dom::from_ref(frame),
            source: Dom::from_ref(source),
        }
    }

    pub fn new(
        global: &GlobalScope,
        type_: Atom,
        bubbles: bool,
        cancelable: bool,
        frame: &XRFrame,
        source: &XRInputSource,
    ) -> DomRoot<XRInputSourceEvent> {
        Self::new_with_proto(global, None, type_, bubbles, cancelable, frame, source)
    }

    fn new_with_proto(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        type_: Atom,
        bubbles: bool,
        cancelable: bool,
        frame: &XRFrame,
        source: &XRInputSource,
    ) -> DomRoot<XRInputSourceEvent> {
        let trackevent = reflect_dom_object_with_proto(
            Box::new(XRInputSourceEvent::new_inherited(frame, source)),
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
        init: &XRInputSourceEventBinding::XRInputSourceEventInit,
    ) -> Fallible<DomRoot<XRInputSourceEvent>> {
        Ok(XRInputSourceEvent::new_with_proto(
            &window.global(),
            proto,
            Atom::from(type_),
            init.parent.bubbles,
            init.parent.cancelable,
            &init.frame,
            &init.inputSource,
        ))
    }
}

impl XRInputSourceEventMethods for XRInputSourceEvent {
    // https://immersive-web.github.io/webxr/#dom-xrinputsourceeventinit-frame
    fn Frame(&self) -> DomRoot<XRFrame> {
        DomRoot::from_ref(&*self.frame)
    }

    // https://immersive-web.github.io/webxr/#dom-xrinputsourceeventinit-inputsource
    fn InputSource(&self) -> DomRoot<XRInputSource> {
        DomRoot::from_ref(&*self.source)
    }

    // https://dom.spec.whatwg.org/#dom-event-istrusted
    fn IsTrusted(&self) -> bool {
        self.event.IsTrusted()
    }
}
