/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use canvas_traits::webgl::{WebGLSLVersion, WebGLVersion};
use dom_struct::dom_struct;

use super::{WebGLExtension, WebGLExtensionSpec, WebGLExtensions};
use crate::dom::bindings::reflector::{reflect_dom_object, DomGlobal, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::webglrenderingcontext::WebGLRenderingContext;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct EXTFragDepth {
    reflector_: Reflector,
}

impl EXTFragDepth {
    fn new_inherited() -> EXTFragDepth {
        Self {
            reflector_: Reflector::new(),
        }
    }
}

impl WebGLExtension for EXTFragDepth {
    type Extension = Self;

    fn new(ctx: &WebGLRenderingContext) -> DomRoot<Self> {
        reflect_dom_object(
            Box::new(Self::new_inherited()),
            &*ctx.global(),
            CanGc::note(),
        )
    }

    fn spec() -> WebGLExtensionSpec {
        WebGLExtensionSpec::Specific(WebGLVersion::WebGL1)
    }

    fn is_supported(ext: &WebGLExtensions) -> bool {
        let min_glsl_version = if ext.is_gles() {
            WebGLSLVersion { major: 3, minor: 0 }
        } else {
            WebGLSLVersion {
                major: 1,
                minor: 10,
            }
        };
        match (
            ext.is_gles(),
            ext.is_min_glsl_version_satisfied(min_glsl_version),
        ) {
            // ANGLE's shader translator can't translate ESSL1 exts to ESSL3. (bug
            // 1524804)
            (true, true) => false,
            (true, false) => ext.supports_gl_extension("GL_EXT_frag_depth"),
            (false, is_min_glsl_version_satisfied) => is_min_glsl_version_satisfied,
        }
    }

    fn enable(_ext: &WebGLExtensions) {}

    fn name() -> &'static str {
        "EXT_frag_depth"
    }
}
