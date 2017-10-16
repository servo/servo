/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::OESTextureFloatLinearBinding;
use dom::bindings::reflector::{DomObject, Reflector, reflect_dom_object};
use dom::bindings::root::DomRoot;
use dom::webglrenderingcontext::WebGLRenderingContext;
use dom_struct::dom_struct;
use super::{constants as webgl, WebGLExtension, WebGLExtensions};

#[dom_struct]
pub struct OESTextureFloatLinear {
    reflector_: Reflector,
}

impl OESTextureFloatLinear {
    fn new_inherited() -> OESTextureFloatLinear {
        Self {
            reflector_: Reflector::new(),
        }
    }
}

impl WebGLExtension for OESTextureFloatLinear {
    type Extension = OESTextureFloatLinear;
    fn new(ctx: &WebGLRenderingContext) -> DomRoot<OESTextureFloatLinear> {
        reflect_dom_object(Box::new(OESTextureFloatLinear::new_inherited()),
                           &*ctx.global(),
                           OESTextureFloatLinearBinding::Wrap)
    }

    fn is_supported(ext: &WebGLExtensions) -> bool {
        ext.supports_any_gl_extension(&["GL_OES_texture_float_linear",
                                        "GL_ARB_texture_float"])
    }

    fn enable(ext: &WebGLExtensions) {
        ext.enable_filterable_tex_type(webgl::FLOAT);
    }

    fn name() -> &'static str {
        "OES_texture_float_linear"
    }
}
