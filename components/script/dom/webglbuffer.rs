/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://www.khronos.org/registry/webgl/specs/latest/1.0/webgl.idl
use canvas_traits::CanvasMsg;
use dom::bindings::codegen::Bindings::WebGLBufferBinding;
use dom::bindings::js::Root;
use dom::bindings::reflector::reflect_dom_object;
use dom::globalscope::GlobalScope;
use dom::webglobject::WebGLObject;
use ipc_channel::ipc::{self, IpcSender};
use std::cell::Cell;
use webrender_traits::{WebGLBufferId, WebGLCommand, WebGLError, WebGLResult};

#[dom_struct]
pub struct WebGLBuffer {
    webgl_object: WebGLObject,
    id: WebGLBufferId,
    /// The target to which this buffer was bound the first time
    target: Cell<Option<u32>>,
    capacity: Cell<usize>,
    is_deleted: Cell<bool>,
    #[ignore_heap_size_of = "Defined in ipc-channel"]
    renderer: IpcSender<CanvasMsg>,
}

impl WebGLBuffer {
    fn new_inherited(renderer: IpcSender<CanvasMsg>,
                     id: WebGLBufferId)
                     -> WebGLBuffer {
        WebGLBuffer {
            webgl_object: WebGLObject::new_inherited(),
            id: id,
            target: Cell::new(None),
            capacity: Cell::new(0),
            is_deleted: Cell::new(false),
            renderer: renderer,
        }
    }

    pub fn maybe_new(global: &GlobalScope, renderer: IpcSender<CanvasMsg>)
                     -> Option<Root<WebGLBuffer>> {
        let (sender, receiver) = ipc::channel().unwrap();
        renderer.send(CanvasMsg::WebGL(WebGLCommand::CreateBuffer(sender))).unwrap();

        let result = receiver.recv().unwrap();
        result.map(|buffer_id| WebGLBuffer::new(global, renderer, buffer_id))
    }

    pub fn new(global: &GlobalScope,
               renderer: IpcSender<CanvasMsg>,
               id: WebGLBufferId)
              -> Root<WebGLBuffer> {
        reflect_dom_object(box WebGLBuffer::new_inherited(renderer, id),
                           global, WebGLBufferBinding::Wrap)
    }
}


impl WebGLBuffer {
    pub fn id(&self) -> WebGLBufferId {
        self.id
    }

    // NB: Only valid buffer targets come here
    pub fn bind(&self, target: u32) -> WebGLResult<()> {
        if let Some(previous_target) = self.target.get() {
            if target != previous_target {
                return Err(WebGLError::InvalidOperation);
            }
        } else {
            self.target.set(Some(target));
        }
        let msg = CanvasMsg::WebGL(WebGLCommand::BindBuffer(target, Some(self.id)));
        self.renderer.send(msg).unwrap();

        Ok(())
    }

    pub fn buffer_data(&self, target: u32, data: &[u8], usage: u32) -> WebGLResult<()> {
        if let Some(previous_target) = self.target.get() {
            if target != previous_target {
                return Err(WebGLError::InvalidOperation);
            }
        }
        self.capacity.set(data.len());
        self.renderer
            .send(CanvasMsg::WebGL(WebGLCommand::BufferData(target, data.to_vec(), usage)))
            .unwrap();

        Ok(())
    }

    pub fn capacity(&self) -> usize {
        self.capacity.get()
    }

    pub fn delete(&self) {
        if !self.is_deleted.get() {
            self.is_deleted.set(true);
            let _ = self.renderer.send(CanvasMsg::WebGL(WebGLCommand::DeleteBuffer(self.id)));
        }
    }

    pub fn is_deleted(&self) -> bool {
        self.is_deleted.get()
    }

    pub fn target(&self) -> Option<u32> {
        self.target.get()
    }
}

impl Drop for WebGLBuffer {
    fn drop(&mut self) {
        self.delete();
    }
}
