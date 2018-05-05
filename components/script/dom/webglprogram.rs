/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://www.khronos.org/registry/webgl/specs/latest/1.0/webgl.idl
use canvas_traits::webgl::{WebGLCommand, WebGLError, WebGLMsgSender, WebGLProgramId, WebGLResult};
use canvas_traits::webgl::webgl_channel;
use dom::bindings::codegen::Bindings::WebGLProgramBinding;
use dom::bindings::codegen::Bindings::WebGLRenderingContextBinding::WebGLRenderingContextConstants as constants;
use dom::bindings::reflector::{DomObject, reflect_dom_object};
use dom::bindings::root::{DomRoot, MutNullableDom};
use dom::bindings::str::DOMString;
use dom::webglactiveinfo::WebGLActiveInfo;
use dom::webglobject::WebGLObject;
use dom::webglrenderingcontext::MAX_UNIFORM_AND_ATTRIBUTE_LEN;
use dom::webglshader::WebGLShader;
use dom::window::Window;
use dom_struct::dom_struct;
use std::cell::Cell;

#[dom_struct]
pub struct WebGLProgram {
    webgl_object: WebGLObject,
    id: WebGLProgramId,
    is_deleted: Cell<bool>,
    link_called: Cell<bool>,
    linked: Cell<bool>,
    fragment_shader: MutNullableDom<WebGLShader>,
    vertex_shader: MutNullableDom<WebGLShader>,
    #[ignore_malloc_size_of = "Defined in ipc-channel"]
    renderer: WebGLMsgSender,
}

/// ANGLE adds a `_u` prefix to variable names:
///
/// https://chromium.googlesource.com/angle/angle/+/855d964bd0d05f6b2cb303f625506cf53d37e94f
///
/// To avoid hard-coding this we would need to use the `sh::GetAttributes` and `sh::GetUniforms`
/// API to look up the `x.name` and `x.mappedName` members,
/// then build a data structure for bi-directional lookup (so either linear scan or two hashmaps).
/// Even then, this would probably only support plain variable names like "foo".
/// Strings passed to e.g. `GetUniformLocation` can be expressions like "foo[0].bar",
/// with the mapping for that "bar" name in yet another part of ANGLE’s API.
const ANGLE_NAME_PREFIX: &'static str = "_u";

fn to_name_in_compiled_shader(s: &str) -> String {
    map_dot_separated(s, |s, mapped| {
        mapped.push_str(ANGLE_NAME_PREFIX);
        mapped.push_str(s);
    })
}

fn from_name_in_compiled_shader(s: &str) -> String {
    map_dot_separated(s, |s, mapped| {
        mapped.push_str(if s.starts_with(ANGLE_NAME_PREFIX) {
            &s[ANGLE_NAME_PREFIX.len()..]
        } else {
            s
        })
    })
}

fn map_dot_separated<F: Fn(&str, &mut String)>(s: &str, f: F) -> String {
    let mut iter = s.split('.');
    let mut mapped = String::new();
    f(iter.next().unwrap(), &mut mapped);
    for s in iter {
        mapped.push('.');
        f(s, &mut mapped);
    }
    mapped
}

impl WebGLProgram {
    fn new_inherited(renderer: WebGLMsgSender,
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

    pub fn maybe_new(window: &Window, renderer: WebGLMsgSender)
                     -> Option<DomRoot<WebGLProgram>> {
        let (sender, receiver) = webgl_channel().unwrap();
        renderer.send(WebGLCommand::CreateProgram(sender)).unwrap();

        let result = receiver.recv().unwrap();
        result.map(|program_id| WebGLProgram::new(window, renderer, program_id))
    }

    pub fn new(window: &Window,
               renderer: WebGLMsgSender,
               id: WebGLProgramId)
               -> DomRoot<WebGLProgram> {
        reflect_dom_object(Box::new(WebGLProgram::new_inherited(renderer, id)),
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
            let _ = self.renderer.send(WebGLCommand::DeleteProgram(self.id));

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
        self.renderer.send(WebGLCommand::LinkProgram(self.id)).unwrap();
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

        self.renderer.send(WebGLCommand::UseProgram(self.id)).unwrap();
        Ok(())
    }

    /// glValidateProgram
    pub fn validate(&self) -> WebGLResult<()> {
        if self.is_deleted() {
            return Err(WebGLError::InvalidOperation);
        }
        self.renderer.send(WebGLCommand::ValidateProgram(self.id)).unwrap();
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

        self.renderer.send(WebGLCommand::AttachShader(self.id, shader.id())).unwrap();

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

        self.renderer.send(WebGLCommand::DetachShader(self.id, shader.id())).unwrap();

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

        let name = to_name_in_compiled_shader(&name);

        self.renderer
            .send(WebGLCommand::BindAttribLocation(self.id, index, name))
            .unwrap();
        Ok(())
    }

    pub fn get_active_uniform(&self, index: u32) -> WebGLResult<DomRoot<WebGLActiveInfo>> {
        if self.is_deleted() {
            return Err(WebGLError::InvalidValue);
        }
        let (sender, receiver) = webgl_channel().unwrap();
        self.renderer
            .send(WebGLCommand::GetActiveUniform(self.id, index, sender))
            .unwrap();

        receiver.recv().unwrap().map(|(size, ty, name)| {
            let name = DOMString::from(from_name_in_compiled_shader(&name));
            WebGLActiveInfo::new(self.global().as_window(), size, ty, name)
        })
    }

    /// glGetActiveAttrib
    pub fn get_active_attrib(&self, index: u32) -> WebGLResult<DomRoot<WebGLActiveInfo>> {
        if self.is_deleted() {
            return Err(WebGLError::InvalidValue);
        }
        let (sender, receiver) = webgl_channel().unwrap();
        self.renderer
            .send(WebGLCommand::GetActiveAttrib(self.id, index, sender))
            .unwrap();

        receiver.recv().unwrap().map(|(size, ty, name)| {
            let name = DOMString::from(from_name_in_compiled_shader(&name));
            WebGLActiveInfo::new(self.global().as_window(), size, ty, name)
        })
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
            return Ok(None);
        }

        // https://www.khronos.org/registry/webgl/specs/latest/1.0/#GLSL_CONSTRUCTS
        if name.starts_with("webgl_") || name.starts_with("_webgl_") {
            return Ok(None);
        }

        let name = to_name_in_compiled_shader(&name);

        let (sender, receiver) = webgl_channel().unwrap();
        self.renderer
            .send(WebGLCommand::GetAttribLocation(self.id, name, sender))
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

        let name = to_name_in_compiled_shader(&name);

        let (sender, receiver) = webgl_channel().unwrap();
        self.renderer
            .send(WebGLCommand::GetUniformLocation(self.id, name, sender))
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
        let (sender, receiver) = webgl_channel().unwrap();
        self.renderer.send(WebGLCommand::GetProgramInfoLog(self.id, sender)).unwrap();
        Ok(receiver.recv().unwrap())
    }

    pub fn attached_shaders(&self) -> WebGLResult<Vec<DomRoot<WebGLShader>>> {
        if self.is_deleted.get() {
            return Err(WebGLError::InvalidValue);
        }
        Ok(match (self.vertex_shader.get(), self.fragment_shader.get()) {
            (Some(vertex_shader), Some(fragment_shader)) => {
                vec![vertex_shader, fragment_shader]
            }
            (Some(shader), None) | (None, Some(shader)) => vec![shader],
            (None, None) => vec![]
        })
    }
}

impl Drop for WebGLProgram {
    fn drop(&mut self) {
        self.delete();
    }
}
