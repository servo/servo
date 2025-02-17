/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::collections::HashMap;
use std::rc::Rc;

use dom_struct::dom_struct;
use js::rust::HandleObject;
use servo_media::streams::registry::MediaStreamId;
use servo_media::streams::MediaStreamType;
use servo_media::webrtc::{
    BundlePolicy, DataChannelEvent, DataChannelId, DataChannelState, GatheringState, IceCandidate,
    IceConnectionState, SdpType, SessionDescription, SignalingState, WebRtcController,
    WebRtcSignaller,
};
use servo_media::ServoMedia;

use crate::conversions::Convert;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::RTCDataChannelBinding::RTCDataChannelInit;
use crate::dom::bindings::codegen::Bindings::RTCIceCandidateBinding::RTCIceCandidateInit;
use crate::dom::bindings::codegen::Bindings::RTCPeerConnectionBinding::{
    RTCAnswerOptions, RTCBundlePolicy, RTCConfiguration, RTCIceConnectionState,
    RTCIceGatheringState, RTCOfferOptions, RTCPeerConnectionMethods, RTCRtpTransceiverInit,
    RTCSignalingState,
};
use crate::dom::bindings::codegen::Bindings::RTCSessionDescriptionBinding::{
    RTCSdpType, RTCSessionDescriptionInit, RTCSessionDescriptionMethods,
};
use crate::dom::bindings::codegen::UnionTypes::{MediaStreamTrackOrString, StringOrStringSequence};
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::{Trusted, TrustedPromise};
use crate::dom::bindings::reflector::{reflect_dom_object_with_proto, DomGlobal};
use crate::dom::bindings::root::{Dom, DomRoot, MutNullableDom};
use crate::dom::bindings::str::USVString;
use crate::dom::event::{Event, EventBubbles, EventCancelable};
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::mediastream::MediaStream;
use crate::dom::mediastreamtrack::MediaStreamTrack;
use crate::dom::promise::Promise;
use crate::dom::rtcdatachannel::RTCDataChannel;
use crate::dom::rtcdatachannelevent::RTCDataChannelEvent;
use crate::dom::rtcicecandidate::RTCIceCandidate;
use crate::dom::rtcpeerconnectioniceevent::RTCPeerConnectionIceEvent;
use crate::dom::rtcrtptransceiver::RTCRtpTransceiver;
use crate::dom::rtcsessiondescription::RTCSessionDescription;
use crate::dom::rtctrackevent::RTCTrackEvent;
use crate::dom::window::Window;
use crate::realms::{enter_realm, InRealm};
use crate::script_runtime::CanGc;
use crate::task_source::SendableTaskSource;

#[dom_struct]
pub(crate) struct RTCPeerConnection {
    eventtarget: EventTarget,
    #[ignore_malloc_size_of = "defined in servo-media"]
    #[no_trace]
    controller: DomRefCell<Option<WebRtcController>>,
    closed: Cell<bool>,
    // Helps track state changes between the time createOffer/createAnswer
    // is called and resolved
    offer_answer_generation: Cell<u32>,
    #[ignore_malloc_size_of = "promises are hard"]
    offer_promises: DomRefCell<Vec<Rc<Promise>>>,
    #[ignore_malloc_size_of = "promises are hard"]
    answer_promises: DomRefCell<Vec<Rc<Promise>>>,
    local_description: MutNullableDom<RTCSessionDescription>,
    remote_description: MutNullableDom<RTCSessionDescription>,
    gathering_state: Cell<RTCIceGatheringState>,
    ice_connection_state: Cell<RTCIceConnectionState>,
    signaling_state: Cell<RTCSignalingState>,
    #[ignore_malloc_size_of = "defined in servo-media"]
    data_channels: DomRefCell<HashMap<DataChannelId, Dom<RTCDataChannel>>>,
}

struct RTCSignaller {
    trusted: Trusted<RTCPeerConnection>,
    task_source: SendableTaskSource,
}

