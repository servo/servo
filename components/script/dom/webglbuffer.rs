/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://www.khronos.org/registry/webgl/specs/latest/1.0/webgl.idl
use canvas_traits::webgl::{WebGLBufferId, WebGLCommand, WebGLError, WebGLResult, WebGLVertexArrayId};
use canvas_traits::webgl::webgl_channel;
use dom::bindings::cell::DomRefCell;
use dom::bindings::codegen::Bindings::WebGLBufferBinding;
use dom::bindings::codegen::Bindings::WebGLRenderingContextBinding::WebGLRenderingContextConstants;
use dom::bindings::inheritance::Castable;
use dom::bindings::reflector::{DomObject, reflect_dom_object};
use dom::bindings::root::DomRoot;
use dom::webglobject::WebGLObject;
use dom::webglrenderingcontext::WebGLRenderingContext;
use dom_struct::dom_struct;
use std::cell::Cell;
use std::collections::HashSet;


#[dom_struct]
pub struct WebGLBuffer {
    webgl_object: WebGLObject,
    id: WebGLBufferId,
    /// The target to which this buffer was bound the first time
    target: Cell<Option<u32>>,
    capacity: Cell<usize>,
    is_deleted: Cell<bool>,
    // The Vertex Array Objects that are referencing this buffer
    vao_references: DomRefCell<Option<HashSet<WebGLVertexArrayId>>>,
    pending_delete: Cell<bool>,
    /// https://www.khronos.org/registry/OpenGL-Refpages/es2.0/xhtml/glGetBufferParameteriv.xml
    usage: Cell<u32>,
}

impl WebGLBuffer {
    fn new_inherited(context: &WebGLRenderingContext, id: WebGLBufferId) -> Self {
        Self {
            webgl_object: WebGLObject::new_inherited(context),
            id: id,
            target: Cell::new(None),
            capacity: Cell::new(0),
            is_deleted: Cell::new(false),
            vao_references: DomRefCell::new(None),
            pending_delete: Cell::new(false),
            usage: Cell::new(WebGLRenderingContextConstants::STATIC_DRAW),
        }
    }

    pub fn maybe_new(context: &WebGLRenderingContext) -> Option<DomRoot<Self>> {
        let (sender, receiver) = webgl_channel().unwrap();
        context.send_command(WebGLCommand::CreateBuffer(sender));
        receiver.recv().unwrap().map(|id| WebGLBuffer::new(context, id))
    }

    pub fn new(context: &WebGLRenderingContext, id: WebGLBufferId) -> DomRoot<Self> {
        reflect_dom_object(
            Box::new(WebGLBuffer::new_inherited(context, id)),
            &*context.global(),
            WebGLBufferBinding::Wrap,
        )
    }
}


impl WebGLBuffer {
    pub fn id(&self) -> WebGLBufferId {
        self.id
    }

    // NB: Only valid buffer targets come here
    pub fn bind(&self, target: u32) -> WebGLResult<()> {
        if self.is_deleted() || self.is_pending_delete() {
            return Err(WebGLError::InvalidOperation);
        }
        if let Some(previous_target) = self.target.get() {
            if target != previous_target {
                return Err(WebGLError::InvalidOperation);
            }
        } else {
            self.target.set(Some(target));
        }
        self.upcast::<WebGLObject>()
            .context()
            .send_command(WebGLCommand::BindBuffer(target, Some(self.id)));
        Ok(())
    }

    pub fn buffer_data<T>(&self, target: u32, data: T, usage: u32) -> WebGLResult<()>
    where
        T: Into<Vec<u8>>,
    {
        match usage {
            WebGLRenderingContextConstants::STREAM_DRAW |
            WebGLRenderingContextConstants::STATIC_DRAW |
            WebGLRenderingContextConstants::DYNAMIC_DRAW => (),
            _ => return Err(WebGLError::InvalidEnum),
        }

        if let Some(previous_target) = self.target.get() {
            if target != previous_target {
                return Err(WebGLError::InvalidOperation);
            }
        }
        let data = data.into();
        self.capacity.set(data.len());
        self.usage.set(usage);
        self.upcast::<WebGLObject>()
            .context()
            .send_command(WebGLCommand::BufferData(target, data.into(), usage));
        Ok(())
    }

    pub fn capacity(&self) -> usize {
        self.capacity.get()
    }

    pub fn delete(&self) {
        if !self.is_deleted.get() {
            self.is_deleted.set(true);
            self.upcast::<WebGLObject>()
                .context()
                .send_command(WebGLCommand::DeleteBuffer(self.id));
        }
    }

    pub fn is_deleted(&self) -> bool {
        self.is_deleted.get()
    }

    pub fn target(&self) -> Option<u32> {
        self.target.get()
    }

    pub fn is_attached_to_vao(&self) -> bool {
        self.vao_references.borrow().as_ref().map_or(false, |vaos| !vaos.is_empty())
    }

    pub fn set_pending_delete(&self) {
        self.pending_delete.set(true);
    }

    pub fn is_pending_delete(&self) -> bool {
        self.pending_delete.get()
    }

    pub fn add_vao_reference(&self, id: WebGLVertexArrayId) {
        let mut vao_refs = self.vao_references.borrow_mut();
        if let Some(ref mut vao_refs) = *vao_refs {
            vao_refs.insert(id);
            return;
        }

        let mut map = HashSet::new();
        map.insert(id);
        *vao_refs = Some(map);
    }

    pub fn remove_vao_reference(&self, id: WebGLVertexArrayId) {
        if let Some(ref mut vao_refs) = *self.vao_references.borrow_mut() {
            if vao_refs.take(&id).is_some() && self.pending_delete.get() {
                // WebGL spec: The deleted buffers should no longer be valid when the VAOs are deleted
                self.upcast::<WebGLObject>()
                    .context()
                    .send_command(WebGLCommand::DeleteBuffer(self.id));
                self.is_deleted.set(true);
            }
        }
    }

    pub fn usage(&self) -> u32 {
        self.usage.get()
    }
}

impl Drop for WebGLBuffer {
    fn drop(&mut self) {
        self.delete();
    }
}
