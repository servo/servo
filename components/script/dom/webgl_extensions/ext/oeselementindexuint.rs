/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use super::{WebGLExtension, WebGLExtensionSpec, WebGLExtensions};
use crate::dom::bindings::codegen::Bindings::OESElementIndexUintBinding;
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::webglrenderingcontext::WebGLRenderingContext;
use canvas_traits::webgl::WebGLVersion;
use dom_struct::dom_struct;

#[dom_struct]
pub struct OESElementIndexUint {
    reflector_: Reflector,
}

impl OESElementIndexUint {
    fn new_inherited() -> Self {
        Self {
            reflector_: Reflector::new(),
        }
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
        // This extension is always available in desktop OpenGL.
        !ext.is_gles() || ext.supports_gl_extension("GL_OES_element_index_uint")
    }

    fn enable(ext: &WebGLExtensions) {
        ext.enable_element_index_uint();
    }

    fn name() -> &'static str {
        "OES_element_index_uint"
    }
}
