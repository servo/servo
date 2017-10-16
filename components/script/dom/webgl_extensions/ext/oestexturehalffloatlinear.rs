/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::OESTextureHalfFloatBinding::OESTextureHalfFloatConstants;
use dom::bindings::codegen::Bindings::OESTextureHalfFloatLinearBinding;
use dom::bindings::reflector::{DomObject, Reflector, reflect_dom_object};
use dom::bindings::root::DomRoot;
use dom::webglrenderingcontext::WebGLRenderingContext;
use dom_struct::dom_struct;
use super::{WebGLExtension, WebGLExtensions};

#[dom_struct]
pub struct OESTextureHalfFloatLinear {
    reflector_: Reflector,
}

impl OESTextureHalfFloatLinear {
    fn new_inherited() -> OESTextureHalfFloatLinear {
        Self {
            reflector_: Reflector::new(),
        }
    }
}

impl WebGLExtension for OESTextureHalfFloatLinear {
    type Extension = OESTextureHalfFloatLinear;
    fn new(ctx: &WebGLRenderingContext) -> DomRoot<OESTextureHalfFloatLinear> {
        reflect_dom_object(Box::new(OESTextureHalfFloatLinear::new_inherited()),
                           &*ctx.global(),
                           OESTextureHalfFloatLinearBinding::Wrap)
    }

    fn is_supported(ext: &WebGLExtensions) -> bool {
        ext.supports_any_gl_extension(&["GL_OES_texture_float_linear",
                                        "GL_ARB_half_float_pixel",
                                        "GL_NV_half_float"])
    }

    fn enable(ext: &WebGLExtensions) {
        ext.enable_filterable_tex_type(OESTextureHalfFloatConstants::HALF_FLOAT_OES);
    }

    fn name() -> &'static str {
        "OES_texture_half_float_linear"
    }
}
