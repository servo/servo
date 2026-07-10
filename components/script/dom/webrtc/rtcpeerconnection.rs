/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::rc::Rc;

use dom_struct::dom_struct;
use js::context::JSContext;
use js::realm::CurrentRealm;
use js::rust::HandleObject;
use rustc_hash::FxHashMap;
use script_bindings::cell::DomRefCell;
use script_bindings::reflector::reflect_dom_object_with_proto_and_cx;
use servo_media::ServoMedia;
use servo_media::streams::MediaStreamType;
use servo_media::streams::registry::MediaStreamId;
use servo_media::webrtc::{
    BundlePolicy, DataChannelEvent, DataChannelId, DataChannelState, GatheringState, IceCandidate,
    IceConnectionState, SdpType, SessionDescription, SignalingState, WebRtcController,
    WebRtcSignaller,
};

use crate::conversions::Convert;
use crate::dom::bindings::codegen::Bindings::RTCDataChannelBinding::RTCDataChannelInit;
use crate::dom::bindings::codegen::Bindings::RTCIceCandidateBinding::RTCIceCandidateInit;
use crate::dom::bindings::codegen::Bindings::RTCPeerConnectionBinding::{
    RTCAnswerOptions, RTCBundlePolicy, RTCConfiguration, RTCIceConnectionState,
    RTCIceGatheringState, RTCOfferOptions, RTCPeerConnectionMethods, RTCRtpTransceiverInit,
    RTCSignalingState,
};
use crate::dom::bindings::codegen::Bindings::RTCSessionDescriptionBinding::{
    RTCSdpType, RTCSessionDescriptionInit,
};
use crate::dom::bindings::codegen::UnionTypes::{MediaStreamTrackOrString, StringOrStringSequence};
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::{Trusted, TrustedPromise};
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::{Dom, DomRoot, MutNullableDom};
use crate::dom::bindings::str::USVString;
use crate::dom::event::{Event, EventBubbles, EventCancelable};
use crate::dom::eventtarget::EventTarget;
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
use crate::realms::enter_auto_realm;
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
    #[conditional_malloc_size_of]
    offer_promises: DomRefCell<Vec<Rc<Promise>>>,
    #[conditional_malloc_size_of]
    answer_promises: DomRefCell<Vec<Rc<Promise>>>,
    local_description: MutNullableDom<RTCSessionDescription>,
    remote_description: MutNullableDom<RTCSessionDescription>,
    gathering_state: Cell<RTCIceGatheringState>,
    ice_connection_state: Cell<RTCIceConnectionState>,
    signaling_state: Cell<RTCSignalingState>,
    #[ignore_malloc_size_of = "defined in servo-media"]
    data_channels: DomRefCell<FxHashMap<DataChannelId, Dom<RTCDataChannel>>>,
}

struct RTCSignaller {
    trusted: Trusted<RTCPeerConnection>,
    task_source: SendableTaskSource,
}

impl WebRtcSignaller for RTCSignaller {
    fn on_ice_candidate(&self, _: &WebRtcController, candidate: IceCandidate) {
        let this = self.trusted.clone();
        self.task_source.queue(task!(on_ice_candidate: move |cx| {
            let this = this.root();
            this.on_ice_candidate(cx, candidate);
        }));
    }

    fn on_negotiation_needed(&self, _: &WebRtcController) {
        let this = self.trusted.clone();
        self.task_source
            .queue(task!(on_negotiation_needed: move |cx| {
                let this = this.root();
                this.on_negotiation_needed(cx);
            }));
    }

    fn update_gathering_state(&self, state: GatheringState) {
        let this = self.trusted.clone();
        self.task_source
            .queue(task!(update_gathering_state: move |cx| {
                let this = this.root();
                this.update_gathering_state(cx, state);
            }));
    }

    fn update_ice_connection_state(&self, state: IceConnectionState) {
        let this = self.trusted.clone();
        self.task_source
            .queue(task!(update_ice_connection_state: move |cx| {
                let this = this.root();
                this.update_ice_connection_state(cx, state);
            }));
    }

