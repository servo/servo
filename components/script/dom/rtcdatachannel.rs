/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::RTCDataChannelBinding::RTCDataChannelInit;
use crate::dom::bindings::codegen::Bindings::RTCDataChannelBinding::RTCDataChannelMethods;
use crate::dom::bindings::codegen::Bindings::RTCDataChannelBinding::RTCDataChannelState;
use crate::dom::bindings::codegen::Bindings::RTCErrorBinding::{RTCErrorDetailType, RTCErrorInit};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::{DOMString, USVString};
use crate::dom::event::{Event, EventBubbles, EventCancelable};
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::messageevent::MessageEvent;
use crate::dom::rtcerror::RTCError;
use crate::dom::rtcerrorevent::RTCErrorEvent;
use crate::dom::rtcpeerconnection::RTCPeerConnection;
use dom_struct::dom_struct;
use js::conversions::ToJSValConvertible;
use js::jsapi::JSAutoRealm;
use js::jsval::UndefinedValue;
use servo_media::webrtc::{
    DataChannelId, DataChannelInit, DataChannelMessage, DataChannelState, WebRtcError,
};

#[dom_struct]
pub struct RTCDataChannel {
    eventtarget: EventTarget,
    #[ignore_malloc_size_of = "defined in servo-media"]
    servo_media_id: DataChannelId,
    peer_connection: Dom<RTCPeerConnection>,
    label: USVString,
    ordered: bool,
    max_packet_life_time: Option<u16>,
    max_retransmits: Option<u16>,
    protocol: USVString,
    negotiated: bool,
    id: Option<u16>,
    ready_state: DomRefCell<RTCDataChannelState>,
}

impl RTCDataChannel {
    #[allow(unrooted_must_root)]
    pub fn new_inherited(
        peer_connection: &RTCPeerConnection,
        label: USVString,
        options: &RTCDataChannelInit,
        servo_media_id: Option<DataChannelId>,
    ) -> RTCDataChannel {
        let mut init: DataChannelInit = options.into();
        init.label = label.to_string();

        let controller = peer_connection.get_webrtc_controller().borrow();
        let servo_media_id = servo_media_id.unwrap_or(
            controller
                .as_ref()
                .unwrap()
                .create_data_channel(init)
                .expect("Expected data channel id"),
        );

        let channel = RTCDataChannel {
            eventtarget: EventTarget::new_inherited(),
            servo_media_id,
            peer_connection: Dom::from_ref(&peer_connection),
            label,
            ordered: options.ordered,
            max_packet_life_time: options.maxPacketLifeTime,
            max_retransmits: options.maxRetransmits,
            protocol: options.protocol.clone(),
            negotiated: options.negotiated,
            id: options.id,
            ready_state: DomRefCell::new(RTCDataChannelState::Connecting),
        };

        peer_connection.register_data_channel(servo_media_id, &channel);

        channel
    }

    pub fn new(
        global: &GlobalScope,
        peer_connection: &RTCPeerConnection,
        label: USVString,
        options: &RTCDataChannelInit,
        servo_media_id: Option<DataChannelId>,
    ) -> DomRoot<RTCDataChannel> {
        let rtc_data_channel = reflect_dom_object(
            Box::new(RTCDataChannel::new_inherited(
                peer_connection,
                label,
                options,
                servo_media_id,
            )),
            global,
        );

        rtc_data_channel
    }

    pub fn on_open(&self) {
        let event = Event::new(
            &self.global(),
            atom!("open"),
            EventBubbles::DoesNotBubble,
            EventCancelable::NotCancelable,
        );
        event.upcast::<Event>().fire(self.upcast());
    }

    pub fn on_close(&self) {
        let event = Event::new(
            &self.global(),
            atom!("close"),
            EventBubbles::DoesNotBubble,
            EventCancelable::NotCancelable,
        );
        event.upcast::<Event>().fire(self.upcast());

        self.peer_connection
            .unregister_data_channel(&self.servo_media_id);
    }

    pub fn on_error(&self, error: WebRtcError) {
        let init = RTCErrorInit {
            errorDetail: RTCErrorDetailType::Data_channel_failure,
            httpRequestStatusCode: None,
            receivedAlert: None,
            sctpCauseCode: None,
            sdpLineNumber: None,
            sentAlert: None,
        };
        let message = match error {
            WebRtcError::Backend(message) => DOMString::from(message),
        };
        let error = RTCError::new(&self.global(), &init, message);
        let event = RTCErrorEvent::new(&self.global(), atom!("error"), false, false, &error);
        event.upcast::<Event>().fire(self.upcast());
    }

    #[allow(unsafe_code)]
    pub fn on_message(&self, message: DataChannelMessage) {
        // XXX(ferjm) Support binary messages
        match message {
            DataChannelMessage::Text(text) => unsafe {
                let global = self.global();
                let cx = global.get_cx();
                let _ac = JSAutoRealm::new(*cx, self.reflector().get_jsobject().get());
                rooted!(in(*cx) let mut message = UndefinedValue());
                text.to_jsval(*cx, message.handle_mut());

                MessageEvent::dispatch_jsval(
                    self.upcast(),
                    &global,
                    message.handle(),
                    Some(&global.origin().immutable().ascii_serialization()),
                    None,
                    vec![],
                );
            },
            DataChannelMessage::Binary(_) => {},
        }
    }

