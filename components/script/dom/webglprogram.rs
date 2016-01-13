/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://www.khronos.org/registry/webgl/specs/latest/1.0/webgl.idl
use canvas_traits::{CanvasMsg, CanvasWebGLMsg, WebGLError, WebGLResult, WebGLParameter};
use dom::bindings::codegen::Bindings::WebGLProgramBinding;
use dom::bindings::codegen::Bindings::WebGLRenderingContextBinding::WebGLRenderingContextConstants as constants;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JS, MutNullableHeap, Root};
use dom::bindings::reflector::reflect_dom_object;
use dom::webglobject::WebGLObject;
use dom::webglrenderingcontext::MAX_UNIFORM_AND_ATTRIBUTE_LEN;
use dom::webglshader::WebGLShader;
use ipc_channel::ipc::{self, IpcSender};
use std::cell::Cell;
use util::str::DOMString;

#[dom_struct]
pub struct WebGLProgram {
    webgl_object: WebGLObject,
    id: u32,
    is_deleted: Cell<bool>,
    fragment_shader: MutNullableHeap<JS<WebGLShader>>,
    vertex_shader: MutNullableHeap<JS<WebGLShader>>,
    #[ignore_heap_size_of = "Defined in ipc-channel"]
    renderer: IpcSender<CanvasMsg>,
}

impl WebGLProgram {
    fn new_inherited(renderer: IpcSender<CanvasMsg>, id: u32) -> WebGLProgram {
        WebGLProgram {
            webgl_object: WebGLObject::new_inherited(),
            id: id,
            is_deleted: Cell::new(false),
            fragment_shader: Default::default(),
            vertex_shader: Default::default(),
            renderer: renderer,
        }
    }

    pub fn maybe_new(global: GlobalRef, renderer: IpcSender<CanvasMsg>)
                     -> Option<Root<WebGLProgram>> {
        let (sender, receiver) = ipc::channel().unwrap();
        renderer.send(CanvasMsg::WebGL(CanvasWebGLMsg::CreateProgram(sender))).unwrap();

        let result = receiver.recv().unwrap();
        result.map(|program_id| WebGLProgram::new(global, renderer, *program_id))
    }

    pub fn new(global: GlobalRef, renderer: IpcSender<CanvasMsg>, id: u32) -> Root<WebGLProgram> {
        reflect_dom_object(box WebGLProgram::new_inherited(renderer, id), global, WebGLProgramBinding::Wrap)
    }
}


impl WebGLProgram {
    pub fn id(&self) -> u32 {
        self.id
    }

    /// glDeleteProgram
    pub fn delete(&self) {
        if !self.is_deleted.get() {
            self.is_deleted.set(true);
            let _ = self.renderer.send(CanvasMsg::WebGL(CanvasWebGLMsg::DeleteProgram(self.id)));
        }
    }

    /// glLinkProgram
    pub fn link(&self) {
        self.renderer.send(CanvasMsg::WebGL(CanvasWebGLMsg::LinkProgram(self.id))).unwrap();
    }

    /// glUseProgram
    pub fn use_program(&self) -> WebGLResult<()> {
        match self.fragment_shader.get() {
            Some(ref shader) if shader.successfully_compiled() => {},
            _ => return Err(WebGLError::InvalidOperation),
        }

        match self.vertex_shader.get() {
            Some(ref shader) if shader.successfully_compiled() => {},
            _ => return Err(WebGLError::InvalidOperation),
        }

        self.renderer.send(CanvasMsg::WebGL(CanvasWebGLMsg::UseProgram(self.id))).unwrap();
        Ok(())
    }

    /// glAttachShader
    pub fn attach_shader(&self, shader: &WebGLShader) -> WebGLResult<()> {
        let shader_slot = match shader.gl_type() {
            constants::FRAGMENT_SHADER => &self.fragment_shader,
            constants::VERTEX_SHADER => &self.vertex_shader,
            _ => return Err(WebGLError::InvalidOperation),
        };

        // TODO(ecoal95): Differentiate between same shader already assigned and other previous
        // shader.
        if shader_slot.get().is_some() {
            return Err(WebGLError::InvalidOperation);
        }

        shader_slot.set(Some(shader));

        self.renderer.send(CanvasMsg::WebGL(CanvasWebGLMsg::AttachShader(self.id, shader.id()))).unwrap();

        Ok(())
    }

    /// glBindAttribLocation
    pub fn bind_attrib_location(&self, index: u32, name: DOMString) -> WebGLResult<()> {
        if name.len() > MAX_UNIFORM_AND_ATTRIBUTE_LEN {
            return Err(WebGLError::InvalidValue);
        }

        // Check if the name is reserved
        if name.starts_with("webgl") || name.starts_with("_webgl_") {
            return Err(WebGLError::InvalidOperation);
        }

        self.renderer
            .send(CanvasMsg::WebGL(CanvasWebGLMsg::BindAttribLocation(self.id, index, String::from(name))))
            .unwrap();
        Ok(())
    }

    /// glGetAttribLocation
    pub fn get_attrib_location(&self, name: DOMString) -> WebGLResult<Option<i32>> {
        if name.len() > MAX_UNIFORM_AND_ATTRIBUTE_LEN {
            return Err(WebGLError::InvalidValue);
        }

        // Check if the name is reserved
        if name.starts_with("webgl") || name.starts_with("_webgl_") {
            return Ok(None);
        }

        let (sender, receiver) = ipc::channel().unwrap();
        self.renderer
            .send(CanvasMsg::WebGL(CanvasWebGLMsg::GetAttribLocation(self.id, String::from(name), sender)))
            .unwrap();
        Ok(receiver.recv().unwrap())
    }

    /// glGetUniformLocation
    pub fn get_uniform_location(&self, name: DOMString) -> WebGLResult<Option<i32>> {
        if name.len() > MAX_UNIFORM_AND_ATTRIBUTE_LEN {
            return Err(WebGLError::InvalidValue);
        }

        // Check if the name is reserved
        if name.starts_with("webgl") || name.starts_with("_webgl_") {
            return Ok(None);
        }

        let (sender, receiver) = ipc::channel().unwrap();
        self.renderer
            .send(CanvasMsg::WebGL(CanvasWebGLMsg::GetUniformLocation(self.id, String::from(name), sender)))
            .unwrap();
        Ok(receiver.recv().unwrap())
    }

    /// glGetProgramParameter
    pub fn parameter(&self, param_id: u32) -> WebGLResult<WebGLParameter> {
        let (sender, receiver) = ipc::channel().unwrap();
        self.renderer.send(CanvasMsg::WebGL(CanvasWebGLMsg::GetProgramParameter(self.id, param_id, sender))).unwrap();
        receiver.recv().unwrap()
    }
}

impl Drop for WebGLProgram {
    fn drop(&mut self) {
        self.delete();
    }
}
