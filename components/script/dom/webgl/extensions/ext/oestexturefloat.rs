/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::JSContext;
use script_bindings::reflector::{Reflector, reflect_dom_object_with_cx};
use servo_canvas_traits::webgl::{TexFormat, WebGLVersion};

use super::{WebGLExtension, WebGLExtensionSpec, WebGLExtensions, constants as webgl};
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::DomRoot;
use crate::dom::webgl::webglrenderingcontext::WebGLRenderingContext;

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
    fn new(cx: &mut JSContext, ctx: &WebGLRenderingContext) -> DomRoot<OESTextureFloat> {
        reflect_dom_object_with_cx(Box::new(Self::new_inherited()), &*ctx.global(), cx)
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
