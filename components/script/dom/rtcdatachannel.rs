/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::RTCDataChannelBinding::RTCDataChannelInit;
use crate::dom::bindings::codegen::Bindings::RTCDataChannelBinding::RTCDataChannelMethods;
use crate::dom::bindings::codegen::Bindings::RTCErrorBinding::{RTCErrorDetailType, RTCErrorInit};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::{DOMString, USVString};
use crate::dom::blob::Blob;
use crate::dom::event::{Event, EventBubbles, EventCancelable};
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::messageevent::MessageEvent;
use crate::dom::rtcerror::RTCError;
use crate::dom::rtcerrorevent::RTCErrorEvent;
use crate::task_source::TaskSource;
use dom_struct::dom_struct;
use js::conversions::ToJSValConvertible;
use js::jsapi::JSAutoRealm;
use js::jsval::UndefinedValue;
use js::rust::CustomAutoRooterGuard;
use js::typedarray::{ArrayBuffer, ArrayBufferView};
use servo_media::webrtc::{
    WebRtcController, WebRtcDataChannelBackend, WebRtcDataChannelInit, WebRtcError,
};
use std::sync::mpsc;

#[dom_struct]
pub struct RTCDataChannel {
    eventtarget: EventTarget,
    #[ignore_malloc_size_of = "defined in servo-media"]
    channel: Box<dyn WebRtcDataChannelBackend>,
    label: USVString,
    ordered: bool,
    max_packet_life_time: Option<u16>,
    max_retransmits: Option<u16>,
    protocol: USVString,
    negotiated: bool,
    id: Option<u16>,
}

impl RTCDataChannel {
    pub fn new_inherited(
        webrtc_controller: &DomRefCell<Option<WebRtcController>>,
        label: USVString,
        options: &RTCDataChannelInit,
        channel: Option<Box<dyn WebRtcDataChannelBackend>>,
    ) -> RTCDataChannel {
        let webrtc = webrtc_controller.borrow();
        let webrtc = webrtc.as_ref().unwrap();

        let (sender, receiver) = mpsc::channel::<Box<dyn WebRtcDataChannelBackend>>();

        let mut init: WebRtcDataChannelInit = options.into();
        init.label = label.to_string();

        let channel = if channel.is_none() {
            webrtc.create_data_channel(init, sender);
            receiver.recv().unwrap()
        } else {
            channel.unwrap()
        };

        RTCDataChannel {
            eventtarget: EventTarget::new_inherited(),
            channel,
            label,
            ordered: options.ordered,
            max_packet_life_time: options.maxPacketLifeTime,
            max_retransmits: options.maxRetransmits,
            protocol: options.protocol.clone(),
            negotiated: options.negotiated,
            id: options.id,
        }
    }

    pub fn new(
        global: &GlobalScope,
        webrtc_controller: &DomRefCell<Option<WebRtcController>>,
        label: USVString,
        options: &RTCDataChannelInit,
        channel: Option<Box<dyn WebRtcDataChannelBackend>>,
    ) -> DomRoot<RTCDataChannel> {
        let rtc_data_channel = reflect_dom_object(
            Box::new(RTCDataChannel::new_inherited(
                webrtc_controller,
                label,
                options,
                channel,
            )),
            global,
        );

        let trusted = Trusted::new(&*rtc_data_channel);
        let (task_source, canceller) = global
            .as_window()
            .task_manager()
            .networking_task_source_with_canceller();

        let this = trusted.clone();
        rtc_data_channel.channel.set_on_open(Box::new(move || {
            let this = this.clone();
            let _ = task_source.queue_with_canceller(
                task!(on_open: move || {
                    this.root().on_open();
                }),
                &canceller,
            );
        }));

        let this = trusted.clone();
        let (task_source, canceller) = global
            .as_window()
            .task_manager()
            .networking_task_source_with_canceller();
        rtc_data_channel.channel.set_on_close(Box::new(move || {
            let this = this.clone();
            let _ = task_source.queue_with_canceller(
                task!(on_close: move || {
                    this.root().on_close();
                }),
                &canceller,
            );
        }));

        let this = trusted.clone();
        let (task_source, canceller) = global
            .as_window()
            .task_manager()
            .networking_task_source_with_canceller();
        rtc_data_channel
            .channel
            .set_on_error(Box::new(move |error| {
                let this = this.clone();
                let _ = task_source.queue_with_canceller(
                    task!(on_error: move || {
                        this.root().on_error(error);
                    }),
                    &canceller,
                );
            }));

        let this = trusted.clone();
        let (task_source, canceller) = global
            .as_window()
            .task_manager()
            .networking_task_source_with_canceller();
        rtc_data_channel
            .channel
            .set_on_message(Box::new(move |message| {
                let this = this.clone();
                let _ = task_source.queue_with_canceller(
                    task!(on_message: move || {
                        this.root().on_message(message);
                    }),
                    &canceller,
                );
            }));

        rtc_data_channel
    }

    fn on_open(&self) {
        let event = Event::new(
            &self.global(),
            atom!("open"),
            EventBubbles::DoesNotBubble,
            EventCancelable::NotCancelable,
        );
        event.upcast::<Event>().fire(self.upcast());
    }

    fn on_close(&self) {
        let event = Event::new(
            &self.global(),
            atom!("close"),
            EventBubbles::DoesNotBubble,
            EventCancelable::NotCancelable,
        );
        event.upcast::<Event>().fire(self.upcast());
    }

    fn on_error(&self, error: WebRtcError) {
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
    fn on_message(&self, text: String) {
        // XXX(ferjm) Support binary messages
        unsafe {
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
        }
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

    //    fn ReadyState(&self) -> RTCDataChannelState;
    //    fn BufferedAmount(&self) -> u32;
    //    fn BufferedAmountLowThreshold(&self) -> u32;
    //    fn SetBufferedAmountLowThreshold(&self, value: u32) -> ();

    // https://www.w3.org/TR/webrtc/#dom-rtcdatachannel-close
    fn Close(&self) -> () {}

    //    fn BinaryType(&self) -> DOMString;
    //    fn SetBinaryType(&self, value: DOMString) -> ();

    // https://www.w3.org/TR/webrtc/#dom-rtcdatachannel-send
    fn Send(&self, data: USVString) -> () {
        if let Err(error) = self.channel.send(&data.0) {
            warn!("Could not send data channel message. Error: {:?}", error);
        }
    }

    // https://www.w3.org/TR/webrtc/#dom-rtcdatachannel-send!overload-1
    fn Send_(&self, data: &Blob) -> () {}

    // https://www.w3.org/TR/webrtc/#dom-rtcdatachannel-send!overload-2
    fn Send__(&self, data: CustomAutoRooterGuard<ArrayBuffer>) -> () {}

    // https://www.w3.org/TR/webrtc/#dom-rtcdatachannel-send!overload-3
    fn Send___(&self, data: CustomAutoRooterGuard<ArrayBufferView>) -> () {}
}

impl From<&RTCDataChannelInit> for WebRtcDataChannelInit {
    fn from(init: &RTCDataChannelInit) -> WebRtcDataChannelInit {
        WebRtcDataChannelInit {
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
