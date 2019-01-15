/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::XRRenderStateBinding::{self, XRRenderStateMethods};
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::reflector::Reflector;
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::globalscope::GlobalScope;
use crate::dom::xrlayer::XRLayer;

use dom_struct::dom_struct;
use std::cell::Cell;

#[dom_struct]
pub struct XRRenderState {
    reflector_: Reflector,
    depth_near: Cell<f64>,
    depth_far: Cell<f64>,
    layer: MutNullableDom<XRLayer>,
}

impl XRRenderState {
    pub fn new_inherited(
        depth_near: f64,
        depth_far: f64,
        layer: Option<&XRLayer>,
    ) -> XRRenderState {
        XRRenderState {
            reflector_: Reflector::new(),
            depth_near: Cell::new(depth_near),
            depth_far: Cell::new(depth_far),
            layer: MutNullableDom::new(layer),
        }
    }

    pub fn new(
        global: &GlobalScope,
        depth_near: f64,
        depth_far: f64,
        layer: Option<&XRLayer>,
    ) -> DomRoot<XRRenderState> {
        reflect_dom_object(
            Box::new(XRRenderState::new_inherited(depth_near, depth_far, layer)),
            global,
            XRRenderStateBinding::Wrap,
        )
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

    /// https://immersive-web.github.io/webxr/#dom-xrrenderstate-baselayer
    fn GetBaseLayer(&self) -> Option<DomRoot<XRLayer>> {
        self.layer.get()
    }
}
