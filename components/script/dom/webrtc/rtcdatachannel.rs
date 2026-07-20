/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::ptr;

use dom_struct::dom_struct;
use js::context::JSContext;
use js::conversions::ToJSValConvertible;
use js::jsapi::JSObject;
use js::jsval::UndefinedValue;
use js::realm::CurrentRealm;
use js::rust::CustomAutoRooterGuard;
use js::typedarray::{ArrayBuffer, ArrayBufferView, CreateWith};
use script_bindings::cell::DomRefCell;
use script_bindings::match_domstring_ascii;
use script_bindings::reflector::reflect_dom_object_with_cx;
use script_bindings::weakref::WeakRef;
use servo_constellation_traits::BlobImpl;
use servo_media::webrtc::{
    DataChannelId, DataChannelInit, DataChannelMessage, DataChannelState, WebRtcError,
};

use crate::conversions::Convert;
use crate::dom::bindings::codegen::Bindings::RTCDataChannelBinding::{
    RTCDataChannelInit, RTCDataChannelMethods, RTCDataChannelState,
};
use crate::dom::bindings::codegen::Bindings::RTCErrorBinding::{RTCErrorDetailType, RTCErrorInit};
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::{DOMString, USVString};
use crate::dom::blob::Blob;
use crate::dom::event::{Event, EventBubbles, EventCancelable};
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::messageevent::MessageEvent;
use crate::dom::rtcerror::RTCError;
use crate::dom::rtcerrorevent::RTCErrorEvent;
use crate::dom::rtcpeerconnection::RTCPeerConnection;

#[derive(JSTraceable, MallocSizeOf)]
struct DroppableRTCDataChannel {
    #[ignore_malloc_size_of = "defined in servo-media"]
    servo_media_id: DataChannelId,
    peer_connection: WeakRef<RTCPeerConnection>,
}

impl DroppableRTCDataChannel {
    fn new(peer_connection: WeakRef<RTCPeerConnection>, servo_media_id: DataChannelId) -> Self {
        DroppableRTCDataChannel {
            servo_media_id,
            peer_connection,
        }
    }

    pub(crate) fn get_servo_media_id(&self) -> DataChannelId {
        self.servo_media_id
    }
}

impl Drop for DroppableRTCDataChannel {
    fn drop(&mut self) {
        if let Some(root) = self.peer_connection.root() {
            root.unregister_data_channel(&self.get_servo_media_id());
        }
    }
}

#[dom_struct]
pub(crate) struct RTCDataChannel {
    eventtarget: EventTarget,
    label: USVString,
    ordered: bool,
    max_packet_life_time: Option<u16>,
    max_retransmits: Option<u16>,
    protocol: USVString,
    negotiated: bool,
    id: Option<u16>,
    ready_state: Cell<RTCDataChannelState>,
    binary_type: DomRefCell<DOMString>,
    peer_connection: Dom<RTCPeerConnection>,
    droppable: DroppableRTCDataChannel,
}

impl RTCDataChannel {
    pub(crate) fn new_inherited(
        peer_connection: &RTCPeerConnection,
        label: USVString,
        options: &RTCDataChannelInit,
        servo_media_id: Option<DataChannelId>,
    ) -> RTCDataChannel {
        let mut init: DataChannelInit = options.convert();
        init.label = label.0.clone();

        let controller = peer_connection.get_webrtc_controller().borrow();
        let servo_media_id = servo_media_id.unwrap_or(
            controller
                .as_ref()
                .unwrap()
                .create_data_channel(init)
                .expect("Expected data channel id"),
        );

        RTCDataChannel {
            eventtarget: EventTarget::new_inherited(),
            label,
            ordered: options.ordered,
            max_packet_life_time: options.maxPacketLifeTime,
            max_retransmits: options.maxRetransmits,
            protocol: options.protocol.clone(),
            negotiated: options.negotiated,
            id: options.id,
            ready_state: Cell::new(RTCDataChannelState::Connecting),
            binary_type: DomRefCell::new(DOMString::from("blob")),
            peer_connection: Dom::from_ref(peer_connection),
            droppable: DroppableRTCDataChannel::new(WeakRef::new(peer_connection), servo_media_id),
        }
    }

    pub(crate) fn new(
        cx: &mut JSContext,
        global: &GlobalScope,
        peer_connection: &RTCPeerConnection,
        label: USVString,
        options: &RTCDataChannelInit,
        servo_media_id: Option<DataChannelId>,
    ) -> DomRoot<RTCDataChannel> {
        let rtc_data_channel = reflect_dom_object_with_cx(
            Box::new(RTCDataChannel::new_inherited(
                peer_connection,
                label,
                options,
                servo_media_id,
            )),
            global,
            cx,
        );

        peer_connection
            .register_data_channel(rtc_data_channel.get_servo_media_id(), &rtc_data_channel);

        rtc_data_channel
    }

