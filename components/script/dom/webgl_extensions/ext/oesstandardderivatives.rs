/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use canvas_traits::webgl::{WebGLVersion, is_gles};
use dom::bindings::codegen::Bindings::OESStandardDerivativesBinding;
use dom::bindings::codegen::Bindings::OESStandardDerivativesBinding::OESStandardDerivativesConstants;
use dom::bindings::reflector::{DomObject, Reflector, reflect_dom_object};
use dom::bindings::root::DomRoot;
use dom::webglrenderingcontext::WebGLRenderingContext;
use dom_struct::dom_struct;
use super::{WebGLExtension, WebGLExtensions, WebGLExtensionSpec};

#[dom_struct]
pub struct OESStandardDerivatives {
    reflector_: Reflector,
}

impl OESStandardDerivatives {
    fn new_inherited() -> OESStandardDerivatives {
        Self {
            reflector_: Reflector::new(),
        }
    }
}

impl WebGLExtension for OESStandardDerivatives {
    type Extension = OESStandardDerivatives;
    fn new(ctx: &WebGLRenderingContext) -> DomRoot<OESStandardDerivatives> {
        reflect_dom_object(
            Box::new(OESStandardDerivatives::new_inherited()),
            &*ctx.global(),
            OESStandardDerivativesBinding::Wrap,
        )
    }

    fn spec() -> WebGLExtensionSpec {
        WebGLExtensionSpec::Specific(WebGLVersion::WebGL1)
    }

    fn is_supported(ext: &WebGLExtensions) -> bool {
        // The standard derivatives are always available in desktop OpenGL.
        !is_gles() || ext.supports_any_gl_extension(&["GL_OES_standard_derivatives"])
    }

    fn enable(ext: &WebGLExtensions) {
        ext.enable_hint_target(
            OESStandardDerivativesConstants::FRAGMENT_SHADER_DERIVATIVE_HINT_OES,
        );
        ext.enable_get_parameter_name(
            OESStandardDerivativesConstants::FRAGMENT_SHADER_DERIVATIVE_HINT_OES,
        );
    }

    fn name() -> &'static str {
        "OES_standard_derivatives"
    }
}
