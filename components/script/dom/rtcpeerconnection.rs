/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::RTCIceCandidateBinding::RTCIceCandidateInit;
use crate::dom::bindings::codegen::Bindings::RTCPeerConnectionBinding;
use crate::dom::bindings::codegen::Bindings::RTCPeerConnectionBinding::RTCPeerConnectionMethods;
use crate::dom::bindings::codegen::Bindings::RTCPeerConnectionBinding::{
    RTCConfiguration, RTCOfferOptions,
};
use crate::dom::bindings::codegen::Bindings::RTCSessionDescriptionBinding::{
    RTCSdpType, RTCSessionDescriptionInit,
};
use crate::dom::bindings::error::Error;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::reflector::DomObject;
use crate::dom::bindings::root::DomRoot;
use crate::dom::event::{Event, EventBubbles, EventCancelable};
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::dom::rtcicecandidate::RTCIceCandidate;
use crate::dom::rtcpeerconnectioniceevent::RTCPeerConnectionIceEvent;
use crate::dom::window::Window;
use crate::task::TaskCanceller;
use crate::task_source::networking::NetworkingTaskSource;
use crate::task_source::TaskSource;
use dom_struct::dom_struct;

use servo_media::webrtc::MediaStream as BackendMediaStream;
use servo_media::webrtc::{
    IceCandidate, SdpType, SessionDescription, WebRtcController, WebRtcSignaller,
};
use servo_media::ServoMedia;

use std::cell::Cell;
use std::rc::Rc;

#[dom_struct]
pub struct RTCPeerConnection {
    eventtarget: EventTarget,
    #[ignore_malloc_size_of = "defined in servo-media"]
    controller: DomRefCell<Option<WebRtcController>>,
    closed: Cell<bool>,
    /// Helps track state changes between the time createOffer
    /// is called and resolved
    offer_generation: Cell<u32>,
    #[ignore_malloc_size_of = "promises are hard"]
    offer_promises: DomRefCell<Vec<Rc<Promise>>>,
}

struct RTCSignaller {
    trusted: Trusted<RTCPeerConnection>,
    task_source: NetworkingTaskSource,
    canceller: TaskCanceller,
}

impl WebRtcSignaller for RTCSignaller {
    fn on_ice_candidate(&self, _: &WebRtcController, candidate: IceCandidate) {
        let this = self.trusted.clone();
        let _ = self.task_source.queue_with_canceller(
            task!(on_ice_candidate: move || {
                let this = this.root();
                this.on_ice_candidate(candidate);
            }),
            &self.canceller,
        );
    }

    fn on_negotiation_needed(&self, _: &WebRtcController) {
        let this = self.trusted.clone();
        let _ = self.task_source.queue_with_canceller(
            task!(on_negotiation_needed: move || {
                let this = this.root();
                this.on_negotiation_needed();
            }),
            &self.canceller,
        );
    }

    fn on_add_stream(&self, _: Box<BackendMediaStream>) {}

    fn close(&self) {
        // do nothing
    }
}

impl RTCPeerConnection {
    pub fn new_inherited() -> RTCPeerConnection {
        RTCPeerConnection {
            eventtarget: EventTarget::new_inherited(),
            controller: DomRefCell::new(None),
            closed: Cell::new(false),
            offer_generation: Cell::new(0),
            offer_promises: DomRefCell::new(vec![]),
        }
    }

    pub fn new(global: &GlobalScope) -> DomRoot<RTCPeerConnection> {
        let this = reflect_dom_object(
            Box::new(RTCPeerConnection::new_inherited()),
            global,
            RTCPeerConnectionBinding::Wrap,
        );
        let signaller = this.make_signaller();
        *this.controller.borrow_mut() = Some(ServoMedia::get().unwrap().create_webrtc(signaller));
        this
    }

    pub fn Constructor(
        window: &Window,
        _config: &RTCConfiguration,
    ) -> Fallible<DomRoot<RTCPeerConnection>> {
        Ok(RTCPeerConnection::new(&window.global()))
    }

    fn make_signaller(&self) -> Box<WebRtcSignaller> {
        let trusted = Trusted::new(self);
        let (task_source, canceller) = self
            .global()
            .as_window()
            .task_manager()
            .networking_task_source_with_canceller();
        Box::new(RTCSignaller {
            trusted,
            task_source,
            canceller,
        })
    }