    pub(crate) fn get_servo_media_id(&self) -> DataChannelId {
        self.droppable.get_servo_media_id()
    }

    pub(crate) fn on_open(&self, cx: &mut JSContext) {
        let event = Event::new(
            cx,
            &self.global(),
            atom!("open"),
            EventBubbles::DoesNotBubble,
            EventCancelable::NotCancelable,
        );
        event.upcast::<Event>().fire(cx, self.upcast());
    }

    pub(crate) fn on_close(&self, cx: &mut JSContext) {
        let event = Event::new(
            cx,
            &self.global(),
            atom!("close"),
            EventBubbles::DoesNotBubble,
            EventCancelable::NotCancelable,
        );
        event.upcast::<Event>().fire(cx, self.upcast());

        self.peer_connection
            .unregister_data_channel(&self.get_servo_media_id());
    }

    pub(crate) fn on_error(&self, cx: &mut CurrentRealm, error: WebRtcError) {
        let global = self.global();
        let window = global.as_window();
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
        let error = RTCError::new(cx, window, &init, message);
        let event = RTCErrorEvent::new(cx, window, atom!("error"), false, false, &error);
        event.upcast::<Event>().fire(cx, self.upcast());
    }

    #[expect(unsafe_code)]
    pub(crate) fn on_message(&self, cx: &mut CurrentRealm, channel_message: DataChannelMessage) {
        let global = self.global();
        rooted!(&in(cx) let mut message = UndefinedValue());

        match channel_message {
            DataChannelMessage::Text(text) => {
                text.safe_to_jsval(cx, message.handle_mut());
            },
            DataChannelMessage::Binary(data) => {
                let binary_type = self.binary_type.borrow();
                match_domstring_ascii!(binary_type,
                    "blob" => {
                        let blob = Blob::new(
                            cx,
                            &global,
                            BlobImpl::new_from_bytes(data, "".to_owned()),
                        );
                        blob.safe_to_jsval(cx, message.handle_mut());
                    },
                    "arraybuffer" => {
                        rooted!(&in(cx) let mut array_buffer = ptr::null_mut::<JSObject>());
                        unsafe {
                            assert!(
                                ArrayBuffer::create(
                                    cx.raw_cx(),
                                    CreateWith::Slice(&data),
                                    array_buffer.handle_mut()
                                )
                                .is_ok()
                            )
                        };
                        (*array_buffer).safe_to_jsval(cx, message.handle_mut());
                    },
                    _ => unreachable!(),
                )
            },
        }

        MessageEvent::dispatch_jsval(
            cx,
            self.upcast(),
            &global,
            message.handle(),
            Some(&global.origin().immutable().ascii_serialization()),
            None,
            vec![],
        );
    }

    pub(crate) fn on_state_change(&self, cx: &mut JSContext, state: DataChannelState) {
        if let DataChannelState::Closing = state {
            let event = Event::new(
                cx,
                &self.global(),
                atom!("closing"),
                EventBubbles::DoesNotBubble,
                EventCancelable::NotCancelable,
            );
            event.upcast::<Event>().fire(cx, self.upcast());
        };
        self.ready_state.set(state.convert());
    }

    fn send(&self, source: &SendSource) -> Fallible<()> {
        if self.ready_state.get() != RTCDataChannelState::Open {
            return Err(Error::InvalidState(None));
        }

        let message = match source {
            SendSource::String(string) => DataChannelMessage::Text(string.0.clone()),
            SendSource::Blob(blob) => {
                DataChannelMessage::Binary(blob.get_bytes().unwrap_or(vec![]))
            },
            SendSource::ArrayBuffer(array) => {
                DataChannelMessage::Binary(array.to_vec().unwrap_or_default())
            },
            SendSource::ArrayBufferView(array) => {
                DataChannelMessage::Binary(array.to_vec().unwrap_or_default())
            },
        };

        let controller = self.peer_connection.get_webrtc_controller().borrow();
        controller
            .as_ref()
            .unwrap()
            .send_data_channel_message(&self.get_servo_media_id(), message);

        Ok(())
    }
}

