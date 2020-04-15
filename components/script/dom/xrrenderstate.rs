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
use canvas_traits::webgl::WebGLContextId;
use dom_struct::dom_struct;
use std::cell::Cell;
use webxr_api::LayerId;
use webxr_api::SubImages;

#[dom_struct]
pub struct XRRenderState {
    reflector_: Reflector,
    depth_near: Cell<f64>,
    depth_far: Cell<f64>,
    inline_vertical_fov: Cell<Option<f64>>,
    base_layer: MutNullableDom<XRWebGLLayer>,
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

    pub(crate) fn layer_id(&self) -> Option<LayerId> {
        match self {
            XRWebGLLayerOrXRLayer::XRWebGLLayer(ref layer) => layer.layer_id(),
            XRWebGLLayerOrXRLayer::XRLayer(ref layer) => Some(layer.layer_id()),
        }
    }
}

impl RootedXRWebGLLayerOrXRLayer {
    pub(crate) fn layer_id(&self) -> Option<LayerId> {
        match self {
            RootedXRWebGLLayerOrXRLayer::XRWebGLLayer(ref layer) => layer.layer_id(),
            RootedXRWebGLLayerOrXRLayer::XRLayer(ref layer) => Some(layer.layer_id()),
        }
    }

    pub(crate) fn context_id(&self) -> WebGLContextId {
        match self {
            RootedXRWebGLLayerOrXRLayer::XRWebGLLayer(ref layer) => layer.context_id(),
            RootedXRWebGLLayerOrXRLayer::XRLayer(ref layer) => layer.context_id(),
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
            base_layer: MutNullableDom::new(layer),
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
            self.base_layer.get().as_ref().map(|x| &**x),
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
    pub fn set_base_layer(&self, layer: Option<&XRWebGLLayer>) {
        self.base_layer.set(layer)
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
    pub fn has_sub_images(&self, sub_images: &[SubImages]) -> bool {
        if let Some(base_layer) = self.base_layer.get() {
            match sub_images.len() {
                // For inline sessions, there may be a base layer, but it won't have a framebuffer
                0 => base_layer.layer_id() == None,
                // For immersive sessions, the base layer will have a framebuffer,
                // so we make sure the layer id's match up
                1 => base_layer.layer_id() == Some(sub_images[0].layer_id),
                _ => false,
            }
        } else {
            // The layers API is only for immersive sessions
            let layers = self.layers.borrow();
            sub_images.len() == layers.len() &&
                sub_images
                    .iter()
                    .zip(layers.iter())
                    .all(|(sub_image, layer)| Some(sub_image.layer_id) == layer.layer_id())
        }
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
        self.base_layer.get()
    }
}
