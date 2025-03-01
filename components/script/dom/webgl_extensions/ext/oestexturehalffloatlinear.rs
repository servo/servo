/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;

use super::{WebGLExtension, WebGLExtensionSpec, WebGLExtensions};
use crate::dom::bindings::codegen::Bindings::OESTextureHalfFloatBinding::OESTextureHalfFloatConstants;
use crate::dom::bindings::reflector::{reflect_dom_object, DomGlobal, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::webglrenderingcontext::WebGLRenderingContext;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct OESTextureHalfFloatLinear {
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
    fn new(ctx: &WebGLRenderingContext, can_gc: CanGc) -> DomRoot<OESTextureHalfFloatLinear> {
        reflect_dom_object(
            Box::new(OESTextureHalfFloatLinear::new_inherited()),
            &*ctx.global(),
            can_gc,
        )
    }

    fn spec() -> WebGLExtensionSpec {
        WebGLExtensionSpec::All
    }

    fn is_supported(ext: &WebGLExtensions) -> bool {
        ext.supports_any_gl_extension(&[
            "GL_OES_texture_float_linear",
            "GL_ARB_half_float_pixel",
            "GL_NV_half_float",
        ])
    }

    fn enable(ext: &WebGLExtensions) {
        ext.enable_filterable_tex_type(OESTextureHalfFloatConstants::HALF_FLOAT_OES);
    }

    fn name() -> &'static str {
        "OES_texture_half_float_linear"
    }
}
