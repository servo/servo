/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::WebGL2RenderingContextBinding::WebGL2RenderingContextConstants as constants;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject};
use crate::dom::bindings::root::DomRoot;
use crate::dom::webglobject::WebGLObject;
use crate::dom::webglrenderingcontext::{Operation, WebGLRenderingContext, WebGLMessageSender, stub_webgl_backtrace};
use crate::task_source::TaskSource;
use canvas_traits::webgl::{webgl_channel, WebGLCommand, WebGLSyncId};
use dom_struct::dom_struct;
use std::cell::Cell;

struct DroppableField {
    sender: WebGLMessageSender,
    marked_for_deletion: Cell<bool>,
    sync_id: WebGLSyncId,
}

impl DroppableField {
    pub fn delete(&self, operation_fallibility: Operation) {
        if self.is_valid() {
            self.marked_for_deletion.set(true);
            let cmd = WebGLCommand::DeleteSync(self.sync_id);
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
}

impl Drop for DroppableField {
    fn drop(&mut self) {
        self.delete(Operation::Fallible);
    }
}

#[dom_struct]
pub struct WebGLSync {
    webgl_object: WebGLObject,
    client_wait_status: Cell<Option<u32>>,
    sync_status: Cell<Option<u32>>,
    droppable_field: DroppableField,
}

impl WebGLSync {
    fn new_inherited(context: &WebGLRenderingContext, sync_id: WebGLSyncId) -> Self {
        Self {
            webgl_object: WebGLObject::new_inherited(context),
            client_wait_status: Cell::new(None),
            sync_status: Cell::new(None),
            droppable_field: DroppableField {
                sender: context.webgl_sender(),
                marked_for_deletion: Cell::new(false),
                sync_id,
            }
        }
    }

    pub fn new(context: &WebGLRenderingContext) -> DomRoot<Self> {
        let (sender, receiver) = webgl_channel().unwrap();
        context.send_command(WebGLCommand::FenceSync(sender));
        let sync_id = receiver.recv().unwrap();

        reflect_dom_object(
            Box::new(WebGLSync::new_inherited(context, sync_id)),
            &*context.global(),
        )
    }
}

impl WebGLSync {
    pub fn client_wait_sync(
        &self,
        context: &WebGLRenderingContext,
        flags: u32,
        timeout: u64,
    ) -> Option<u32> {
        match self.client_wait_status.get() {
            Some(constants::TIMEOUT_EXPIRED) | Some(constants::WAIT_FAILED) | None => {
                let global = self.global();
                let this = Trusted::new(self);
                let context = Trusted::new(context);
                let task = task!(request_client_wait_status: move || {
                    let this = this.root();
                    let context = context.root();
                    let (sender, receiver) = webgl_channel().unwrap();
                    context.send_command(WebGLCommand::ClientWaitSync(
                        this.droppable_field.sync_id,
                        flags,
                        timeout,
                        sender,
                    ));
                    this.client_wait_status.set(Some(receiver.recv().unwrap()));
                });
                global
                    .as_window()
                    .task_manager()
                    .dom_manipulation_task_source()
                    .queue(task, global.upcast())
                    .unwrap();
            },
            _ => {},
        }
        self.client_wait_status.get()
    }

    pub fn delete(&self, operation_fallibility: Operation) {
        self.droppable_field.delete(operation_fallibility);
    }

    pub fn get_sync_status(&self, pname: u32, context: &WebGLRenderingContext) -> Option<u32> {
        match self.sync_status.get() {
            Some(constants::UNSIGNALED) | None => {
                let global = self.global();
                let this = Trusted::new(self);
                let context = Trusted::new(context);
                let task = task!(request_sync_status: move || {
                    let this = this.root();
                    let context = context.root();
                    let (sender, receiver) = webgl_channel().unwrap();
                    context.send_command(WebGLCommand::GetSyncParameter(this.droppable_field.sync_id, pname, sender));
                    this.sync_status.set(Some(receiver.recv().unwrap()));
                });
                global
                    .as_window()
                    .task_manager()
                    .dom_manipulation_task_source()
                    .queue(task, global.upcast())
                    .unwrap();
            },
            _ => {},
        }
        self.sync_status.get()
    }

    pub fn is_valid(&self) -> bool {
        !self.droppable_field.is_valid()
    }

    pub fn id(&self) -> WebGLSyncId {
        self.droppable_field.sync_id
    }
}