    fn on_ice_candidate(&self, candidate: IceCandidate) {
        if self.closed.get() {
            return;
        }
        let candidate = RTCIceCandidate::new(
            &self.global(),
            candidate.candidate.into(),
            None,
            Some(candidate.sdp_mline_index as u16),
            None,
        );
        let event = RTCPeerConnectionIceEvent::new(
            &self.global(),
            atom!("icecandidate"),
            Some(&candidate),
            None,
            true,
        );
        event
            .upcast::<Event>()
            .fire(self.upcast());
    }

    fn on_negotiation_needed(&self) {
        if self.closed.get() {
            return;
        }
        let event = Event::new(
            &self.global(),
            atom!("negotiationneeded"),
            EventBubbles::DoesNotBubble,
            EventCancelable::NotCancelable,
        );
        event
            .upcast::<Event>()
            .fire(self.upcast());
    }

    fn create_offer(&self) {
        let generation = self.offer_generation.get();
        let (task_source, canceller) = self
            .global()
            .as_window()
            .task_manager()
            .networking_task_source_with_canceller();
        let this = Trusted::new(self);
        self.controller.borrow_mut().as_ref().unwrap().create_offer(
            (move |desc: SessionDescription| {
                let _ = task_source.queue_with_canceller(
                    task!(offer_created: move || {
                        let this = this.root();
                        if this.offer_generation.get() != generation {
                            // the state has changed since we last created the offer,
                            // create a fresh one
                            this.create_offer();
                        } else {
                            let init: RTCSessionDescriptionInit = desc.into();
                            for promise in this.offer_promises.borrow_mut().drain(..) {
                                promise.resolve_native(&init);
                            }
                        }
                    }),
                    &canceller,
                );
            })
            .into(),
        );
    }
}

impl RTCPeerConnectionMethods for RTCPeerConnection {
    /// https://www.w3.org/TR/webrtc/#dom-rtcpeerconnection-icecandidate
    event_handler!(icecandidate, GetOnicecandidate, SetOnicecandidate);

    /// https://www.w3.org/TR/webrtc/#dom-rtcpeerconnection-onnegotiationneeded
    event_handler!(
        negotiationneeded,
        GetOnnegotiationneeded,
        SetOnnegotiationneeded
    );

    /// https://www.w3.org/TR/webrtc/#dom-rtcpeerconnection-addicecandidate
    fn AddIceCandidate(&self, candidate: &RTCIceCandidateInit) -> Rc<Promise> {
        let p = Promise::new(&self.global());
        if candidate.sdpMid.is_none() && candidate.sdpMLineIndex.is_none() {
            p.reject_error(Error::Type(format!(
                "one of sdpMid and sdpMLineIndex must be set"
            )));
            return p;
        }

        // XXXManishearth add support for sdpMid
        if candidate.sdpMLineIndex.is_none() {
            p.reject_error(Error::Type(format!(
                "servo only supports sdpMLineIndex right now"
            )));
            return p;
        }

        // XXXManishearth this should be enqueued
        // https://www.w3.org/TR/webrtc/#enqueue-an-operation

        self.controller
            .borrow_mut()
            .as_ref()
            .unwrap()
            .add_ice_candidate(IceCandidate {
                sdp_mline_index: candidate.sdpMLineIndex.unwrap() as u32,
                candidate: candidate.candidate.to_string(),
            });

        // XXXManishearth add_ice_candidate should have a callback
        p.resolve_native(&());
        p
    }

    /// https://www.w3.org/TR/webrtc/#dom-rtcpeerconnection-createoffer
    fn CreateOffer(&self, _options: &RTCOfferOptions) -> Rc<Promise> {
        let p = Promise::new(&self.global());
        if self.closed.get() {
            p.reject_error(Error::InvalidState);
            return p;
        }
        self.offer_promises.borrow_mut().push(p.clone());
        self.create_offer();
        p
    }
}

impl From<SessionDescription> for RTCSessionDescriptionInit {
    fn from(desc: SessionDescription) -> Self {
        let type_ = match desc.type_ {
            SdpType::Answer => RTCSdpType::Answer,
            SdpType::Offer => RTCSdpType::Offer,
            SdpType::Pranswer => RTCSdpType::Pranswer,
            SdpType::Rollback => RTCSdpType::Rollback,
        };
        RTCSessionDescriptionInit {
            type_,
            sdp: desc.sdp.into(),
        }
    }
}
