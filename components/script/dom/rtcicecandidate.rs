/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::rust::HandleObject;

use crate::dom::bindings::codegen::Bindings::RTCIceCandidateBinding::{
    RTCIceCandidateInit, RTCIceCandidateMethods,
};
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::reflector::{reflect_dom_object_with_proto, DomGlobal, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct RTCIceCandidate {
    reflector: Reflector,
    candidate: DOMString,
    sdp_m_id: Option<DOMString>,
    sdp_m_line_index: Option<u16>,
    username_fragment: Option<DOMString>,
}

impl RTCIceCandidate {
    pub(crate) fn new_inherited(
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

    pub(crate) fn new(
        global: &GlobalScope,
        candidate: DOMString,
        sdp_m_id: Option<DOMString>,
        sdp_m_line_index: Option<u16>,
        username_fragment: Option<DOMString>,
        can_gc: CanGc,
    ) -> DomRoot<RTCIceCandidate> {
        Self::new_with_proto(
            global,
            None,
            candidate,
            sdp_m_id,
            sdp_m_line_index,
            username_fragment,
            can_gc,
        )
    }

    fn new_with_proto(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        candidate: DOMString,
        sdp_m_id: Option<DOMString>,
        sdp_m_line_index: Option<u16>,
        username_fragment: Option<DOMString>,
        can_gc: CanGc,
    ) -> DomRoot<RTCIceCandidate> {
        reflect_dom_object_with_proto(
            Box::new(RTCIceCandidate::new_inherited(
                candidate,
                sdp_m_id,
                sdp_m_line_index,
                username_fragment,
            )),
            global,
            proto,
            can_gc,
        )
    }
}

impl RTCIceCandidateMethods<crate::DomTypeHolder> for RTCIceCandidate {
    /// <https://w3c.github.io/webrtc-pc/#dom-rtcicecandidate-constructor>
    fn Constructor(
        window: &Window,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        config: &RTCIceCandidateInit,
    ) -> Fallible<DomRoot<RTCIceCandidate>> {
        if config.sdpMid.is_none() && config.sdpMLineIndex.is_none() {
            return Err(Error::Type(
                "one of sdpMid and sdpMLineIndex must be set".to_string(),
            ));
        }
        Ok(RTCIceCandidate::new_with_proto(
            &window.global(),
            proto,
            config.candidate.clone(),
            config.sdpMid.clone(),
            config.sdpMLineIndex,
            config.usernameFragment.clone(),
            can_gc,
        ))
    }

    /// <https://w3c.github.io/webrtc-pc/#dom-rtcicecandidate-candidate>
    fn Candidate(&self) -> DOMString {
        self.candidate.clone()
    }

    /// <https://w3c.github.io/webrtc-pc/#dom-rtcicecandidate-sdpmid>
    fn GetSdpMid(&self) -> Option<DOMString> {
        self.sdp_m_id.clone()
    }

    /// <https://w3c.github.io/webrtc-pc/#dom-rtcicecandidate-sdpmlineindex>
    fn GetSdpMLineIndex(&self) -> Option<u16> {
        self.sdp_m_line_index
    }

    /// <https://w3c.github.io/webrtc-pc/#dom-rtcicecandidate-usernamefragment>
    fn GetUsernameFragment(&self) -> Option<DOMString> {
        self.username_fragment.clone()
    }

    /// <https://w3c.github.io/webrtc-pc/#dom-rtcicecandidate-tojson>
    fn ToJSON(&self) -> RTCIceCandidateInit {
        RTCIceCandidateInit {
            candidate: self.candidate.clone(),
            sdpMid: self.sdp_m_id.clone(),
            sdpMLineIndex: self.sdp_m_line_index,
            usernameFragment: self.username_fragment.clone(),
        }
    }
}
