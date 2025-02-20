/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use canvas_traits::webgl::WebGLVersion;
use dom_struct::dom_struct;

use super::{WebGLExtension, WebGLExtensionSpec, WebGLExtensions};
use crate::dom::bindings::codegen::Bindings::EXTTextureFilterAnisotropicBinding::EXTTextureFilterAnisotropicConstants;
use crate::dom::bindings::reflector::{reflect_dom_object, DomGlobal, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::webglrenderingcontext::WebGLRenderingContext;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct EXTTextureFilterAnisotropic {
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

    fn new(ctx: &WebGLRenderingContext, can_gc: CanGc) -> DomRoot<Self> {
        reflect_dom_object(Box::new(Self::new_inherited()), &*ctx.global(), can_gc)
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
