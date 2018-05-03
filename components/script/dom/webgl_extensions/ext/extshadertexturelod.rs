/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use canvas_traits::webgl::WebGLVersion;
use dom::bindings::codegen::Bindings::EXTShaderTextureLodBinding;
use dom::bindings::reflector::{DomObject, Reflector, reflect_dom_object};
use dom::bindings::root::DomRoot;
use dom::webglrenderingcontext::{WebGLRenderingContext, is_gles};
use dom_struct::dom_struct;
use super::{WebGLExtension, WebGLExtensions, WebGLExtensionSpec};
use typeholder::TypeHolderTrait;

#[dom_struct]
pub struct EXTShaderTextureLod<TH: TypeHolderTrait> {
    reflector_: Reflector<TH>,
}

impl<TH: TypeHolderTrait> EXTShaderTextureLod<TH> {
    fn new_inherited() -> Self {
        Self { reflector_: Reflector::new() }
    }
}

impl<TH: TypeHolderTrait> WebGLExtension<TH> for EXTShaderTextureLod<TH> {
    type Extension = Self;

    fn new(ctx: &WebGLRenderingContext<TH>) -> DomRoot<Self> {
        reflect_dom_object(
            Box::new(Self::new_inherited()),
            &*ctx.global(),
            EXTShaderTextureLodBinding::Wrap,
        )
    }

    fn spec() -> WebGLExtensionSpec {
        WebGLExtensionSpec::Specific(WebGLVersion::WebGL1)
    }

    fn is_supported(ext: &WebGLExtensions<TH>) -> bool {
        // This extension is always available on desktop GL.
        !is_gles() || ext.supports_gl_extension("GL_EXT_shader_texture_lod")
    }

    fn enable(_ext: &WebGLExtensions<TH>) {}

    fn name() -> &'static str {
        "EXT_shader_texture_lod"
    }
}
