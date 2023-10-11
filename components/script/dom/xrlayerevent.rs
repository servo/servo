/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::rust::HandleObject;
use servo_atoms::Atom;

use crate::dom::bindings::codegen::Bindings::EventBinding::Event_Binding::EventMethods;
use crate::dom::bindings::codegen::Bindings::XRLayerEventBinding::{
    XRLayerEventInit, XRLayerEventMethods,
};
use crate::dom::bindings::reflector::reflect_dom_object_with_proto;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::event::Event;
use crate::dom::window::Window;
use crate::dom::xrlayer::XRLayer;

// https://w3c.github.io/uievents/#interface-uievent
#[dom_struct]
pub struct XRLayerEvent {
    event: Event,
    layer: Dom<XRLayer>,
}

impl XRLayerEvent {
    pub fn new_inherited(layer: &XRLayer) -> XRLayerEvent {
        XRLayerEvent {
            event: Event::new_inherited(),
            layer: Dom::from_ref(layer),
        }
    }

    fn new(window: &Window, proto: Option<HandleObject>, layer: &XRLayer) -> DomRoot<XRLayerEvent> {
        reflect_dom_object_with_proto(Box::new(XRLayerEvent::new_inherited(layer)), window, proto)
    }

    #[allow(non_snake_case)]
    pub fn Constructor(
        window: &Window,
        proto: Option<HandleObject>,
        type_: DOMString,
        init: &XRLayerEventInit,
    ) -> DomRoot<XRLayerEvent> {
        let event = XRLayerEvent::new(window, proto, &init.layer);
        let type_ = Atom::from(type_);
        let bubbles = init.parent.bubbles;
        let cancelable = init.parent.cancelable;
        event.event.init_event(type_, bubbles, cancelable);
        event
    }
}

impl XRLayerEventMethods for XRLayerEvent {
    // https://immersive-web.github.io/layers/#dom-xrlayerevent-layer
    fn Layer(&self) -> DomRoot<XRLayer> {
        DomRoot::from_ref(&self.layer)
    }

    // https://dom.spec.whatwg.org/#dom-event-istrusted
    fn IsTrusted(&self) -> bool {
        self.event.IsTrusted()
    }
}
