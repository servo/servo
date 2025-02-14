/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use dom_struct::dom_struct;

use crate::dom::bindings::codegen::Bindings::RTCRtpSenderBinding::{
    RTCRtcpParameters, RTCRtpParameters, RTCRtpSendParameters, RTCRtpSenderMethods,
};
use crate::dom::bindings::reflector::{reflect_dom_object, DomGlobal, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct RTCRtpSender {
    reflector_: Reflector,
}

impl RTCRtpSender {
    fn new_inherited() -> Self {
        Self {
            reflector_: Reflector::new(),
        }
    }

    pub(crate) fn new(global: &GlobalScope) -> DomRoot<Self> {
        reflect_dom_object(Box::new(Self::new_inherited()), global, CanGc::note())
    }
}

impl RTCRtpSenderMethods<crate::DomTypeHolder> for RTCRtpSender {
    // https://w3c.github.io/webrtc-pc/#dom-rtcrtpsender-getparameters
    fn GetParameters(&self) -> RTCRtpSendParameters {
        RTCRtpSendParameters {
            parent: RTCRtpParameters {
                headerExtensions: vec![],
                rtcp: RTCRtcpParameters {
                    cname: None,
                    reducedSize: None,
                },
                codecs: vec![],
            },
            transactionId: DOMString::new(),
            encodings: vec![],
        }
    }

    // https://w3c.github.io/webrtc-pc/#dom-rtcrtpsender-setparameters
    fn SetParameters(&self, _parameters: &RTCRtpSendParameters, can_gc: CanGc) -> Rc<Promise> {
        let promise = Promise::new(&self.global(), can_gc);
        promise.resolve_native(&());
        promise
    }
}