impl WebRtcSignaller for RTCSignaller {
    fn on_ice_candidate(&self, _: &WebRtcController, candidate: IceCandidate) {
        let this = self.trusted.clone();
        self.task_source.queue(task!(on_ice_candidate: move || {
            let this = this.root();
            this.on_ice_candidate(candidate, CanGc::note());
        }));
    }

    fn on_negotiation_needed(&self, _: &WebRtcController) {
        let this = self.trusted.clone();
        self.task_source
            .queue(task!(on_negotiation_needed: move || {
                let this = this.root();
                this.on_negotiation_needed(CanGc::note());
            }));
    }

    fn update_gathering_state(&self, state: GatheringState) {
        let this = self.trusted.clone();
        self.task_source
            .queue(task!(update_gathering_state: move || {
                let this = this.root();
                this.update_gathering_state(state, CanGc::note());
            }));
    }

    fn update_ice_connection_state(&self, state: IceConnectionState) {
        let this = self.trusted.clone();
        self.task_source
            .queue(task!(update_ice_connection_state: move || {
                let this = this.root();
                this.update_ice_connection_state(state, CanGc::note());
            }));
    }

    fn update_signaling_state(&self, state: SignalingState) {
        let this = self.trusted.clone();
        self.task_source
            .queue(task!(update_signaling_state: move || {
                let this = this.root();
                this.update_signaling_state(state, CanGc::note());
            }));
    }

    fn on_add_stream(&self, id: &MediaStreamId, ty: MediaStreamType) {
        let this = self.trusted.clone();
        let id = *id;
        self.task_source.queue(task!(on_add_stream: move || {
            let this = this.root();
            this.on_add_stream(id, ty, CanGc::note());
        }));
    }

    fn on_data_channel_event(
        &self,
        channel: DataChannelId,
        event: DataChannelEvent,
        _: &WebRtcController,
    ) {
        // XXX(ferjm) get label and options from channel properties.
        let this = self.trusted.clone();
        self.task_source
            .queue(task!(on_data_channel_event: move || {
                let this = this.root();
                let global = this.global();
                let _ac = enter_realm(&*global);
                this.on_data_channel_event(channel, event, CanGc::note());
            }));
    }

    fn close(&self) {
        // do nothing
    }
}

impl RTCPeerConnection {
    pub(crate) fn new_inherited() -> RTCPeerConnection {
        RTCPeerConnection {
            eventtarget: EventTarget::new_inherited(),
            controller: DomRefCell::new(None),
            closed: Cell::new(false),
            offer_answer_generation: Cell::new(0),
            offer_promises: DomRefCell::new(vec![]),
            answer_promises: DomRefCell::new(vec![]),
            local_description: Default::default(),
            remote_description: Default::default(),
            gathering_state: Cell::new(RTCIceGatheringState::New),
            ice_connection_state: Cell::new(RTCIceConnectionState::New),
            signaling_state: Cell::new(RTCSignalingState::Stable),
            data_channels: DomRefCell::new(HashMap::new()),
        }
    }

    fn new(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        config: &RTCConfiguration,
        can_gc: CanGc,
    ) -> DomRoot<RTCPeerConnection> {
        let this = reflect_dom_object_with_proto(
            Box::new(RTCPeerConnection::new_inherited()),
            global,
            proto,
            can_gc,
        );
        let signaller = this.make_signaller();
        *this.controller.borrow_mut() = Some(ServoMedia::get().create_webrtc(signaller));
        if let Some(ref servers) = config.iceServers {
            if let Some(server) = servers.first() {
                let server = match server.urls {
                    StringOrStringSequence::String(ref s) => Some(s.clone()),
                    StringOrStringSequence::StringSequence(ref s) => s.first().cloned(),
                };
                if let Some(server) = server {
                    let policy = match config.bundlePolicy {
                        RTCBundlePolicy::Balanced => BundlePolicy::Balanced,
                        RTCBundlePolicy::Max_compat => BundlePolicy::MaxCompat,
                        RTCBundlePolicy::Max_bundle => BundlePolicy::MaxBundle,
                    };
                    this.controller
                        .borrow()
                        .as_ref()
                        .unwrap()
                        .configure(server.to_string(), policy);
                }
            }
        }
        this
    }

