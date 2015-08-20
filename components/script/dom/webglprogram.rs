/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://www.khronos.org/registry/webgl/specs/latest/1.0/webgl.idl
use dom::bindings::codegen::Bindings::WebGLProgramBinding;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JS, MutNullableHeap, Root};
use dom::bindings::utils::reflect_dom_object;
use dom::webglobject::WebGLObject;
use dom::webglrenderingcontext::MAX_UNIFORM_AND_ATTRIBUTE_LEN;
use dom::webglshader::{WebGLShader, WebGLShaderHelpers};

use dom::bindings::codegen::Bindings::WebGLRenderingContextBinding::WebGLRenderingContextConstants as constants;

use canvas_traits::{CanvasMsg, CanvasWebGLMsg, WebGLResult, WebGLError};
use ipc_channel::ipc::{self, IpcSender};
use std::cell::Cell;

#[dom_struct]
#[derive(HeapSizeOf)]
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

pub trait WebGLProgramHelpers {
    fn delete(self);
    fn link(self);
    fn use_program(self);
    fn attach_shader(self, shader: &WebGLShader) -> WebGLResult<()>;
    fn get_attrib_location(self, name: String) -> WebGLResult<Option<i32>>;
    fn get_uniform_location(self, name: String) -> WebGLResult<Option<i32>>;
}

impl<'a> WebGLProgramHelpers for &'a WebGLProgram {
    /// glDeleteProgram
    fn delete(self) {
        if !self.is_deleted.get() {
            self.is_deleted.set(true);
            self.renderer.send(CanvasMsg::WebGL(CanvasWebGLMsg::DeleteProgram(self.id))).unwrap();
        }
    }

    /// glLinkProgram
    fn link(self) {
        self.renderer.send(CanvasMsg::WebGL(CanvasWebGLMsg::LinkProgram(self.id))).unwrap();
    }

    /// glUseProgram
    fn use_program(self) {
        self.renderer.send(CanvasMsg::WebGL(CanvasWebGLMsg::UseProgram(self.id))).unwrap();
    }

    /// glAttachShader
    fn attach_shader(self, shader: &WebGLShader) -> WebGLResult<()> {
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

        shader_slot.set(Some(JS::from_ref(shader)));

        self.renderer.send(CanvasMsg::WebGL(CanvasWebGLMsg::AttachShader(self.id, shader.id()))).unwrap();

        Ok(())
    }

    /// glGetAttribLocation
    fn get_attrib_location(self, name: String) -> WebGLResult<Option<i32>> {
        if name.len() > MAX_UNIFORM_AND_ATTRIBUTE_LEN {
            return Err(WebGLError::InvalidValue);
        }

        // Check if the name is reserved
        if name.starts_with("webgl") || name.starts_with("_webgl_") {
            return Ok(None);
        }

        let (sender, receiver) = ipc::channel().unwrap();
        self.renderer.send(CanvasMsg::WebGL(CanvasWebGLMsg::GetAttribLocation(self.id, name, sender))).unwrap();
        Ok(receiver.recv().unwrap())
    }

    /// glGetUniformLocation
    fn get_uniform_location(self, name: String) -> WebGLResult<Option<i32>> {
        if name.len() > MAX_UNIFORM_AND_ATTRIBUTE_LEN {
            return Err(WebGLError::InvalidValue);
        }

        // Check if the name is reserved
        if name.starts_with("webgl") || name.starts_with("_webgl_") {
            return Ok(None);
        }

        let (sender, receiver) = ipc::channel().unwrap();
        self.renderer.send(CanvasMsg::WebGL(CanvasWebGLMsg::GetUniformLocation(self.id, name, sender))).unwrap();
        Ok(receiver.recv().unwrap())
    }
}
