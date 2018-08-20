/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://www.khronos.org/registry/webgl/specs/latest/1.0/webgl.idl
use canvas_traits::webgl::{ActiveAttribInfo, ActiveUniformInfo, WebGLCommand, WebGLError};
use canvas_traits::webgl::{WebGLProgramId, WebGLResult, webgl_channel};
use dom::bindings::cell::DomRefCell;
use dom::bindings::codegen::Bindings::WebGLProgramBinding;
use dom::bindings::codegen::Bindings::WebGLRenderingContextBinding::WebGLRenderingContextConstants as constants;
use dom::bindings::inheritance::Castable;
use dom::bindings::reflector::{DomObject, reflect_dom_object};
use dom::bindings::root::{DomRoot, MutNullableDom};
use dom::bindings::str::DOMString;
use dom::webglactiveinfo::WebGLActiveInfo;
use dom::webglobject::WebGLObject;
use dom::webglrenderingcontext::{MAX_UNIFORM_AND_ATTRIBUTE_LEN, WebGLRenderingContext};
use dom::webglshader::WebGLShader;
use dom::webgluniformlocation::WebGLUniformLocation;
use dom_struct::dom_struct;
use fnv::FnvHashSet;
use std::cell::{Cell, Ref};
use typeholder::TypeHolderTrait;

#[dom_struct]
pub struct WebGLProgram<TH: TypeHolderTrait> {
    webgl_object: WebGLObject<TH>,
    id: WebGLProgramId,
    is_in_use: Cell<bool>,
    marked_for_deletion: Cell<bool>,
    link_called: Cell<bool>,
    linked: Cell<bool>,
    link_generation: Cell<u64>,
    fragment_shader: MutNullableDom<WebGLShader<TH>>,
    vertex_shader: MutNullableDom<WebGLShader<TH>>,
    active_attribs: DomRefCell<Box<[ActiveAttribInfo]>>,
    active_uniforms: DomRefCell<Box<[ActiveUniformInfo]>>,
}

impl<TH: TypeHolderTrait> WebGLProgram<TH> {
    fn new_inherited(context: &WebGLRenderingContext<TH>, id: WebGLProgramId) -> Self {
        Self {
            webgl_object: WebGLObject::new_inherited(context),
            id: id,
            is_in_use: Default::default(),
            marked_for_deletion: Default::default(),
            link_called: Default::default(),
            linked: Default::default(),
            link_generation: Default::default(),
            fragment_shader: Default::default(),
            vertex_shader: Default::default(),
            active_attribs: DomRefCell::new(vec![].into()),
            active_uniforms: DomRefCell::new(vec![].into()),
        }
    }

    pub fn maybe_new(context: &WebGLRenderingContext<TH>) -> Option<DomRoot<Self>> {
        let (sender, receiver) = webgl_channel().unwrap();
        context.send_command(WebGLCommand::CreateProgram(sender));
        receiver.recv().unwrap().map(|id| WebGLProgram::new(context, id))
    }

    pub fn new(context: &WebGLRenderingContext<TH>, id: WebGLProgramId) -> DomRoot<Self> {
        reflect_dom_object(
            Box::new(WebGLProgram::new_inherited(context, id)),
            &*context.global(),
            WebGLProgramBinding::Wrap,
        )
    }
}


impl<TH: TypeHolderTrait> WebGLProgram<TH> {
    pub fn id(&self) -> WebGLProgramId {
        self.id
    }

    /// glDeleteProgram
    pub fn mark_for_deletion(&self) {
        if self.marked_for_deletion.get() {
            return;
        }
        self.marked_for_deletion.set(true);
        self.upcast::<WebGLObject<TH>>()
            .context()
            .send_command(WebGLCommand::DeleteProgram(self.id));
        if self.is_deleted() {
            self.detach_shaders();
        }
    }

    pub fn in_use(&self, value: bool) {
        if self.is_in_use.get() == value {
            return;
        }
        self.is_in_use.set(value);
        if self.is_deleted() {
            self.detach_shaders();
        }
    }

    fn detach_shaders(&self) {
        assert!(self.is_deleted());
        if let Some(shader) = self.fragment_shader.get() {
            shader.decrement_attached_counter();
            self.fragment_shader.set(None);
        }
        if let Some(shader) = self.vertex_shader.get() {
            shader.decrement_attached_counter();
            self.vertex_shader.set(None);
        }
    }

