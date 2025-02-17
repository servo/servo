/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use canvas_traits::webgl::{TexFormat, WebGLVersion};
use dom_struct::dom_struct;

use super::{WebGLExtension, WebGLExtensionSpec, WebGLExtensions};
use crate::dom::bindings::codegen::Bindings::OESTextureHalfFloatBinding::OESTextureHalfFloatConstants;
use crate::dom::bindings::reflector::{reflect_dom_object, DomGlobal, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::webglrenderingcontext::WebGLRenderingContext;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct OESTextureHalfFloat {
    reflector_: Reflector,
}

impl OESTextureHalfFloat {
    fn new_inherited() -> OESTextureHalfFloat {
        Self {
            reflector_: Reflector::new(),
        }
    }
}

impl WebGLExtension for OESTextureHalfFloat {
    type Extension = OESTextureHalfFloat;
    fn new(ctx: &WebGLRenderingContext) -> DomRoot<OESTextureHalfFloat> {
        reflect_dom_object(
            Box::new(OESTextureHalfFloat::new_inherited()),
            &*ctx.global(),
            CanGc::note(),
        )
    }

    fn spec() -> WebGLExtensionSpec {
        WebGLExtensionSpec::Specific(WebGLVersion::WebGL1)
    }

    fn is_supported(ext: &WebGLExtensions) -> bool {
        ext.supports_any_gl_extension(&[
            "GL_OES_texture_half_float",
            "GL_ARB_half_float_pixel",
            "GL_NV_half_float",
            "GL_EXT_color_buffer_half_float",
        ])
    }

    fn enable(ext: &WebGLExtensions) {
        let hf = OESTextureHalfFloatConstants::HALF_FLOAT_OES;
        ext.enable_tex_type(hf);
        ext.add_effective_tex_internal_format(TexFormat::RGBA, hf, TexFormat::RGBA16f);
        ext.add_effective_tex_internal_format(TexFormat::RGB, hf, TexFormat::RGB16f);
        ext.add_effective_tex_internal_format(TexFormat::Luminance, hf, TexFormat::Luminance16f);
        ext.add_effective_tex_internal_format(TexFormat::Alpha, hf, TexFormat::Alpha16f);
        ext.add_effective_tex_internal_format(
            TexFormat::LuminanceAlpha,
            hf,
            TexFormat::LuminanceAlpha16f,
        );
    }

    fn name() -> &'static str {
        "OES_texture_half_float"
    }
}
