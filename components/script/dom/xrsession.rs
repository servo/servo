/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::compartments::InCompartment;
use crate::dom::bindings::codegen::Bindings::XRBinding::XRSessionMode;
use crate::dom::bindings::codegen::Bindings::XRReferenceSpaceBinding::XRReferenceSpaceType;
use crate::dom::bindings::codegen::Bindings::XRRenderStateBinding::XRRenderStateInit;
use crate::dom::bindings::codegen::Bindings::XRSessionBinding;
use crate::dom::bindings::codegen::Bindings::XRSessionBinding::XREnvironmentBlendMode;
use crate::dom::bindings::codegen::Bindings::XRSessionBinding::XRFrameRequestCallback;
use crate::dom::bindings::codegen::Bindings::XRSessionBinding::XRSessionMethods;
use crate::dom::bindings::error::Error;
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject};
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::dom::xrinputsource::XRInputSource;
use crate::dom::xrlayer::XRLayer;
use crate::dom::xrreferencespace::XRReferenceSpace;
use crate::dom::xrrenderstate::XRRenderState;
use crate::dom::xrspace::XRSpace;
use dom_struct::dom_struct;
use euclid::Vector3D;
use std::rc::Rc;
use webxr_api::Session;

#[dom_struct]
pub struct XRSession {
    eventtarget: EventTarget,
    base_layer: MutNullableDom<XRLayer>,
    blend_mode: XREnvironmentBlendMode,
    viewer_space: MutNullableDom<XRSpace>,
    #[ignore_malloc_size_of = "defined in webxr"]
    session: Session,
}

impl XRSession {
    fn new_inherited(session: Session) -> XRSession {
        XRSession {
            eventtarget: EventTarget::new_inherited(),
            base_layer: Default::default(),
            // we don't yet support any AR devices
            blend_mode: XREnvironmentBlendMode::Opaque,
            viewer_space: Default::default(),
            session,
        }
    }

    pub fn new(global: &GlobalScope, session: Session) -> DomRoot<XRSession> {
        reflect_dom_object(
            Box::new(XRSession::new_inherited(session)),
            global,
            XRSessionBinding::Wrap,
        )
    }

    pub fn set_layer(&self, layer: &XRLayer) {
        self.base_layer.set(Some(layer))
    }

    pub fn left_eye_params_offset(&self) -> Vector3D<f64> {
        unimplemented!()
    }

    pub fn right_eye_params_offset(&self) -> Vector3D<f64> {
        unimplemented!()
    }
}

impl XRSessionMethods for XRSession {
    /// https://immersive-web.github.io/webxr/#dom-xrsession-mode
    fn Mode(&self) -> XRSessionMode {
        XRSessionMode::Immersive_vr
    }

    // https://immersive-web.github.io/webxr/#dom-xrsession-renderstate
    fn RenderState(&self) -> DomRoot<XRRenderState> {
        unimplemented!()
    }

    /// https://immersive-web.github.io/webxr/#dom-xrsession-requestanimationframe
    fn UpdateRenderState(&self, init: &XRRenderStateInit, comp: InCompartment) -> Rc<Promise> {
        unimplemented!()
    }

    /// https://immersive-web.github.io/webxr/#dom-xrsession-requestanimationframe
    fn RequestAnimationFrame(&self, callback: Rc<XRFrameRequestCallback>) -> i32 {
        unimplemented!()
    }

    /// https://immersive-web.github.io/webxr/#dom-xrsession-cancelanimationframe
    fn CancelAnimationFrame(&self, frame: i32) {
        unimplemented!()
    }

    /// https://immersive-web.github.io/webxr/#dom-xrsession-environmentblendmode
    fn EnvironmentBlendMode(&self) -> XREnvironmentBlendMode {
        self.blend_mode
    }

    /// https://immersive-web.github.io/webxr/#dom-xrsession-requestreferencespace
    fn RequestReferenceSpace(&self, ty: XRReferenceSpaceType, comp: InCompartment) -> Rc<Promise> {
        let p = Promise::new_in_current_compartment(&self.global(), comp);

        // https://immersive-web.github.io/webxr/#create-a-reference-space

        // XXXManishearth reject based on session type
        // https://github.com/immersive-web/webxr/blob/master/spatial-tracking-explainer.md#practical-usage-guidelines

        match ty {
            XRReferenceSpaceType::Bounded_floor | XRReferenceSpaceType::Unbounded => {
                // XXXManishearth eventually support these
                p.reject_error(Error::NotSupported)
            },
            ty => {
                p.resolve_native(&XRReferenceSpace::new(&self.global(), self, ty));
            },
        }

        p
    }

    /// https://immersive-web.github.io/webxr/#dom-xrsession-getinputsources
    fn GetInputSources(&self) -> Vec<DomRoot<XRInputSource>> {
        unimplemented!()
    }
}
