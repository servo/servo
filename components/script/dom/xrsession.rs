/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::VRDisplayBinding::VRDisplayMethods;
use crate::dom::bindings::codegen::Bindings::XRBinding::XRSessionMode;
use crate::dom::bindings::codegen::Bindings::XRRenderStateBinding::XRRenderStateInit;
use crate::dom::bindings::codegen::Bindings::XRSessionBinding;
use crate::dom::bindings::codegen::Bindings::XRSessionBinding::XREnvironmentBlendMode;
use crate::dom::bindings::codegen::Bindings::XRSessionBinding::XRFrameRequestCallback;
use crate::dom::bindings::codegen::Bindings::XRSessionBinding::XRReferenceSpaceOptions;
use crate::dom::bindings::codegen::Bindings::XRSessionBinding::XRReferenceSpaceType;
use crate::dom::bindings::codegen::Bindings::XRSessionBinding::XRSessionMethods;
use crate::dom::bindings::error::Error;
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject};
use crate::dom::bindings::root::{Dom, DomRoot, MutNullableDom};
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::dom::vrdisplay::VRDisplay;
use crate::dom::xrlayer::XRLayer;
use crate::dom::xrreferencespace::XRReferenceSpace;
use crate::dom::xrrenderstate::XRRenderState;
use crate::dom::xrstationaryreferencespace::XRStationaryReferenceSpace;
use dom_struct::dom_struct;
use std::rc::Rc;

#[dom_struct]
pub struct XRSession {
    eventtarget: EventTarget,
    display: Dom<VRDisplay>,
    base_layer: MutNullableDom<XRLayer>,
    blend_mode: XREnvironmentBlendMode,
}

impl XRSession {
    fn new_inherited(display: &VRDisplay) -> XRSession {
        XRSession {
            eventtarget: EventTarget::new_inherited(),
            display: Dom::from_ref(display),
            base_layer: Default::default(),
            // we don't yet support any AR devices
            blend_mode: XREnvironmentBlendMode::Opaque,
        }
    }

    pub fn new(global: &GlobalScope, display: &VRDisplay) -> DomRoot<XRSession> {
        reflect_dom_object(
            Box::new(XRSession::new_inherited(display)),
            global,
            XRSessionBinding::Wrap,
        )
    }

    pub fn xr_present(&self, p: Rc<Promise>) {
        self.display.xr_present(self, None, Some(p));
    }

    pub fn display(&self) -> &VRDisplay {
        &self.display
    }

    pub fn set_layer(&self, layer: &XRLayer) {
        self.base_layer.set(Some(layer))
    }
}

impl XRSessionMethods for XRSession {
    /// https://immersive-web.github.io/webxr/#dom-xrsession-mode
    fn Mode(&self) -> XRSessionMode {
        XRSessionMode::Immersive_vr
    }

    // https://immersive-web.github.io/webxr/#dom-xrsession-renderstate
    fn RenderState(&self) -> DomRoot<XRRenderState> {
        // XXXManishearth maybe cache this
        XRRenderState::new(
            &self.global(),
            *self.display.DepthNear(),
            *self.display.DepthFar(),
            self.base_layer.get().as_ref().map(|l| &**l),
        )
    }

    /// https://immersive-web.github.io/webxr/#dom-xrsession-requestanimationframe
    fn UpdateRenderState(&self, init: &XRRenderStateInit) -> Rc<Promise> {
        let p = Promise::new(&self.global());
        self.display.queue_renderstate(init, p.clone());
        p
    }

    /// https://immersive-web.github.io/webxr/#dom-xrsession-requestanimationframe
    fn RequestAnimationFrame(&self, callback: Rc<XRFrameRequestCallback>) -> i32 {
        self.display.xr_raf(callback) as i32
    }

    /// https://immersive-web.github.io/webxr/#dom-xrsession-cancelanimationframe
    fn CancelAnimationFrame(&self, frame: i32) {
        self.display.xr_cancel_raf(frame)
    }

    /// https://immersive-web.github.io/webxr/#dom-xrsession-environmentblendmode
    fn EnvironmentBlendMode(&self) -> XREnvironmentBlendMode {
        self.blend_mode
    }

    /// https://immersive-web.github.io/webxr/#dom-xrsession-requestreferencespace
    fn RequestReferenceSpace(&self, options: &XRReferenceSpaceOptions) -> Rc<Promise> {
        let p = Promise::new(&self.global());

        // https://immersive-web.github.io/webxr/#create-a-reference-space

        // XXXManishearth reject based on session type
        // https://github.com/immersive-web/webxr/blob/master/spatial-tracking-explainer.md#practical-usage-guidelines

        match options.type_ {
            XRReferenceSpaceType::Identity => {
                p.resolve_native(&XRReferenceSpace::identity(&self.global(), self));
            },
            XRReferenceSpaceType::Stationary => {
                if let Some(subtype) = options.subtype {
                    p.resolve_native(&XRStationaryReferenceSpace::new(
                        &self.global(),
                        self,
                        subtype,
                    ));
                } else {
                    p.reject_error(Error::Type(format!(
                        "stationary XRReferenceSpaces must specify a subtype"
                    )))
                }
            },
            XRReferenceSpaceType::Bounded | XRReferenceSpaceType::Unbounded => {
                // XXXManishearth eventually support these
                p.reject_error(Error::NotSupported)
            },
        }

        p
    }
}
