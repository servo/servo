/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::XRRenderStateBinding::{self, XRRenderStateMethods};
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject, Reflector};
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::globalscope::GlobalScope;
use crate::dom::xrwebgllayer::XRWebGLLayer;

use dom_struct::dom_struct;
use std::cell::Cell;

#[dom_struct]
pub struct XRRenderState {
    reflector_: Reflector,
    depth_near: Cell<f64>,
    depth_far: Cell<f64>,
    inline_vertical_fov: Cell<Option<f64>>,
    layer: MutNullableDom<XRWebGLLayer>,
}

impl XRRenderState {
    pub fn new_inherited(
        depth_near: f64,
        depth_far: f64,
        inline_vertical_fov: Option<f64>,
        layer: Option<&XRWebGLLayer>,
    ) -> XRRenderState {
        XRRenderState {
            reflector_: Reflector::new(),
            depth_near: Cell::new(depth_near),
            depth_far: Cell::new(depth_far),
            inline_vertical_fov: Cell::new(inline_vertical_fov),
            layer: MutNullableDom::new(layer),
        }
    }

    pub fn new(
        global: &GlobalScope,
        depth_near: f64,
        depth_far: f64,
        inline_vertical_fov: Option<f64>,
        layer: Option<&XRWebGLLayer>,
    ) -> DomRoot<XRRenderState> {
        reflect_dom_object(
            Box::new(XRRenderState::new_inherited(
                depth_near,
                depth_far,
                inline_vertical_fov,
                layer,
            )),
            global,
            XRRenderStateBinding::Wrap,
        )
    }

    pub fn clone_object(&self) -> DomRoot<Self> {
        XRRenderState::new(
            &self.global(),
            self.depth_near.get(),
            self.depth_far.get(),
            self.inline_vertical_fov.get(),
            self.layer.get().as_ref().map(|x| &**x),
        )
    }

    pub fn set_depth_near(&self, depth: f64) {
        self.depth_near.set(depth)
    }
    pub fn set_depth_far(&self, depth: f64) {
        self.depth_far.set(depth)
    }
    pub fn set_inline_vertical_fov(&self, fov: f64) {
        debug_assert!(self.inline_vertical_fov.get().is_some());
        self.inline_vertical_fov.set(Some(fov))
    }
    pub fn set_layer(&self, layer: Option<&XRWebGLLayer>) {
        self.layer.set(layer)
    }
}

impl XRRenderStateMethods for XRRenderState {
    /// https://immersive-web.github.io/webxr/#dom-xrrenderstate-depthnear
    fn DepthNear(&self) -> Finite<f64> {
        Finite::wrap(self.depth_near.get())
    }

    /// https://immersive-web.github.io/webxr/#dom-xrrenderstate-depthfar
    fn DepthFar(&self) -> Finite<f64> {
        Finite::wrap(self.depth_far.get())
    }

    /// https://immersive-web.github.io/webxr/#dom-xrrenderstate-inlineverticalfieldofview
    fn GetInlineVerticalFieldOfView(&self) -> Option<Finite<f64>> {
        self.inline_vertical_fov.get().map(Finite::wrap)
    }

    /// https://immersive-web.github.io/webxr/#dom-xrrenderstate-baselayer
    fn GetBaseLayer(&self) -> Option<DomRoot<XRWebGLLayer>> {
        self.layer.get()
    }
}