    pub(crate) fn get_webrtc_controller(&self) -> &DomRefCell<Option<WebRtcController>> {
        &self.controller
    }

    fn make_signaller(&self) -> Box<dyn WebRtcSignaller> {
        let trusted = Trusted::new(self);
        Box::new(RTCSignaller {
            trusted,
            task_source: self.global().task_manager().networking_task_source().into(),
        })
    }

    fn on_ice_candidate(&self, candidate: IceCandidate, can_gc: CanGc) {
        if self.closed.get() {
            return;
        }
        let candidate = RTCIceCandidate::new(
            &self.global(),
            candidate.candidate.into(),
            None,
            Some(candidate.sdp_mline_index as u16),
            None,
            can_gc,
        );
        let event = RTCPeerConnectionIceEvent::new(
            &self.global(),
            atom!("icecandidate"),
            Some(&candidate),
            None,
            true,
            can_gc,
        );
        event.upcast::<Event>().fire(self.upcast(), can_gc);
    }

    fn on_negotiation_needed(&self, can_gc: CanGc) {
        if self.closed.get() {
            return;
        }
        let event = Event::new(
            &self.global(),
            atom!("negotiationneeded"),
            EventBubbles::DoesNotBubble,
            EventCancelable::NotCancelable,
            can_gc,
        );
        event.upcast::<Event>().fire(self.upcast(), can_gc);
    }

    fn on_add_stream(&self, id: MediaStreamId, ty: MediaStreamType, can_gc: CanGc) {
        if self.closed.get() {
            return;
        }
        let track = MediaStreamTrack::new(&self.global(), id, ty);
        let event =
            RTCTrackEvent::new(&self.global(), atom!("track"), false, false, &track, can_gc);
        event.upcast::<Event>().fire(self.upcast(), can_gc);
    }

    fn on_data_channel_event(
        &self,
        channel_id: DataChannelId,
        event: DataChannelEvent,
        can_gc: CanGc,
    ) {
        if self.closed.get() {
            return;
        }

        match event {
            DataChannelEvent::NewChannel => {
                let channel = RTCDataChannel::new(
                    &self.global(),
                    self,
                    USVString::from("".to_owned()),
                    &RTCDataChannelInit::empty(),
                    Some(channel_id),
                );

                let event = RTCDataChannelEvent::new(
                    &self.global(),
                    atom!("datachannel"),
                    false,
                    false,
                    &channel,
                    can_gc,
                );
                event.upcast::<Event>().fire(self.upcast(), can_gc);
            },
            _ => {
                let channel = if let Some(channel) = self.data_channels.borrow().get(&channel_id) {
                    DomRoot::from_ref(&**channel)
                } else {
                    warn!(
                        "Got an event for an unregistered data channel {:?}",
                        channel_id
                    );
                    return;
                };

                match event {
                    DataChannelEvent::Open => channel.on_open(can_gc),
                    DataChannelEvent::Close => channel.on_close(can_gc),
                    DataChannelEvent::Error(error) => channel.on_error(error, can_gc),
                    DataChannelEvent::OnMessage(message) => channel.on_message(message, can_gc),
                    DataChannelEvent::StateChange(state) => channel.on_state_change(state, can_gc),
                    DataChannelEvent::NewChannel => unreachable!(),
                }
            },
        };
    }

    pub(crate) fn register_data_channel(&self, id: DataChannelId, channel: &RTCDataChannel) {
        if self
            .data_channels
            .borrow_mut()
            .insert(id, Dom::from_ref(channel))
            .is_some()
        {
            warn!("Data channel already registered {:?}", id);
        }
    }

    pub(crate) fn unregister_data_channel(&self, id: &DataChannelId) {
        self.data_channels.borrow_mut().remove(id);
    }

