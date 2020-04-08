/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::XRLayerBinding::XRLayerBinding::XRLayerMethods;
use crate::dom::bindings::reflector::Reflector;
use crate::dom::bindings::root::Dom;
use crate::dom::webglrenderingcontext::WebGLRenderingContext;
use crate::dom::xrsession::XRSession;
use dom_struct::dom_struct;
use euclid::Size2D;
use webxr_api::Viewport;

#[dom_struct]
pub struct XRLayer {
    reflector: Reflector,
    session: Dom<XRSession>,
    context: Dom<WebGLRenderingContext>,
    size: Size2D<u32, Viewport>,
}

impl XRLayerMethods for XRLayer {
    /// https://immersive-web.github.io/layers/#dom-xrlayer-pixelwidth
    fn PixelWidth(&self) -> u32 {
        self.size.width
    }

    /// https://immersive-web.github.io/layers/#dom-xrlayer-pixelheight
    fn PixelHeight(&self) -> u32 {
        self.size.height
    }

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
        size: Size2D<u32, Viewport>,
    ) -> XRLayer {
        XRLayer {
            reflector: Reflector::new(),
            session: Dom::from_ref(session),
            context: Dom::from_ref(context),
            size: size,
        }
    }
}
