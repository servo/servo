/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://www.khronos.org/registry/webgl/specs/latest/1.0/webgl.idl
use angle::hl::{BuiltInResources, Output, ShaderValidator};
use canvas_traits::CanvasMsg;
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::WebGLShaderBinding;
use dom::bindings::js::Root;
use dom::bindings::reflector::reflect_dom_object;
use dom::bindings::str::DOMString;
use dom::webglobject::WebGLObject;
use dom::window::Window;
use dom_struct::dom_struct;
use ipc_channel::ipc::IpcSender;
use std::cell::Cell;
use std::sync::{ONCE_INIT, Once};
use webrender_traits;
use webrender_traits::{WebGLCommand, WebGLParameter, WebGLResult, WebGLShaderId};

#[derive(Clone, Copy, PartialEq, Debug, JSTraceable, HeapSizeOf)]
pub enum ShaderCompilationStatus {
    NotCompiled,
    Succeeded,
    Failed,
}

#[dom_struct]
pub struct WebGLShader {
    webgl_object: WebGLObject,
    id: WebGLShaderId,
    gl_type: u32,
    source: DOMRefCell<Option<DOMString>>,
    info_log: DOMRefCell<Option<String>>,
    is_deleted: Cell<bool>,
    attached_counter: Cell<u32>,
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
    fn new_inherited(renderer: IpcSender<CanvasMsg>,
                     id: WebGLShaderId,
                     shader_type: u32)
                     -> WebGLShader {
        GLSLANG_INITIALIZATION.call_once(|| ::angle::hl::initialize().unwrap());
        WebGLShader {
            webgl_object: WebGLObject::new_inherited(),
            id: id,
            gl_type: shader_type,
            source: DOMRefCell::new(None),
            info_log: DOMRefCell::new(None),
            is_deleted: Cell::new(false),
            attached_counter: Cell::new(0),
            compilation_status: Cell::new(ShaderCompilationStatus::NotCompiled),
            renderer: renderer,
        }
    }

    pub fn maybe_new(window: &Window,
                     renderer: IpcSender<CanvasMsg>,
                     shader_type: u32)
                     -> Option<Root<WebGLShader>> {
        let (sender, receiver) = webrender_traits::channel::msg_channel().unwrap();
        renderer.send(CanvasMsg::WebGL(WebGLCommand::CreateShader(shader_type, sender))).unwrap();

        let result = receiver.recv().unwrap();
        result.map(|shader_id| WebGLShader::new(window, renderer, shader_id, shader_type))
    }

    pub fn new(window: &Window,
               renderer: IpcSender<CanvasMsg>,
               id: WebGLShaderId,
               shader_type: u32)
               -> Root<WebGLShader> {
        reflect_dom_object(box WebGLShader::new_inherited(renderer, id, shader_type),
                           window,
                           WebGLShaderBinding::Wrap)
    }
}


impl WebGLShader {
    pub fn id(&self) -> WebGLShaderId {
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
            let mut params = BuiltInResources::default();
            params.FragmentPrecisionHigh = 1;
            let validator = ShaderValidator::for_webgl(self.gl_type,
                                                       SHADER_OUTPUT_FORMAT,
                                                       &params).unwrap();
            match validator.compile_and_translate(&[source]) {
                Ok(translated_source) => {
                    debug!("Shader translated: {}", translated_source);
                    // NOTE: At this point we should be pretty sure that the compilation in the paint thread
                    // will succeed.
                    // It could be interesting to retrieve the info log from the paint thread though
                    let msg = WebGLCommand::CompileShader(self.id, translated_source);
                    self.renderer.send(CanvasMsg::WebGL(msg)).unwrap();
                    self.compilation_status.set(ShaderCompilationStatus::Succeeded);
                },
                Err(error) => {
                    self.compilation_status.set(ShaderCompilationStatus::Failed);
                    debug!("Shader {} compilation failed: {}", self.id, error);
                },
            }

            *self.info_log.borrow_mut() = Some(validator.info_log());
            // TODO(emilio): More data (like uniform data) should be collected
            // here to properly validate uniforms.
            //
            // This requires a more complex interface with ANGLE, using C++
            // bindings and being extremely cautious about destructing things.
        }
    }

    /// Mark this shader as deleted (if it wasn't previously)
    /// and delete it as if calling glDeleteShader.
    /// Currently does not check if shader is attached
    pub fn delete(&self) {
        if !self.is_deleted.get() {
            self.is_deleted.set(true);
            let _ = self.renderer.send(CanvasMsg::WebGL(WebGLCommand::DeleteShader(self.id)));
        }
    }

    pub fn is_deleted(&self) -> bool {
        self.is_deleted.get()
    }

    pub fn is_attached(&self) -> bool {
        self.attached_counter.get() > 0
    }

    pub fn increment_attached_counter(&self) {
        self.attached_counter.set(self.attached_counter.get() + 1);
    }

    pub fn decrement_attached_counter(&self) {
        assert!(self.attached_counter.get() > 0);
        self.attached_counter.set(self.attached_counter.get() - 1);
    }

    /// glGetShaderInfoLog
    pub fn info_log(&self) -> Option<String> {
        self.info_log.borrow().clone()
    }

    /// glGetParameter
    pub fn parameter(&self, param_id: u32) -> WebGLResult<WebGLParameter> {
        let (sender, receiver) = webrender_traits::channel::msg_channel().unwrap();
        self.renderer.send(CanvasMsg::WebGL(WebGLCommand::GetShaderParameter(self.id, param_id, sender))).unwrap();
        receiver.recv().unwrap()
    }

    /// Get the shader source
    pub fn source(&self) -> Option<DOMString> {
        self.source.borrow().clone()
    }

    /// glShaderSource
    pub fn set_source(&self, source: DOMString) {
        *self.source.borrow_mut() = Some(source);
    }

    pub fn successfully_compiled(&self) -> bool {
        self.compilation_status.get() == ShaderCompilationStatus::Succeeded
    }
}

impl Drop for WebGLShader {
    fn drop(&mut self) {
        assert!(self.attached_counter.get() == 0);
        self.delete();
    }
}
