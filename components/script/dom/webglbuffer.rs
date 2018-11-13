/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://www.khronos.org/registry/webgl/specs/latest/1.0/webgl.idl
use canvas_traits::webgl::{WebGLBufferId, WebGLCommand, WebGLError, WebGLResult};
use canvas_traits::webgl::webgl_channel;
use dom::bindings::codegen::Bindings::WebGLBufferBinding;
use dom::bindings::codegen::Bindings::WebGLRenderingContextBinding::WebGLRenderingContextConstants;
use dom::bindings::inheritance::Castable;
use dom::bindings::reflector::{DomObject, reflect_dom_object};
use dom::bindings::root::DomRoot;
use dom::webglobject::WebGLObject;
use dom::webglrenderingcontext::WebGLRenderingContext;
use dom_struct::dom_struct;
use ipc_channel::ipc;
use std::cell::Cell;

#[dom_struct]
pub struct WebGLBuffer {
    webgl_object: WebGLObject,
    id: WebGLBufferId,
    /// The target to which this buffer was bound the first time
    target: Cell<Option<u32>>,
    capacity: Cell<usize>,
    marked_for_deletion: Cell<bool>,
    attached_counter: Cell<u32>,
    /// https://www.khronos.org/registry/OpenGL-Refpages/es2.0/xhtml/glGetBufferParameteriv.xml
    usage: Cell<u32>,
}

impl WebGLBuffer {
    fn new_inherited(context: &WebGLRenderingContext, id: WebGLBufferId) -> Self {
        Self {
            webgl_object: WebGLObject::new_inherited(context),
            id,
            target: Default::default(),
            capacity: Default::default(),
            marked_for_deletion: Default::default(),
            attached_counter: Default::default(),
            usage: Cell::new(WebGLRenderingContextConstants::STATIC_DRAW),
        }
    }

    pub fn maybe_new(context: &WebGLRenderingContext) -> Option<DomRoot<Self>> {
        let (sender, receiver) = webgl_channel().unwrap();
        context.send_command(WebGLCommand::CreateBuffer(sender));
        receiver
            .recv()
            .unwrap()
            .map(|id| WebGLBuffer::new(context, id))
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

    pub fn buffer_data(&self, data: &[u8], usage: u32) -> WebGLResult<()> {
        match usage {
            WebGLRenderingContextConstants::STREAM_DRAW |
            WebGLRenderingContextConstants::STATIC_DRAW |
            WebGLRenderingContextConstants::DYNAMIC_DRAW => (),
            _ => return Err(WebGLError::InvalidEnum),
        }

        self.capacity.set(data.len());
        self.usage.set(usage);
        let (sender, receiver) = ipc::bytes_channel().unwrap();
        self.upcast::<WebGLObject>()
            .context()
            .send_command(WebGLCommand::BufferData(
                self.target.get().unwrap(),
                receiver,
                usage,
            ));
        sender.send(data).unwrap();
        Ok(())
    }

    pub fn capacity(&self) -> usize {
        self.capacity.get()
    }

    pub fn mark_for_deletion(&self) {
        if self.marked_for_deletion.get() {
            return;
        }
        self.marked_for_deletion.set(true);
        if self.is_deleted() {
            self.delete();
        }
    }

    fn delete(&self) {
        assert!(self.is_deleted());
        self.upcast::<WebGLObject>()
            .context()
            .send_command(WebGLCommand::DeleteBuffer(self.id));
    }

    pub fn is_marked_for_deletion(&self) -> bool {
        self.marked_for_deletion.get()
    }

    pub fn is_deleted(&self) -> bool {
        self.marked_for_deletion.get() && !self.is_attached()
    }

    pub fn target(&self) -> Option<u32> {
        self.target.get()
    }

    pub fn set_target(&self, target: u32) -> WebGLResult<()> {
        if self.target.get().map_or(false, |t| t != target) {
            return Err(WebGLError::InvalidOperation);
        }
        self.target.set(Some(target));
        Ok(())
    }

    pub fn is_attached(&self) -> bool {
        self.attached_counter.get() != 0
    }

    pub fn increment_attached_counter(&self) {
        self.attached_counter.set(
            self.attached_counter
                .get()
                .checked_add(1)
                .expect("refcount overflowed"),
        );
    }

    pub fn decrement_attached_counter(&self) {
        self.attached_counter.set(
            self.attached_counter
                .get()
                .checked_sub(1)
                .expect("refcount underflowed"),
        );
        if self.is_deleted() {
            self.delete();
        }
    }

    pub fn usage(&self) -> u32 {
        self.usage.get()
    }
}

impl Drop for WebGLBuffer {
    fn drop(&mut self) {
        self.mark_for_deletion();
        assert!(self.is_deleted());
    }
}
