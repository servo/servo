/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::OESTextureFloatLinearBinding;
use dom::bindings::reflector::{DomObject, Reflector, reflect_dom_object};
use dom::bindings::root::DomRoot;
use dom::webglrenderingcontext::WebGLRenderingContext;
use dom_struct::dom_struct;
use super::{constants as webgl, WebGLExtension, WebGLExtensions, WebGLExtensionSpec};
use typeholder::TypeHolderTrait;
use std::marker::PhantomData;

#[dom_struct]
pub struct OESTextureFloatLinear<TH: TypeHolderTrait> {
    reflector_: Reflector<TH>,
    _p: PhantomData<TH>,
}

impl<TH: TypeHolderTrait> OESTextureFloatLinear<TH> {
    fn new_inherited() -> OESTextureFloatLinear<TH> {
        Self {
            reflector_: Reflector::new(),
            _p: Default::default(),
        }
    }
}

impl<TH: TypeHolderTrait> WebGLExtension<TH> for OESTextureFloatLinear<TH> {
    type Extension = OESTextureFloatLinear<TH>;
    fn new(ctx: &WebGLRenderingContext<TH>) -> DomRoot<OESTextureFloatLinear<TH>> {
        reflect_dom_object(Box::new(OESTextureFloatLinear::new_inherited()),
                           &*ctx.global(),
                           OESTextureFloatLinearBinding::Wrap)
    }

    fn spec() -> WebGLExtensionSpec {
        WebGLExtensionSpec::All
    }

    fn is_supported(ext: &WebGLExtensions<TH>) -> bool {
        ext.supports_any_gl_extension(&["GL_OES_texture_float_linear",
                                        "GL_ARB_texture_float"])
    }

    fn enable(ext: &WebGLExtensions<TH>) {
        ext.enable_filterable_tex_type(webgl::FLOAT);
    }

    fn name() -> &'static str {
        "OES_texture_float_linear"
    }
}