enum SendSource<'a, 'b> {
    String(&'a USVString),
    Blob(&'a Blob),
    ArrayBuffer(CustomAutoRooterGuard<'b, ArrayBuffer>),
    ArrayBufferView(CustomAutoRooterGuard<'b, ArrayBufferView>),
}

impl RTCDataChannelMethods<crate::DomTypeHolder> for RTCDataChannel {
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

    /// <https://www.w3.org/TR/webrtc/#dom-datachannel-label>
    fn Label(&self) -> USVString {
        self.label.clone()
    }
    /// <https://www.w3.org/TR/webrtc/#dom-datachannel-ordered>
    fn Ordered(&self) -> bool {
        self.ordered
    }

    /// <https://www.w3.org/TR/webrtc/#dom-datachannel-maxpacketlifetime>
    fn GetMaxPacketLifeTime(&self) -> Option<u16> {
        self.max_packet_life_time
    }

    /// <https://www.w3.org/TR/webrtc/#dom-datachannel-maxretransmits>
    fn GetMaxRetransmits(&self) -> Option<u16> {
        self.max_retransmits
    }

    /// <https://www.w3.org/TR/webrtc/#dom-datachannel-protocol>
    fn Protocol(&self) -> USVString {
        self.protocol.clone()
    }

    /// <https://www.w3.org/TR/webrtc/#dom-datachannel-negotiated>
    fn Negotiated(&self) -> bool {
        self.negotiated
    }

    /// <https://www.w3.org/TR/webrtc/#dom-rtcdatachannel-id>
    fn GetId(&self) -> Option<u16> {
        self.id
    }

    /// <https://www.w3.org/TR/webrtc/#dom-datachannel-readystate>
    fn ReadyState(&self) -> RTCDataChannelState {
        self.ready_state.get()
    }

    // XXX We need a way to know when the underlying data transport
    // actually sends data from its queue to decrease buffered amount.

    //    fn BufferedAmount(&self) -> u32;
    //    fn BufferedAmountLowThreshold(&self) -> u32;
    //    fn SetBufferedAmountLowThreshold(&self, value: u32) -> ();

    /// <https://www.w3.org/TR/webrtc/#dom-rtcdatachannel-close>
    fn Close(&self) {
        let controller = self.peer_connection.get_webrtc_controller().borrow();
        controller
            .as_ref()
            .unwrap()
            .close_data_channel(&self.get_servo_media_id());
    }

    /// <https://www.w3.org/TR/webrtc/#dom-datachannel-binarytype>
    fn BinaryType(&self) -> DOMString {
        self.binary_type.borrow().clone()
    }

    /// <https://www.w3.org/TR/webrtc/#dom-datachannel-binarytype>
    fn SetBinaryType(&self, value: DOMString) -> Fallible<()> {
        if value != "blob" || value != "arraybuffer" {
            return Err(Error::Syntax(None));
        }
        *self.binary_type.borrow_mut() = value;
        Ok(())
    }

    /// <https://www.w3.org/TR/webrtc/#dom-rtcdatachannel-send>
    fn Send(&self, data: USVString) -> Fallible<()> {
        self.send(&SendSource::String(&data))
    }

    /// <https://www.w3.org/TR/webrtc/#dom-rtcdatachannel-send!overload-1>
    fn Send_(&self, data: &Blob) -> Fallible<()> {
        self.send(&SendSource::Blob(data))
    }

    /// <https://www.w3.org/TR/webrtc/#dom-rtcdatachannel-send!overload-2>
    fn Send__(&self, data: CustomAutoRooterGuard<ArrayBuffer>) -> Fallible<()> {
        self.send(&SendSource::ArrayBuffer(data))
    }

    /// <https://www.w3.org/TR/webrtc/#dom-rtcdatachannel-send!overload-3>
    fn Send___(&self, data: CustomAutoRooterGuard<ArrayBufferView>) -> Fallible<()> {
        self.send(&SendSource::ArrayBufferView(data))
    }
}

impl Convert<DataChannelInit> for &RTCDataChannelInit {
    fn convert(self) -> DataChannelInit {
        DataChannelInit {
            label: String::new(),
            id: self.id,
            max_packet_life_time: self.maxPacketLifeTime,
            max_retransmits: self.maxRetransmits,
            negotiated: self.negotiated,
            ordered: self.ordered,
            protocol: self.protocol.to_string(),
        }
    }
}

impl Convert<RTCDataChannelState> for DataChannelState {
    fn convert(self) -> RTCDataChannelState {
        match self {
            DataChannelState::Connecting | DataChannelState::__Unknown(_) => {
                RTCDataChannelState::Connecting
            },
            DataChannelState::Open => RTCDataChannelState::Open,
            DataChannelState::Closing => RTCDataChannelState::Closing,
            DataChannelState::Closed => RTCDataChannelState::Closed,
        }
    }
}
