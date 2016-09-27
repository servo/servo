/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://www.khronos.org/registry/webgl/specs/latest/1.0/webgl.idl
use canvas_traits::CanvasMsg;
use dom::bindings::codegen::Bindings::WebGLFramebufferBinding;
use dom::bindings::codegen::Bindings::WebGLRenderingContextBinding::WebGLRenderingContextConstants as constants;
use dom::bindings::js::Root;
use dom::bindings::reflector::reflect_dom_object;
use dom::globalscope::GlobalScope;
use dom::webglobject::WebGLObject;
use ipc_channel::ipc::{self, IpcSender};
use std::cell::Cell;
use webrender_traits::{WebGLCommand, WebGLFramebufferBindingRequest, WebGLFramebufferId};

#[dom_struct]
pub struct WebGLFramebuffer {
    webgl_object: WebGLObject,
    id: WebGLFramebufferId,
    /// target can only be gl::FRAMEBUFFER at the moment
    target: Cell<Option<u32>>,
    is_deleted: Cell<bool>,
    #[ignore_heap_size_of = "Defined in ipc-channel"]
    renderer: IpcSender<CanvasMsg>,
}

impl WebGLFramebuffer {
    fn new_inherited(renderer: IpcSender<CanvasMsg>,
                     id: WebGLFramebufferId)
                     -> WebGLFramebuffer {
        WebGLFramebuffer {
            webgl_object: WebGLObject::new_inherited(),
            id: id,
            target: Cell::new(None),
            is_deleted: Cell::new(false),
            renderer: renderer,
        }
    }

    pub fn maybe_new(global: &GlobalScope, renderer: IpcSender<CanvasMsg>)
                     -> Option<Root<WebGLFramebuffer>> {
        let (sender, receiver) = ipc::channel().unwrap();
        renderer.send(CanvasMsg::WebGL(WebGLCommand::CreateFramebuffer(sender))).unwrap();

        let result = receiver.recv().unwrap();
        result.map(|fb_id| WebGLFramebuffer::new(global, renderer, fb_id))
    }

    pub fn new(global: &GlobalScope,
               renderer: IpcSender<CanvasMsg>,
               id: WebGLFramebufferId)
               -> Root<WebGLFramebuffer> {
        reflect_dom_object(box WebGLFramebuffer::new_inherited(renderer, id),
                           global,
                           WebGLFramebufferBinding::Wrap)
    }
}


impl WebGLFramebuffer {
    pub fn id(&self) -> WebGLFramebufferId {
        self.id
    }

    pub fn bind(&self, target: u32) {
        self.target.set(Some(target));
        let cmd = WebGLCommand::BindFramebuffer(target, WebGLFramebufferBindingRequest::Explicit(self.id));
        self.renderer.send(CanvasMsg::WebGL(cmd)).unwrap();
    }

    pub fn delete(&self) {
        if !self.is_deleted.get() {
            self.is_deleted.set(true);
            let _ = self.renderer.send(CanvasMsg::WebGL(WebGLCommand::DeleteFramebuffer(self.id)));
        }
    }

    pub fn is_deleted(&self) -> bool {
        self.is_deleted.get()
    }

    pub fn check_status(&self) -> u32 {
        // Until we build support for attaching renderbuffers or
        // textures, all user FBOs are incomplete.
        return constants::FRAMEBUFFER_UNSUPPORTED;
    }

    pub fn target(&self) -> Option<u32> {
        self.target.get()
    }
}

impl Drop for WebGLFramebuffer {
    fn drop(&mut self) {
        self.delete();
    }
}
