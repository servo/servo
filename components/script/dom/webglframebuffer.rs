/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://www.khronos.org/registry/webgl/specs/latest/1.0/webgl.idl
use canvas_traits::{CanvasMsg};
use dom::bindings::codegen::Bindings::WebGLFramebufferBinding;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::Root;
use dom::bindings::reflector::reflect_dom_object;
use dom::webglobject::WebGLObject;
use ipc_channel::ipc::{self, IpcSender};
use std::cell::Cell;
use webrender_traits::{WebGLCommand, WebGLFramebufferBindingRequest};

#[dom_struct]
pub struct WebGLFramebuffer {
    webgl_object: WebGLObject,
    id: u32,
    is_deleted: Cell<bool>,
    #[ignore_heap_size_of = "Defined in ipc-channel"]
    renderer: IpcSender<CanvasMsg>,
}

impl WebGLFramebuffer {
    fn new_inherited(renderer: IpcSender<CanvasMsg>, id: u32) -> WebGLFramebuffer {
        WebGLFramebuffer {
            webgl_object: WebGLObject::new_inherited(),
            id: id,
            is_deleted: Cell::new(false),
            renderer: renderer,
        }
    }

    pub fn maybe_new(global: GlobalRef, renderer: IpcSender<CanvasMsg>)
                     -> Option<Root<WebGLFramebuffer>> {
        let (sender, receiver) = ipc::channel().unwrap();
        renderer.send(CanvasMsg::WebGL(WebGLCommand::CreateFramebuffer(sender))).unwrap();

        let result = receiver.recv().unwrap();
        result.map(|fb_id| WebGLFramebuffer::new(global, renderer, *fb_id))
    }

    pub fn new(global: GlobalRef, renderer: IpcSender<CanvasMsg>, id: u32)
               -> Root<WebGLFramebuffer> {
        reflect_dom_object(box WebGLFramebuffer::new_inherited(renderer, id), global, WebGLFramebufferBinding::Wrap)
    }
}


impl WebGLFramebuffer {
    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn bind(&self, target: u32) {
        let cmd = WebGLCommand::BindFramebuffer(target, WebGLFramebufferBindingRequest::Explicit(self.id));
        self.renderer.send(CanvasMsg::WebGL(cmd)).unwrap();
    }

    pub fn delete(&self) {
        if !self.is_deleted.get() {
            self.is_deleted.set(true);
            let _ = self.renderer.send(CanvasMsg::WebGL(WebGLCommand::DeleteFramebuffer(self.id)));
        }
    }
}

impl Drop for WebGLFramebuffer {
    fn drop(&mut self) {
        self.delete();
    }
}
