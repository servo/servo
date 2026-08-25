/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::JSContext;
use js::rust::HandleObject;
use script_bindings::reflector::reflect_dom_object_with_proto;
use stylo_atoms::Atom;

use crate::dom::bindings::codegen::Bindings::EventBinding::Event_Binding::EventMethods;
use crate::dom::bindings::codegen::Bindings::XRLayerEventBinding::{
    XRLayerEventInit, XRLayerEventMethods,
};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::event::Event;
use crate::dom::window::Window;
use crate::dom::xrlayer::XRLayer;

// https://w3c.github.io/uievents/#interface-uievent
#[dom_struct]
pub(crate) struct XRLayerEvent {
    event: Event,
    layer: Dom<XRLayer>,
}

impl XRLayerEvent {
    pub(crate) fn new_inherited(layer: &XRLayer) -> XRLayerEvent {
        XRLayerEvent {
            event: Event::new_inherited(),
            layer: Dom::from_ref(layer),
        }
    }

    fn new(
        cx: &mut JSContext,
        window: &Window,
        proto: Option<HandleObject>,
        layer: &XRLayer,
    ) -> DomRoot<XRLayerEvent> {
        reflect_dom_object_with_proto(
            cx,
            Box::new(XRLayerEvent::new_inherited(layer)),
            window,
            proto,
        )
    }
}

impl XRLayerEventMethods<crate::DomTypeHolder> for XRLayerEvent {
    /// <https://immersive-web.github.io/layers/#dom-xrlayerevent-xrlayerevent>
    fn Constructor(
        cx: &mut JSContext,
        window: &Window,
        proto: Option<HandleObject>,
        type_: DOMString,
        init: &XRLayerEventInit,
    ) -> DomRoot<XRLayerEvent> {
        let event = XRLayerEvent::new(cx, window, proto, &init.layer);
        let type_ = Atom::from(type_);
        let bubbles = init.parent.bubbles;
        let cancelable = init.parent.cancelable;
        event.event.init_event(type_, bubbles, cancelable);
        event
    }

    /// <https://immersive-web.github.io/layers/#dom-xrlayerevent-layer>
    fn Layer(&self) -> DomRoot<XRLayer> {
        DomRoot::from_ref(&self.layer)
    }

    /// <https://dom.spec.whatwg.org/#dom-event-istrusted>
    fn IsTrusted(&self) -> bool {
        self.event.IsTrusted()
    }
}
