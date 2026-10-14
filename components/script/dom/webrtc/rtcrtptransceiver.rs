/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use dom_struct::dom_struct;
use js::context::JSContext;
use script_bindings::cell::DomRefCell;
use script_bindings::reflector::{Reflector, reflect_dom_object};

use crate::dom::bindings::codegen::Bindings::RTCRtpTransceiverBinding::{
    RTCRtpTransceiverDirection, RTCRtpTransceiverMethods,
};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::rtcrtpreceiver::RTCRtpReceiver;
use crate::dom::rtcrtpsender::RTCRtpSender;

#[dom_struct]
pub(crate) struct RTCRtpTransceiver {
    reflector_: Reflector,
    sender: Dom<RTCRtpSender>,
    receiver: Dom<RTCRtpReceiver>,
    direction: Cell<RTCRtpTransceiverDirection>,
    current_direction: Cell<Option<RTCRtpTransceiverDirection>>,
    mid: DomRefCell<Option<String>>,
    stopped: Cell<bool>,
}

impl RTCRtpTransceiver {
    fn new_inherited(
        direction: RTCRtpTransceiverDirection,
        sender: &RTCRtpSender,
        receiver: &RTCRtpReceiver,
    ) -> Self {
        Self {
            reflector_: Reflector::new(),
            direction: Cell::new(direction),
            current_direction: Cell::new(Some(direction)),
            sender: Dom::from_ref(sender),
            receiver: Dom::from_ref(receiver),
            mid: DomRefCell::new(None),
            stopped: Cell::new(false),
        }
    }

    pub(crate) fn new(
        cx: &mut JSContext,
        global: &GlobalScope,
        direction: RTCRtpTransceiverDirection,
        sender: &RTCRtpSender,
        receiver: &RTCRtpReceiver,
    ) -> DomRoot<Self> {
        reflect_dom_object(
            cx,
            Box::new(Self::new_inherited(direction, sender, receiver)),
            global,
        )
    }

    pub(crate) fn set_mid(&self, mid: Option<String>) {
        *self.mid.borrow_mut() = mid;
    }

    pub(crate) fn set_current_direction(&self, direction: Option<RTCRtpTransceiverDirection>) {
        self.current_direction.set(direction);
    }

    pub(crate) fn is_stopped(&self) -> bool {
        self.stopped.get()
    }
}

impl RTCRtpTransceiverMethods<crate::DomTypeHolder> for RTCRtpTransceiver {
    /// <https://w3c.github.io/webrtc-pc/#dom-rtcrtptransceiver-mid>
    fn GetMid(&self) -> Option<DOMString> {
        self.mid
            .borrow()
            .as_ref()
            .map(|s| DOMString::from(s.clone()))
    }

    /// <https://w3c.github.io/webrtc-pc/#dom-rtcrtptransceiver-sender>
    fn Sender(&self) -> DomRoot<RTCRtpSender> {
        DomRoot::from_ref(&*self.sender)
    }

    /// <https://w3c.github.io/webrtc-pc/#dom-rtcrtptransceiver-receiver>
    fn Receiver(&self) -> DomRoot<RTCRtpReceiver> {
        DomRoot::from_ref(&*self.receiver)
    }

    /// <https://w3c.github.io/webrtc-pc/#dom-rtcrtptransceiver-direction>
    fn Direction(&self) -> RTCRtpTransceiverDirection {
        self.direction.get()
    }

    /// <https://w3c.github.io/webrtc-pc/#dom-rtcrtptransceiver-direction>
    fn SetDirection(&self, direction: RTCRtpTransceiverDirection) {
        self.direction.set(direction);
    }

    /// <https://w3c.github.io/webrtc-pc/#dom-rtcrtptransceiver-currentdirection>
    fn GetCurrentDirection(&self) -> Option<RTCRtpTransceiverDirection> {
        self.current_direction.get()
    }

    /// <https://w3c.github.io/webrtc-pc/#dom-rtcrtptransceiver-stop>
    fn Stop(&self) {
        self.stopped.set(true);
        self.current_direction.set(None);
        self.direction.set(RTCRtpTransceiverDirection::Stopped);
    }
}