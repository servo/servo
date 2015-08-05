/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://www.khronos.org/registry/webgl/specs/latest/1.0/webgl.idl
use dom::bindings::codegen::Bindings::WebGLTextureBinding;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::Root;
use dom::bindings::utils::reflect_dom_object;
use dom::webglobject::WebGLObject;

use canvas_traits::{CanvasMsg, CanvasWebGLMsg};
use ipc_channel::ipc::{self, IpcSender};
use std::cell::Cell;

#[dom_struct]
#[derive(HeapSizeOf)]
pub struct WebGLTexture {
    webgl_object: WebGLObject,
    id: u32,
    is_deleted: Cell<bool>,
    #[ignore_heap_size_of = "Defined in ipc-channel"]
    renderer: IpcSender<CanvasMsg>,
}

impl WebGLTexture {
    fn new_inherited(renderer: IpcSender<CanvasMsg>, id: u32) -> WebGLTexture {
        WebGLTexture {
            webgl_object: WebGLObject::new_inherited(),
            id: id,
            is_deleted: Cell::new(false),
            renderer: renderer,
        }
    }

    pub fn maybe_new(global: GlobalRef, renderer: IpcSender<CanvasMsg>)
                     -> Option<Root<WebGLTexture>> {
        let (sender, receiver) = ipc::channel().unwrap();
        renderer.send(CanvasMsg::WebGL(CanvasWebGLMsg::CreateTexture(sender))).unwrap();

        let result = receiver.recv().unwrap();
        result.map(|texture_id| WebGLTexture::new(global, renderer, *texture_id))
    }

    pub fn new(global: GlobalRef, renderer: IpcSender<CanvasMsg>, id: u32) -> Root<WebGLTexture> {
        reflect_dom_object(box WebGLTexture::new_inherited(renderer, id), global, WebGLTextureBinding::Wrap)
    }
}

pub trait WebGLTextureHelpers {
    fn id(self) -> u32;
    fn bind(self, target: u32);
    fn delete(self);
}

impl<'a> WebGLTextureHelpers for &'a WebGLTexture {
    fn id(self) -> u32 {
        self.id
    }

    fn bind(self, target: u32) {
        self.renderer.send(CanvasMsg::WebGL(CanvasWebGLMsg::BindTexture(self.id, target))).unwrap();
    }

    fn delete(self) {
        if !self.is_deleted.get() {
            self.is_deleted.set(true);
            self.renderer.send(CanvasMsg::WebGL(CanvasWebGLMsg::DeleteTexture(self.id))).unwrap();
        }
    }
}