    /// <https://www.w3.org/TR/webrtc/#update-ice-gathering-state>
    fn update_gathering_state(&self, state: GatheringState, can_gc: CanGc) {
        // step 1
        if self.closed.get() {
            return;
        }

        // step 2 (state derivation already done by gstreamer)
        let state: RTCIceGatheringState = state.convert();

        // step 3
        if state == self.gathering_state.get() {
            return;
        }

        // step 4
        self.gathering_state.set(state);

        // step 5
        let event = Event::new(
            &self.global(),
            atom!("icegatheringstatechange"),
            EventBubbles::DoesNotBubble,
            EventCancelable::NotCancelable,
            can_gc,
        );
        event.upcast::<Event>().fire(self.upcast(), can_gc);

        // step 6
        if state == RTCIceGatheringState::Complete {
            let event = RTCPeerConnectionIceEvent::new(
                &self.global(),
                atom!("icecandidate"),
                None,
                None,
                true,
                can_gc,
            );
            event.upcast::<Event>().fire(self.upcast(), can_gc);
        }
    }

    /// <https://www.w3.org/TR/webrtc/#update-ice-connection-state>
    fn update_ice_connection_state(&self, state: IceConnectionState, can_gc: CanGc) {
        // step 1
        if self.closed.get() {
            return;
        }

        // step 2 (state derivation already done by gstreamer)
        let state: RTCIceConnectionState = state.convert();

        // step 3
        if state == self.ice_connection_state.get() {
            return;
        }

        // step 4
        self.ice_connection_state.set(state);

        // step 5
        let event = Event::new(
            &self.global(),
            atom!("iceconnectionstatechange"),
            EventBubbles::DoesNotBubble,
            EventCancelable::NotCancelable,
            can_gc,
        );
        event.upcast::<Event>().fire(self.upcast(), can_gc);
    }

    fn update_signaling_state(&self, state: SignalingState, can_gc: CanGc) {
        if self.closed.get() {
            return;
        }

        let state: RTCSignalingState = state.convert();

        if state == self.signaling_state.get() {
            return;
        }

        self.signaling_state.set(state);

        let event = Event::new(
            &self.global(),
            atom!("signalingstatechange"),
            EventBubbles::DoesNotBubble,
            EventCancelable::NotCancelable,
            can_gc,
        );
        event.upcast::<Event>().fire(self.upcast(), can_gc);
    }

    fn create_offer(&self) {
        let generation = self.offer_answer_generation.get();
        let task_source = self
            .global()
            .task_manager()
            .networking_task_source()
            .to_sendable();
        let this = Trusted::new(self);
        self.controller
            .borrow_mut()
            .as_ref()
            .unwrap()
            .create_offer(Box::new(move |desc: SessionDescription| {
                task_source.queue(task!(offer_created: move || {
                    let this = this.root();
                    if this.offer_answer_generation.get() != generation {
                        // the state has changed since we last created the offer,
                        // create a fresh one
                        this.create_offer();
                    } else {
                        let init: RTCSessionDescriptionInit = desc.convert();
                        for promise in this.offer_promises.borrow_mut().drain(..) {
                            promise.resolve_native(&init);
                        }
                    }
                }));
            }));
    }

    fn create_answer(&self) {
        let generation = self.offer_answer_generation.get();
        let task_source = self
            .global()
            .task_manager()
            .networking_task_source()
            .to_sendable();
        let this = Trusted::new(self);
        self.controller
            .borrow_mut()
            .as_ref()
            .unwrap()
            .create_answer(Box::new(move |desc: SessionDescription| {
                task_source.queue(task!(answer_created: move || {
                    let this = this.root();
                    if this.offer_answer_generation.get() != generation {
                        // the state has changed since we last created the offer,
                        // create a fresh one
                        this.create_answer();
                    } else {
                        let init: RTCSessionDescriptionInit = desc.convert();
                        for promise in this.answer_promises.borrow_mut().drain(..) {
                            promise.resolve_native(&init);
                        }
                    }
                }));
            }));
    }
}

