/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::OESTextureHalfFloatBinding::OESTextureHalfFloatConstants;
use dom::bindings::codegen::Bindings::OESTextureHalfFloatLinearBinding;
use dom::bindings::reflector::{DomObject, Reflector, reflect_dom_object};
use dom::bindings::root::DomRoot;
use dom::webglrenderingcontext::WebGLRenderingContext;
use dom_struct::dom_struct;
use super::{WebGLExtension, WebGLExtensions, WebGLExtensionSpec};
use typeholder::TypeHolderTrait;

#[dom_struct]
pub struct OESTextureHalfFloatLinear<TH: TypeHolderTrait> {
    reflector_: Reflector<TH>,
}

impl<TH: TypeHolderTrait> OESTextureHalfFloatLinear<TH> {
    fn new_inherited() -> OESTextureHalfFloatLinear<TH> {
        Self {
            reflector_: Reflector::new(),
        }
    }
}

impl<TH: TypeHolderTrait> WebGLExtension<TH> for OESTextureHalfFloatLinear<TH> {
    type Extension = OESTextureHalfFloatLinear<TH>;
    fn new(ctx: &WebGLRenderingContext<TH>) -> DomRoot<OESTextureHalfFloatLinear<TH>> {
        reflect_dom_object(Box::new(OESTextureHalfFloatLinear::new_inherited()),
                           &*ctx.global(),
                           OESTextureHalfFloatLinearBinding::Wrap)
    }

    fn spec() -> WebGLExtensionSpec {
        WebGLExtensionSpec::All
    }

    fn is_supported(ext: &WebGLExtensions<TH>) -> bool {
        ext.supports_any_gl_extension(&["GL_OES_texture_float_linear",
                                        "GL_ARB_half_float_pixel",
                                        "GL_NV_half_float"])
    }

    fn enable(ext: &WebGLExtensions<TH>) {
        ext.enable_filterable_tex_type(OESTextureHalfFloatConstants::HALF_FLOAT_OES);
    }

    fn name() -> &'static str {
        "OES_texture_half_float_linear"
    }
}
