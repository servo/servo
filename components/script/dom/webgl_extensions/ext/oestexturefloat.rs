/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::OESTextureFloatBinding;
use dom::bindings::js::Root;
use dom::bindings::reflector::{DomObject, Reflector, reflect_dom_object};
use dom::webglrenderingcontext::WebGLRenderingContext;
use dom_struct::dom_struct;
use super::{constants as webgl, ext_constants as gl, WebGLExtension, WebGLExtensionManager};

#[dom_struct]
pub struct OESTextureFloat {
    reflector_: Reflector,
}

impl OESTextureFloat {
    fn new_inherited() -> OESTextureFloat {
        Self {
            reflector_: Reflector::new(),
        }
    }
}

impl WebGLExtension for OESTextureFloat {
    type Extension = OESTextureFloat;
    fn new(ctx: &WebGLRenderingContext) -> Root<OESTextureFloat> {
        reflect_dom_object(box OESTextureFloat::new_inherited(),
                           &*ctx.global(),
                           OESTextureFloatBinding::Wrap)
    }

    fn is_supported(manager: &WebGLExtensionManager) -> bool {
        manager.supports_any_gl_extension(&["GL_OES_texture_float",
                                            "GL_ARB_texture_float"])
    }

    fn enable(manager: &WebGLExtensionManager) {
        // Enable FLOAT text data type
        manager.enable_tex_type(webgl::FLOAT);
        let needs_replace = !manager.supports_gl_extension("GL_OES_texture_float");
        if needs_replace {
            // Special internal formast must be used to avoid clamped float values
            manager.add_effective_tex_internal_format(webgl::RGBA, webgl::FLOAT, gl::RGBA32F);
            manager.add_effective_tex_internal_format(webgl::RGB, webgl::FLOAT, gl::RGB32F);
            manager.add_effective_tex_internal_format(webgl::LUMINANCE, webgl::FLOAT, gl::LUMINANCE32F_ARB);
            manager.add_effective_tex_internal_format(webgl::ALPHA, webgl::FLOAT, gl::ALPHA32F_ARB);
            manager.add_effective_tex_internal_format(webgl::LUMINANCE_ALPHA, webgl::FLOAT,
                                                      gl::LUMINANCE_ALPHA32F_ARB);
        }
    }

    fn name() -> &'static str {
        "OES_texture_float"
    }
}
