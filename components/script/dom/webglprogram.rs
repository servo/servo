/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://www.khronos.org/registry/webgl/specs/latest/1.0/webgl.idl
use canvas_traits::CanvasMsg;
use dom::bindings::codegen::Bindings::WebGLProgramBinding;
use dom::bindings::codegen::Bindings::WebGLRenderingContextBinding::WebGLRenderingContextConstants as constants;
use dom::bindings::js::{MutNullableJS, Root};
use dom::bindings::reflector::{DomObject, reflect_dom_object};
use dom::bindings::str::DOMString;
use dom::webglactiveinfo::WebGLActiveInfo;
use dom::webglobject::WebGLObject;
use dom::webglrenderingcontext::MAX_UNIFORM_AND_ATTRIBUTE_LEN;
use dom::webglshader::WebGLShader;
use dom::window::Window;
use dom_struct::dom_struct;
use ipc_channel::ipc::IpcSender;
use std::cell::Cell;
use webrender_api;
use webrender_api::{WebGLCommand, WebGLError, WebGLParameter, WebGLProgramId, WebGLResult};

#[dom_struct]
pub struct WebGLProgram {
    webgl_object: WebGLObject,
    id: WebGLProgramId,
    is_deleted: Cell<bool>,
    link_called: Cell<bool>,
    linked: Cell<bool>,
    fragment_shader: MutNullableJS<WebGLShader>,
    vertex_shader: MutNullableJS<WebGLShader>,
    #[ignore_heap_size_of = "Defined in ipc-channel"]
    renderer: IpcSender<CanvasMsg>,
}

impl WebGLProgram {
    fn new_inherited(renderer: IpcSender<CanvasMsg>,
                     id: WebGLProgramId)
                     -> WebGLProgram {
        WebGLProgram {
            webgl_object: WebGLObject::new_inherited(),
            id: id,
            is_deleted: Cell::new(false),
            link_called: Cell::new(false),
            linked: Cell::new(false),
            fragment_shader: Default::default(),
            vertex_shader: Default::default(),
            renderer: renderer,
        }
    }

    pub fn maybe_new(window: &Window, renderer: IpcSender<CanvasMsg>)
                     -> Option<Root<WebGLProgram>> {
        let (sender, receiver) = webrender_api::channel::msg_channel().unwrap();
        renderer.send(CanvasMsg::WebGL(WebGLCommand::CreateProgram(sender))).unwrap();

        let result = receiver.recv().unwrap();
        result.map(|program_id| WebGLProgram::new(window, renderer, program_id))
    }

    pub fn new(window: &Window,
               renderer: IpcSender<CanvasMsg>,
               id: WebGLProgramId)
               -> Root<WebGLProgram> {
        reflect_dom_object(box WebGLProgram::new_inherited(renderer, id),
                           window,
                           WebGLProgramBinding::Wrap)
    }
}


impl WebGLProgram {
    pub fn id(&self) -> WebGLProgramId {
        self.id
    }

    /// glDeleteProgram
    pub fn delete(&self) {
        if !self.is_deleted.get() {
            self.is_deleted.set(true);
            let _ = self.renderer.send(CanvasMsg::WebGL(WebGLCommand::DeleteProgram(self.id)));

            if let Some(shader) = self.fragment_shader.get() {
                shader.decrement_attached_counter();
            }

            if let Some(shader) = self.vertex_shader.get() {
                shader.decrement_attached_counter();
            }
        }
    }

    pub fn is_deleted(&self) -> bool {
        self.is_deleted.get()
    }

    pub fn is_linked(&self) -> bool {
        self.linked.get()
    }

    /// glLinkProgram
    pub fn link(&self) -> WebGLResult<()>  {
        if self.is_deleted() {
            return Err(WebGLError::InvalidOperation);
        }
        self.linked.set(false);
        self.link_called.set(true);

        match self.fragment_shader.get() {
            Some(ref shader) if shader.successfully_compiled() => {},
            _ => return Ok(()), // callers use gl.LINK_STATUS to check link errors
        }

        match self.vertex_shader.get() {
            Some(ref shader) if shader.successfully_compiled() => {},
            _ => return Ok(()), // callers use gl.LINK_STATUS to check link errors
        }

        self.linked.set(true);
        self.renderer.send(CanvasMsg::WebGL(WebGLCommand::LinkProgram(self.id))).unwrap();
        Ok(())
    }