    pub fn is_in_use(&self) -> bool {
        self.is_in_use.get()
    }

    pub fn is_marked_for_deletion(&self) -> bool {
        self.marked_for_deletion.get()
    }

    pub fn is_deleted(&self) -> bool {
        self.marked_for_deletion.get() && !self.is_in_use.get()
    }

    pub fn is_linked(&self) -> bool {
        self.linked.get()
    }

    /// glLinkProgram
    pub fn link(&self) -> WebGLResult<()> {
        self.linked.set(false);
        self.link_generation.set(self.link_generation.get().checked_add(1).unwrap());
        *self.active_attribs.borrow_mut() = Box::new([]);
        *self.active_uniforms.borrow_mut() = Box::new([]);

        match self.fragment_shader.get() {
            Some(ref shader) if shader.successfully_compiled() => {},
            _ => return Ok(()), // callers use gl.LINK_STATUS to check link errors
        }

        match self.vertex_shader.get() {
            Some(ref shader) if shader.successfully_compiled() => {},
            _ => return Ok(()), // callers use gl.LINK_STATUS to check link errors
        }

        let (sender, receiver) = webgl_channel().unwrap();
        self.upcast::<WebGLObject<TH>>()
            .context()
            .send_command(WebGLCommand::LinkProgram(self.id, sender));
        let link_info = receiver.recv().unwrap();

        {
            let mut used_locs = FnvHashSet::default();
            let mut used_names = FnvHashSet::default();
            for active_attrib in &*link_info.active_attribs {
                if active_attrib.location == -1 {
                    continue;
                }
                let columns = match active_attrib.type_ {
                    constants::FLOAT_MAT2 => 2,
                    constants::FLOAT_MAT3 => 3,
                    constants::FLOAT_MAT4 => 4,
                    _ => 1,
                };
                assert!(used_names.insert(&*active_attrib.name));
                for column in 0..columns {
                    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#6.31
                    if !used_locs.insert(active_attrib.location as u32 + column) {
                        return Ok(());
                    }
                }
            }
            for active_uniform in &*link_info.active_uniforms {
                // https://www.khronos.org/registry/webgl/specs/latest/1.0/#6.41
                if !used_names.insert(&*active_uniform.base_name) {
                    return Ok(());
                }
            }
        }

        self.linked.set(link_info.linked);
        *self.active_attribs.borrow_mut() = link_info.active_attribs;
        *self.active_uniforms.borrow_mut() = link_info.active_uniforms;
        Ok(())
    }

    pub fn active_attribs(&self) -> Ref<[ActiveAttribInfo]> {
        Ref::map(self.active_attribs.borrow(), |attribs| &**attribs)
    }

    pub fn active_uniforms(&self) -> Ref<[ActiveUniformInfo]> {
        Ref::map(self.active_uniforms.borrow(), |uniforms| &**uniforms)
    }

    /// glValidateProgram
    pub fn validate(&self) -> WebGLResult<()> {
        if self.is_deleted() {
            return Err(WebGLError::InvalidOperation);
        }
        self.upcast::<WebGLObject<TH>>()
            .context()
            .send_command(WebGLCommand::ValidateProgram(self.id));
        Ok(())
    }

    /// glAttachShader
    pub fn attach_shader(&self, shader: &WebGLShader<TH>) -> WebGLResult<()> {
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

        if shader_slot.get().is_some() {
            return Err(WebGLError::InvalidOperation);
        }

        shader_slot.set(Some(shader));
        shader.increment_attached_counter();

        self.upcast::<WebGLObject<TH>>()
            .context()
            .send_command(WebGLCommand::AttachShader(self.id, shader.id()));

        Ok(())
    }

