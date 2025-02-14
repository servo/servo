/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use canvas_traits::webgl::WebGLVersion;
use dom_struct::dom_struct;

use super::{WebGLExtension, WebGLExtensionSpec, WebGLExtensions};
use crate::dom::bindings::codegen::Bindings::OESVertexArrayObjectBinding::{
    OESVertexArrayObjectConstants, OESVertexArrayObjectMethods,
};
use crate::dom::bindings::reflector::{reflect_dom_object, DomGlobal, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::webglrenderingcontext::WebGLRenderingContext;
use crate::dom::webglvertexarrayobjectoes::WebGLVertexArrayObjectOES;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct OESVertexArrayObject {
    reflector_: Reflector,
    ctx: Dom<WebGLRenderingContext>,
}

impl OESVertexArrayObject {
    fn new_inherited(ctx: &WebGLRenderingContext) -> OESVertexArrayObject {
        Self {
            reflector_: Reflector::new(),
            ctx: Dom::from_ref(ctx),
        }
    }
}

impl OESVertexArrayObjectMethods<crate::DomTypeHolder> for OESVertexArrayObject {
    // https://www.khronos.org/registry/webgl/extensions/OES_vertex_array_object/
    fn CreateVertexArrayOES(&self) -> Option<DomRoot<WebGLVertexArrayObjectOES>> {
        self.ctx.create_vertex_array()
    }

    // https://www.khronos.org/registry/webgl/extensions/OES_vertex_array_object/
    fn DeleteVertexArrayOES(&self, vao: Option<&WebGLVertexArrayObjectOES>) {
        self.ctx.delete_vertex_array(vao);
    }

    // https://www.khronos.org/registry/webgl/extensions/OES_vertex_array_object/
    fn IsVertexArrayOES(&self, vao: Option<&WebGLVertexArrayObjectOES>) -> bool {
        self.ctx.is_vertex_array(vao)
    }

    // https://www.khronos.org/registry/webgl/extensions/OES_vertex_array_object/
    fn BindVertexArrayOES(&self, vao: Option<&WebGLVertexArrayObjectOES>) {
        self.ctx.bind_vertex_array(vao);
    }
}

impl WebGLExtension for OESVertexArrayObject {
    type Extension = OESVertexArrayObject;
    fn new(ctx: &WebGLRenderingContext) -> DomRoot<OESVertexArrayObject> {
        reflect_dom_object(
            Box::new(OESVertexArrayObject::new_inherited(ctx)),
            &*ctx.global(),
            CanGc::note(),
        )
    }

    fn spec() -> WebGLExtensionSpec {
        WebGLExtensionSpec::Specific(WebGLVersion::WebGL1)
    }

    fn is_supported(ext: &WebGLExtensions) -> bool {
        ext.supports_any_gl_extension(&[
            "GL_OES_vertex_array_object",
            "GL_ARB_vertex_array_object",
            "GL_APPLE_vertex_array_object",
        ])
    }

    fn enable(ext: &WebGLExtensions) {
        ext.enable_get_parameter_name(OESVertexArrayObjectConstants::VERTEX_ARRAY_BINDING_OES);
    }

    fn name() -> &'static str {
        "OES_vertex_array_object"
    }
}
