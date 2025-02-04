/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use canvas_traits::webgl::{TexFormat, WebGLVersion};
use dom_struct::dom_struct;

use super::{constants as webgl, WebGLExtension, WebGLExtensionSpec, WebGLExtensions};
use crate::dom::bindings::reflector::{reflect_dom_object, DomGlobal, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::webglrenderingcontext::WebGLRenderingContext;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct OESTextureFloat {
    reflector_: Reflector,
}

impl OESTextureFloat {
    fn new_inherited() -> OESTextureFloat {
        Self {
            reflector_: Reflector::new(),
        }
    }
}

impl WebGLExtension for OESTextureFloat {
    type Extension = OESTextureFloat;
    fn new(ctx: &WebGLRenderingContext) -> DomRoot<OESTextureFloat> {
        reflect_dom_object(
            Box::new(OESTextureFloat::new_inherited()),
            &*ctx.global(),
            CanGc::note(),
        )
    }

    fn spec() -> WebGLExtensionSpec {
        WebGLExtensionSpec::Specific(WebGLVersion::WebGL1)
    }

    fn is_supported(ext: &WebGLExtensions) -> bool {
        ext.supports_any_gl_extension(&[
            "GL_OES_texture_float",
            "GL_ARB_texture_float",
            "GL_EXT_color_buffer_float",
        ])
    }

    fn enable(ext: &WebGLExtensions) {
        ext.enable_tex_type(webgl::FLOAT);
        ext.add_effective_tex_internal_format(TexFormat::RGBA, webgl::FLOAT, TexFormat::RGBA32f);
        ext.add_effective_tex_internal_format(TexFormat::RGB, webgl::FLOAT, TexFormat::RGB32f);
        ext.add_effective_tex_internal_format(
            TexFormat::Luminance,
            webgl::FLOAT,
            TexFormat::Luminance32f,
        );
        ext.add_effective_tex_internal_format(TexFormat::Alpha, webgl::FLOAT, TexFormat::Alpha32f);
        ext.add_effective_tex_internal_format(
            TexFormat::LuminanceAlpha,
            webgl::FLOAT,
            TexFormat::LuminanceAlpha32f,
        );
    }

    fn name() -> &'static str {
        "OES_texture_float"
    }
}
