/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://www.khronos.org/registry/webgl/specs/latest/1.0/webgl.idl
use std::cell::Cell;

use canvas_traits::webgl::{WebGLBufferId, WebGLCommand, WebGLError, WebGLResult, webgl_channel};
use dom_struct::dom_struct;
use ipc_channel::ipc;
use script_bindings::weakref::WeakRef;

use crate::dom::bindings::codegen::Bindings::WebGL2RenderingContextBinding::WebGL2RenderingContextConstants;
use crate::dom::bindings::codegen::Bindings::WebGLRenderingContextBinding::WebGLRenderingContextConstants;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::{DomGlobal, reflect_dom_object};
use crate::dom::bindings::root::DomRoot;
use crate::dom::webglobject::WebGLObject;
use crate::dom::webglrenderingcontext::{Operation, WebGLRenderingContext};
use crate::script_runtime::CanGc;

fn target_is_copy_buffer(target: u32) -> bool {
    target == WebGL2RenderingContextConstants::COPY_READ_BUFFER ||
        target == WebGL2RenderingContextConstants::COPY_WRITE_BUFFER
}

#[derive(JSTraceable, MallocSizeOf)]
struct DroppableWebGLBuffer {
    #[no_trace]
    id: WebGLBufferId,
    marked_for_deletion: Cell<bool>,
    attached_counter: Cell<u32>,
    context: WeakRef<WebGLRenderingContext>,
}

impl DroppableWebGLBuffer {
    pub(crate) fn new(
        id: WebGLBufferId,
        marked_for_deletion: Cell<bool>,
        attached_counter: Cell<u32>,
        context: WeakRef<WebGLRenderingContext>,
    ) -> Self {
        Self {
            id,
            marked_for_deletion,
            attached_counter,
            context,
        }
    }
}

impl DroppableWebGLBuffer {
    pub(crate) fn is_marked_for_deletion(&self) -> bool {
        self.marked_for_deletion.get()
    }

    pub(crate) fn set_marked_for_deletion(&self, marked_for_deletion: bool) {
        self.marked_for_deletion.set(marked_for_deletion);
    }

    pub(crate) fn get_attached_counter(&self) -> u32 {
        self.attached_counter.get()
    }

    pub(crate) fn set_attached_counter(&self, attached_counter: u32) {
        self.attached_counter.set(attached_counter);
    }

    pub(crate) fn id(&self) -> WebGLBufferId {
        self.id
    }

    pub(crate) fn is_attached(&self) -> bool {
        self.get_attached_counter() != 0
    }

    pub(crate) fn is_deleted(&self) -> bool {
        self.is_marked_for_deletion() && !self.is_attached()
    }

    pub(crate) fn delete(&self, operation_fallibility: Operation) {
        assert!(self.is_deleted());
        if let Some(context) = self.context.root() {
            let cmd = WebGLCommand::DeleteBuffer(self.id);
            match operation_fallibility {
                Operation::Fallible => context.send_command_ignored(cmd),
                Operation::Infallible => context.send_command(cmd),
            }
        }
    }

    pub(crate) fn mark_for_deletion(&self, operation_fallibility: Operation) {
        if self.is_marked_for_deletion() {
            return;
        }
        self.set_marked_for_deletion(true);
        if self.is_deleted() {
            self.delete(operation_fallibility);
        }
    }
}

impl Drop for DroppableWebGLBuffer {
    fn drop(&mut self) {
        self.mark_for_deletion(Operation::Fallible);
    }
}

#[dom_struct]
pub(crate) struct WebGLBuffer {
    webgl_object: WebGLObject,
    /// The target to which this buffer was bound the first time
    target: Cell<Option<u32>>,
    capacity: Cell<usize>,
    /// <https://www.khronos.org/registry/OpenGL-Refpages/es2.0/xhtml/glGetBufferParameteriv.xml>
    usage: Cell<u32>,
    droppable: DroppableWebGLBuffer,
}

impl WebGLBuffer {
    fn new_inherited(context: &WebGLRenderingContext, id: WebGLBufferId) -> Self {
        Self {
            webgl_object: WebGLObject::new_inherited(context),
            target: Default::default(),
            capacity: Default::default(),
            usage: Cell::new(WebGLRenderingContextConstants::STATIC_DRAW),
            droppable: DroppableWebGLBuffer::new(
                id,
                Default::default(),
                Default::default(),
                WeakRef::new(context),
            ),
        }
    }

    pub(crate) fn maybe_new(
        context: &WebGLRenderingContext,
        can_gc: CanGc,
    ) -> Option<DomRoot<Self>> {
        let (sender, receiver) = webgl_channel().unwrap();
        context.send_command(WebGLCommand::CreateBuffer(sender));
        receiver
            .recv()
            .unwrap()
            .map(|id| WebGLBuffer::new(context, id, can_gc))
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
        self.droppable.id()
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
        self.droppable.mark_for_deletion(operation_fallibility);
    }

    fn delete(&self, operation_fallibility: Operation) {
        self.droppable.delete(operation_fallibility);
    }

    pub(crate) fn is_marked_for_deletion(&self) -> bool {
        self.droppable.is_marked_for_deletion()
    }

    fn get_attached_counter(&self) -> u32 {
        self.droppable.get_attached_counter()
    }

    fn set_attached_counter(&self, attached_counter: u32) {
        self.droppable.set_attached_counter(attached_counter);
    }

    pub(crate) fn is_deleted(&self) -> bool {
        self.droppable.is_deleted()
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

    pub(crate) fn increment_attached_counter(&self) {
        self.set_attached_counter(
            self.get_attached_counter()
                .checked_add(1)
                .expect("refcount overflowed"),
        );
    }

    pub(crate) fn decrement_attached_counter(&self, operation_fallibility: Operation) {
        self.set_attached_counter(
            self.get_attached_counter()
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
