/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::{WebGLExtension, WebGLExtensionSpec, WebGLExtensions};
use canvas_traits::webgl::{is_gles, WebGLVersion};
use crate::dom::bindings::codegen::Bindings::EXTShaderTextureLodBinding;
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::webglrenderingcontext::WebGLRenderingContext;
use dom_struct::dom_struct;

#[dom_struct]
pub struct EXTShaderTextureLod {
    reflector_: Reflector,
}

impl EXTShaderTextureLod {
    fn new_inherited() -> Self {
        Self {
            reflector_: Reflector::new(),
        }
    }
}

impl WebGLExtension for EXTShaderTextureLod {
    type Extension = Self;

    fn new(ctx: &WebGLRenderingContext) -> DomRoot<Self> {
        reflect_dom_object(
            Box::new(Self::new_inherited()),
            &*ctx.global(),
            EXTShaderTextureLodBinding::Wrap,
        )
    }

    fn spec() -> WebGLExtensionSpec {
        WebGLExtensionSpec::Specific(WebGLVersion::WebGL1)
    }

    fn is_supported(ext: &WebGLExtensions) -> bool {
        // This extension is always available on desktop GL.
        !is_gles() || ext.supports_gl_extension("GL_EXT_shader_texture_lod")
    }

    fn enable(_ext: &WebGLExtensions) {}

    fn name() -> &'static str {
        "EXT_shader_texture_lod"
    }
}
