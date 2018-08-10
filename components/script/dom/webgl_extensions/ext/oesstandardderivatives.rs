/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use canvas_traits::webgl::WebGLVersion;
use dom::bindings::codegen::Bindings::OESStandardDerivativesBinding;
use dom::bindings::codegen::Bindings::OESStandardDerivativesBinding::OESStandardDerivativesConstants;
use dom::bindings::reflector::{DomObject, Reflector, reflect_dom_object};
use dom::bindings::root::DomRoot;
use dom::webglrenderingcontext::{WebGLRenderingContext, is_gles};
use dom_struct::dom_struct;
use super::{WebGLExtension, WebGLExtensions, WebGLExtensionSpec};
use typeholder::TypeHolderTrait;
use std::marker::PhantomData;

#[dom_struct]
pub struct OESStandardDerivatives<TH: TypeHolderTrait> {
    reflector_: Reflector<TH>,
    _p: PhantomData<TH>,
}

impl<TH: TypeHolderTrait> OESStandardDerivatives<TH> {
    fn new_inherited() -> OESStandardDerivatives<TH> {
        Self {
            reflector_: Reflector::new(),
            _p: Default::default(),
        }
    }
}

impl<TH: TypeHolderTrait> WebGLExtension<TH> for OESStandardDerivatives<TH> {
    type Extension = OESStandardDerivatives<TH>;
    fn new(ctx: &WebGLRenderingContext<TH>) -> DomRoot<OESStandardDerivatives<TH>> {
        reflect_dom_object(Box::new(OESStandardDerivatives::new_inherited()),
                           &*ctx.global(),
                           OESStandardDerivativesBinding::Wrap)
    }

    fn spec() -> WebGLExtensionSpec {
        WebGLExtensionSpec::Specific(WebGLVersion::WebGL1)
    }

    fn is_supported(ext: &WebGLExtensions<TH>) -> bool {
        // The standard derivatives are always available in desktop OpenGL.
        !is_gles() || ext.supports_any_gl_extension(&["GL_OES_standard_derivatives"])
    }

    fn enable(ext: &WebGLExtensions<TH>) {
        ext.enable_hint_target(OESStandardDerivativesConstants::FRAGMENT_SHADER_DERIVATIVE_HINT_OES);
        ext.enable_get_parameter_name(OESStandardDerivativesConstants::FRAGMENT_SHADER_DERIVATIVE_HINT_OES);
    }

    fn name() -> &'static str {
        "OES_standard_derivatives"
    }
}