    pub fn on_state_change(&self, state: DataChannelState) {
        match state {
            DataChannelState::Closing => {
                let event = Event::new(
                    &self.global(),
                    atom!("closing"),
                    EventBubbles::DoesNotBubble,
                    EventCancelable::NotCancelable,
                );
                event.upcast::<Event>().fire(self.upcast());
            },
            _ => {},
        };
        *self.ready_state.borrow_mut() = state.into();
    }
}

impl Drop for RTCDataChannel {
    fn drop(&mut self) {
        self.peer_connection
            .unregister_data_channel(&self.servo_media_id);
    }
}

impl RTCDataChannelMethods for RTCDataChannel {
    // https://www.w3.org/TR/webrtc/#dom-rtcdatachannel-onopen
    event_handler!(open, GetOnopen, SetOnopen);
    // https://www.w3.org/TR/webrtc/#dom-rtcdatachannel-onbufferedamountlow
    event_handler!(
        bufferedamountlow,
        GetOnbufferedamountlow,
        SetOnbufferedamountlow
    );
    // https://www.w3.org/TR/webrtc/#dom-rtcdatachannel-onerror
    event_handler!(error, GetOnerror, SetOnerror);
    // https://www.w3.org/TR/webrtc/#dom-rtcdatachannel-onclosing
    event_handler!(closing, GetOnclosing, SetOnclosing);
    // https://www.w3.org/TR/webrtc/#dom-rtcdatachannel-onclose
    event_handler!(close, GetOnclose, SetOnclose);
    // https://www.w3.org/TR/webrtc/#dom-rtcdatachannel-onmessage
    event_handler!(message, GetOnmessage, SetOnmessage);

    // https://www.w3.org/TR/webrtc/#dom-datachannel-label
    fn Label(&self) -> USVString {
        self.label.clone()
    }
    // https://www.w3.org/TR/webrtc/#dom-datachannel-ordered
    fn Ordered(&self) -> bool {
        self.ordered
    }

    // https://www.w3.org/TR/webrtc/#dom-datachannel-maxpacketlifetime
    fn GetMaxPacketLifeTime(&self) -> Option<u16> {
        self.max_packet_life_time
    }

    // https://www.w3.org/TR/webrtc/#dom-datachannel-maxretransmits
    fn GetMaxRetransmits(&self) -> Option<u16> {
        self.max_retransmits
    }

    // https://www.w3.org/TR/webrtc/#dom-datachannel-protocol
    fn Protocol(&self) -> USVString {
        self.protocol.clone()
    }

    // https://www.w3.org/TR/webrtc/#dom-datachannel-negotiated
    fn Negotiated(&self) -> bool {
        self.negotiated
    }

    // https://www.w3.org/TR/webrtc/#dom-rtcdatachannel-id
    fn GetId(&self) -> Option<u16> {
        self.id
    }

    fn ReadyState(&self) -> RTCDataChannelState {
        *self.ready_state.borrow()
    }

    // XXX We need a way to know when the underlying data transport
    // actually sends data from its queue to decrease buffered amount.

    //    fn BufferedAmount(&self) -> u32;
    //    fn BufferedAmountLowThreshold(&self) -> u32;
    //    fn SetBufferedAmountLowThreshold(&self, value: u32) -> ();

    // https://www.w3.org/TR/webrtc/#dom-rtcdatachannel-close
    fn Close(&self) {
        let controller = self.peer_connection.get_webrtc_controller().borrow();
        controller
            .as_ref()
            .unwrap()
            .close_data_channel(&self.servo_media_id);
    }

    //    fn BinaryType(&self) -> DOMString;
    //    fn SetBinaryType(&self, value: DOMString) -> ();

    // https://www.w3.org/TR/webrtc/#dom-rtcdatachannel-send
    fn Send(&self, data: USVString) {
        let controller = self.peer_connection.get_webrtc_controller().borrow();
        controller
            .as_ref()
            .unwrap()
            .send_data_channel_message(&self.servo_media_id, DataChannelMessage::Text(data.0));
    }

    // https://www.w3.org/TR/webrtc/#dom-rtcdatachannel-send!overload-1
    // fn Send_(&self, data: &Blob) -> () {}

    // https://www.w3.org/TR/webrtc/#dom-rtcdatachannel-send!overload-2
    // fn Send__(&self, data: CustomAutoRooterGuard<ArrayBuffer>) -> () {}

    // https://www.w3.org/TR/webrtc/#dom-rtcdatachannel-send!overload-3
    // fn Send___(&self, data: CustomAutoRooterGuard<ArrayBufferView>) -> () {}
}

impl From<&RTCDataChannelInit> for DataChannelInit {
    fn from(init: &RTCDataChannelInit) -> DataChannelInit {
        DataChannelInit {
            label: String::new(),
            id: init.id,
            max_packet_life_time: init.maxPacketLifeTime,
            max_retransmits: init.maxRetransmits,
            negotiated: init.negotiated,
            ordered: init.ordered,
            protocol: init.protocol.to_string(),
        }
    }
}

impl From<DataChannelState> for RTCDataChannelState {
    fn from(state: DataChannelState) -> RTCDataChannelState {
        match state {
            DataChannelState::New |
            DataChannelState::Connecting |
            DataChannelState::__Unknown(_) => RTCDataChannelState::Connecting,
            DataChannelState::Open => RTCDataChannelState::Open,
            DataChannelState::Closing => RTCDataChannelState::Closing,
            DataChannelState::Closed => RTCDataChannelState::Closed,
        }
    }
}
