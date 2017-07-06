/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use core::cell::Ref;
use core::iter::FromIterator;
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::WebGLVertexArrayObjectOESBinding;
use dom::bindings::js::{JS, MutNullableJS};
use dom::bindings::js::Root;
use dom::bindings::reflector::reflect_dom_object;
use dom::globalscope::GlobalScope;
use dom::webglbuffer::WebGLBuffer;
use dom::webglobject::WebGLObject;
use dom_struct::dom_struct;
use std::cell::Cell;
use std::collections::HashMap;
use webrender_api::WebGLVertexArrayId;

#[dom_struct]
pub struct WebGLVertexArrayObjectOES {
    webgl_object_: WebGLObject,
    id: WebGLVertexArrayId,
    ever_bound: Cell<bool>,
    is_deleted: Cell<bool>,
    bound_attrib_buffers: DOMRefCell<HashMap<u32, JS<WebGLBuffer>>>,
    bound_buffer_element_array: MutNullableJS<WebGLBuffer>,
}

impl WebGLVertexArrayObjectOES {
    fn new_inherited(id: WebGLVertexArrayId) -> WebGLVertexArrayObjectOES {
        Self {
            webgl_object_: WebGLObject::new_inherited(),
            id: id,
            ever_bound: Cell::new(false),
            is_deleted: Cell::new(false),
            bound_attrib_buffers: DOMRefCell::new(HashMap::new()),
            bound_buffer_element_array: MutNullableJS::new(None),
        }
    }

    pub fn new(global: &GlobalScope, id: WebGLVertexArrayId) -> Root<WebGLVertexArrayObjectOES> {
        reflect_dom_object(box WebGLVertexArrayObjectOES::new_inherited(id),
                           global,
                           WebGLVertexArrayObjectOESBinding::Wrap)
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

    pub fn borrow_bound_attrib_buffers(&self) -> Ref<HashMap<u32, JS<WebGLBuffer>>> {
        self.bound_attrib_buffers.borrow()
    }

    pub fn bound_attrib_buffers(&self) -> Vec<Root<WebGLBuffer>> {
        self.bound_attrib_buffers.borrow().iter().map(|(_, b)| Root::from_ref(&**b)).collect()
    }

    pub fn set_bound_attrib_buffers<'a, T>(&self, iter: T) where T: Iterator<Item=(u32, &'a WebGLBuffer)> {
        *self.bound_attrib_buffers.borrow_mut() = HashMap::from_iter(iter.map(|(k,v)| (k, JS::from_ref(v))));
    }

    pub fn bound_buffer_element_array(&self) -> Option<Root<WebGLBuffer>> {
        self.bound_buffer_element_array.get()
    }

    pub fn set_bound_buffer_element_array(&self, buffer: Option<&WebGLBuffer>) {
        self.bound_buffer_element_array.set(buffer);
    }
}