    fn update_signaling_state(&self, state: SignalingState) {
        let this = self.trusted.clone();
        self.task_source
            .queue(task!(update_signaling_state: move |cx| {
                let this = this.root();
                this.update_signaling_state(cx, state);
            }));
    }

    fn on_add_stream(&self, id: &MediaStreamId, ty: MediaStreamType) {
        let this = self.trusted.clone();
        let id = *id;
        self.task_source.queue(task!(on_add_stream: move |cx| {
            let this = this.root();
            this.on_add_stream(cx, id, ty, );
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
            .queue(task!(on_data_channel_event: move |cx| {
                let this = this.root();
                let global = this.global();
                let mut realm = enter_auto_realm(cx, &*global);
                this.on_data_channel_event(&mut realm.current_realm(), channel, event);
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
            data_channels: DomRefCell::new(FxHashMap::default()),
        }
    }

    fn new(
        cx: &mut JSContext,
        window: &Window,
        proto: Option<HandleObject>,
        config: &RTCConfiguration,
    ) -> DomRoot<RTCPeerConnection> {
        let this = reflect_dom_object_with_proto_and_cx(
            Box::new(RTCPeerConnection::new_inherited()),
            window,
            proto,
            cx,
        );
        let signaller = this.make_signaller();
        *this.controller.borrow_mut() = Some(ServoMedia::get().create_webrtc(signaller));
        if let Some(ref servers) = config.iceServers
            && let Some(server) = servers.first()
        {
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
                    .configure(String::from(server), policy);
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

    fn on_ice_candidate(&self, cx: &mut JSContext, candidate: IceCandidate) {
        if self.closed.get() {
            return;
        }
        let candidate = RTCIceCandidate::new(
            cx,
            self.global().as_window(),
            candidate.candidate.into(),
            None,
            Some(candidate.sdp_mline_index as u16),
            None,
        );
        let event = RTCPeerConnectionIceEvent::new(
            cx,
            self.global().as_window(),
            atom!("icecandidate"),
            Some(&candidate),
            None,
            true,
        );
        event.upcast::<Event>().fire(cx, self.upcast());
    }

    fn on_negotiation_needed(&self, cx: &mut JSContext) {
        if self.closed.get() {
            return;
        }
        let event = Event::new(
            cx,
            &self.global(),
            atom!("negotiationneeded"),
            EventBubbles::DoesNotBubble,
            EventCancelable::NotCancelable,
        );
        event.upcast::<Event>().fire(cx, self.upcast());
    }

    fn on_add_stream(&self, cx: &mut JSContext, id: MediaStreamId, ty: MediaStreamType) {
        if self.closed.get() {
            return;
        }
        let track = MediaStreamTrack::new(cx, &self.global(), id, ty);
        let event = RTCTrackEvent::new(
            cx,
            self.global().as_window(),
            atom!("track"),
            false,
            false,
            &track,
        );
        event.upcast::<Event>().fire(cx, self.upcast());
    }

    fn on_data_channel_event(
        &self,
        cx: &mut CurrentRealm,
        channel_id: DataChannelId,
        event: DataChannelEvent,
    ) {
        if self.closed.get() {
            return;
        }

        match event {
            DataChannelEvent::NewChannel => {
                let channel = RTCDataChannel::new(
                    cx,
                    &self.global(),
                    self,
                    USVString::from("".to_owned()),
                    &RTCDataChannelInit::empty(),
                    Some(channel_id),
                );

                let event = RTCDataChannelEvent::new(
                    cx,
                    self.global().as_window(),
                    atom!("datachannel"),
                    false,
                    false,
                    &channel,
                );
                event.upcast::<Event>().fire(cx, self.upcast());
            },
            _ => {
                let channel: DomRoot<RTCDataChannel> =
                    if let Some(channel) = self.data_channels.borrow().get(&channel_id) {
                        DomRoot::from_ref(&**channel)
                    } else {
                        warn!(
                            "Got an event for an unregistered data channel {:?}",
                            channel_id
                        );
                        return;
                    };

                match event {
                    DataChannelEvent::Open => channel.on_open(cx),
                    DataChannelEvent::Close => channel.on_close(cx),
                    DataChannelEvent::Error(error) => channel.on_error(cx, error),
                    DataChannelEvent::OnMessage(message) => channel.on_message(cx, message),
                    DataChannelEvent::StateChange(state) => channel.on_state_change(cx, state),
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
    fn update_gathering_state(&self, cx: &mut JSContext, state: GatheringState) {
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
            cx,
            &self.global(),
            atom!("icegatheringstatechange"),
            EventBubbles::DoesNotBubble,
            EventCancelable::NotCancelable,
        );
        event.upcast::<Event>().fire(cx, self.upcast());

        // step 6
        if state == RTCIceGatheringState::Complete {
            let event = RTCPeerConnectionIceEvent::new(
                cx,
                self.global().as_window(),
                atom!("icecandidate"),
                None,
                None,
                true,
            );
            event.upcast::<Event>().fire(cx, self.upcast());
        }
    }

    /// <https://www.w3.org/TR/webrtc/#update-ice-connection-state>
    fn update_ice_connection_state(&self, cx: &mut JSContext, state: IceConnectionState) {
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
            cx,
            &self.global(),
            atom!("iceconnectionstatechange"),
            EventBubbles::DoesNotBubble,
            EventCancelable::NotCancelable,
        );
        event.upcast::<Event>().fire(cx, self.upcast());
    }

    fn update_signaling_state(&self, cx: &mut JSContext, state: SignalingState) {
        if self.closed.get() {
            return;
        }

        let state: RTCSignalingState = state.convert();

        if state == self.signaling_state.get() {
            return;
        }

        self.signaling_state.set(state);

        let event = Event::new(
            cx,
            &self.global(),
            atom!("signalingstatechange"),
            EventBubbles::DoesNotBubble,
            EventCancelable::NotCancelable,
        );
        event.upcast::<Event>().fire(cx, self.upcast());
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
                task_source.queue(task!(offer_created: move |cx| {
                    let this = this.root();
                    if this.offer_answer_generation.get() != generation {
                        // the state has changed since we last created the offer,
                        // create a fresh one
                        this.create_offer();
                    } else {
                        let init: RTCSessionDescriptionInit = desc.convert();
                        for promise in this.offer_promises.borrow_mut().drain(..) {
                            promise.resolve_native(cx, &init);
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
                task_source.queue(task!(answer_created: move |cx| {
                    let this = this.root();
                    if this.offer_answer_generation.get() != generation {
                        // the state has changed since we last created the offer,
                        // create a fresh one
                        this.create_answer();
                    } else {
                        let init: RTCSessionDescriptionInit = desc.convert();
                        for promise in this.answer_promises.borrow_mut().drain(..) {
                            promise.resolve_native(cx, &init);
                        }
                    }
                }));
            }));
    }
}

impl RTCPeerConnectionMethods<crate::DomTypeHolder> for RTCPeerConnection {
    /// <https://w3c.github.io/webrtc-pc/#dom-peerconnection>
    fn Constructor(
        cx: &mut JSContext,
        window: &Window,
        proto: Option<HandleObject>,
        config: &RTCConfiguration,
    ) -> Fallible<DomRoot<RTCPeerConnection>> {
        Ok(RTCPeerConnection::new(cx, window, proto, config))
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
        current_realm: &mut CurrentRealm,
        candidate: &RTCIceCandidateInit,
    ) -> Rc<Promise> {
        let p = Promise::new_in_realm(current_realm);
        if candidate.sdpMid.is_none() && candidate.sdpMLineIndex.is_none() {
            p.reject_error(
                current_realm,
                Error::Type(c"one of sdpMid and sdpMLineIndex must be set".to_owned()),
            );
            return p;
        }

        // XXXManishearth add support for sdpMid
        if candidate.sdpMLineIndex.is_none() {
            p.reject_error(
                current_realm,
                Error::Type(c"servo only supports sdpMLineIndex right now".to_owned()),
            );
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
        p.resolve_native(current_realm, &());
        p
    }

    /// <https://w3c.github.io/webrtc-pc/#dom-rtcpeerconnection-createoffer>
    fn CreateOffer(
        &self,
        current_realm: &mut CurrentRealm,
        _options: &RTCOfferOptions,
    ) -> Rc<Promise> {
        let p = Promise::new_in_realm(current_realm);
        if self.closed.get() {
            p.reject_error(current_realm, Error::InvalidState(None));
            return p;
        }
        self.offer_promises.borrow_mut().push(p.clone());
        self.create_offer();
        p
    }

    /// <https://w3c.github.io/webrtc-pc/#dom-rtcpeerconnection-createoffer>
    fn CreateAnswer(
        &self,
        current_realm: &mut CurrentRealm,
        _options: &RTCAnswerOptions,
    ) -> Rc<Promise> {
        let p = Promise::new_in_realm(current_realm);
        if self.closed.get() {
            p.reject_error(current_realm, Error::InvalidState(None));
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
        current_realm: &mut CurrentRealm,
        desc: &RTCSessionDescriptionInit,
    ) -> Rc<Promise> {
        // XXXManishearth validate the current state
        let p = Promise::new_in_realm(current_realm);
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
                    task_source.queue(task!(local_description_set: move |current_realm| {
                        // XXXManishearth spec actually asks for an intricate
                        // dance between pending/current local/remote descriptions
                        let this = this.root();
                        let desc = desc.convert();
                        let desc = RTCSessionDescription::new(
                            current_realm,
                            this.global().as_window(),
                            None,
                            desc.type_,
                            desc.sdp,
                        );
                        this.local_description.set(Some(&desc));
                        trusted_promise.root().resolve_native(current_realm, &())
                    }));
                }),
            );
        p
    }

    /// <https://w3c.github.io/webrtc-pc/#dom-rtcpeerconnection-setremotedescription>
    fn SetRemoteDescription(
        &self,
        current_realm: &mut CurrentRealm,
        desc: &RTCSessionDescriptionInit,
    ) -> Rc<Promise> {
        // XXXManishearth validate the current state
        let p = Promise::new_in_realm(current_realm);
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
                    task_source.queue(task!(remote_description_set: move |current_realm| {
                        // XXXManishearth spec actually asks for an intricate
                        // dance between pending/current local/remote descriptions
                        let this = this.root();
                        let desc = desc.convert();
                        let desc = RTCSessionDescription::new(
                            current_realm,
                            this.global().as_window(),
                            None,
                            desc.type_,
                            desc.sdp,
                        );
                        this.remote_description.set(Some(&desc));
                        trusted_promise.root().resolve_native(current_realm, &())
                    }));
                }),
            );
        p
    }

    /// <https://w3c.github.io/webrtc-pc/#legacy-interface-extensions>
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
    fn Close(&self, cx: &mut JSContext) {
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
            val.on_state_change(cx, DataChannelState::Closed);
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
        cx: &mut JSContext,
        label: USVString,
        init: &RTCDataChannelInit,
    ) -> DomRoot<RTCDataChannel> {
        RTCDataChannel::new(cx, &self.global(), self, label, init, None)
    }

    /// <https://w3c.github.io/webrtc-pc/#dom-rtcpeerconnection-addtransceiver>
    fn AddTransceiver(
        &self,
        cx: &mut JSContext,
        _track_or_kind: MediaStreamTrackOrString,
        init: &RTCRtpTransceiverInit,
    ) -> DomRoot<RTCRtpTransceiver> {
        RTCRtpTransceiver::new(cx, &self.global(), init.direction)
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
