/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::JSContext;
use script_bindings::reflector::{Reflector, reflect_dom_object_with_cx};
use servo_canvas_traits::webgl::{TexFormat, WebGLVersion};

use super::{WebGLExtension, WebGLExtensionSpec, WebGLExtensions};
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::DomRoot;
use crate::dom::webgl::webglrenderingcontext::WebGLRenderingContext;
use crate::dom::webgl::webgltexture::{TexCompression, TexCompressionValidation};

#[dom_struct]
pub(crate) struct WEBGLCompressedTextureETC1 {
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
    fn new(cx: &mut JSContext, ctx: &WebGLRenderingContext) -> DomRoot<WEBGLCompressedTextureETC1> {
        reflect_dom_object_with_cx(Box::new(Self::new_inherited()), &*ctx.global(), cx)
    }

    fn spec() -> WebGLExtensionSpec {
        WebGLExtensionSpec::Specific(WebGLVersion::WebGL1)
    }

    fn is_supported(ext: &WebGLExtensions) -> bool {
        ext.supports_gl_extension("GL_OES_compressed_ETC1_RGB8_texture")
    }

    fn enable(ext: &WebGLExtensions) {
        ext.add_tex_compression_formats(&[TexCompression {
            format: TexFormat::CompressedRgbEtc1,
            bytes_per_block: 8,
            block_width: 4,
            block_height: 4,
            validation: TexCompressionValidation::None,
        }]);
    }

    fn name() -> &'static str {
        "WEBGL_compressed_texture_etc1"
    }
}
