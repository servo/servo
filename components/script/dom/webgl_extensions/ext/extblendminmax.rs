/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use canvas_traits::webgl::WebGLVersion;
use dom::bindings::codegen::Bindings::EXTBlendMinmaxBinding;
use dom::bindings::reflector::{DomObject, Reflector, reflect_dom_object};
use dom::bindings::root::DomRoot;
use dom::webglrenderingcontext::WebGLRenderingContext;
use dom_struct::dom_struct;
use super::{WebGLExtension, WebGLExtensions, WebGLExtensionSpec};

#[dom_struct]
pub struct EXTBlendMinmax {
    reflector_: Reflector,
}

impl EXTBlendMinmax {
    fn new_inherited() -> Self {
        Self {
            reflector_: Reflector::new(),
        }
    }
}

impl WebGLExtension for EXTBlendMinmax {
    type Extension = Self;

    fn new(ctx: &WebGLRenderingContext) -> DomRoot<Self> {
        reflect_dom_object(
            Box::new(Self::new_inherited()),
            &*ctx.global(),
            EXTBlendMinmaxBinding::Wrap,
        )
    }

    fn spec() -> WebGLExtensionSpec {
        WebGLExtensionSpec::Specific(WebGLVersion::WebGL1)
    }

    fn is_supported(ext: &WebGLExtensions) -> bool {
        ext.supports_gl_extension("GL_EXT_blend_minmax")
    }

    fn enable(ext: &WebGLExtensions) {
        ext.enable_blend_minmax();
    }

    fn name() -> &'static str {
        "EXT_blend_minmax"
    }
}
