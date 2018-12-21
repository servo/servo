/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::XRBinding::XRSessionMode;
use crate::dom::bindings::codegen::Bindings::XRSessionBinding;
use crate::dom::bindings::codegen::Bindings::XRSessionBinding::XRFrameRequestCallback;
use crate::dom::bindings::codegen::Bindings::XRSessionBinding::XRSessionMethods;
use crate::dom::bindings::codegen::Bindings::XRWebGLLayerBinding::XRWebGLLayerMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::root::{Dom, DomRoot, MutNullableDom};
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::vrdisplay::VRDisplay;
use crate::dom::xrlayer::XRLayer;
use crate::dom::xrwebgllayer::XRWebGLLayer;
use dom_struct::dom_struct;
use std::cell::Cell;
use std::rc::Rc;

#[dom_struct]
pub struct XRSession {
    eventtarget: EventTarget,
    display: Dom<VRDisplay>,
    depth_near: Cell<f64>,
    depth_far: Cell<f64>,
    base_layer: MutNullableDom<XRLayer>,
}

impl XRSession {
    fn new_inherited(display: &VRDisplay) -> XRSession {
        XRSession {
            eventtarget: EventTarget::new_inherited(),
            display: Dom::from_ref(display),
            depth_near: Cell::new(0.1),
            depth_far: Cell::new(1000.),
            base_layer: Default::default(),
        }
    }

    pub fn new(global: &GlobalScope, display: &VRDisplay) -> DomRoot<XRSession> {
        reflect_dom_object(
            Box::new(XRSession::new_inherited(display)),
            global,
            XRSessionBinding::Wrap,
        )
    }
}

impl XRSessionMethods for XRSession {
    fn DepthNear(&self) -> Finite<f64> {
        Finite::wrap(self.depth_near.get())
    }

    fn DepthFar(&self) -> Finite<f64> {
        Finite::wrap(self.depth_far.get())
    }

    fn SetDepthNear(&self, d: Finite<f64>) {
        self.depth_near.set(*d)
    }

    fn SetDepthFar(&self, d: Finite<f64>) {
        self.depth_far.set(*d)
    }

    fn Mode(&self) -> XRSessionMode {
        XRSessionMode::Immersive_vr
    }

    fn SetBaseLayer(&self, layer: Option<&XRLayer>) {
        self.base_layer.set(layer);
        if let Some(layer) = layer {
            let layer = layer.downcast::<XRWebGLLayer>().unwrap();
            self.display.xr_present(&layer.Context());
        } else {
            // steps unknown
            // https://github.com/immersive-web/webxr/issues/453
        }
    }

    fn GetBaseLayer(&self) -> Option<DomRoot<XRLayer>> {
        self.base_layer.get()
    }

    fn RequestAnimationFrame(&self, callback: Rc<XRFrameRequestCallback>) -> i32 {
        self.display.xr_raf(callback) as i32
    }

    fn CancelAnimationFrame(&self, frame: i32) {
        self.display.xr_cancel_raf(frame)
    }
}
