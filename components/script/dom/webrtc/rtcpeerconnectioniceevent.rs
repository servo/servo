/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::JSContext;
use js::rust::HandleObject;
use script_bindings::reflector::reflect_dom_object_with_proto;
use stylo_atoms::Atom;

use crate::dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use crate::dom::bindings::codegen::Bindings::RTCPeerConnectionIceEventBinding::{
    RTCPeerConnectionIceEventInit, RTCPeerConnectionIceEventMethods,
};
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::event::Event;
use crate::dom::rtcicecandidate::RTCIceCandidate;
use crate::dom::window::Window;

#[dom_struct]
pub(crate) struct RTCPeerConnectionIceEvent {
    event: Event,
    candidate: Option<Dom<RTCIceCandidate>>,
    url: Option<DOMString>,
}

impl RTCPeerConnectionIceEvent {
    pub(crate) fn new_inherited(
        candidate: Option<&RTCIceCandidate>,
        url: Option<DOMString>,
    ) -> RTCPeerConnectionIceEvent {
        RTCPeerConnectionIceEvent {
            event: Event::new_inherited(),
            candidate: candidate.map(Dom::from_ref),
            url,
        }
    }

    pub(crate) fn new(
        cx: &mut JSContext,
        window: &Window,
        ty: Atom,
        candidate: Option<&RTCIceCandidate>,
        url: Option<DOMString>,
        trusted: bool,
    ) -> DomRoot<RTCPeerConnectionIceEvent> {
        Self::new_with_proto(cx, window, None, ty, candidate, url, trusted)
    }

    fn new_with_proto(
        cx: &mut JSContext,
        window: &Window,
        proto: Option<HandleObject>,
        ty: Atom,
        candidate: Option<&RTCIceCandidate>,
        url: Option<DOMString>,
        trusted: bool,
    ) -> DomRoot<RTCPeerConnectionIceEvent> {
        let e = reflect_dom_object_with_proto(
            cx,
            Box::new(RTCPeerConnectionIceEvent::new_inherited(candidate, url)),
            window,
            proto,
        );
        let evt = e.upcast::<Event>();
        evt.init_event(ty, false, false); // XXXManishearth bubbles/cancelable?
        evt.set_trusted(trusted);
        e
    }
}

impl RTCPeerConnectionIceEventMethods<crate::DomTypeHolder> for RTCPeerConnectionIceEvent {
    /// <https://w3c.github.io/webrtc-pc/#dom-rtcpeerconnectioniceevent-constructor>
    fn Constructor(
        cx: &mut JSContext,
        window: &Window,
        proto: Option<HandleObject>,
        ty: DOMString,
        init: &RTCPeerConnectionIceEventInit,
    ) -> Fallible<DomRoot<RTCPeerConnectionIceEvent>> {
        Ok(RTCPeerConnectionIceEvent::new_with_proto(
            cx,
            window,
            proto,
            ty.into(),
            init.candidate
                .as_ref()
                .and_then(|x| x.as_ref())
                .map(|x| &**x),
            init.url.as_ref().and_then(|x| x.clone()),
            false,
        ))
    }

    /// <https://w3c.github.io/webrtc-pc/#dom-rtcpeerconnectioniceevent-candidate>
    fn GetCandidate(&self) -> Option<DomRoot<RTCIceCandidate>> {
        self.candidate.as_ref().map(|x| DomRoot::from_ref(&**x))
    }

    /// <https://w3c.github.io/webrtc-pc/#dom-rtcpeerconnectioniceevent-url>
    fn GetUrl(&self) -> Option<DOMString> {
        self.url.clone()
    }

    /// <https://dom.spec.whatwg.org/#dom-event-istrusted>
    fn IsTrusted(&self) -> bool {
        self.event.IsTrusted()
    }
}
