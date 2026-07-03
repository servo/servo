/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use dom_struct::dom_struct;
use js::context::JSContext;
use script_bindings::reflector::{Reflector, reflect_dom_object_with_cx};

use crate::dom::bindings::codegen::Bindings::RTCRtpTransceiverBinding::{
    RTCRtpTransceiverDirection, RTCRtpTransceiverMethods,
};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::globalscope::GlobalScope;
use crate::dom::rtcrtpsender::RTCRtpSender;

#[dom_struct]
pub(crate) struct RTCRtpTransceiver {
    reflector_: Reflector,
    sender: Dom<RTCRtpSender>,
    direction: Cell<RTCRtpTransceiverDirection>,
}

impl RTCRtpTransceiver {
    fn new_inherited(
        cx: &mut JSContext,
        global: &GlobalScope,
        direction: RTCRtpTransceiverDirection,
    ) -> Self {
        let sender = RTCRtpSender::new(cx, global);
        Self {
            reflector_: Reflector::new(),
            direction: Cell::new(direction),
            sender: Dom::from_ref(&*sender),
        }
    }

    pub(crate) fn new(
        cx: &mut JSContext,
        global: &GlobalScope,
        direction: RTCRtpTransceiverDirection,
    ) -> DomRoot<Self> {
        reflect_dom_object_with_cx(
            Box::new(Self::new_inherited(cx, global, direction)),
            global,
            cx,
        )
    }
}

impl RTCRtpTransceiverMethods<crate::DomTypeHolder> for RTCRtpTransceiver {
    /// <https://w3c.github.io/webrtc-pc/#dom-rtcrtptransceiver-direction>
    fn Direction(&self) -> RTCRtpTransceiverDirection {
        self.direction.get()
    }

    /// <https://w3c.github.io/webrtc-pc/#dom-rtcrtptransceiver-direction>
    fn SetDirection(&self, direction: RTCRtpTransceiverDirection) {
        self.direction.set(direction);
    }

    /// <https://w3c.github.io/webrtc-pc/#dom-rtcrtptransceiver-sender>
    fn Sender(&self) -> DomRoot<RTCRtpSender> {
        DomRoot::from_ref(&*self.sender)
    }
}