impl RTCPeerConnectionMethods<crate::DomTypeHolder> for RTCPeerConnection {
    // https://w3c.github.io/webrtc-pc/#dom-peerconnection
    fn Constructor(
        window: &Window,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        config: &RTCConfiguration,
    ) -> Fallible<DomRoot<RTCPeerConnection>> {
        Ok(RTCPeerConnection::new(
            &window.global(),
            proto,
            config,
            can_gc,
        ))
    }

    // https://w3c.github.io/webrtc-pc/#dom-rtcpeerconnection-icecandidate
    event_handler!(icecandidate, GetOnicecandidate, SetOnicecandidate);

    // https://www.w3.org/TR/webrtc/#dom-rtcpeerconnection-ontrack
    event_handler!(track, GetOntrack, SetOntrack);

    // https://w3c.github.io/webrtc-pc/#dom-rtcpeerconnection-iceconnectionstatechange
    event_handler!(
        iceconnectionstatechange,
        GetOniceconnectionstatechange,
        SetOniceconnectionstatechange
    );

    // https://w3c.github.io/webrtc-pc/#dom-rtcpeerconnection-icegatheringstatechange
    event_handler!(
        icegatheringstatechange,
        GetOnicegatheringstatechange,
        SetOnicegatheringstatechange
    );

    // https://w3c.github.io/webrtc-pc/#dom-rtcpeerconnection-onnegotiationneeded
    event_handler!(
        negotiationneeded,
        GetOnnegotiationneeded,
        SetOnnegotiationneeded
    );

    // https://w3c.github.io/webrtc-pc/#dom-rtcpeerconnection-signalingstatechange
    event_handler!(
        signalingstatechange,
        GetOnsignalingstatechange,
        SetOnsignalingstatechange
    );

    // https://www.w3.org/TR/webrtc/#dom-rtcpeerconnection-ondatachannel
    event_handler!(datachannel, GetOndatachannel, SetOndatachannel);

