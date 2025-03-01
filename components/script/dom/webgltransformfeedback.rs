/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use canvas_traits::webgl::{webgl_channel, WebGLCommand};
use dom_struct::dom_struct;

use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::{reflect_dom_object, DomGlobal};
use crate::dom::bindings::root::DomRoot;
use crate::dom::webglobject::WebGLObject;
use crate::dom::webglrenderingcontext::{Operation, WebGLRenderingContext};
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct WebGLTransformFeedback {
    webgl_object: WebGLObject,
    id: u32,
    marked_for_deletion: Cell<bool>,
    has_been_bound: Cell<bool>,
    is_active: Cell<bool>,
    is_paused: Cell<bool>,
}

impl WebGLTransformFeedback {
    fn new_inherited(context: &WebGLRenderingContext, id: u32) -> Self {
        Self {
            webgl_object: WebGLObject::new_inherited(context),
            id,
            marked_for_deletion: Cell::new(false),
            has_been_bound: Cell::new(false),
            is_active: Cell::new(false),
            is_paused: Cell::new(false),
        }
    }

    pub(crate) fn new(context: &WebGLRenderingContext, can_gc: CanGc) -> DomRoot<Self> {
        let (sender, receiver) = webgl_channel().unwrap();
        context.send_command(WebGLCommand::CreateTransformFeedback(sender));
        let id = receiver.recv().unwrap();

        reflect_dom_object(
            Box::new(WebGLTransformFeedback::new_inherited(context, id)),
            &*context.global(),
            can_gc,
        )
    }
}

impl WebGLTransformFeedback {
    pub(crate) fn bind(&self, context: &WebGLRenderingContext, target: u32) {
        context.send_command(WebGLCommand::BindTransformFeedback(target, self.id()));
        self.has_been_bound.set(true);
    }

    pub(crate) fn begin(&self, context: &WebGLRenderingContext, primitive_mode: u32) {
        if self.has_been_bound.get() && !self.is_active() {
            context.send_command(WebGLCommand::BeginTransformFeedback(primitive_mode));
            self.set_active(true);
        }
    }

    pub(crate) fn end(&self, context: &WebGLRenderingContext) {
        if self.has_been_bound.get() && self.is_active() {
            if self.is_paused() {
                context.send_command(WebGLCommand::ResumeTransformFeedback());
            }
            context.send_command(WebGLCommand::EndTransformFeedback());
            self.set_active(false);
        }
    }

    pub(crate) fn resume(&self, context: &WebGLRenderingContext) {
        if self.is_active() && self.is_paused() {
            context.send_command(WebGLCommand::ResumeTransformFeedback());
            self.set_pause(false);
        }
    }

    pub(crate) fn pause(&self, context: &WebGLRenderingContext) {
        if self.is_active() && !self.is_paused() {
            context.send_command(WebGLCommand::PauseTransformFeedback());
            self.set_pause(true);
        }
    }

    pub(crate) fn id(&self) -> u32 {
        self.id
    }

    pub(crate) fn is_valid(&self) -> bool {
        !self.marked_for_deletion.get()
    }

    pub(crate) fn is_active(&self) -> bool {
        self.is_active.get()
    }

    pub(crate) fn is_paused(&self) -> bool {
        self.is_paused.get()
    }

    pub(crate) fn delete(&self, operation_fallibility: Operation) {
        if self.is_valid() && self.id() != 0 {
            self.marked_for_deletion.set(true);
            let context = self.upcast::<WebGLObject>().context();
            let cmd = WebGLCommand::DeleteTransformFeedback(self.id);
            match operation_fallibility {
                Operation::Fallible => context.send_command_ignored(cmd),
                Operation::Infallible => context.send_command(cmd),
            }
        }
    }

    pub(crate) fn set_active(&self, value: bool) {
        if self.is_valid() && self.has_been_bound.get() {
            self.is_active.set(value);
        }
    }

    pub(crate) fn set_pause(&self, value: bool) {
        if self.is_valid() && self.is_active() {
            self.is_active.set(value);
        }
    }
}

impl Drop for WebGLTransformFeedback {
    fn drop(&mut self) {
        self.delete(Operation::Fallible);
    }
}
