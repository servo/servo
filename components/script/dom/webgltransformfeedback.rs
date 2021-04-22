/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::reflector::{reflect_dom_object, DomObject};
use crate::dom::bindings::root::DomRoot;
use crate::dom::webglobject::WebGLObject;
use crate::dom::webglrenderingcontext::{Operation, WebGLRenderingContext, WebGLMessageSender, stub_webgl_backtrace};
use canvas_traits::webgl::{webgl_channel, WebGLCommand};
use dom_struct::dom_struct;
use std::cell::Cell;

#[derive(JSTraceable, MallocSizeOf)]
struct DroppableField {
    sender: WebGLMessageSender,
    marked_for_deletion: Cell<bool>,
    id: u32,
}

impl DroppableField {
    pub fn delete(&self, operation_fallibility: Operation) {
        if self.is_valid() && self.id() != 0 {
            self.marked_for_deletion.set(true);
            let cmd = WebGLCommand::DeleteTransformFeedback(self.id);
            match operation_fallibility {
                Operation::Fallible => {
                    let _ = self.sender.send(cmd, stub_webgl_backtrace());
                },
                Operation::Infallible => {
                    self.sender.send(cmd, stub_webgl_backtrace()).unwrap();
                },
            }
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.marked_for_deletion.get()
    }

    pub fn id(&self) -> u32 {
        self.id
    }
}

impl Drop for DroppableField {
    fn drop(&mut self) {
        self.delete(Operation::Fallible);
    }
}
#[dom_struct]
pub struct WebGLTransformFeedback {
    webgl_object: WebGLObject,
    has_been_bound: Cell<bool>,
    is_active: Cell<bool>,
    is_paused: Cell<bool>,
    droppable_field: DroppableField,
}

impl WebGLTransformFeedback {
    fn new_inherited(context: &WebGLRenderingContext, id: u32) -> Self {
        Self {
            webgl_object: WebGLObject::new_inherited(context),
            has_been_bound: Cell::new(false),
            is_active: Cell::new(false),
            is_paused: Cell::new(false),
            droppable_field: DroppableField {
                sender: context.webgl_sender(),
                id,
                marked_for_deletion: Cell::new(false),
            }
        }
    }

    pub fn new(context: &WebGLRenderingContext) -> DomRoot<Self> {
        let (sender, receiver) = webgl_channel().unwrap();
        context.send_command(WebGLCommand::CreateTransformFeedback(sender));
        let id = receiver.recv().unwrap();

        reflect_dom_object(
            Box::new(WebGLTransformFeedback::new_inherited(context, id)),
            &*context.global(),
        )
    }
}

impl WebGLTransformFeedback {
    pub fn bind(&self, context: &WebGLRenderingContext, target: u32) {
        context.send_command(WebGLCommand::BindTransformFeedback(target, self.id()));
        self.has_been_bound.set(true);
    }

    pub fn begin(&self, context: &WebGLRenderingContext, primitive_mode: u32) {
        if self.has_been_bound.get() && !self.is_active() {
            context.send_command(WebGLCommand::BeginTransformFeedback(primitive_mode));
            self.set_active(true);
        }
    }

    pub fn end(&self, context: &WebGLRenderingContext) {
        if self.has_been_bound.get() && self.is_active() {
            if self.is_paused() {
                context.send_command(WebGLCommand::ResumeTransformFeedback());
            }
            context.send_command(WebGLCommand::EndTransformFeedback());
            self.set_active(false);
        }
    }

    pub fn resume(&self, context: &WebGLRenderingContext) {
        if self.is_active() && self.is_paused() {
            context.send_command(WebGLCommand::ResumeTransformFeedback());
            self.set_pause(false);
        }
    }

    pub fn pause(&self, context: &WebGLRenderingContext) {
        if self.is_active() && !self.is_paused() {
            context.send_command(WebGLCommand::PauseTransformFeedback());
            self.set_pause(true);
        }
    }

    pub fn id(&self) -> u32 {
        self.droppable_field.id()
    }

    pub fn is_valid(&self) -> bool {
        self.droppable_field.is_valid()
    }

    pub fn is_active(&self) -> bool {
        self.is_active.get()
    }

    pub fn is_paused(&self) -> bool {
        self.is_paused.get()
    }

    pub fn delete(&self, operation_fallibility: Operation) {
        self.droppable_field.delete(operation_fallibility);
    }

    pub fn set_active(&self, value: bool) {
        if self.is_valid() && self.has_been_bound.get() {
            self.is_active.set(value);
        }
    }

    pub fn set_pause(&self, value: bool) {
        if self.is_valid() && self.is_active() {
            self.is_active.set(value);
        }
    }
}
