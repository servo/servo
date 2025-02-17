/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://www.khronos.org/registry/webgl/specs/latest/1.0/webgl.idl
use std::cell::Cell;

use canvas_traits::webgl::{
    webgl_channel, ActiveAttribInfo, ActiveUniformBlockInfo, ActiveUniformInfo, WebGLCommand,
    WebGLError, WebGLProgramId, WebGLResult,
};
use dom_struct::dom_struct;
use fnv::FnvHashSet;

use crate::dom::bindings::cell::{DomRefCell, Ref};
use crate::dom::bindings::codegen::Bindings::WebGL2RenderingContextBinding::WebGL2RenderingContextConstants as constants2;
use crate::dom::bindings::codegen::Bindings::WebGLRenderingContextBinding::WebGLRenderingContextConstants as constants;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::{reflect_dom_object, DomGlobal};
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::webglactiveinfo::WebGLActiveInfo;
use crate::dom::webglobject::WebGLObject;
use crate::dom::webglrenderingcontext::{Operation, WebGLRenderingContext};
use crate::dom::webglshader::WebGLShader;
use crate::dom::webgluniformlocation::WebGLUniformLocation;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct WebGLProgram {
    webgl_object: WebGLObject,
    #[no_trace]
    id: WebGLProgramId,
    is_in_use: Cell<bool>,
    marked_for_deletion: Cell<bool>,
    link_called: Cell<bool>,
    linked: Cell<bool>,
    link_generation: Cell<u64>,
    fragment_shader: MutNullableDom<WebGLShader>,
    vertex_shader: MutNullableDom<WebGLShader>,
    #[no_trace]
    active_attribs: DomRefCell<Box<[ActiveAttribInfo]>>,
    #[no_trace]
    active_uniforms: DomRefCell<Box<[ActiveUniformInfo]>>,
    #[no_trace]
    active_uniform_blocks: DomRefCell<Box<[ActiveUniformBlockInfo]>>,
    transform_feedback_varyings_length: Cell<i32>,
    transform_feedback_mode: Cell<i32>,
}

impl WebGLProgram {
    fn new_inherited(context: &WebGLRenderingContext, id: WebGLProgramId) -> Self {
        Self {
            webgl_object: WebGLObject::new_inherited(context),
            id,
            is_in_use: Default::default(),
            marked_for_deletion: Default::default(),
            link_called: Default::default(),
            linked: Default::default(),
            link_generation: Default::default(),
            fragment_shader: Default::default(),
            vertex_shader: Default::default(),
            active_attribs: DomRefCell::new(vec![].into()),
            active_uniforms: DomRefCell::new(vec![].into()),
            active_uniform_blocks: DomRefCell::new(vec![].into()),
            transform_feedback_varyings_length: Default::default(),
            transform_feedback_mode: Default::default(),
        }
    }

    pub(crate) fn maybe_new(context: &WebGLRenderingContext) -> Option<DomRoot<Self>> {
        let (sender, receiver) = webgl_channel().unwrap();
        context.send_command(WebGLCommand::CreateProgram(sender));
        receiver
            .recv()
            .unwrap()
            .map(|id| WebGLProgram::new(context, id))
    }

    pub(crate) fn new(context: &WebGLRenderingContext, id: WebGLProgramId) -> DomRoot<Self> {
        reflect_dom_object(
            Box::new(WebGLProgram::new_inherited(context, id)),
            &*context.global(),
            CanGc::note(),
        )
    }
}

impl WebGLProgram {
    pub(crate) fn id(&self) -> WebGLProgramId {
        self.id
    }

    /// glDeleteProgram
    pub(crate) fn mark_for_deletion(&self, operation_fallibility: Operation) {
        if self.marked_for_deletion.get() {
            return;
        }
        self.marked_for_deletion.set(true);
        let cmd = WebGLCommand::DeleteProgram(self.id);
        let context = self.upcast::<WebGLObject>().context();
        match operation_fallibility {
            Operation::Fallible => context.send_command_ignored(cmd),
            Operation::Infallible => context.send_command(cmd),
        }
        if self.is_deleted() {
            self.detach_shaders();
        }
    }

