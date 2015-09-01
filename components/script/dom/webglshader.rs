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

use angle::hl::{BuiltInResources, Output, ShaderValidator};
use canvas_traits::{CanvasMsg, CanvasWebGLMsg, WebGLResult, WebGLError, WebGLShaderParameter};
use ipc_channel::ipc::{self, IpcSender};
use std::cell::Cell;
use std::cell::RefCell;
use std::sync::{Once, ONCE_INIT};

#[derive(Clone, Copy, PartialEq, Debug, JSTraceable, HeapSizeOf)]
pub enum ShaderCompilationStatus {
    NotCompiled,
    Succeeded,
    Failed,
}

#[dom_struct]
pub struct WebGLShader {
    webgl_object: WebGLObject,
    id: u32,
    gl_type: u32,
    source: RefCell<Option<String>>,
    info_log: RefCell<Option<String>>,
    is_deleted: Cell<bool>,
    compilation_status: Cell<ShaderCompilationStatus>,
    #[ignore_heap_size_of = "Defined in ipc-channel"]
    renderer: IpcSender<CanvasMsg>,
}

#[cfg(not(target_os = "android"))]
const SHADER_OUTPUT_FORMAT: Output = Output::Glsl;

#[cfg(target_os = "android")]
const SHADER_OUTPUT_FORMAT: Output = Output::Essl;

static GLSLANG_INITIALIZATION: Once = ONCE_INIT;

impl WebGLShader {
    fn new_inherited(renderer: IpcSender<CanvasMsg>, id: u32, shader_type: u32) -> WebGLShader {
        GLSLANG_INITIALIZATION.call_once(|| ::angle::hl::initialize().unwrap());
        WebGLShader {
            webgl_object: WebGLObject::new_inherited(),
            id: id,
            gl_type: shader_type,
            source: RefCell::new(None),
            info_log: RefCell::new(None),
            is_deleted: Cell::new(false),
            compilation_status: Cell::new(ShaderCompilationStatus::NotCompiled),
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


impl WebGLShader {
    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn gl_type(&self) -> u32 {
        self.gl_type
    }

    /// glCompileShader
    pub fn compile(&self) {
        if self.compilation_status.get() != ShaderCompilationStatus::NotCompiled {
            debug!("Compiling already compiled shader {}", self.id);
        }

        if let Some(ref source) = *self.source.borrow() {
            let validator = ShaderValidator::for_webgl(self.gl_type,
                                                       SHADER_OUTPUT_FORMAT,
                                                       &BuiltInResources::default()).unwrap();
            match validator.compile_and_translate(&[source.as_bytes()]) {
                Ok(translated_source) => {
                    // NOTE: At this point we should be pretty sure that the compilation in the paint task
                    // will succeed.
                    // It could be interesting to retrieve the info log from the paint task though
                    let msg = CanvasWebGLMsg::CompileShader(self.id, translated_source);
                    self.renderer.send(CanvasMsg::WebGL(msg)).unwrap();
                    self.compilation_status.set(ShaderCompilationStatus::Succeeded);
                },
                Err(error) => {
                    self.compilation_status.set(ShaderCompilationStatus::Failed);
                    debug!("Shader {} compilation failed: {}", self.id, error);
                },
            }

            *self.info_log.borrow_mut() = Some(validator.info_log());
        }
    }

    /// Mark this shader as deleted (if it wasn't previously)
    /// and delete it as if calling glDeleteShader.
    pub fn delete(&self) {
        if !self.is_deleted.get() {
            self.is_deleted.set(true);
            self.renderer.send(CanvasMsg::WebGL(CanvasWebGLMsg::DeleteShader(self.id))).unwrap()
        }
    }

    /// glGetShaderInfoLog
    pub fn info_log(&self) -> Option<String> {
        self.info_log.borrow().clone()
    }

    /// glGetShaderParameter
    pub fn parameter(&self, param_id: u32) -> WebGLResult<WebGLShaderParameter> {
        match param_id {
            constants::SHADER_TYPE | constants::DELETE_STATUS | constants::COMPILE_STATUS => {},
            _ => return Err(WebGLError::InvalidEnum),
        }

        let (sender, receiver) = ipc::channel().unwrap();
        self.renderer.send(CanvasMsg::WebGL(CanvasWebGLMsg::GetShaderParameter(self.id, param_id, sender))).unwrap();
        Ok(receiver.recv().unwrap())
    }

    /// Get the shader source
    pub fn source(&self) -> Option<String> {
        self.source.borrow().clone()
    }

    /// glShaderSource
    pub fn set_source(&self, source: String) {
        *self.source.borrow_mut() = Some(source);
    }
}
