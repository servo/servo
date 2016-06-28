/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://www.khronos.org/registry/webgl/specs/latest/1.0/webgl.idl
use canvas_traits::CanvasMsg;
use dom::bindings::codegen::Bindings::WebGLRenderbufferBinding;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::Root;
use dom::bindings::reflector::reflect_dom_object;
use dom::webglobject::WebGLObject;
use ipc_channel::ipc::{self, IpcSender};
use std::cell::Cell;
use webrender_traits::{WebGLCommand, WebGLRenderbufferId};

#[dom_struct]
pub struct WebGLRenderbuffer {
    webgl_object: WebGLObject,
    id: WebGLRenderbufferId,
    ever_bound: Cell<bool>,
    is_deleted: Cell<bool>,
    #[ignore_heap_size_of = "Defined in ipc-channel"]
    renderer: IpcSender<CanvasMsg>,
}

impl WebGLRenderbuffer {
    fn new_inherited(renderer: IpcSender<CanvasMsg>,
                     id: WebGLRenderbufferId)
                     -> WebGLRenderbuffer {
        WebGLRenderbuffer {
            webgl_object: WebGLObject::new_inherited(),
            id: id,
            ever_bound: Cell::new(false),
            is_deleted: Cell::new(false),
            renderer: renderer,
        }
    }

    pub fn maybe_new(global: GlobalRef, renderer: IpcSender<CanvasMsg>)
                     -> Option<Root<WebGLRenderbuffer>> {
        let (sender, receiver) = ipc::channel().unwrap();
        renderer.send(CanvasMsg::WebGL(WebGLCommand::CreateRenderbuffer(sender))).unwrap();

        let result = receiver.recv().unwrap();
        result.map(|renderbuffer_id| WebGLRenderbuffer::new(global, renderer, renderbuffer_id))
    }

    pub fn new(global: GlobalRef,
               renderer: IpcSender<CanvasMsg>,
               id: WebGLRenderbufferId)
               -> Root<WebGLRenderbuffer> {
        reflect_dom_object(box WebGLRenderbuffer::new_inherited(renderer, id),
                           global,
                           WebGLRenderbufferBinding::Wrap)
    }
}


impl WebGLRenderbuffer {
    pub fn id(&self) -> WebGLRenderbufferId {
        self.id
    }

    pub fn bind(&self, target: u32) {
        self.ever_bound.set(true);
        let msg = CanvasMsg::WebGL(WebGLCommand::BindRenderbuffer(target, Some(self.id)));
        self.renderer.send(msg).unwrap();
    }

    pub fn delete(&self) {
        if !self.is_deleted.get() {
            self.is_deleted.set(true);
            let _ = self.renderer.send(CanvasMsg::WebGL(WebGLCommand::DeleteRenderbuffer(self.id)));
        }
    }

    pub fn is_deleted(&self) -> bool {
        self.is_deleted.get()
    }

    pub fn ever_bound(&self) -> bool {
        self.ever_bound.get()
    }
}
