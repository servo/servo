/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::WebGLSyncBinding;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject};
use crate::dom::bindings::root::DomRoot;
use crate::dom::webglobject::WebGLObject;
use crate::dom::webglrenderingcontext::WebGLRenderingContext;
use crate::task_source::TaskSource;
use canvas_traits::webgl::{webgl_channel, WebGLCommand, WebGLSyncId};
use dom_struct::dom_struct;
use std::cell::Cell;

#[dom_struct]
pub struct WebGLSync {
    webgl_object: WebGLObject,
    sync_id: WebGLSyncId,
    marked_for_deletion: Cell<bool>,
    sync_status: Cell<Option<u32>>,
}

impl WebGLSync {
    fn new_inherited(context: &WebGLRenderingContext, sync_id: WebGLSyncId) -> Self {
        Self {
            webgl_object: WebGLObject::new_inherited(context),
            sync_id,
            marked_for_deletion: Cell::new(false),
            sync_status: Cell::new(None),
        }
    }

    pub fn new(context: &WebGLRenderingContext) -> DomRoot<Self> {
        let (sender, receiver) = webgl_channel().unwrap();
        context.send_command(WebGLCommand::FenceSync(sender));
        let sync_id = receiver.recv().unwrap();

        reflect_dom_object(
            Box::new(WebGLSync::new_inherited(context, sync_id)),
            &*context.global(),
            WebGLSyncBinding::Wrap,
        )
    }
}

impl WebGLSync {
    pub fn client_wait_sync(&self, context: &WebGLRenderingContext, flags: u32, timeout: u64) {
        if self.get_client_sync_status().is_none() {
            let global = self.global();
            let this = Trusted::new(self);
            let context = Trusted::new(context);
            let task = task!(request_client_sync_status: move || {
                let this = this.root();
                let context = context.root();
                let (sender, receiver) = webgl_channel().unwrap();
                context.send_command(WebGLCommand::ClientWaitSync(
                    this.sync_id,
                    flags,
                    timeout,
                    sender,
                ));
                this.sync_status.set(Some(receiver.recv().unwrap()));
            });
            global
                .as_window()
                .task_manager()
                .dom_manipulation_task_source()
                .queue(task, global.upcast())
                .unwrap();
        }
    }

    pub fn delete(&self, context: &WebGLRenderingContext) {
        if self.is_valid() {
            self.marked_for_deletion.set(true);
            context.send_command(WebGLCommand::DeleteSync(self.sync_id));
        }
    }

    pub fn get_client_sync_status(&self) -> Option<u32> {
        self.sync_status.get()
    }

    pub fn is_valid(&self) -> bool {
        !self.marked_for_deletion.get()
    }

    pub fn id(&self) -> WebGLSyncId {
        self.sync_id
    }
}
