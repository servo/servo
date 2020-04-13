/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::XRWebGLSubImageBinding::XRWebGLSubImageBinding::XRWebGLSubImageMethods;
use crate::dom::bindings::root::Dom;
use crate::dom::bindings::root::DomRoot;
use crate::dom::webgltexture::WebGLTexture;
use crate::dom::xrsubimage::XRSubImage;
use dom_struct::dom_struct;

#[dom_struct]
pub struct XRWebGLSubImage {
    xr_sub_image: XRSubImage,
    color_texture: Dom<WebGLTexture>,
    depth_stencil_texture: Option<Dom<WebGLTexture>>,
    image_index: Option<u32>,
}

impl XRWebGLSubImageMethods for XRWebGLSubImage {
    /// https://immersive-web.github.io/layers/#dom-xrwebglsubimage-colortexture
    fn ColorTexture(&self) -> DomRoot<WebGLTexture> {
        DomRoot::from_ref(&self.color_texture)
    }

    /// https://immersive-web.github.io/layers/#dom-xrwebglsubimage-depthstenciltexture
    fn GetDepthStencilTexture(&self) -> Option<DomRoot<WebGLTexture>> {
        self.depth_stencil_texture.as_deref().map(DomRoot::from_ref)
    }

    /// https://immersive-web.github.io/layers/#dom-xrwebglsubimage-imageindex
    fn GetImageIndex(&self) -> Option<u32> {
        self.image_index
    }
}