    /// <https://w3c.github.io/webrtc-pc/#dom-rtcpeerconnection-addicecandidate>
    fn AddIceCandidate(
        &self,
        candidate: &RTCIceCandidateInit,
        comp: InRealm,
        can_gc: CanGc,
    ) -> Rc<Promise> {
        let p = Promise::new_in_current_realm(comp, can_gc);
        if candidate.sdpMid.is_none() && candidate.sdpMLineIndex.is_none() {
            p.reject_error(Error::Type(
                "one of sdpMid and sdpMLineIndex must be set".to_string(),
            ));
            return p;
        }

        // XXXManishearth add support for sdpMid
        if candidate.sdpMLineIndex.is_none() {
            p.reject_error(Error::Type(
                "servo only supports sdpMLineIndex right now".to_string(),
            ));
            return p;
        }

        // XXXManishearth this should be enqueued
        // https://w3c.github.io/webrtc-pc/#enqueue-an-operation

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

    /// <https://w3c.github.io/webrtc-pc/#dom-rtcpeerconnection-createoffer>
    fn CreateOffer(&self, _options: &RTCOfferOptions, comp: InRealm, can_gc: CanGc) -> Rc<Promise> {
        let p = Promise::new_in_current_realm(comp, can_gc);
        if self.closed.get() {
            p.reject_error(Error::InvalidState);
            return p;
        }
        self.offer_promises.borrow_mut().push(p.clone());
        self.create_offer();
        p
    }

    /// <https://w3c.github.io/webrtc-pc/#dom-rtcpeerconnection-createoffer>
    fn CreateAnswer(
        &self,
        _options: &RTCAnswerOptions,
        comp: InRealm,
        can_gc: CanGc,
    ) -> Rc<Promise> {
        let p = Promise::new_in_current_realm(comp, can_gc);
        if self.closed.get() {
            p.reject_error(Error::InvalidState);
            return p;
        }
        self.answer_promises.borrow_mut().push(p.clone());
        self.create_answer();
        p
    }

    /// <https://w3c.github.io/webrtc-pc/#dom-rtcpeerconnection-localdescription>
    fn GetLocalDescription(&self) -> Option<DomRoot<RTCSessionDescription>> {
        self.local_description.get()
    }

    /// <https://w3c.github.io/webrtc-pc/#dom-rtcpeerconnection-remotedescription>
    fn GetRemoteDescription(&self) -> Option<DomRoot<RTCSessionDescription>> {
        self.remote_description.get()
    }

    /// <https://w3c.github.io/webrtc-pc/#dom-rtcpeerconnection-setlocaldescription>
    fn SetLocalDescription(
        &self,
        desc: &RTCSessionDescriptionInit,
        comp: InRealm,
        can_gc: CanGc,
    ) -> Rc<Promise> {
        // XXXManishearth validate the current state
        let p = Promise::new_in_current_realm(comp, can_gc);
        let this = Trusted::new(self);
        let desc: SessionDescription = desc.convert();
        let trusted_promise = TrustedPromise::new(p.clone());
        let task_source = self
            .global()
            .task_manager()
            .networking_task_source()
            .to_sendable();
        self.controller
            .borrow_mut()
            .as_ref()
            .unwrap()
            .set_local_description(
                desc.clone(),
                Box::new(move || {
                    task_source.queue(task!(local_description_set: move || {
                        // XXXManishearth spec actually asks for an intricate
                        // dance between pending/current local/remote descriptions
                        let this = this.root();
                        let desc = desc.convert();
                        let desc = RTCSessionDescription::Constructor(
                            this.global().as_window(),
                            None,
                            CanGc::note(),
                            &desc,
                        ).unwrap();
                        this.local_description.set(Some(&desc));
                        trusted_promise.root().resolve_native(&())
                    }));
                }),
            );
        p
    }

    /// <https://w3c.github.io/webrtc-pc/#dom-rtcpeerconnection-setremotedescription>
    fn SetRemoteDescription(
        &self,
        desc: &RTCSessionDescriptionInit,
        comp: InRealm,
        can_gc: CanGc,
    ) -> Rc<Promise> {
        // XXXManishearth validate the current state
        let p = Promise::new_in_current_realm(comp, can_gc);
        let this = Trusted::new(self);
        let desc: SessionDescription = desc.convert();
        let trusted_promise = TrustedPromise::new(p.clone());
        let task_source = self
            .global()
            .task_manager()
            .networking_task_source()
            .to_sendable();
        self.controller
            .borrow_mut()
            .as_ref()
            .unwrap()
            .set_remote_description(
                desc.clone(),
                Box::new(move || {
                    task_source.queue(task!(remote_description_set: move || {
                        // XXXManishearth spec actually asks for an intricate
                        // dance between pending/current local/remote descriptions
                        let this = this.root();
                        let desc = desc.convert();
                        let desc = RTCSessionDescription::Constructor(
                            this.global().as_window(),
                            None,
                            CanGc::note(),
                            &desc,
                        ).unwrap();
                        this.remote_description.set(Some(&desc));
                        trusted_promise.root().resolve_native(&())
                    }));
                }),
            );
        p
    }

    // https://w3c.github.io/webrtc-pc/#legacy-interface-extensions
    fn AddStream(&self, stream: &MediaStream) {
        for track in &*stream.get_tracks() {
            self.controller
                .borrow()
                .as_ref()
                .unwrap()
                .add_stream(&track.id());
        }
    }

    /// <https://www.w3.org/TR/webrtc/#dom-rtcpeerconnection-icegatheringstate>
    fn IceGatheringState(&self) -> RTCIceGatheringState {
        self.gathering_state.get()
    }

    /// <https://www.w3.org/TR/webrtc/#dom-rtcpeerconnection-iceconnectionstate>
    fn IceConnectionState(&self) -> RTCIceConnectionState {
        self.ice_connection_state.get()
    }

    /// <https://www.w3.org/TR/webrtc/#dom-rtcpeerconnection-signalingstate>
    fn SignalingState(&self) -> RTCSignalingState {
        self.signaling_state.get()
    }

    /// <https://www.w3.org/TR/webrtc/#dom-rtcpeerconnection-close>
    fn Close(&self, can_gc: CanGc) {
        // Step 1
        if self.closed.get() {
            return;
        }
        // Step 2
        self.closed.set(true);

        // Step 4
        self.signaling_state.set(RTCSignalingState::Closed);

        // Step 5 handled by backend
        self.controller.borrow_mut().as_ref().unwrap().quit();

        // Step 6
        for (_, val) in self.data_channels.borrow().iter() {
            val.on_state_change(DataChannelState::Closed, can_gc);
        }

        // Step 7-10
        // (no current support for transports, etc)

        // Step 11
        self.ice_connection_state.set(RTCIceConnectionState::Closed);

        // Step 11
        // (no current support for connection state)
    }

    /// <https://www.w3.org/TR/webrtc/#dom-peerconnection-createdatachannel>
    fn CreateDataChannel(
        &self,
        label: USVString,
        init: &RTCDataChannelInit,
    ) -> DomRoot<RTCDataChannel> {
        RTCDataChannel::new(&self.global(), self, label, init, None)
    }

    /// <https://w3c.github.io/webrtc-pc/#dom-rtcpeerconnection-addtransceiver>
    fn AddTransceiver(
        &self,
        _track_or_kind: MediaStreamTrackOrString,
        init: &RTCRtpTransceiverInit,
    ) -> DomRoot<RTCRtpTransceiver> {
        RTCRtpTransceiver::new(&self.global(), init.direction)
    }
}

impl Convert<RTCSessionDescriptionInit> for SessionDescription {
    fn convert(self) -> RTCSessionDescriptionInit {
        let type_ = match self.type_ {
            SdpType::Answer => RTCSdpType::Answer,
            SdpType::Offer => RTCSdpType::Offer,
            SdpType::Pranswer => RTCSdpType::Pranswer,
            SdpType::Rollback => RTCSdpType::Rollback,
        };
        RTCSessionDescriptionInit {
            type_,
            sdp: self.sdp.into(),
        }
    }
}

impl Convert<SessionDescription> for &RTCSessionDescriptionInit {
    fn convert(self) -> SessionDescription {
        let type_ = match self.type_ {
            RTCSdpType::Answer => SdpType::Answer,
            RTCSdpType::Offer => SdpType::Offer,
            RTCSdpType::Pranswer => SdpType::Pranswer,
            RTCSdpType::Rollback => SdpType::Rollback,
        };
        SessionDescription {
            type_,
            sdp: self.sdp.to_string(),
        }
    }
}

impl Convert<RTCIceGatheringState> for GatheringState {
    fn convert(self) -> RTCIceGatheringState {
        match self {
            GatheringState::New => RTCIceGatheringState::New,
            GatheringState::Gathering => RTCIceGatheringState::Gathering,
            GatheringState::Complete => RTCIceGatheringState::Complete,
        }
    }
}

impl Convert<RTCIceConnectionState> for IceConnectionState {
    fn convert(self) -> RTCIceConnectionState {
        match self {
            IceConnectionState::New => RTCIceConnectionState::New,
            IceConnectionState::Checking => RTCIceConnectionState::Checking,
            IceConnectionState::Connected => RTCIceConnectionState::Connected,
            IceConnectionState::Completed => RTCIceConnectionState::Completed,
            IceConnectionState::Disconnected => RTCIceConnectionState::Disconnected,
            IceConnectionState::Failed => RTCIceConnectionState::Failed,
            IceConnectionState::Closed => RTCIceConnectionState::Closed,
        }
    }
}

impl Convert<RTCSignalingState> for SignalingState {
    fn convert(self) -> RTCSignalingState {
        match self {
            SignalingState::Stable => RTCSignalingState::Stable,
            SignalingState::HaveLocalOffer => RTCSignalingState::Have_local_offer,
            SignalingState::HaveRemoteOffer => RTCSignalingState::Have_remote_offer,
            SignalingState::HaveLocalPranswer => RTCSignalingState::Have_local_pranswer,
            SignalingState::HaveRemotePranswer => RTCSignalingState::Have_remote_pranswer,
            SignalingState::Closed => RTCSignalingState::Closed,
        }
    }
}
