/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;

use super::{constants as webgl, WebGLExtension, WebGLExtensionSpec, WebGLExtensions};
use crate::dom::bindings::reflector::{reflect_dom_object, DomGlobal, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::webglrenderingcontext::WebGLRenderingContext;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct OESTextureFloatLinear {
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
        reflect_dom_object(
            Box::new(OESTextureFloatLinear::new_inherited()),
            &*ctx.global(),
            CanGc::note(),
        )
    }

    fn spec() -> WebGLExtensionSpec {
        WebGLExtensionSpec::All
    }

    fn is_supported(ext: &WebGLExtensions) -> bool {
        ext.supports_any_gl_extension(&["GL_OES_texture_float_linear", "GL_ARB_texture_float"])
    }

    fn enable(ext: &WebGLExtensions) {
        ext.enable_filterable_tex_type(webgl::FLOAT);
    }

    fn name() -> &'static str {
        "OES_texture_float_linear"
    }
}
