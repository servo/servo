/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://www.khronos.org/registry/webgl/specs/latest/1.0/webgl.idl
use canvas_traits::CanvasMsg;
use dom::bindings::codegen::Bindings::WebGLRenderbufferBinding;
use dom::bindings::codegen::Bindings::WebGLRenderingContextBinding::WebGLRenderingContextConstants as constants;
use dom::bindings::js::Root;
use dom::bindings::reflector::reflect_dom_object;
use dom::webglobject::WebGLObject;
use dom::window::Window;
use dom_struct::dom_struct;
use ipc_channel::ipc::IpcSender;
use std::cell::Cell;
use webrender_api;
use webrender_api::{WebGLCommand, WebGLRenderbufferId, WebGLResult, WebGLError};

#[dom_struct]
pub struct WebGLRenderbuffer {
    webgl_object: WebGLObject,
    id: WebGLRenderbufferId,
    ever_bound: Cell<bool>,
    is_deleted: Cell<bool>,
    size: Cell<Option<(i32, i32)>>,
    internal_format: Cell<Option<u32>>,
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
            internal_format: Cell::new(None),
            size: Cell::new(None),
        }
    }

    pub fn maybe_new(window: &Window, renderer: IpcSender<CanvasMsg>)
                     -> Option<Root<WebGLRenderbuffer>> {
        let (sender, receiver) = webrender_api::channel::msg_channel().unwrap();
        renderer.send(CanvasMsg::WebGL(WebGLCommand::CreateRenderbuffer(sender))).unwrap();

        let result = receiver.recv().unwrap();
        result.map(|renderbuffer_id| WebGLRenderbuffer::new(window, renderer, renderbuffer_id))
    }

    pub fn new(window: &Window,
               renderer: IpcSender<CanvasMsg>,
               id: WebGLRenderbufferId)
               -> Root<WebGLRenderbuffer> {
        reflect_dom_object(box WebGLRenderbuffer::new_inherited(renderer, id),
                           window,
                           WebGLRenderbufferBinding::Wrap)
    }
}


impl WebGLRenderbuffer {
    pub fn id(&self) -> WebGLRenderbufferId {
        self.id
    }

    pub fn size(&self) -> Option<(i32, i32)> {
        self.size.get()
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

    pub fn storage(&self, internal_format: u32, width: i32, height: i32) -> WebGLResult<()> {
        // Validate the internal_format, and save it for completeness
        // validation.
        match internal_format {
            constants::RGBA4 |
            constants::DEPTH_STENCIL |
            constants::DEPTH_COMPONENT16 |
            constants::STENCIL_INDEX8 =>
                self.internal_format.set(Some(internal_format)),

            _ => return Err(WebGLError::InvalidEnum),
        };

        // FIXME: Check that w/h are < MAX_RENDERBUFFER_SIZE

        // FIXME: Invalidate completeness after the call

        let msg = CanvasMsg::WebGL(WebGLCommand::RenderbufferStorage(constants::RENDERBUFFER,
                                                                     internal_format, width, height));
        self.renderer.send(msg).unwrap();

        self.size.set(Some((width, height)));

        Ok(())
    }
}
