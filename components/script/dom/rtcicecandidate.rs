/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::RTCIceCandidateBinding;
use crate::dom::bindings::codegen::Bindings::RTCIceCandidateBinding::RTCIceCandidateInit;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::reflector::{DomObject, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;
use crate::dom::window::Window;
use dom_struct::dom_struct;

#[dom_struct]
pub struct RTCIceCandidate {
    reflector: Reflector,
}

impl RTCIceCandidate {
    pub fn new_inherited() -> RTCIceCandidate {
        RTCIceCandidate {
            reflector: Reflector::new(),
        }
    }

    pub fn new(global: &GlobalScope) -> DomRoot<RTCIceCandidate> {
        reflect_dom_object(
            Box::new(RTCIceCandidate::new_inherited()),
            global,
            RTCIceCandidateBinding::Wrap,
        )
    }

    pub fn Constructor(
        window: &Window,
        _config: &RTCIceCandidateInit,
    ) -> Fallible<DomRoot<RTCIceCandidate>> {
        Ok(RTCIceCandidate::new(&window.global()))
    }
}
