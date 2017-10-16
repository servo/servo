/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::OESTextureHalfFloatBinding::{self, OESTextureHalfFloatConstants};
use dom::bindings::reflector::{DomObject, Reflector, reflect_dom_object};
use dom::bindings::root::DomRoot;
use dom::webglrenderingcontext::WebGLRenderingContext;
use dom_struct::dom_struct;
use super::{constants as webgl, ext_constants as gl, WebGLExtension, WebGLExtensions};

#[dom_struct]
pub struct OESTextureHalfFloat {
    reflector_: Reflector,
}

impl OESTextureHalfFloat {
    fn new_inherited() -> OESTextureHalfFloat {
        Self {
            reflector_: Reflector::new(),
        }
    }
}

impl WebGLExtension for OESTextureHalfFloat {
    type Extension = OESTextureHalfFloat;
    fn new(ctx: &WebGLRenderingContext) -> DomRoot<OESTextureHalfFloat> {
        reflect_dom_object(Box::new(OESTextureHalfFloat::new_inherited()),
                           &*ctx.global(),
                           OESTextureHalfFloatBinding::Wrap)
    }

    fn is_supported(ext: &WebGLExtensions) -> bool {
        ext.supports_any_gl_extension(&["GL_OES_texture_half_float",
                                        "GL_ARB_half_float_pixel",
                                        "GL_NV_half_float",
                                        "GL_EXT_color_buffer_half_float"])
    }

    fn enable(ext: &WebGLExtensions) {
        // Enable FLOAT text data type
        let hf = OESTextureHalfFloatConstants::HALF_FLOAT_OES;
        ext.enable_tex_type(hf);
        let needs_replace = !ext.supports_gl_extension("GL_OES_texture_float");
        if needs_replace {
            // Special internal formats must be used to avoid clamped float values
            ext.add_effective_tex_internal_format(webgl::RGBA, hf, gl::RGBA16F);
            ext.add_effective_tex_internal_format(webgl::RGB, hf, gl::RGB16F);
            ext.add_effective_tex_internal_format(webgl::LUMINANCE, hf, gl::LUMINANCE16F_ARB);
            ext.add_effective_tex_internal_format(webgl::ALPHA, hf, gl::ALPHA16F_ARB);
            ext.add_effective_tex_internal_format(webgl::LUMINANCE_ALPHA, hf, gl::LUMINANCE_ALPHA16F_ARB);
        }
    }

    fn name() -> &'static str {
        "OES_texture_half_float"
    }
}
