/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use canvas_traits::webgl::WebGLVersion;
use dom::bindings::codegen::Bindings::EXTTextureFilterAnisotropicBinding;
use dom::bindings::codegen::Bindings::EXTTextureFilterAnisotropicBinding::EXTTextureFilterAnisotropicConstants;
use dom::bindings::reflector::{DomObject, Reflector, reflect_dom_object};
use dom::bindings::root::DomRoot;
use dom::webglrenderingcontext::WebGLRenderingContext;
use dom_struct::dom_struct;
use super::{WebGLExtension, WebGLExtensions, WebGLExtensionSpec};

#[dom_struct]
pub struct EXTTextureFilterAnisotropic {
    reflector_: Reflector,
}

impl EXTTextureFilterAnisotropic {
    fn new_inherited() -> EXTTextureFilterAnisotropic {
        Self {
            reflector_: Reflector::new(),
        }
    }
}

impl WebGLExtension for EXTTextureFilterAnisotropic {
    type Extension = EXTTextureFilterAnisotropic;

    fn new(ctx: &WebGLRenderingContext) -> DomRoot<Self> {
        reflect_dom_object(
            Box::new(Self::new_inherited()),
            &*ctx.global(),
            EXTTextureFilterAnisotropicBinding::Wrap,
        )
    }

    fn spec() -> WebGLExtensionSpec {
        WebGLExtensionSpec::Specific(WebGLVersion::WebGL1)
    }

    fn is_supported(ext: &WebGLExtensions) -> bool {
        ext.supports_gl_extension("GL_EXT_texture_filter_anisotropic")
    }

    fn enable(ext: &WebGLExtensions) {
        ext.enable_get_tex_parameter_name(
            EXTTextureFilterAnisotropicConstants::TEXTURE_MAX_ANISOTROPY_EXT,
        );
        ext.enable_get_parameter_name(
            EXTTextureFilterAnisotropicConstants::MAX_TEXTURE_MAX_ANISOTROPY_EXT,
        );
    }

    fn name() -> &'static str {
        "EXT_texture_filter_anisotropic"
    }
}
