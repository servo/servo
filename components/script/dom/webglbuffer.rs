/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://www.khronos.org/registry/webgl/specs/latest/1.0/webgl.idl
use std::cell::Cell;

use canvas_traits::webgl::{webgl_channel, WebGLBufferId, WebGLCommand, WebGLError, WebGLResult};
use dom_struct::dom_struct;
use ipc_channel::ipc;

use crate::dom::bindings::codegen::Bindings::WebGL2RenderingContextBinding::WebGL2RenderingContextConstants;
use crate::dom::bindings::codegen::Bindings::WebGLRenderingContextBinding::WebGLRenderingContextConstants;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::{reflect_dom_object, DomGlobal};
use crate::dom::bindings::root::DomRoot;
use crate::dom::webglobject::WebGLObject;
use crate::dom::webglrenderingcontext::{Operation, WebGLRenderingContext};
use crate::script_runtime::CanGc;

fn target_is_copy_buffer(target: u32) -> bool {
    target == WebGL2RenderingContextConstants::COPY_READ_BUFFER ||
        target == WebGL2RenderingContextConstants::COPY_WRITE_BUFFER
}

#[dom_struct]
pub(crate) struct WebGLBuffer {
    webgl_object: WebGLObject,
    #[no_trace]
    id: WebGLBufferId,
    /// The target to which this buffer was bound the first time
    target: Cell<Option<u32>>,
    capacity: Cell<usize>,
    marked_for_deletion: Cell<bool>,
    attached_counter: Cell<u32>,
    /// <https://www.khronos.org/registry/OpenGL-Refpages/es2.0/xhtml/glGetBufferParameteriv.xml>
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

    pub(crate) fn maybe_new(context: &WebGLRenderingContext) -> Option<DomRoot<Self>> {
        let (sender, receiver) = webgl_channel().unwrap();
        context.send_command(WebGLCommand::CreateBuffer(sender));
        receiver
            .recv()
            .unwrap()
            .map(|id| WebGLBuffer::new(context, id, CanGc::note()))
    }

    pub(crate) fn new(
        context: &WebGLRenderingContext,
        id: WebGLBufferId,
        can_gc: CanGc,
    ) -> DomRoot<Self> {
        reflect_dom_object(
            Box::new(WebGLBuffer::new_inherited(context, id)),
            &*context.global(),
            can_gc,
        )
    }
}

impl WebGLBuffer {
    pub(crate) fn id(&self) -> WebGLBufferId {
        self.id
    }

    pub(crate) fn buffer_data(&self, target: u32, data: &[u8], usage: u32) -> WebGLResult<()> {
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

    pub(crate) fn capacity(&self) -> usize {
        self.capacity.get()
    }

    pub(crate) fn mark_for_deletion(&self, operation_fallibility: Operation) {
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
        let context = self.upcast::<WebGLObject>().context();
        let cmd = WebGLCommand::DeleteBuffer(self.id);
        match operation_fallibility {
            Operation::Fallible => context.send_command_ignored(cmd),
            Operation::Infallible => context.send_command(cmd),
        }
    }

    pub(crate) fn is_marked_for_deletion(&self) -> bool {
        self.marked_for_deletion.get()
    }

    pub(crate) fn is_deleted(&self) -> bool {
        self.marked_for_deletion.get() && !self.is_attached()
    }

    pub(crate) fn target(&self) -> Option<u32> {
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

    pub(crate) fn set_target_maybe(&self, target: u32) -> WebGLResult<()> {
        if !self.can_bind_to(target) {
            return Err(WebGLError::InvalidOperation);
        }
        if !target_is_copy_buffer(target) {
            self.target.set(Some(target));
        }
        Ok(())
    }

    pub(crate) fn is_attached(&self) -> bool {
        self.attached_counter.get() != 0
    }

    pub(crate) fn increment_attached_counter(&self) {
        self.attached_counter.set(
            self.attached_counter
                .get()
                .checked_add(1)
                .expect("refcount overflowed"),
        );
    }

    pub(crate) fn decrement_attached_counter(&self, operation_fallibility: Operation) {
        self.attached_counter.set(
            self.attached_counter
                .get()
                .checked_sub(1)
                .expect("refcount underflowed"),
        );
        if self.is_deleted() {
            self.delete(operation_fallibility);
        }
    }

    pub(crate) fn usage(&self) -> u32 {
        self.usage.get()
    }
}

impl Drop for WebGLBuffer {
    fn drop(&mut self) {
        self.mark_for_deletion(Operation::Fallible);
    }
}
