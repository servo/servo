/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use super::{WebGLExtension, WebGLExtensionSpec, WebGLExtensions};
use crate::dom::bindings::codegen::Bindings::{
    WEBGLCompressedTextureETC1Binding,
    WEBGLCompressedTextureETC1Binding::WEBGLCompressedTextureETC1Constants,
};
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::webglrenderingcontext::WebGLRenderingContext;
use crate::dom::webgltexture::{TexCompression, TexCompressionValidation};
use canvas_traits::webgl::WebGLVersion;
use dom_struct::dom_struct;

#[dom_struct]
pub struct WEBGLCompressedTextureETC1 {
    reflector_: Reflector,
}

impl WEBGLCompressedTextureETC1 {
    fn new_inherited() -> WEBGLCompressedTextureETC1 {
        Self {
            reflector_: Reflector::new(),
        }
    }
}

impl WebGLExtension for WEBGLCompressedTextureETC1 {
    type Extension = WEBGLCompressedTextureETC1;
    fn new(ctx: &WebGLRenderingContext) -> DomRoot<WEBGLCompressedTextureETC1> {
        reflect_dom_object(
            Box::new(WEBGLCompressedTextureETC1::new_inherited()),
            &*ctx.global(),
            WEBGLCompressedTextureETC1Binding::Wrap,
        )
    }

    fn spec() -> WebGLExtensionSpec {
        WebGLExtensionSpec::Specific(WebGLVersion::WebGL1)
    }

    fn is_supported(ext: &WebGLExtensions) -> bool {
        ext.supports_gl_extension("GL_OES_compressed_ETC1_RGB8_texture")
    }

    fn enable(ext: &WebGLExtensions) {
        ext.add_tex_compression_formats(&[(
            WEBGLCompressedTextureETC1Constants::COMPRESSED_RGB_ETC1_WEBGL,
            TexCompression {
                bytes_per_block: 8,
                block_width: 4,
                block_height: 4,
                validation: TexCompressionValidation::None,
            },
        )]);
    }

    fn name() -> &'static str {
        "WEBGL_compressed_texture_etc1"
    }
}
