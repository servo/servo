/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use canvas_traits::webgl::WebGLVersion;
use dom::bindings::codegen::Bindings::ANGLEInstancedArraysBinding;
use dom::bindings::codegen::Bindings::ANGLEInstancedArraysBinding::ANGLEInstancedArraysConstants;
use dom::bindings::codegen::Bindings::ANGLEInstancedArraysBinding::ANGLEInstancedArraysMethods;
use dom::bindings::reflector::{DomObject, Reflector, reflect_dom_object};
use dom::bindings::root::{Dom, DomRoot};
use dom::webglrenderingcontext::WebGLRenderingContext;
use dom_struct::dom_struct;
use super::{WebGLExtension, WebGLExtensions, WebGLExtensionSpec};

#[dom_struct]
pub struct ANGLEInstancedArrays {
    reflector_: Reflector,
    ctx: Dom<WebGLRenderingContext>,
}

impl ANGLEInstancedArrays {
    fn new_inherited(ctx: &WebGLRenderingContext) -> Self {
        Self {
            reflector_: Reflector::new(),
            ctx: Dom::from_ref(ctx),
        }
    }
}

impl WebGLExtension for ANGLEInstancedArrays {
    type Extension = Self;

    fn new(ctx: &WebGLRenderingContext) -> DomRoot<Self> {
        reflect_dom_object(
            Box::new(ANGLEInstancedArrays::new_inherited(ctx)),
            &*ctx.global(),
            ANGLEInstancedArraysBinding::Wrap,
        )
    }

    fn spec() -> WebGLExtensionSpec {
        WebGLExtensionSpec::Specific(WebGLVersion::WebGL1)
    }

    fn is_supported(ext: &WebGLExtensions) -> bool {
        ext.supports_any_gl_extension(&[
            "GL_ANGLE_instanced_arrays",
            "GL_ARB_instanced_arrays",
            "GL_EXT_instanced_arrays",
            "GL_NV_instanced_arrays",
        ])
    }

    fn enable(ext: &WebGLExtensions) {
        ext.enable_get_vertex_attrib_name(
            ANGLEInstancedArraysConstants::VERTEX_ATTRIB_ARRAY_DIVISOR_ANGLE,
        );
    }

    fn name() -> &'static str {
        "ANGLE_instanced_arrays"
    }
}

impl ANGLEInstancedArraysMethods for ANGLEInstancedArrays {
    // https://www.khronos.org/registry/webgl/extensions/ANGLE_instanced_arrays/
    fn DrawArraysInstancedANGLE(&self, mode: u32, first: i32, count: i32, primcount: i32) {
        handle_potential_webgl_error!(
            self.ctx,
            self.ctx
                .draw_arrays_instanced(mode, first, count, primcount)
        )
    }

    // https://www.khronos.org/registry/webgl/extensions/ANGLE_instanced_arrays/
    fn DrawElementsInstancedANGLE(
        &self,
        mode: u32,
        count: i32,
        type_: u32,
        offset: i64,
        primcount: i32,
    ) {
        handle_potential_webgl_error!(
            self.ctx,
            self.ctx
                .draw_elements_instanced(mode, count, type_, offset, primcount)
        )
    }

    fn VertexAttribDivisorANGLE(&self, index: u32, divisor: u32) {
        self.ctx.vertex_attrib_divisor(index, divisor);
    }
}
