/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://www.khronos.org/registry/webgl/specs/latest/1.0/webgl.idl
use dom::bindings::codegen::Bindings::WebGLShaderBinding;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::Root;
use dom::bindings::utils::reflect_dom_object;
use dom::webglobject::WebGLObject;

use dom::bindings::codegen::Bindings::WebGLRenderingContextBinding::WebGLRenderingContextConstants as constants;

use canvas_traits::{CanvasMsg, CanvasWebGLMsg, WebGLResult, WebGLError, WebGLShaderParameter};
use ipc_channel::ipc::{self, IpcSender};
use std::cell::Cell;
use std::cell::RefCell;

#[dom_struct]
pub struct WebGLShader {
    webgl_object: WebGLObject,
    id: u32,
    gl_type: u32,
    source: RefCell<Option<String>>,
    is_deleted: Cell<bool>,
    // TODO(ecoal95): Evaluate moving this to `WebGLObject`
    #[ignore_heap_size_of = "Defined in ipc-channel"]
    renderer: IpcSender<CanvasMsg>,
}

impl WebGLShader {
    fn new_inherited(renderer: IpcSender<CanvasMsg>, id: u32, shader_type: u32) -> WebGLShader {
        WebGLShader {
            webgl_object: WebGLObject::new_inherited(),
            id: id,
            gl_type: shader_type,
            source: RefCell::new(None),
            is_deleted: Cell::new(false),
            renderer: renderer,
        }
    }

    pub fn maybe_new(global: GlobalRef,
                     renderer: IpcSender<CanvasMsg>,
                     shader_type: u32) -> Option<Root<WebGLShader>> {
        let (sender, receiver) = ipc::channel().unwrap();
        renderer.send(CanvasMsg::WebGL(CanvasWebGLMsg::CreateShader(shader_type, sender))).unwrap();

        let result = receiver.recv().unwrap();
        result.map(|shader_id| WebGLShader::new(global, renderer, *shader_id, shader_type))
    }

    pub fn new(global: GlobalRef,
               renderer: IpcSender<CanvasMsg>,
               id: u32,
               shader_type: u32) -> Root<WebGLShader> {
        reflect_dom_object(
            box WebGLShader::new_inherited(renderer, id, shader_type), global, WebGLShaderBinding::Wrap)
    }
}

pub trait WebGLShaderHelpers {
    fn id(self) -> u32;
    fn gl_type(self) -> u32;
    fn compile(self);
    fn delete(self);
    fn info_log(self) -> Option<String>;
    fn parameter(self, param_id: u32) -> WebGLResult<WebGLShaderParameter>;
    fn source(self) -> Option<String>;
    fn set_source(self, src: String);
}

impl<'a> WebGLShaderHelpers for &'a WebGLShader {
    fn id(self) -> u32 {
        self.id
    }

    fn gl_type(self) -> u32 {
        self.gl_type
    }

    // TODO(ecoal95): Validate shaders to be conforming to the WebGL spec
    /// glCompileShader
    fn compile(self) {
        self.renderer.send(CanvasMsg::WebGL(CanvasWebGLMsg::CompileShader(self.id))).unwrap()
    }

    /// Mark this shader as deleted (if it wasn't previously)
    /// and delete it as if calling glDeleteShader.
    fn delete(self) {
        if !self.is_deleted.get() {
            self.is_deleted.set(true);
            self.renderer.send(CanvasMsg::WebGL(CanvasWebGLMsg::DeleteShader(self.id))).unwrap()
        }
    }

    /// glGetShaderInfoLog
    fn info_log(self) -> Option<String> {
        let (sender, receiver) = ipc::channel().unwrap();
        self.renderer.send(CanvasMsg::WebGL(CanvasWebGLMsg::GetShaderInfoLog(self.id, sender))).unwrap();
        receiver.recv().unwrap()
    }

    /// glGetShaderParameter
    fn parameter(self, param_id: u32) -> WebGLResult<WebGLShaderParameter> {
        match param_id {
            constants::SHADER_TYPE | constants::DELETE_STATUS | constants::COMPILE_STATUS => {},
            _ => return Err(WebGLError::InvalidEnum),
        }

        let (sender, receiver) = ipc::channel().unwrap();
        self.renderer.send(CanvasMsg::WebGL(CanvasWebGLMsg::GetShaderParameter(self.id, param_id, sender))).unwrap();
        Ok(receiver.recv().unwrap())
    }

    /// Get the shader source
    fn source(self) -> Option<String> {
        self.source.borrow().clone()
    }

    /// glShaderSource
    fn set_source(self, source: String) {
        *self.source.borrow_mut() = Some(source.clone());
        self.renderer.send(CanvasMsg::WebGL(CanvasWebGLMsg::ShaderSource(self.id, source))).unwrap()
    }
}
