/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use canvas_traits::webgl::WebGLVertexArrayId;
use dom::bindings::codegen::Bindings::WebGLVertexArrayObjectOESBinding;
use dom::bindings::reflector::reflect_dom_object;
use dom::bindings::root::{DomRoot, MutNullableDom};
use dom::globalscope::GlobalScope;
use dom::webglbuffer::WebGLBuffer;
use dom::webglobject::WebGLObject;
use dom::webglrenderingcontext::BoundAttribBuffers;
use dom_struct::dom_struct;
use std::cell::Cell;

#[dom_struct]
pub struct WebGLVertexArrayObjectOES {
    webgl_object_: WebGLObject,
    id: WebGLVertexArrayId,
    ever_bound: Cell<bool>,
    is_deleted: Cell<bool>,
    bound_attrib_buffers: BoundAttribBuffers,
    bound_buffer_element_array: MutNullableDom<WebGLBuffer>,
}

impl WebGLVertexArrayObjectOES {
    fn new_inherited(id: WebGLVertexArrayId) -> WebGLVertexArrayObjectOES {
        Self {
            webgl_object_: WebGLObject::new_inherited(),
            id: id,
            ever_bound: Cell::new(false),
            is_deleted: Cell::new(false),
            bound_attrib_buffers: Default::default(),
            bound_buffer_element_array: MutNullableDom::new(None),
        }
    }

    pub fn new(global: &GlobalScope, id: WebGLVertexArrayId) -> DomRoot<WebGLVertexArrayObjectOES> {
        reflect_dom_object(Box::new(WebGLVertexArrayObjectOES::new_inherited(id)),
                           global,
                           WebGLVertexArrayObjectOESBinding::Wrap)
    }

    pub fn bound_attrib_buffers(&self) -> &BoundAttribBuffers {
        &self.bound_attrib_buffers
    }

    pub fn id(&self) -> WebGLVertexArrayId {
        self.id
    }

    pub fn is_deleted(&self) -> bool {
        self.is_deleted.get()
    }

    pub fn set_deleted(&self) {
        self.is_deleted.set(true)
    }

    pub fn ever_bound(&self) -> bool {
        return self.ever_bound.get()
    }

    pub fn set_ever_bound(&self) {
        self.ever_bound.set(true);
    }

    pub fn bound_buffer_element_array(&self) -> Option<DomRoot<WebGLBuffer>> {
        self.bound_buffer_element_array.get()
    }

    pub fn set_bound_buffer_element_array(&self, buffer: Option<&WebGLBuffer>) {
        self.bound_buffer_element_array.set(buffer);
    }
}
