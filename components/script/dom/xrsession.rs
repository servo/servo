/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::VRDisplayBinding::VRDisplayMethods;
use crate::dom::bindings::codegen::Bindings::XRBinding::XRSessionMode;
use crate::dom::bindings::codegen::Bindings::XRSessionBinding;
use crate::dom::bindings::codegen::Bindings::XRSessionBinding::XREnvironmentBlendMode;
use crate::dom::bindings::codegen::Bindings::XRSessionBinding::XRFrameRequestCallback;
use crate::dom::bindings::codegen::Bindings::XRSessionBinding::XRSessionMethods;
use crate::dom::bindings::codegen::Bindings::XRWebGLLayerBinding::XRWebGLLayerMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::root::{Dom, DomRoot, MutNullableDom};
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::dom::vrdisplay::VRDisplay;
use crate::dom::xrlayer::XRLayer;
use crate::dom::xrwebgllayer::XRWebGLLayer;
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
}

impl XRSessionMethods for XRSession {
    /// https://immersive-web.github.io/webxr/#dom-xrsession-depthnear
    fn DepthNear(&self) -> Finite<f64> {
        self.display.DepthNear()
    }

    /// https://immersive-web.github.io/webxr/#dom-xrsession-depthfar
    fn DepthFar(&self) -> Finite<f64> {
        self.display.DepthFar()
    }

    /// https://immersive-web.github.io/webxr/#dom-xrsession-depthnear
    fn SetDepthNear(&self, d: Finite<f64>) {
        self.display.SetDepthNear(d)
    }

    /// https://immersive-web.github.io/webxr/#dom-xrsession-depthfar
    fn SetDepthFar(&self, d: Finite<f64>) {
        self.display.SetDepthFar(d)
    }

    /// https://immersive-web.github.io/webxr/#dom-xrsession-mode
    fn Mode(&self) -> XRSessionMode {
        XRSessionMode::Immersive_vr
    }

    /// https://immersive-web.github.io/webxr/#dom-xrsession-baselayer
    fn SetBaseLayer(&self, layer: Option<&XRLayer>) {
        self.base_layer.set(layer);
        if let Some(layer) = layer {
            let layer = layer.downcast::<XRWebGLLayer>().unwrap();
            self.display.xr_present(&self, Some(&layer.Context()), None);
        } else {
            // steps unknown
            // https://github.com/immersive-web/webxr/issues/453
        }
    }

    /// https://immersive-web.github.io/webxr/#dom-xrsession-baselayer
    fn GetBaseLayer(&self) -> Option<DomRoot<XRLayer>> {
        self.base_layer.get()
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
}
