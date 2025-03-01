/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use canvas_traits::webgl::{ActiveAttribInfo, WebGLResult, WebGLVertexArrayId};
use dom_struct::dom_struct;

use crate::dom::bindings::cell::Ref;
use crate::dom::bindings::reflector::{reflect_dom_object, DomGlobal};
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::vertexarrayobject::{VertexArrayObject, VertexAttribData};
use crate::dom::webglbuffer::WebGLBuffer;
use crate::dom::webglobject::WebGLObject;
use crate::dom::webglrenderingcontext::{Operation, WebGLRenderingContext};
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct WebGLVertexArrayObjectOES {
    webgl_object_: WebGLObject,
    array_object: VertexArrayObject,
}

impl WebGLVertexArrayObjectOES {
    fn new_inherited(context: &WebGLRenderingContext, id: Option<WebGLVertexArrayId>) -> Self {
        Self {
            webgl_object_: WebGLObject::new_inherited(context),
            array_object: VertexArrayObject::new(context, id),
        }
    }

    pub(crate) fn new(
        context: &WebGLRenderingContext,
        id: Option<WebGLVertexArrayId>,
        can_gc: CanGc,
    ) -> DomRoot<Self> {
        reflect_dom_object(
            Box::new(WebGLVertexArrayObjectOES::new_inherited(context, id)),
            &*context.global(),
            can_gc,
        )
    }

    pub(crate) fn id(&self) -> Option<WebGLVertexArrayId> {
        self.array_object.id()
    }

    pub(crate) fn is_deleted(&self) -> bool {
        self.array_object.is_deleted()
    }

    pub(crate) fn delete(&self, operation_fallibility: Operation) {
        self.array_object.delete(operation_fallibility);
    }

    pub(crate) fn ever_bound(&self) -> bool {
        self.array_object.ever_bound()
    }

    pub(crate) fn set_ever_bound(&self) {
        self.array_object.set_ever_bound();
    }

    pub(crate) fn element_array_buffer(&self) -> &MutNullableDom<WebGLBuffer> {
        self.array_object.element_array_buffer()
    }

    pub(crate) fn get_vertex_attrib(&self, index: u32) -> Option<Ref<VertexAttribData>> {
        self.array_object.get_vertex_attrib(index)
    }

    pub(crate) fn set_vertex_attrib_type(&self, index: u32, type_: u32) {
        self.array_object.set_vertex_attrib_type(index, type_);
    }

    pub(crate) fn vertex_attrib_pointer(
        &self,
        index: u32,
        size: i32,
        type_: u32,
        normalized: bool,
        stride: i32,
        offset: i64,
    ) -> WebGLResult<()> {
        self.array_object
            .vertex_attrib_pointer(index, size, type_, normalized, stride, offset)
    }

    pub(crate) fn vertex_attrib_divisor(&self, index: u32, value: u32) {
        self.array_object.vertex_attrib_divisor(index, value);
    }

    pub(crate) fn enabled_vertex_attrib_array(&self, index: u32, value: bool) {
        self.array_object.enabled_vertex_attrib_array(index, value);
    }

    pub(crate) fn unbind_buffer(&self, buffer: &WebGLBuffer) {
        self.array_object.unbind_buffer(buffer);
    }

    pub(crate) fn validate_for_draw(
        &self,
        required_len: u32,
        instance_count: u32,
        active_attribs: &[ActiveAttribInfo],
    ) -> WebGLResult<()> {
        self.array_object
            .validate_for_draw(required_len, instance_count, active_attribs)
    }
}
