/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::RTCIceCandidateBinding::RTCIceCandidateInit;
use crate::dom::bindings::codegen::Bindings::RTCIceCandidateBinding::RTCIceCandidateMethods;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::reflector::{DomObject, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::window::Window;
use dom_struct::dom_struct;

#[dom_struct]
pub struct RTCIceCandidate {
    reflector: Reflector,
    candidate: DOMString,
    sdp_m_id: Option<DOMString>,
    sdp_m_line_index: Option<u16>,
    username_fragment: Option<DOMString>,
}

impl RTCIceCandidate {
    pub fn new_inherited(
        candidate: DOMString,
        sdp_m_id: Option<DOMString>,
        sdp_m_line_index: Option<u16>,
        username_fragment: Option<DOMString>,
    ) -> RTCIceCandidate {
        RTCIceCandidate {
            reflector: Reflector::new(),
            candidate,
            sdp_m_id,
            sdp_m_line_index,
            username_fragment,
        }
    }

    pub fn new(
        global: &GlobalScope,
        candidate: DOMString,
        sdp_m_id: Option<DOMString>,
        sdp_m_line_index: Option<u16>,
        username_fragment: Option<DOMString>,
    ) -> DomRoot<RTCIceCandidate> {
        reflect_dom_object(
            Box::new(RTCIceCandidate::new_inherited(
                candidate,
                sdp_m_id,
                sdp_m_line_index,
                username_fragment,
            )),
            global,
        )
    }

    #[allow(non_snake_case)]
    pub fn Constructor(
        window: &Window,
        config: &RTCIceCandidateInit,
    ) -> Fallible<DomRoot<RTCIceCandidate>> {
        if config.sdpMid.is_none() && config.sdpMLineIndex.is_none() {
            return Err(Error::Type(format!(
                "one of sdpMid and sdpMLineIndex must be set"
            )));
        }
        Ok(RTCIceCandidate::new(
            &window.global(),
            config.candidate.clone(),
            config.sdpMid.clone(),
            config.sdpMLineIndex,
            config.usernameFragment.clone(),
        ))
    }
}

impl RTCIceCandidateMethods for RTCIceCandidate {
    /// https://w3c.github.io/webrtc-pc/#dom-rtcicecandidate-candidate
    fn Candidate(&self) -> DOMString {
        self.candidate.clone()
    }

    /// https://w3c.github.io/webrtc-pc/#dom-rtcicecandidate-sdpmid
    fn GetSdpMid(&self) -> Option<DOMString> {
        self.sdp_m_id.clone()
    }

    /// https://w3c.github.io/webrtc-pc/#dom-rtcicecandidate-sdpmlineindex
    fn GetSdpMLineIndex(&self) -> Option<u16> {
        self.sdp_m_line_index.clone()
    }

    /// https://w3c.github.io/webrtc-pc/#dom-rtcicecandidate-usernamefragment
    fn GetUsernameFragment(&self) -> Option<DOMString> {
        self.username_fragment.clone()
    }

    /// https://w3c.github.io/webrtc-pc/#dom-rtcicecandidate-tojson
    fn ToJSON(&self) -> RTCIceCandidateInit {
        RTCIceCandidateInit {
            candidate: self.candidate.clone(),
            sdpMid: self.sdp_m_id.clone(),
            sdpMLineIndex: self.sdp_m_line_index.clone(),
            usernameFragment: self.username_fragment.clone(),
        }
    }
}