    pub(crate) fn in_use(&self, value: bool) {
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

    pub(crate) fn is_in_use(&self) -> bool {
        self.is_in_use.get()
    }

    pub(crate) fn is_marked_for_deletion(&self) -> bool {
        self.marked_for_deletion.get()
    }

    pub(crate) fn is_deleted(&self) -> bool {
        self.marked_for_deletion.get() && !self.is_in_use.get()
    }

    pub(crate) fn is_linked(&self) -> bool {
        self.linked.get()
    }

    /// glLinkProgram
    pub(crate) fn link(&self) -> WebGLResult<()> {
        self.linked.set(false);
        self.link_generation
            .set(self.link_generation.get().checked_add(1).unwrap());
        *self.active_attribs.borrow_mut() = Box::new([]);
        *self.active_uniforms.borrow_mut() = Box::new([]);
        *self.active_uniform_blocks.borrow_mut() = Box::new([]);

        match self.fragment_shader.get() {
            Some(ref shader) if shader.successfully_compiled() => {},
            _ => return Ok(()), // callers use gl.LINK_STATUS to check link errors
        }

        match self.vertex_shader.get() {
            Some(ref shader) if shader.successfully_compiled() => {},
            _ => return Ok(()), // callers use gl.LINK_STATUS to check link errors
        }

        let (sender, receiver) = webgl_channel().unwrap();
        self.upcast::<WebGLObject>()
            .context()
            .send_command(WebGLCommand::LinkProgram(self.id, sender));
        let link_info = receiver.recv().unwrap();

        {
            let mut used_locs = FnvHashSet::default();
            let mut used_names = FnvHashSet::default();
            for active_attrib in &*link_info.active_attribs {
                let Some(location) = active_attrib.location else {
                    continue;
                };
                let columns = match active_attrib.type_ {
                    constants::FLOAT_MAT2 => 2,
                    constants::FLOAT_MAT3 => 3,
                    constants::FLOAT_MAT4 => 4,
                    _ => 1,
                };
                assert!(used_names.insert(&*active_attrib.name));
                for column in 0..columns {
                    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#6.31
                    if !used_locs.insert(location + column) {
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
        self.link_called.set(true);
        self.transform_feedback_varyings_length
            .set(link_info.transform_feedback_length);
        self.transform_feedback_mode
            .set(link_info.transform_feedback_mode);
        *self.active_attribs.borrow_mut() = link_info.active_attribs;
        *self.active_uniforms.borrow_mut() = link_info.active_uniforms;
        *self.active_uniform_blocks.borrow_mut() = link_info.active_uniform_blocks;
        Ok(())
    }

    pub(crate) fn active_attribs(&self) -> Ref<[ActiveAttribInfo]> {
        Ref::map(self.active_attribs.borrow(), |attribs| &**attribs)
    }

    pub(crate) fn active_uniforms(&self) -> Ref<[ActiveUniformInfo]> {
        Ref::map(self.active_uniforms.borrow(), |uniforms| &**uniforms)
    }

    pub(crate) fn active_uniform_blocks(&self) -> Ref<[ActiveUniformBlockInfo]> {
        Ref::map(self.active_uniform_blocks.borrow(), |blocks| &**blocks)
    }

    /// glValidateProgram
    pub(crate) fn validate(&self) -> WebGLResult<()> {
        if self.is_deleted() {
            return Err(WebGLError::InvalidOperation);
        }
        self.upcast::<WebGLObject>()
            .context()
            .send_command(WebGLCommand::ValidateProgram(self.id));
        Ok(())
    }

    /// glAttachShader
    pub(crate) fn attach_shader(&self, shader: &WebGLShader) -> WebGLResult<()> {
        if self.is_deleted() || shader.is_deleted() {
            return Err(WebGLError::InvalidOperation);
        }
        let shader_slot = match shader.gl_type() {
            constants::FRAGMENT_SHADER => &self.fragment_shader,
            constants::VERTEX_SHADER => &self.vertex_shader,
            _ => {
                error!("detachShader: Unexpected shader type");
                return Err(WebGLError::InvalidValue);
            },
        };

        if shader_slot.get().is_some() {
            return Err(WebGLError::InvalidOperation);
        }

        shader_slot.set(Some(shader));
        shader.increment_attached_counter();

        self.upcast::<WebGLObject>()
            .context()
            .send_command(WebGLCommand::AttachShader(self.id, shader.id()));

        Ok(())
    }

    /// glDetachShader
    pub(crate) fn detach_shader(&self, shader: &WebGLShader) -> WebGLResult<()> {
        if self.is_deleted() {
            return Err(WebGLError::InvalidOperation);
        }
        let shader_slot = match shader.gl_type() {
            constants::FRAGMENT_SHADER => &self.fragment_shader,
            constants::VERTEX_SHADER => &self.vertex_shader,
            _ => return Err(WebGLError::InvalidValue),
        };

        match shader_slot.get() {
            Some(ref attached_shader) if attached_shader.id() != shader.id() => {
                return Err(WebGLError::InvalidOperation);
            },
            None => return Err(WebGLError::InvalidOperation),
            _ => {},
        }

        shader_slot.set(None);
        shader.decrement_attached_counter();

        self.upcast::<WebGLObject>()
            .context()
            .send_command(WebGLCommand::DetachShader(self.id, shader.id()));

        Ok(())
    }

    /// glBindAttribLocation
    pub(crate) fn bind_attrib_location(&self, index: u32, name: DOMString) -> WebGLResult<()> {
        if self.is_deleted() {
            return Err(WebGLError::InvalidOperation);
        }

        if !validate_glsl_name(&name)? {
            return Ok(());
        }
        if name.starts_with("gl_") {
            return Err(WebGLError::InvalidOperation);
        }

        self.upcast::<WebGLObject>()
            .context()
            .send_command(WebGLCommand::BindAttribLocation(
                self.id,
                index,
                name.into(),
            ));
        Ok(())
    }

    pub(crate) fn get_active_uniform(&self, index: u32) -> WebGLResult<DomRoot<WebGLActiveInfo>> {
        if self.is_deleted() {
            return Err(WebGLError::InvalidValue);
        }
        let uniforms = self.active_uniforms.borrow();
        let data = uniforms
            .get(index as usize)
            .ok_or(WebGLError::InvalidValue)?;
        Ok(WebGLActiveInfo::new(
            self.global().as_window(),
            data.size.unwrap_or(1),
            data.type_,
            data.name().into(),
        ))
    }

    /// glGetActiveAttrib
    pub(crate) fn get_active_attrib(&self, index: u32) -> WebGLResult<DomRoot<WebGLActiveInfo>> {
        if self.is_deleted() {
            return Err(WebGLError::InvalidValue);
        }
        let attribs = self.active_attribs.borrow();
        let data = attribs
            .get(index as usize)
            .ok_or(WebGLError::InvalidValue)?;
        Ok(WebGLActiveInfo::new(
            self.global().as_window(),
            data.size,
            data.type_,
            data.name.clone().into(),
        ))
    }

    /// glGetAttribLocation
    pub(crate) fn get_attrib_location(&self, name: DOMString) -> WebGLResult<i32> {
        if !self.is_linked() || self.is_deleted() {
            return Err(WebGLError::InvalidOperation);
        }

        if !validate_glsl_name(&name)? {
            return Ok(-1);
        }
        if name.starts_with("gl_") {
            return Ok(-1);
        }

        let location = self
            .active_attribs
            .borrow()
            .iter()
            .find(|attrib| attrib.name == *name)
            .and_then(|attrib| attrib.location.map(|l| l as i32))
            .unwrap_or(-1);
        Ok(location)
    }

    /// glGetFragDataLocation
    pub(crate) fn get_frag_data_location(&self, name: DOMString) -> WebGLResult<i32> {
        if !self.is_linked() || self.is_deleted() {
            return Err(WebGLError::InvalidOperation);
        }

        if !validate_glsl_name(&name)? {
            return Ok(-1);
        }
        if name.starts_with("gl_") {
            return Ok(-1);
        }

        let (sender, receiver) = webgl_channel().unwrap();
        self.upcast::<WebGLObject>()
            .context()
            .send_command(WebGLCommand::GetFragDataLocation(
                self.id,
                name.into(),
                sender,
            ));
        Ok(receiver.recv().unwrap())
    }

    /// glGetUniformLocation
    pub(crate) fn get_uniform_location(
        &self,
        name: DOMString,
    ) -> WebGLResult<Option<DomRoot<WebGLUniformLocation>>> {
        if !self.is_linked() || self.is_deleted() {
            return Err(WebGLError::InvalidOperation);
        }

        if !validate_glsl_name(&name)? {
            return Ok(None);
        }
        if name.starts_with("gl_") {
            return Ok(None);
        }

        let (size, type_) = {
            let (base_name, array_index) = match parse_uniform_name(&name) {
                Some((name, index)) if index.map_or(true, |i| i >= 0) => (name, index),
                _ => return Ok(None),
            };

            let uniforms = self.active_uniforms.borrow();
            match uniforms
                .iter()
                .find(|attrib| &*attrib.base_name == base_name)
            {
                Some(uniform) if array_index.is_none() || array_index < uniform.size => (
                    uniform
                        .size
                        .map(|size| size - array_index.unwrap_or_default()),
                    uniform.type_,
                ),
                _ => return Ok(None),
            }
        };

        let (sender, receiver) = webgl_channel().unwrap();
        self.upcast::<WebGLObject>()
            .context()
            .send_command(WebGLCommand::GetUniformLocation(
                self.id,
                name.into(),
                sender,
            ));
        let location = receiver.recv().unwrap();
        let context_id = self.upcast::<WebGLObject>().context().context_id();

        Ok(Some(WebGLUniformLocation::new(
            self.global().as_window(),
            location,
            context_id,
            self.id,
            self.link_generation.get(),
            size,
            type_,
        )))
    }

    pub(crate) fn get_uniform_block_index(&self, name: DOMString) -> WebGLResult<u32> {
        if !self.link_called.get() || self.is_deleted() {
            return Err(WebGLError::InvalidOperation);
        }

        if !validate_glsl_name(&name)? {
            return Ok(constants2::INVALID_INDEX);
        }

        let (sender, receiver) = webgl_channel().unwrap();
        self.upcast::<WebGLObject>()
            .context()
            .send_command(WebGLCommand::GetUniformBlockIndex(
                self.id,
                name.into(),
                sender,
            ));
        Ok(receiver.recv().unwrap())
    }

    pub(crate) fn get_uniform_indices(&self, names: Vec<DOMString>) -> WebGLResult<Vec<u32>> {
        if !self.link_called.get() || self.is_deleted() {
            return Err(WebGLError::InvalidOperation);
        }

        let validation_errors = names
            .iter()
            .map(|name| validate_glsl_name(name))
            .collect::<Vec<_>>();
        let first_validation_error = validation_errors.iter().find(|result| result.is_err());
        if let Some(error) = first_validation_error {
            return Err(error.unwrap_err());
        }

        let names = names
            .iter()
            .map(|name| name.to_string())
            .collect::<Vec<_>>();

        let (sender, receiver) = webgl_channel().unwrap();
        self.upcast::<WebGLObject>()
            .context()
            .send_command(WebGLCommand::GetUniformIndices(self.id, names, sender));
        Ok(receiver.recv().unwrap())
    }

    pub(crate) fn get_active_uniforms(
        &self,
        indices: Vec<u32>,
        pname: u32,
    ) -> WebGLResult<Vec<i32>> {
        if !self.is_linked() || self.is_deleted() {
            return Err(WebGLError::InvalidOperation);
        }

        match pname {
            constants2::UNIFORM_TYPE |
            constants2::UNIFORM_SIZE |
            constants2::UNIFORM_BLOCK_INDEX |
            constants2::UNIFORM_OFFSET |
            constants2::UNIFORM_ARRAY_STRIDE |
            constants2::UNIFORM_MATRIX_STRIDE |
            constants2::UNIFORM_IS_ROW_MAJOR => {},
            _ => return Err(WebGLError::InvalidEnum),
        }

        if indices.len() > self.active_uniforms.borrow().len() {
            return Err(WebGLError::InvalidValue);
        }

        let (sender, receiver) = webgl_channel().unwrap();
        self.upcast::<WebGLObject>()
            .context()
            .send_command(WebGLCommand::GetActiveUniforms(
                self.id, indices, pname, sender,
            ));
        Ok(receiver.recv().unwrap())
    }

    pub(crate) fn get_active_uniform_block_parameter(
        &self,
        block_index: u32,
        pname: u32,
    ) -> WebGLResult<Vec<i32>> {
        if !self.link_called.get() || self.is_deleted() {
            return Err(WebGLError::InvalidOperation);
        }

        if block_index as usize >= self.active_uniform_blocks.borrow().len() {
            return Err(WebGLError::InvalidValue);
        }

        match pname {
            constants2::UNIFORM_BLOCK_BINDING |
            constants2::UNIFORM_BLOCK_DATA_SIZE |
            constants2::UNIFORM_BLOCK_ACTIVE_UNIFORMS |
            constants2::UNIFORM_BLOCK_ACTIVE_UNIFORM_INDICES |
            constants2::UNIFORM_BLOCK_REFERENCED_BY_VERTEX_SHADER |
            constants2::UNIFORM_BLOCK_REFERENCED_BY_FRAGMENT_SHADER => {},
            _ => return Err(WebGLError::InvalidEnum),
        }

        let (sender, receiver) = webgl_channel().unwrap();
        self.upcast::<WebGLObject>().context().send_command(
            WebGLCommand::GetActiveUniformBlockParameter(self.id, block_index, pname, sender),
        );
        Ok(receiver.recv().unwrap())
    }

    pub(crate) fn get_active_uniform_block_name(&self, block_index: u32) -> WebGLResult<String> {
        if !self.link_called.get() || self.is_deleted() {
            return Err(WebGLError::InvalidOperation);
        }

        if block_index as usize >= self.active_uniform_blocks.borrow().len() {
            return Err(WebGLError::InvalidValue);
        }

        let (sender, receiver) = webgl_channel().unwrap();
        self.upcast::<WebGLObject>().context().send_command(
            WebGLCommand::GetActiveUniformBlockName(self.id, block_index, sender),
        );
        Ok(receiver.recv().unwrap())
    }

    pub(crate) fn bind_uniform_block(
        &self,
        block_index: u32,
        block_binding: u32,
    ) -> WebGLResult<()> {
        if block_index as usize >= self.active_uniform_blocks.borrow().len() {
            return Err(WebGLError::InvalidValue);
        }

        let mut active_uniforms = self.active_uniforms.borrow_mut();
        if active_uniforms.len() > block_binding as usize {
            active_uniforms[block_binding as usize].bind_index = Some(block_binding);
        }

        self.upcast::<WebGLObject>()
            .context()
            .send_command(WebGLCommand::UniformBlockBinding(
                self.id,
                block_index,
                block_binding,
            ));
        Ok(())
    }

    /// glGetProgramInfoLog
    pub(crate) fn get_info_log(&self) -> WebGLResult<String> {
        if self.is_deleted() {
            return Err(WebGLError::InvalidValue);
        }
        if self.link_called.get() {
            let shaders_compiled = match (self.fragment_shader.get(), self.vertex_shader.get()) {
                (Some(fs), Some(vs)) => fs.successfully_compiled() && vs.successfully_compiled(),
                _ => false,
            };
            if !shaders_compiled {
                return Ok("One or more shaders failed to compile".to_string());
            }
        }
        let (sender, receiver) = webgl_channel().unwrap();
        self.upcast::<WebGLObject>()
            .context()
            .send_command(WebGLCommand::GetProgramInfoLog(self.id, sender));
        Ok(receiver.recv().unwrap())
    }

    pub(crate) fn attached_shaders(&self) -> WebGLResult<Vec<DomRoot<WebGLShader>>> {
        if self.marked_for_deletion.get() {
            return Err(WebGLError::InvalidValue);
        }
        Ok(
            match (self.vertex_shader.get(), self.fragment_shader.get()) {
                (Some(vertex_shader), Some(fragment_shader)) => {
                    vec![vertex_shader, fragment_shader]
                },
                (Some(shader), None) | (None, Some(shader)) => vec![shader],
                (None, None) => vec![],
            },
        )
    }

    pub(crate) fn link_generation(&self) -> u64 {
        self.link_generation.get()
    }

    pub(crate) fn transform_feedback_varyings_length(&self) -> i32 {
        self.transform_feedback_varyings_length.get()
    }

    pub(crate) fn transform_feedback_buffer_mode(&self) -> i32 {
        self.transform_feedback_mode.get()
    }
}

impl Drop for WebGLProgram {
    fn drop(&mut self) {
        self.in_use(false);
        self.mark_for_deletion(Operation::Fallible);
    }
}

fn validate_glsl_name(name: &str) -> WebGLResult<bool> {
    if name.is_empty() {
        return Ok(false);
    }
    if name.len() > MAX_UNIFORM_AND_ATTRIBUTE_LEN {
        return Err(WebGLError::InvalidValue);
    }
    for c in name.chars() {
        validate_glsl_char(c)?;
    }
    if name.starts_with("webgl_") || name.starts_with("_webgl_") {
        return Err(WebGLError::InvalidOperation);
    }
    Ok(true)
}

fn validate_glsl_char(c: char) -> WebGLResult<()> {
    match c {
        'a'..='z' |
        'A'..='Z' |
        '0'..='9' |
        ' ' |
        '\t' |
        '\u{11}' |
        '\u{12}' |
        '\r' |
        '\n' |
        '_' |
        '.' |
        '+' |
        '-' |
        '/' |
        '*' |
        '%' |
        '<' |
        '>' |
        '[' |
        ']' |
        '(' |
        ')' |
        '{' |
        '}' |
        '^' |
        '|' |
        '&' |
        '~' |
        '=' |
        '!' |
        ':' |
        ';' |
        ',' |
        '?' => Ok(()),
        _ => Err(WebGLError::InvalidValue),
    }
}

fn parse_uniform_name(name: &str) -> Option<(&str, Option<i32>)> {
    if !name.ends_with(']') {
        return Some((name, None));
    }
    let bracket_pos = name[..name.len() - 1].rfind('[')?;
    let index = name[(bracket_pos + 1)..(name.len() - 1)]
        .parse::<i32>()
        .ok()?;
    Some((&name[..bracket_pos], Some(index)))
}

pub(crate) const MAX_UNIFORM_AND_ATTRIBUTE_LEN: usize = 256;