    /// glDetachShader
    pub fn detach_shader(&self, shader: &WebGLShader<TH>) -> WebGLResult<()> {
        if self.is_deleted() {
            return Err(WebGLError::InvalidOperation);
        }
        let shader_slot = match shader.gl_type() {
            constants::FRAGMENT_SHADER => &self.fragment_shader,
            constants::VERTEX_SHADER => &self.vertex_shader,
            _ => return Err(WebGLError::InvalidValue),
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

        self.upcast::<WebGLObject<TH>>()
            .context()
            .send_command(WebGLCommand::DetachShader(self.id, shader.id()));

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

        self.upcast::<WebGLObject<TH>>()
            .context()
            .send_command(WebGLCommand::BindAttribLocation(self.id, index, name.into()));
        Ok(())
    }

    pub fn get_active_uniform(&self, index: u32) -> WebGLResult<DomRoot<WebGLActiveInfo<TH>>> {
        if self.is_deleted() {
            return Err(WebGLError::InvalidValue);
        }
        let uniforms = self.active_uniforms.borrow();
        let data = uniforms.get(index as usize).ok_or(WebGLError::InvalidValue)?;
        Ok(WebGLActiveInfo::new(
            self.global().as_window(),
            data.size.unwrap_or(1),
            data.type_,
            data.name().into(),
        ))
    }

    /// glGetActiveAttrib
    pub fn get_active_attrib(&self, index: u32) -> WebGLResult<DomRoot<WebGLActiveInfo<TH>>> {
        if self.is_deleted() {
            return Err(WebGLError::InvalidValue);
        }
        let attribs = self.active_attribs.borrow();
        let data = attribs.get(index as usize).ok_or(WebGLError::InvalidValue)?;
        Ok(WebGLActiveInfo::new(
            self.global().as_window(),
            data.size,
            data.type_,
            data.name.clone().into(),
        ))
    }

    /// glGetAttribLocation
    pub fn get_attrib_location(&self, name: DOMString) -> WebGLResult<i32> {
        if !self.is_linked() || self.is_deleted() {
            return Err(WebGLError::InvalidOperation);
        }
        if name.len() > MAX_UNIFORM_AND_ATTRIBUTE_LEN {
            return Err(WebGLError::InvalidValue);
        }

        // Check if the name is reserved
        if name.starts_with("gl_") {
            return Ok(-1);
        }

        // https://www.khronos.org/registry/webgl/specs/latest/1.0/#GLSL_CONSTRUCTS
        if name.starts_with("webgl_") || name.starts_with("_webgl_") {
            return Ok(-1);
        }

        let location = self.active_attribs
            .borrow()
            .iter()
            .find(|attrib| attrib.name == &*name)
            .map_or(-1, |attrib| attrib.location);
        Ok(location)
    }

    /// glGetUniformLocation
    pub fn get_uniform_location(
        &self,
        name: DOMString,
    ) -> WebGLResult<Option<DomRoot<WebGLUniformLocation<TH>>>> {
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

        let (size, type_) = {
            let (base_name, array_index) = match parse_uniform_name(&name) {
                Some((name, index)) if index.map_or(true, |i| i >= 0) => (name, index),
                _ => return Ok(None),
            };

            let uniforms = self.active_uniforms.borrow();
            match uniforms.iter().find(|attrib| &*attrib.base_name == base_name) {
                Some(uniform) if array_index.is_none() || array_index < uniform.size => {
                    (uniform.size.map(|size| size - array_index.unwrap_or_default()), uniform.type_)
                },
                _ => return Ok(None),
            }
        };

        let (sender, receiver) = webgl_channel().unwrap();
        self.upcast::<WebGLObject<TH>>()
            .context()
            .send_command(WebGLCommand::GetUniformLocation(self.id, name.into(), sender));
        let location = receiver.recv().unwrap();

        Ok(Some(WebGLUniformLocation::new(
            self.global().as_window(),
            location,
            self.id,
            self.link_generation.get(),
            size,
            type_,
        )))
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
        self.upcast::<WebGLObject<TH>>()
            .context()
            .send_command(WebGLCommand::GetProgramInfoLog(self.id, sender));
        Ok(receiver.recv().unwrap())
    }

    pub fn attached_shaders(&self) -> WebGLResult<Vec<DomRoot<WebGLShader<TH>>>> {
        if self.marked_for_deletion.get() {
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

    pub fn link_generation(&self) -> u64 {
        self.link_generation.get()
    }
}

impl<TH: TypeHolderTrait> Drop for WebGLProgram<TH> {
    fn drop(&mut self) {
        self.in_use(false);
        self.mark_for_deletion();
    }
}


fn parse_uniform_name(name: &str) -> Option<(&str, Option<i32>)> {
    if !name.ends_with(']') {
        return Some((name, None));
    }
    let bracket_pos = name[..name.len() - 1].rfind('[')?;
    let index = name[(bracket_pos + 1)..(name.len() - 1)].parse::<i32>().ok()?;
    Some((&name[..bracket_pos], Some(index)))
}
