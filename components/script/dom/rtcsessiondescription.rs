/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::rust::HandleObject;

use crate::dom::bindings::codegen::Bindings::RTCSessionDescriptionBinding::{
    RTCSdpType, RTCSessionDescriptionInit, RTCSessionDescriptionMethods,
};
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::reflector::{reflect_dom_object_with_proto, DomObject, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::window::Window;

#[dom_struct]
pub struct RTCSessionDescription {
    reflector: Reflector,
    ty: RTCSdpType,
    sdp: DOMString,
}

impl RTCSessionDescription {
    pub fn new_inherited(ty: RTCSdpType, sdp: DOMString) -> RTCSessionDescription {
        RTCSessionDescription {
            reflector: Reflector::new(),
            ty,
            sdp,
        }
    }

    fn new(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        ty: RTCSdpType,
        sdp: DOMString,
    ) -> DomRoot<RTCSessionDescription> {
        reflect_dom_object_with_proto(
            Box::new(RTCSessionDescription::new_inherited(ty, sdp)),
            global,
            proto,
        )
    }

    #[allow(non_snake_case)]
    pub fn Constructor(
        window: &Window,
        proto: Option<HandleObject>,
        config: &RTCSessionDescriptionInit,
    ) -> Fallible<DomRoot<RTCSessionDescription>> {
        Ok(RTCSessionDescription::new(
            &window.global(),
            proto,
            config.type_,
            config.sdp.clone(),
        ))
    }
}

impl RTCSessionDescriptionMethods for RTCSessionDescription {
    /// <https://w3c.github.io/webrtc-pc/#dom-rtcsessiondescription-type>
    fn Type(&self) -> RTCSdpType {
        self.ty
    }

    /// <https://w3c.github.io/webrtc-pc/#dom-rtcsessiondescription-sdp>
    fn Sdp(&self) -> DOMString {
        self.sdp.clone()
    }
}
