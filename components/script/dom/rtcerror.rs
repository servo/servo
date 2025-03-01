/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::rust::HandleObject;

use crate::dom::bindings::codegen::Bindings::RTCErrorBinding::{
    RTCErrorDetailType, RTCErrorInit, RTCErrorMethods,
};
use crate::dom::bindings::reflector::{reflect_dom_object_with_proto, DomGlobal};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::domexception::DOMException;
use crate::dom::globalscope::GlobalScope;
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct RTCError {
    exception: DOMException,
    error_detail: RTCErrorDetailType,
    sdp_line_number: Option<i32>,
    http_request_status_code: Option<i32>,
    sctp_cause_code: Option<i32>,
    received_alert: Option<u32>,
    sent_alert: Option<u32>,
}

impl RTCError {
    fn new_inherited(init: &RTCErrorInit, message: DOMString) -> RTCError {
        RTCError {
            exception: DOMException::new_inherited(message, "OperationError".into()),
            error_detail: init.errorDetail,
            sdp_line_number: init.sdpLineNumber,
            http_request_status_code: init.httpRequestStatusCode,
            sctp_cause_code: init.sctpCauseCode,
            received_alert: init.receivedAlert,
            sent_alert: init.sentAlert,
        }
    }

    pub(crate) fn new(
        global: &GlobalScope,
        init: &RTCErrorInit,
        message: DOMString,
        can_gc: CanGc,
    ) -> DomRoot<RTCError> {
        Self::new_with_proto(global, None, init, message, can_gc)
    }

    fn new_with_proto(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        init: &RTCErrorInit,
        message: DOMString,
        can_gc: CanGc,
    ) -> DomRoot<RTCError> {
        reflect_dom_object_with_proto(
            Box::new(RTCError::new_inherited(init, message)),
            global,
            proto,
            can_gc,
        )
    }
}

impl RTCErrorMethods<crate::DomTypeHolder> for RTCError {
    // https://www.w3.org/TR/webrtc/#dom-rtcerror-constructor
    fn Constructor(
        window: &Window,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        init: &RTCErrorInit,
        message: DOMString,
    ) -> DomRoot<RTCError> {
        RTCError::new_with_proto(&window.global(), proto, init, message, can_gc)
    }

    // https://www.w3.org/TR/webrtc/#dom-rtcerror-errordetail
    fn ErrorDetail(&self) -> RTCErrorDetailType {
        self.error_detail
    }

    // https://www.w3.org/TR/webrtc/#dom-rtcerror-sdplinenumber
    fn GetSdpLineNumber(&self) -> Option<i32> {
        self.sdp_line_number
    }

    // https://www.w3.org/TR/webrtc/#dom-rtcerror
    fn GetHttpRequestStatusCode(&self) -> Option<i32> {
        self.http_request_status_code
    }

    // https://www.w3.org/TR/webrtc/#dom-rtcerror-sctpcausecode
    fn GetSctpCauseCode(&self) -> Option<i32> {
        self.sctp_cause_code
    }

    // https://www.w3.org/TR/webrtc/#dom-rtcerror-receivedalert
    fn GetReceivedAlert(&self) -> Option<u32> {
        self.received_alert
    }

    // https://www.w3.org/TR/webrtc/#dom-rtcerror-sentalert
    fn GetSentAlert(&self) -> Option<u32> {
        self.sent_alert
    }
}
