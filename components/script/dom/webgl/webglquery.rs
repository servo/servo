/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use dom_struct::dom_struct;
use script_bindings::weakref::WeakRef;
use servo_canvas_traits::webgl::WebGLError::*;
use servo_canvas_traits::webgl::{WebGLCommand, WebGLQueryId, webgl_channel};

use crate::dom::bindings::codegen::Bindings::WebGL2RenderingContextBinding::WebGL2RenderingContextConstants as constants;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::{DomGlobal, reflect_dom_object};
use crate::dom::bindings::root::DomRoot;
use crate::dom::webgl::webglobject::WebGLObject;
use crate::dom::webgl::webglrenderingcontext::{Operation, WebGLRenderingContext};
use crate::dom::webglrenderingcontext::capture_webgl_backtrace;
use crate::script_runtime::CanGc;

#[derive(JSTraceable, MallocSizeOf)]
struct DroppableWebGLQuery {
    context: WeakRef<WebGLRenderingContext>,
    #[no_trace]
    gl_id: WebGLQueryId,
    marked_for_deletion: Cell<bool>,
}

impl DroppableWebGLQuery {
    fn send_with_fallibility(&self, command: WebGLCommand, fallibility: Operation) {
        if let Some(root) = self.context.root() {
            let result = root.sender().send(command, capture_webgl_backtrace());
            if matches!(fallibility, Operation::Infallible) {
                result.expect("Operation failed");
            }
        }
    }

    fn delete(&self, operation_fallibility: Operation) {
        if !self.marked_for_deletion.get() {
            self.marked_for_deletion.set(true);
            self.send_with_fallibility(
                WebGLCommand::DeleteQuery(self.gl_id),
                operation_fallibility,
            );
        }
    }
}

impl Drop for DroppableWebGLQuery {
    fn drop(&mut self) {
        self.delete(Operation::Fallible);
    }
}

#[dom_struct(associated_memory)]
pub(crate) struct WebGLQuery {
    webgl_object: WebGLObject,
    gl_target: Cell<Option<u32>>,
    query_result_available: Cell<Option<u32>>,
    query_result: Cell<u32>,
    droppable: DroppableWebGLQuery,
}

impl WebGLQuery {
    fn new_inherited(context: &WebGLRenderingContext, id: WebGLQueryId) -> Self {
        Self {
            webgl_object: WebGLObject::new_inherited(context),
            gl_target: Cell::new(None),
            query_result_available: Cell::new(None),
            query_result: Cell::new(0),
            droppable: DroppableWebGLQuery {
                context: WeakRef::new(context),
                gl_id: id,
                marked_for_deletion: Cell::new(false),
            },
        }
    }

    pub(crate) fn new(context: &WebGLRenderingContext, can_gc: CanGc) -> DomRoot<Self> {
        let (sender, receiver) = webgl_channel().unwrap();
        context.send_command(WebGLCommand::GenerateQuery(sender));
        let id = receiver.recv().unwrap();

        reflect_dom_object(
            Box::new(Self::new_inherited(context, id)),
            &*context.global(),
            can_gc,
        )
    }

    pub(crate) fn begin(
        &self,
        context: &WebGLRenderingContext,
        target: u32,
    ) -> Result<(), servo_canvas_traits::webgl::WebGLError> {
        if self.droppable.marked_for_deletion.get() {
            return Err(InvalidOperation);
        }
        if let Some(current_target) = self.gl_target.get() {
            if current_target != target {
                return Err(InvalidOperation);
            }
        }
        match target {
            constants::ANY_SAMPLES_PASSED |
            constants::ANY_SAMPLES_PASSED_CONSERVATIVE |
            constants::TRANSFORM_FEEDBACK_PRIMITIVES_WRITTEN => (),
            _ => return Err(InvalidEnum),
        }
        self.gl_target.set(Some(target));

        context.send_command(WebGLCommand::BeginQuery(target, self.droppable.gl_id));
        Ok(())
    }

    pub(crate) fn end(
        &self,
        context: &WebGLRenderingContext,
        target: u32,
    ) -> Result<(), servo_canvas_traits::webgl::WebGLError> {
        if self.droppable.marked_for_deletion.get() {
            return Err(InvalidOperation);
        }
        if let Some(current_target) = self.gl_target.get() {
            if current_target != target {
                return Err(InvalidOperation);
            }
        }
        match target {
            constants::ANY_SAMPLES_PASSED |
            constants::ANY_SAMPLES_PASSED_CONSERVATIVE |
            constants::TRANSFORM_FEEDBACK_PRIMITIVES_WRITTEN => (),
            _ => return Err(InvalidEnum),
        }
        context.send_command(WebGLCommand::EndQuery(target));
        Ok(())
    }

    pub(crate) fn delete(&self, operation_fallibility: Operation) {
        self.droppable.delete(operation_fallibility);
    }

    pub(crate) fn is_valid(&self) -> bool {
        !self.droppable.marked_for_deletion.get() && self.target().is_some()
    }

    pub(crate) fn target(&self) -> Option<u32> {
        self.gl_target.get()
    }

    fn update_results(&self, context: &WebGLRenderingContext) {
        let (sender, receiver) = webgl_channel().unwrap();
        context.send_command(WebGLCommand::GetQueryState(
            sender,
            self.droppable.gl_id,
            constants::QUERY_RESULT_AVAILABLE,
        ));
        let is_available = receiver.recv().unwrap();
        if is_available == 0 {
            self.query_result_available.set(None);
            return;
        }

        let (sender, receiver) = webgl_channel().unwrap();
        context.send_command(WebGLCommand::GetQueryState(
            sender,
            self.droppable.gl_id,
            constants::QUERY_RESULT,
        ));

        self.query_result.set(receiver.recv().unwrap());
        self.query_result_available.set(Some(is_available));
    }

    #[rustfmt::skip]
    pub(crate) fn get_parameter(
        &self,
        context: &WebGLRenderingContext,
        pname: u32,
    ) -> Result<u32, servo_canvas_traits::webgl::WebGLError> {
        if !self.is_valid() {
            return Err(InvalidOperation);
        }
        match pname {
            constants::QUERY_RESULT |
            constants::QUERY_RESULT_AVAILABLE => {},
            _ => return Err(InvalidEnum),
        }

        if self.query_result_available.get().is_none() {
            self.query_result_available.set(Some(0));

            let this = Trusted::new(self);
            let context = Trusted::new(context);
            let task = task!(request_query_state: move || {
                let this = this.root();
                let context = context.root();
                this.update_results(&context);
            });

            self.global()
                .task_manager()
                .dom_manipulation_task_source()
                .queue(task);
        }

        match pname {
            constants::QUERY_RESULT => Ok(self.query_result.get()),
            constants::QUERY_RESULT_AVAILABLE => Ok(self.query_result_available.get().unwrap()),
            _ => unreachable!(),
        }
    }
}
