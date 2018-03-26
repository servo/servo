/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use canvas_traits::webgl::WebGLVersion;
use dom::bindings::codegen::Bindings::OESElementIndexUintBinding;
use dom::bindings::reflector::{DomObject, Reflector, reflect_dom_object};
use dom::bindings::root::DomRoot;
use dom::webglrenderingcontext::WebGLRenderingContext;
use dom_struct::dom_struct;
use super::{WebGLExtension, WebGLExtensions, WebGLExtensionSpec};

#[dom_struct]
pub struct OESElementIndexUint {
    reflector_: Reflector,
}

impl OESElementIndexUint {
    fn new_inherited() -> Self {
        Self { reflector_: Reflector::new() }
    }
}

impl WebGLExtension for OESElementIndexUint {
    type Extension = Self;

    fn new(ctx: &WebGLRenderingContext) -> DomRoot<Self> {
        reflect_dom_object(
            Box::new(OESElementIndexUint::new_inherited()),
            &*ctx.global(),
            OESElementIndexUintBinding::Wrap,
        )
    }

    fn spec() -> WebGLExtensionSpec {
        WebGLExtensionSpec::Specific(WebGLVersion::WebGL1)
    }

    fn is_supported(ext: &WebGLExtensions) -> bool {
        ext.supports_gl_extension("GL_OES_element_index_uint")
    }

    fn enable(ext: &WebGLExtensions) {
        ext.enable_element_index_uint();
    }

    fn name() -> &'static str {
        "OES_element_index_uint"
    }
}