    /// glUseProgram
    pub fn use_program(&self) -> WebGLResult<()> {
        if self.is_deleted() {
            return Err(WebGLError::InvalidOperation);
        }
        if !self.linked.get() {
            return Err(WebGLError::InvalidOperation);
        }

        self.renderer.send(CanvasMsg::WebGL(WebGLCommand::UseProgram(self.id))).unwrap();
        Ok(())
    }

    /// glValidateProgram
    pub fn validate(&self) -> WebGLResult<()> {
        if self.is_deleted() {
            return Err(WebGLError::InvalidOperation);
        }
        self.renderer.send(CanvasMsg::WebGL(WebGLCommand::ValidateProgram(self.id))).unwrap();
        Ok(())
    }

    /// glAttachShader
    pub fn attach_shader(&self, shader: &WebGLShader) -> WebGLResult<()> {
        if self.is_deleted() || shader.is_deleted() {
            return Err(WebGLError::InvalidOperation);
        }
        let shader_slot = match shader.gl_type() {
            constants::FRAGMENT_SHADER => &self.fragment_shader,
            constants::VERTEX_SHADER => &self.vertex_shader,
            _ => {
                error!("detachShader: Unexpected shader type");
                return Err(WebGLError::InvalidValue);
            }
        };

        // TODO(emilio): Differentiate between same shader already assigned and other previous
        // shader.
        if shader_slot.get().is_some() {
            return Err(WebGLError::InvalidOperation);
        }

        shader_slot.set(Some(shader));
        shader.increment_attached_counter();

        self.renderer.send(CanvasMsg::WebGL(WebGLCommand::AttachShader(self.id, shader.id()))).unwrap();

        Ok(())
    }

    /// glDetachShader
    pub fn detach_shader(&self, shader: &WebGLShader) -> WebGLResult<()> {
        if self.is_deleted() {
            return Err(WebGLError::InvalidOperation);
        }
        let shader_slot = match shader.gl_type() {
            constants::FRAGMENT_SHADER => &self.fragment_shader,
            constants::VERTEX_SHADER => &self.vertex_shader,
            _ => {
                error!("detachShader: Unexpected shader type");
                return Err(WebGLError::InvalidValue);
            }
        };

        match shader_slot.get() {
            Some(ref attached_shader) if attached_shader.id() != shader.id() =>
                return Err(WebGLError::InvalidOperation),
            None =>
                return Err(WebGLError::InvalidOperation),
            _ => {}
        }

        shader_slot.set(None);
        shader.decrement_attached_counter();

        self.renderer.send(CanvasMsg::WebGL(WebGLCommand::DetachShader(self.id, shader.id()))).unwrap();

        Ok(())
    }

    /// glBindAttribLocation
    pub fn bind_attrib_location(&self, index: u32, name: DOMString) -> WebGLResult<()> {
        if self.is_deleted() {
            return Err(WebGLError::InvalidOperation);
        }
        if name.len() > MAX_UNIFORM_AND_ATTRIBUTE_LEN {
            return Err(WebGLError::InvalidValue);
        }

        // Check if the name is reserved
        if name.starts_with("gl_") || name.starts_with("webgl") || name.starts_with("_webgl_") {
            return Err(WebGLError::InvalidOperation);
        }

        self.renderer
            .send(CanvasMsg::WebGL(WebGLCommand::BindAttribLocation(self.id, index, String::from(name))))
            .unwrap();
        Ok(())
    }

