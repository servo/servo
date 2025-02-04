/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use canvas_traits::webgl::{TexFormat, WebGLVersion};
use dom_struct::dom_struct;

use super::{WebGLExtension, WebGLExtensionSpec, WebGLExtensions};
use crate::dom::bindings::reflector::{reflect_dom_object, DomGlobal, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::webglrenderingcontext::WebGLRenderingContext;
use crate::dom::webgltexture::{TexCompression, TexCompressionValidation};
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct WEBGLCompressedTextureS3TC {
    reflector_: Reflector,
}

impl WEBGLCompressedTextureS3TC {
    fn new_inherited() -> WEBGLCompressedTextureS3TC {
        Self {
            reflector_: Reflector::new(),
        }
    }
}

impl WebGLExtension for WEBGLCompressedTextureS3TC {
    type Extension = WEBGLCompressedTextureS3TC;
    fn new(ctx: &WebGLRenderingContext) -> DomRoot<WEBGLCompressedTextureS3TC> {
        reflect_dom_object(
            Box::new(WEBGLCompressedTextureS3TC::new_inherited()),
            &*ctx.global(),
            CanGc::note(),
        )
    }

    fn spec() -> WebGLExtensionSpec {
        WebGLExtensionSpec::Specific(WebGLVersion::WebGL1)
    }

    fn is_supported(ext: &WebGLExtensions) -> bool {
        ext.supports_gl_extension("GL_EXT_texture_compression_s3tc") ||
            ext.supports_all_gl_extension(&[
                "GL_EXT_texture_compression_dxt1",
                "GL_ANGLE_texture_compression_dxt3",
                "GL_ANGLE_texture_compression_dxt5",
            ])
    }

    fn enable(ext: &WebGLExtensions) {
        ext.add_tex_compression_formats(&[
            TexCompression {
                format: TexFormat::CompressedRgbS3tcDxt1,
                bytes_per_block: 8,
                block_width: 4,
                block_height: 4,
                validation: TexCompressionValidation::S3TC,
            },
            TexCompression {
                format: TexFormat::CompressedRgbaS3tcDxt1,
                bytes_per_block: 8,
                block_width: 4,
                block_height: 4,
                validation: TexCompressionValidation::S3TC,
            },
            TexCompression {
                format: TexFormat::CompressedRgbaS3tcDxt3,
                bytes_per_block: 16,
                block_width: 4,
                block_height: 4,
                validation: TexCompressionValidation::S3TC,
            },
            TexCompression {
                format: TexFormat::CompressedRgbaS3tcDxt5,
                bytes_per_block: 16,
                block_width: 4,
                block_height: 4,
                validation: TexCompressionValidation::S3TC,
            },
        ]);
    }

    fn name() -> &'static str {
        "WEBGL_compressed_texture_s3tc"
    }
}
