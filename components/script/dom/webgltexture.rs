/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://www.khronos.org/registry/webgl/specs/latest/1.0/webgl.idl
use dom::bindings::codegen::Bindings::WebGLRenderingContextBinding::WebGLRenderingContextConstants as constants;
use dom::bindings::codegen::Bindings::WebGLTextureBinding;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::Root;
use dom::bindings::utils::reflect_dom_object;
use dom::webglobject::WebGLObject;

use canvas_traits::{CanvasMsg, CanvasWebGLMsg, WebGLError, WebGLResult};
use ipc_channel::ipc::{self, IpcSender};
use std::cell::Cell;

pub enum TexParameterValue {
    Float(f32),
    Int(i32),
}

#[dom_struct]
#[derive(HeapSizeOf)]
pub struct WebGLTexture {
    webgl_object: WebGLObject,
    id: u32,
    /// The target to which this texture was bound the first time
    target: Cell<Option<u32>>,
    is_deleted: Cell<bool>,
    #[ignore_heap_size_of = "Defined in ipc-channel"]
    renderer: IpcSender<CanvasMsg>,
}

impl WebGLTexture {
    fn new_inherited(renderer: IpcSender<CanvasMsg>, id: u32) -> WebGLTexture {
        WebGLTexture {
            webgl_object: WebGLObject::new_inherited(),
            id: id,
            target: Cell::new(None),
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
    fn bind(self, target: u32) -> WebGLResult<()>;
    fn delete(self);
    fn tex_parameter(self,
                     target: u32,
                     name: u32,
                     value: TexParameterValue) -> WebGLResult<()>;
}

impl<'a> WebGLTextureHelpers for &'a WebGLTexture {
    fn id(self) -> u32 {
        self.id
    }

    // NB: Only valid texture targets come here
    fn bind(self, target: u32) -> WebGLResult<()> {
        if let Some(previous_target) = self.target.get() {
            if target != previous_target {
                return Err(WebGLError::InvalidOperation);
            }
        } else {
            self.target.set(Some(target));
        }

        self.renderer.send(CanvasMsg::WebGL(CanvasWebGLMsg::BindTexture(self.id, target))).unwrap();

        Ok(())
    }

    fn delete(self) {
        if !self.is_deleted.get() {
            self.is_deleted.set(true);
            self.renderer.send(CanvasMsg::WebGL(CanvasWebGLMsg::DeleteTexture(self.id))).unwrap();
        }
    }

    /// We have to follow the conversion rules for GLES 2.0. See:
    ///   https://www.khronos.org/webgl/public-mailing-list/archives/1008/msg00014.html
    ///
    fn tex_parameter(self,
                     target: u32,
                     name: u32,
                     value: TexParameterValue) -> WebGLResult<()> {
        let (int_value, _float_value) = match value {
            TexParameterValue::Int(int_value) => (int_value, int_value as f32),
            TexParameterValue::Float(float_value) => (float_value as i32, float_value),
        };

        match name {
            constants::TEXTURE_MIN_FILTER => {
                match int_value as u32 {
                    constants::NEAREST |
                    constants::LINEAR |
                    constants::NEAREST_MIPMAP_NEAREST |
                    constants::LINEAR_MIPMAP_NEAREST |
                    constants::NEAREST_MIPMAP_LINEAR |
                    constants::LINEAR_MIPMAP_LINEAR => {
                        self.renderer
                            .send(CanvasMsg::WebGL(CanvasWebGLMsg::TexParameteri(target, name, int_value)))
                            .unwrap();
                        return Ok(());
                    },

                    _ => return Err(WebGLError::InvalidEnum),
                }
            },
            constants::TEXTURE_MAG_FILTER => {
                match int_value as u32 {
                    constants::NEAREST |
                    constants::LINEAR => {
                        self.renderer
                            .send(CanvasMsg::WebGL(CanvasWebGLMsg::TexParameteri(target, name, int_value)))
                            .unwrap();
                        return Ok(());
                    },

                    _ => return Err(WebGLError::InvalidEnum),
                }
            },
            constants::TEXTURE_WRAP_S |
            constants::TEXTURE_WRAP_T => {
                match int_value as u32 {
                    constants::CLAMP_TO_EDGE |
                    constants::MIRRORED_REPEAT |
                    constants::REPEAT => {
                        self.renderer
                            .send(CanvasMsg::WebGL(CanvasWebGLMsg::TexParameteri(target, name, int_value)))
                            .unwrap();
                        return Ok(());
                    },

                    _ => return Err(WebGLError::InvalidEnum),
                }
            },

            _ => return Err(WebGLError::InvalidEnum),
        }
    }
}
