/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://www.khronos.org/registry/webgl/specs/latest/1.0/webgl.idl
use crate::dom::bindings::codegen::Bindings::WebGL2RenderingContextBinding::WebGL2RenderingContextConstants;
use crate::dom::bindings::codegen::Bindings::WebGLRenderingContextBinding::WebGLRenderingContextConstants;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject};
use crate::dom::bindings::root::{DomRoot};
use crate::dom::webglobject::WebGLObject;
use crate::dom::webglrenderingcontext::{Operation, WebGLRenderingContext, WebGLMessageSender, stub_webgl_backtrace};
use canvas_traits::webgl::webgl_channel;
use canvas_traits::webgl::{WebGLBufferId, WebGLCommand, WebGLError, WebGLResult};
use dom_struct::dom_struct;
use ipc_channel::ipc;
use std::cell::Cell;

fn target_is_copy_buffer(target: u32) -> bool {
    target == WebGL2RenderingContextConstants::COPY_READ_BUFFER ||
        target == WebGL2RenderingContextConstants::COPY_WRITE_BUFFER
}

struct DroppableField {
    id: WebGLBufferId,
    marked_for_deletion: Cell<bool>,
    attached_counter: Cell<u32>,
    sender: WebGLMessageSender,
}

impl DroppableField {
    fn mark_for_deletion(&self, operation_fallibility: Operation) {
        if self.marked_for_deletion.get() {
            return;
        }
        self.marked_for_deletion.set(true);
        if self.is_deleted() {
            self.delete(operation_fallibility);
        }
    }

    fn delete(&self, operation_fallibility: Operation) {
        assert!(self.is_deleted());
        let cmd = WebGLCommand::DeleteBuffer(self.id);
        match operation_fallibility {
            Operation::Fallible => {
                let _ = self.sender.send(cmd, stub_webgl_backtrace());
            },
            Operation::Infallible => {
                self.sender.send(cmd, stub_webgl_backtrace()).unwrap();
            },
        }
    }

    fn is_deleted(&self) -> bool {
        self.marked_for_deletion.get() && !self.is_attached()
    }

    fn is_attached(&self) -> bool {
        self.attached_counter.get() != 0
    }
}

impl Drop for DroppableField {
    fn drop(&mut self) {
        self.mark_for_deletion(Operation::Fallible);
    }
}

#[dom_struct]
pub struct WebGLBuffer {
    webgl_object: WebGLObject,
    /// The target to which this buffer was bound the first time
    target: Cell<Option<u32>>,
    capacity: Cell<usize>,
    /// https://www.khronos.org/registry/OpenGL-Refpages/es2.0/xhtml/glGetBufferParameteriv.xml
    usage: Cell<u32>,
    droppable_field: DroppableField,
}

impl WebGLBuffer {
    fn new_inherited(context: &WebGLRenderingContext, id: WebGLBufferId) -> Self {
        Self {
            webgl_object: WebGLObject::new_inherited(context),
            target: Default::default(),
            capacity: Default::default(),
            usage: Cell::new(WebGLRenderingContextConstants::STATIC_DRAW),
            droppable_field: DroppableField {
                id,
                marked_for_deletion: Default::default(),
                attached_counter: Default::default(),
                sender: context.webgl_sender(),
            },
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
        )
    }
}

impl WebGLBuffer {
    pub fn id(&self) -> WebGLBufferId {
        self.droppable_field.id
    }

    pub fn buffer_data(&self, target: u32, data: &[u8], usage: u32) -> WebGLResult<()> {
        match usage {
            WebGLRenderingContextConstants::STREAM_DRAW |
            WebGLRenderingContextConstants::STATIC_DRAW |
            WebGLRenderingContextConstants::DYNAMIC_DRAW |
            WebGL2RenderingContextConstants::STATIC_READ |
            WebGL2RenderingContextConstants::DYNAMIC_READ |
            WebGL2RenderingContextConstants::STREAM_READ |
            WebGL2RenderingContextConstants::STATIC_COPY |
            WebGL2RenderingContextConstants::DYNAMIC_COPY |
            WebGL2RenderingContextConstants::STREAM_COPY => (),
            _ => return Err(WebGLError::InvalidEnum),
        }

        self.capacity.set(data.len());
        self.usage.set(usage);
        let (sender, receiver) = ipc::bytes_channel().unwrap();
        self.upcast::<WebGLObject>()
            .context()
            .send_command(WebGLCommand::BufferData(target, receiver, usage));
        sender.send(data).unwrap();
        Ok(())
    }

    pub fn capacity(&self) -> usize {
        self.capacity.get()
    }

    pub fn mark_for_deletion(&self, operation_fallibility: Operation) {
        self.droppable_field
            .mark_for_deletion(operation_fallibility);
    }

    fn delete(&self, operation_fallibility: Operation) {
        self.droppable_field.delete(operation_fallibility);
    }

    pub fn is_marked_for_deletion(&self) -> bool {
        self.droppable_field.marked_for_deletion.get()
    }

    pub fn is_deleted(&self) -> bool {
        self.droppable_field.is_deleted()
    }

    pub fn target(&self) -> Option<u32> {
        self.target.get()
    }

    fn can_bind_to(&self, new_target: u32) -> bool {
        if let Some(current_target) = self.target.get() {
            if [current_target, new_target]
                .contains(&WebGLRenderingContextConstants::ELEMENT_ARRAY_BUFFER)
            {
                return target_is_copy_buffer(new_target) || new_target == current_target;
            }
        }
        true
    }

    pub fn set_target_maybe(&self, target: u32) -> WebGLResult<()> {
        if !self.can_bind_to(target) {
            return Err(WebGLError::InvalidOperation);
        }
        if !target_is_copy_buffer(target) {
            self.target.set(Some(target));
        }
        Ok(())
    }

    pub fn is_attached(&self) -> bool {
        self.droppable_field.attached_counter.get() != 0
    }

    pub fn increment_attached_counter(&self) {
        self.droppable_field.attached_counter.set(
            self.droppable_field
                .attached_counter
                .get()
                .checked_add(1)
                .expect("refcount overflowed"),
        );
    }

    pub fn decrement_attached_counter(&self, operation_fallibility: Operation) {
        self.droppable_field.attached_counter.set(
            self.droppable_field
                .attached_counter
                .get()
                .checked_sub(1)
                .expect("refcount underflowed"),
        );
        if self.droppable_field.is_deleted() {
            self.droppable_field.delete(operation_fallibility);
        }
    }

    pub fn usage(&self) -> u32 {
        self.usage.get()
    }
}
