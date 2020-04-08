/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::XRRenderStateBinding::XRRenderStateMethods;
use crate::dom::bindings::codegen::UnionTypes::XRWebGLLayerOrXRLayer as RootedXRWebGLLayerOrXRLayer;
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot, MutNullableDom};
use crate::dom::globalscope::GlobalScope;
use crate::dom::xrlayer::XRLayer;
use crate::dom::xrwebgllayer::XRWebGLLayer;
use dom_struct::dom_struct;
use std::cell::Cell;
use webxr_api::SwapChainId;

#[dom_struct]
pub struct XRRenderState {
    reflector_: Reflector,
    depth_near: Cell<f64>,
    depth_far: Cell<f64>,
    inline_vertical_fov: Cell<Option<f64>>,
    layer: MutNullableDom<XRWebGLLayer>,
    layers: DomRefCell<Vec<XRWebGLLayerOrXRLayer>>,
}

#[unrooted_must_root_lint::must_root]
#[derive(Clone, JSTraceable, MallocSizeOf)]
pub enum XRWebGLLayerOrXRLayer {
    XRWebGLLayer(Dom<XRWebGLLayer>),
    XRLayer(Dom<XRLayer>),
}

impl XRWebGLLayerOrXRLayer {
    #[allow(unrooted_must_root)]
    fn from_ref(layer: &RootedXRWebGLLayerOrXRLayer) -> XRWebGLLayerOrXRLayer {
        match layer {
            RootedXRWebGLLayerOrXRLayer::XRWebGLLayer(ref layer) => {
                XRWebGLLayerOrXRLayer::XRWebGLLayer(Dom::from_ref(layer))
            },
            RootedXRWebGLLayerOrXRLayer::XRLayer(ref layer) => {
                XRWebGLLayerOrXRLayer::XRLayer(Dom::from_ref(layer))
            },
        }
    }

    pub fn swap_chain_id(&self) -> Option<SwapChainId> {
        match self {
            XRWebGLLayerOrXRLayer::XRWebGLLayer(layer) => Some(layer.swap_chain_id()),
            XRWebGLLayerOrXRLayer::XRLayer(_) => None,
        }
    }

    pub fn swap_buffers(&self) {
        match self {
            XRWebGLLayerOrXRLayer::XRWebGLLayer(layer) => layer.swap_buffers(),
            XRWebGLLayerOrXRLayer::XRLayer(_) => (),
        }
    }
}

impl XRRenderState {
    pub fn new_inherited(
        depth_near: f64,
        depth_far: f64,
        inline_vertical_fov: Option<f64>,
        layer: Option<&XRWebGLLayer>,
        layers: &[XRWebGLLayerOrXRLayer],
    ) -> XRRenderState {
        debug_assert!(layer.is_none() || layers.is_empty());
        XRRenderState {
            reflector_: Reflector::new(),
            depth_near: Cell::new(depth_near),
            depth_far: Cell::new(depth_far),
            inline_vertical_fov: Cell::new(inline_vertical_fov),
            layer: MutNullableDom::new(layer),
            layers: DomRefCell::new(layers.iter().cloned().collect()),
        }
    }

    pub fn new(
        global: &GlobalScope,
        depth_near: f64,
        depth_far: f64,
        inline_vertical_fov: Option<f64>,
        layer: Option<&XRWebGLLayer>,
        layers: &[XRWebGLLayerOrXRLayer],
    ) -> DomRoot<XRRenderState> {
        reflect_dom_object(
            Box::new(XRRenderState::new_inherited(
                depth_near,
                depth_far,
                inline_vertical_fov,
                layer,
                layers,
            )),
            global,
        )
    }

    pub fn clone_object(&self) -> DomRoot<Self> {
        let layers = self.layers.borrow();
        XRRenderState::new(
            &self.global(),
            self.depth_near.get(),
            self.depth_far.get(),
            self.inline_vertical_fov.get(),
            self.layer.get().as_ref().map(|x| &**x),
            &layers,
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
    pub fn set_layers(&self, layers: &[RootedXRWebGLLayerOrXRLayer]) {
        *self.layers.borrow_mut() = layers.iter().map(XRWebGLLayerOrXRLayer::from_ref).collect();
    }
    pub fn with_layers<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&[XRWebGLLayerOrXRLayer]) -> R,
    {
        let layers = self.layers.borrow();
        f(&*layers)
    }
    pub fn has_layer(&self) -> bool {
        self.layer.get().is_some() || !self.layers.borrow().is_empty()
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