    pub fn get_active_uniform(&self, index: u32) -> WebGLResult<Root<WebGLActiveInfo>> {
        if self.is_deleted() {
            return Err(WebGLError::InvalidValue);
        }
        let (sender, receiver) = webrender_api::channel::msg_channel().unwrap();
        self.renderer
            .send(CanvasMsg::WebGL(WebGLCommand::GetActiveUniform(self.id, index, sender)))
            .unwrap();

        receiver.recv().unwrap().map(|(size, ty, name)|
            WebGLActiveInfo::new(self.global().as_window(), size, ty, DOMString::from(name)))
    }

    /// glGetActiveAttrib
    pub fn get_active_attrib(&self, index: u32) -> WebGLResult<Root<WebGLActiveInfo>> {
        if self.is_deleted() {
            return Err(WebGLError::InvalidValue);
        }
        let (sender, receiver) = webrender_api::channel::msg_channel().unwrap();
        self.renderer
            .send(CanvasMsg::WebGL(WebGLCommand::GetActiveAttrib(self.id, index, sender)))
            .unwrap();

        receiver.recv().unwrap().map(|(size, ty, name)|
            WebGLActiveInfo::new(self.global().as_window(), size, ty, DOMString::from(name)))
    }

    /// glGetAttribLocation
    pub fn get_attrib_location(&self, name: DOMString) -> WebGLResult<Option<i32>> {
        if !self.is_linked() || self.is_deleted() {
            return Err(WebGLError::InvalidOperation);
        }
        if name.len() > MAX_UNIFORM_AND_ATTRIBUTE_LEN {
            return Err(WebGLError::InvalidValue);
        }

        // Check if the name is reserved
        if name.starts_with("gl_") {
            return Err(WebGLError::InvalidOperation);
        }

        if name.starts_with("webgl") || name.starts_with("_webgl_") {
            return Ok(None);
        }

        let (sender, receiver) = webrender_api::channel::msg_channel().unwrap();
        self.renderer
            .send(CanvasMsg::WebGL(WebGLCommand::GetAttribLocation(self.id, String::from(name), sender)))
            .unwrap();
        Ok(receiver.recv().unwrap())
    }

    /// glGetUniformLocation
    pub fn get_uniform_location(&self, name: DOMString) -> WebGLResult<Option<i32>> {
        if !self.is_linked() || self.is_deleted() {
            return Err(WebGLError::InvalidOperation);
        }
        if name.len() > MAX_UNIFORM_AND_ATTRIBUTE_LEN {
            return Err(WebGLError::InvalidValue);
        }

        // Check if the name is reserved
        if name.starts_with("webgl") || name.starts_with("_webgl_") {
            return Ok(None);
        }

        let (sender, receiver) = webrender_api::channel::msg_channel().unwrap();
        self.renderer
            .send(CanvasMsg::WebGL(WebGLCommand::GetUniformLocation(self.id, String::from(name), sender)))
            .unwrap();
        Ok(receiver.recv().unwrap())
    }

    /// glGetProgramInfoLog
    pub fn get_info_log(&self) -> WebGLResult<String> {
        if self.is_deleted() {
            return Err(WebGLError::InvalidOperation);
        }
        if self.link_called.get() {
            let shaders_compiled = match (self.fragment_shader.get(), self.vertex_shader.get()) {
                (Some(fs), Some(vs)) => fs.successfully_compiled() && vs.successfully_compiled(),
                _ => false
            };
            if !shaders_compiled {
                return Ok("One or more shaders failed to compile".to_string());
            }
        }
        let (sender, receiver) = webrender_api::channel::msg_channel().unwrap();
        self.renderer.send(CanvasMsg::WebGL(WebGLCommand::GetProgramInfoLog(self.id, sender))).unwrap();
        Ok(receiver.recv().unwrap())
    }

    /// glGetProgramParameter
    pub fn parameter(&self, param_id: u32) -> WebGLResult<WebGLParameter> {
        let (sender, receiver) = webrender_api::channel::msg_channel().unwrap();
        self.renderer.send(CanvasMsg::WebGL(WebGLCommand::GetProgramParameter(self.id, param_id, sender))).unwrap();
        receiver.recv().unwrap()
    }
}

impl Drop for WebGLProgram {
    fn drop(&mut self) {
        self.delete();
    }
}
