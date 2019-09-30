/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::WebGL2RenderingContextBinding::WebGL2RenderingContextConstants as constants;
use crate::dom::bindings::codegen::Bindings::WebGLQueryBinding;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject};
use crate::dom::bindings::root::DomRoot;
use crate::dom::webglobject::WebGLObject;
use crate::dom::webglrenderingcontext::WebGLRenderingContext;
use crate::task_source::TaskSource;
use canvas_traits::webgl::WebGLError::*;
use canvas_traits::webgl::{webgl_channel, WebGLCommand, WebGLQueryId};
use dom_struct::dom_struct;
use std::cell::Cell;

#[dom_struct]
pub struct WebGLQuery {
    webgl_object: WebGLObject,
    gl_id: WebGLQueryId,
    gl_target: Cell<Option<u32>>,
    marked_for_deletion: Cell<bool>,
    query_result_available: Cell<Option<u32>>,
    query_result: Cell<u32>,
}

impl WebGLQuery {
    fn new_inherited(context: &WebGLRenderingContext, id: WebGLQueryId) -> Self {
        Self {
            webgl_object: WebGLObject::new_inherited(context),
            gl_id: id,
            gl_target: Cell::new(None),
            marked_for_deletion: Cell::new(false),
            query_result_available: Cell::new(None),
            query_result: Cell::new(0),
        }
    }

    pub fn new(context: &WebGLRenderingContext) -> DomRoot<Self> {
        let (sender, receiver) = webgl_channel().unwrap();
        context.send_command(WebGLCommand::GenerateQuery(sender));
        let id = receiver.recv().unwrap();

        reflect_dom_object(
            Box::new(Self::new_inherited(context, id)),
            &*context.global(),
            WebGLQueryBinding::Wrap,
        )
    }

    pub fn begin(
        &self,
        context: &WebGLRenderingContext,
        target: u32,
    ) -> Result<(), canvas_traits::webgl::WebGLError> {
        if self.marked_for_deletion.get() {
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

        context.send_command(WebGLCommand::BeginQuery(target, self.gl_id));
        Ok(())
    }

    pub fn end(
        &self,
        context: &WebGLRenderingContext,
        target: u32,
    ) -> Result<(), canvas_traits::webgl::WebGLError> {
        if self.marked_for_deletion.get() {
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

    pub fn delete(&self, fallible: bool) {
        if !self.marked_for_deletion.get() {
            self.marked_for_deletion.set(true);

            let context = self.upcast::<WebGLObject>().context();
            let command = WebGLCommand::DeleteQuery(self.gl_id);
            if fallible {
                context.send_command_ignored(command);
            } else {
                context.send_command(command);
            }
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.marked_for_deletion.get() && self.target().is_some()
    }

    pub fn target(&self) -> Option<u32> {
        self.gl_target.get()
    }

    fn update_results(&self, context: &WebGLRenderingContext) {
        let (sender, receiver) = webgl_channel().unwrap();
        context.send_command(WebGLCommand::GetQueryState(
            sender,
            self.gl_id,
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
            self.gl_id,
            constants::QUERY_RESULT,
        ));

        self.query_result.set(receiver.recv().unwrap());
        self.query_result_available.set(Some(is_available));
    }

    #[cfg_attr(rustfmt, rustfmt_skip)]
    pub fn get_parameter(
        &self,
        context: &WebGLRenderingContext,
        pname: u32,
    ) -> Result<u32, canvas_traits::webgl::WebGLError> {
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

            let global = self.global();
            global
                .as_window()
                .task_manager()
                .dom_manipulation_task_source()
                .queue(task, global.upcast())
                .unwrap();
        }

        match pname {
            constants::QUERY_RESULT => Ok(self.query_result.get()),
            constants::QUERY_RESULT_AVAILABLE => Ok(self.query_result_available.get().unwrap()),
            _ => unreachable!(),
        }
    }
}

impl Drop for WebGLQuery {
    fn drop(&mut self) {
        self.delete(true);
    }
}
