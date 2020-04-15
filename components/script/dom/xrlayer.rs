/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::XRLayerBinding::XRLayerBinding::XRLayerMethods;
use crate::dom::bindings::reflector::Reflector;
use crate::dom::bindings::root::Dom;
use crate::dom::webglrenderingcontext::WebGLRenderingContext;
use crate::dom::xrframe::XRFrame;
use crate::dom::xrsession::XRSession;
use canvas_traits::webgl::WebGLContextId;
use dom_struct::dom_struct;
use webxr_api::LayerId;

#[dom_struct]
pub struct XRLayer {
    reflector: Reflector,
    session: Dom<XRSession>,
    context: Dom<WebGLRenderingContext>,
    #[ignore_malloc_size_of = "Layers don't heap-allocate"]
    layer_id: LayerId,
}

impl XRLayerMethods for XRLayer {
    /// https://immersive-web.github.io/layers/#dom-xrlayer-destroy
    fn Destroy(&self) {
        // TODO: Implement this
    }
}

impl XRLayer {
    #[allow(dead_code)]
    pub fn new_inherited(
        session: &XRSession,
        context: &WebGLRenderingContext,
        layer_id: LayerId,
    ) -> XRLayer {
        XRLayer {
            reflector: Reflector::new(),
            session: Dom::from_ref(session),
            context: Dom::from_ref(context),
            layer_id,
        }
    }

    pub(crate) fn layer_id(&self) -> LayerId {
        self.layer_id
    }

    pub(crate) fn context_id(&self) -> WebGLContextId {
        self.context.context_id()
    }

    pub fn begin_frame(&self, _frame: &XRFrame) -> Option<()> {
        // TODO: Implement this
        None
    }

    pub fn end_frame(&self, _frame: &XRFrame) -> Option<()> {
        // TODO: Implement this
        None
    }
}
