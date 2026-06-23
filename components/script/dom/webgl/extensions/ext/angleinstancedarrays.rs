/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::JSContext;
use script_bindings::reflector::{Reflector, reflect_dom_object_with_cx};
use script_bindings::script_runtime::temp_cx;
use servo_canvas_traits::webgl::WebGLVersion;

use super::{WebGLExtension, WebGLExtensionSpec, WebGLExtensions};
use crate::dom::bindings::codegen::Bindings::ANGLEInstancedArraysBinding::{
    ANGLEInstancedArraysConstants, ANGLEInstancedArraysMethods,
};
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::webgl::webglrenderingcontext::WebGLRenderingContext;

#[dom_struct]
pub(crate) struct ANGLEInstancedArrays {
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

    fn new(cx: &mut JSContext, ctx: &WebGLRenderingContext) -> DomRoot<Self> {
        reflect_dom_object_with_cx(
            Box::new(ANGLEInstancedArrays::new_inherited(ctx)),
            &*ctx.global(),
            cx,
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

impl ANGLEInstancedArraysMethods<crate::DomTypeHolder> for ANGLEInstancedArrays {
    /// <https://www.khronos.org/registry/webgl/extensions/ANGLE_instanced_arrays/>
    #[expect(unsafe_code, reason = "transfer to jscontext")]
    fn DrawArraysInstancedANGLE(&self, mode: u32, first: i32, count: i32, primcount: i32) {
        let mut cx = unsafe { temp_cx() };
        let cx = &mut cx;
        handle_potential_webgl_error!(
            self.ctx,
            self.ctx
                .draw_arrays_instanced(cx, mode, first, count, primcount)
        )
    }

    /// <https://www.khronos.org/registry/webgl/extensions/ANGLE_instanced_arrays/>
    #[expect(unsafe_code, reason = "transfer to jscontext")]
    fn DrawElementsInstancedANGLE(
        &self,
        mode: u32,
        count: i32,
        type_: u32,
        offset: i64,
        primcount: i32,
    ) {
        let mut cx = unsafe { temp_cx() };
        let cx = &mut cx;
        handle_potential_webgl_error!(
            self.ctx,
            self.ctx
                .draw_elements_instanced(cx, mode, count, type_, offset, primcount)
        )
    }

    #[expect(unsafe_code, reason = "transfer to jscontext")]
    fn VertexAttribDivisorANGLE(&self, index: u32, divisor: u32) {
        let mut cx = unsafe { temp_cx() };
        let cx = &mut cx;
        self.ctx.vertex_attrib_divisor(cx, index, divisor);
    }
}
