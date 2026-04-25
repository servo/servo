/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use dom_struct::dom_struct;
use script_bindings::weakref::WeakRef;
use servo_canvas_traits::webgl::{WebGLCommand, WebGLSyncId, webgl_channel};

use crate::dom::bindings::codegen::Bindings::WebGL2RenderingContextBinding::WebGL2RenderingContextConstants as constants;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::{DomGlobal, reflect_dom_object};
use crate::dom::bindings::root::DomRoot;
use crate::dom::webgl::webglobject::WebGLObject;
use crate::dom::webgl::webglrenderingcontext::{Operation, WebGLRenderingContext};
use crate::dom::webglrenderingcontext::capture_webgl_backtrace;
use crate::script_runtime::CanGc;

#[derive(JSTraceable, MallocSizeOf)]
struct DroppableWebGLSync {
    context: WeakRef<WebGLRenderingContext>,
    #[no_trace]
    sync_id: WebGLSyncId,
    marked_for_deletion: Cell<bool>,
}

impl DroppableWebGLSync {
    fn send_with_fallibility(&self, command: WebGLCommand, fallibility: Operation) {
        if let Some(root) = self.context.root() {
            let result = root.sender().send(command, capture_webgl_backtrace());
            if matches!(fallibility, Operation::Infallible) {
                result.expect("Operation failed");
            }
        }
    }

    fn delete(&self, operation_fallibility: Operation) {
        if self.is_valid() {
            self.marked_for_deletion.set(true);
            self.send_with_fallibility(
                WebGLCommand::DeleteSync(self.sync_id),
                operation_fallibility,
            );
        }
    }

    fn is_valid(&self) -> bool {
        !self.marked_for_deletion.get()
    }
}

impl Drop for DroppableWebGLSync {
    fn drop(&mut self) {
        self.delete(Operation::Fallible);
    }
}

#[dom_struct(associated_memory)]
pub(crate) struct WebGLSync {
    webgl_object: WebGLObject,
    client_wait_status: Cell<Option<u32>>,
    sync_status: Cell<Option<u32>>,
    droppable: DroppableWebGLSync,
}

impl WebGLSync {
    fn new_inherited(context: &WebGLRenderingContext, sync_id: WebGLSyncId) -> Self {
        Self {
            webgl_object: WebGLObject::new_inherited(context),
            client_wait_status: Cell::new(None),
            sync_status: Cell::new(None),
            droppable: DroppableWebGLSync {
                context: WeakRef::new(context),
                sync_id,
                marked_for_deletion: Cell::new(false),
            },
        }
    }

    pub(crate) fn new(context: &WebGLRenderingContext, can_gc: CanGc) -> DomRoot<Self> {
        let (sender, receiver) = webgl_channel().unwrap();
        context.send_command(WebGLCommand::FenceSync(sender));
        let sync_id = receiver.recv().unwrap();

        reflect_dom_object(
            Box::new(WebGLSync::new_inherited(context, sync_id)),
            &*context.global(),
            can_gc,
        )
    }
}

impl WebGLSync {
    pub(crate) fn client_wait_sync(
        &self,
        context: &WebGLRenderingContext,
        flags: u32,
        timeout: u64,
    ) -> Option<u32> {
        match self.client_wait_status.get() {
            Some(constants::TIMEOUT_EXPIRED) | Some(constants::WAIT_FAILED) | None => {
                let this = Trusted::new(self);
                let context = Trusted::new(context);
                let task = task!(request_client_wait_status: move || {
                    let this = this.root();
                    let context = context.root();
                    let (sender, receiver) = webgl_channel().unwrap();
                    context.send_command(WebGLCommand::ClientWaitSync(
                        this.id(),
                        flags,
                        timeout,
                        sender,
                    ));
                    this.client_wait_status.set(Some(receiver.recv().unwrap()));
                });
                self.global()
                    .task_manager()
                    .dom_manipulation_task_source()
                    .queue(task);
            },
            _ => {},
        }
        self.client_wait_status.get()
    }

    pub(crate) fn delete(&self, operation_fallibility: Operation) {
        self.droppable.delete(operation_fallibility);
    }

    pub(crate) fn get_sync_status(
        &self,
        pname: u32,
        context: &WebGLRenderingContext,
    ) -> Option<u32> {
        match self.sync_status.get() {
            Some(constants::UNSIGNALED) | None => {
                let this = Trusted::new(self);
                let context = Trusted::new(context);
                let task = task!(request_sync_status: move || {
                    let this = this.root();
                    let context = context.root();
                    let (sender, receiver) = webgl_channel().unwrap();
                    context.send_command(WebGLCommand::GetSyncParameter(this.id(), pname, sender));
                    this.sync_status.set(Some(receiver.recv().unwrap()));
                });
                self.global()
                    .task_manager()
                    .dom_manipulation_task_source()
                    .queue(task);
            },
            _ => {},
        }
        self.sync_status.get()
    }

    pub(crate) fn is_valid(&self) -> bool {
        self.droppable.is_valid()
    }

    pub(crate) fn id(&self) -> WebGLSyncId {
        self.droppable.sync_id
    }
}
