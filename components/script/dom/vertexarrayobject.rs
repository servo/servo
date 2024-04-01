/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use canvas_traits::webgl::{
    ActiveAttribInfo, WebGLCommand, WebGLError, WebGLResult, WebGLVersion, WebGLVertexArrayId,
};

use crate::dom::bindings::cell::{ref_filter_map, DomRefCell, Ref};
use crate::dom::bindings::codegen::Bindings::WebGL2RenderingContextBinding::WebGL2RenderingContextConstants as constants2;
use crate::dom::bindings::codegen::Bindings::WebGLRenderingContextBinding::WebGLRenderingContextConstants as constants;
use crate::dom::bindings::root::{Dom, MutNullableDom};
use crate::dom::webglbuffer::WebGLBuffer;
use crate::dom::webglrenderingcontext::{Operation, WebGLRenderingContext};

#[derive(JSTraceable, MallocSizeOf)]
#[crown::unrooted_must_root_lint::must_root]
pub struct VertexArrayObject {
    context: Dom<WebGLRenderingContext>,
    #[no_trace]
    id: Option<WebGLVertexArrayId>,
    ever_bound: Cell<bool>,
    is_deleted: Cell<bool>,
    vertex_attribs: DomRefCell<Box<[VertexAttribData]>>,
    element_array_buffer: MutNullableDom<WebGLBuffer>,
}

impl VertexArrayObject {
    pub fn new(context: &WebGLRenderingContext, id: Option<WebGLVertexArrayId>) -> Self {
        let max_vertex_attribs = context.limits().max_vertex_attribs as usize;
        Self {
            context: Dom::from_ref(context),
            id,
            ever_bound: Default::default(),
            is_deleted: Default::default(),
            vertex_attribs: DomRefCell::new(vec![Default::default(); max_vertex_attribs].into()),
            element_array_buffer: Default::default(),
        }
    }

    pub fn id(&self) -> Option<WebGLVertexArrayId> {
        self.id
    }

    pub fn is_deleted(&self) -> bool {
        self.is_deleted.get()
    }

    pub fn delete(&self, operation_fallibility: Operation) {
        assert!(self.id.is_some());
        if self.is_deleted.get() {
            return;
        }
        self.is_deleted.set(true);
        let cmd = WebGLCommand::DeleteVertexArray(self.id.unwrap());
        match operation_fallibility {
            Operation::Fallible => self.context.send_command_ignored(cmd),
            Operation::Infallible => self.context.send_command(cmd),
        }

        for attrib_data in &**self.vertex_attribs.borrow() {
            if let Some(buffer) = attrib_data.buffer() {
                buffer.decrement_attached_counter(operation_fallibility);
            }
        }
        if let Some(buffer) = self.element_array_buffer.get() {
            buffer.decrement_attached_counter(operation_fallibility);
        }
    }

    pub fn ever_bound(&self) -> bool {
        self.ever_bound.get()
    }

    pub fn set_ever_bound(&self) {
        self.ever_bound.set(true);
    }

    pub fn element_array_buffer(&self) -> &MutNullableDom<WebGLBuffer> {
        &self.element_array_buffer
    }

    pub fn get_vertex_attrib(&self, index: u32) -> Option<Ref<VertexAttribData>> {
        ref_filter_map(self.vertex_attribs.borrow(), |attribs| {
            attribs.get(index as usize)
        })
    }

    pub fn set_vertex_attrib_type(&self, index: u32, type_: u32) {
        self.vertex_attribs.borrow_mut()[index as usize].type_ = type_;
    }

    pub fn vertex_attrib_pointer(
        &self,
        index: u32,
        size: i32,
        type_: u32,
        normalized: bool,
        stride: i32,
        offset: i64,
    ) -> WebGLResult<()> {
        let mut attribs = self.vertex_attribs.borrow_mut();
        let data = attribs
            .get_mut(index as usize)
            .ok_or(WebGLError::InvalidValue)?;

        if !(1..=4).contains(&size) {
            return Err(WebGLError::InvalidValue);
        }

        // https://www.khronos.org/registry/webgl/specs/latest/1.0/#BUFFER_OFFSET_AND_STRIDE
        // https://www.khronos.org/registry/webgl/specs/latest/1.0/#VERTEX_STRIDE
        if !(0..=255).contains(&stride) || offset < 0 {
            return Err(WebGLError::InvalidValue);
        }

        let is_webgl2 = matches!(self.context.webgl_version(), WebGLVersion::WebGL2);

        let bytes_per_component: i32 = match type_ {
            constants::BYTE | constants::UNSIGNED_BYTE => 1,
            constants::SHORT | constants::UNSIGNED_SHORT => 2,
            constants::FLOAT => 4,
            constants::INT | constants::UNSIGNED_INT if is_webgl2 => 4,
            constants2::HALF_FLOAT if is_webgl2 => 2,
            sparkle::gl::FIXED if is_webgl2 => 4,
            constants2::INT_2_10_10_10_REV | constants2::UNSIGNED_INT_2_10_10_10_REV
                if is_webgl2 && size == 4 =>
            {
                4
            },
            _ => return Err(WebGLError::InvalidEnum),
        };

        if offset % bytes_per_component as i64 > 0 || stride % bytes_per_component > 0 {
            return Err(WebGLError::InvalidOperation);
        }

        let buffer = self.context.array_buffer();
        match buffer {
            Some(ref buffer) => buffer.increment_attached_counter(),
            None if offset != 0 => {
                // https://github.com/KhronosGroup/WebGL/pull/2228
                return Err(WebGLError::InvalidOperation);
            },
            _ => {},
        }
        self.context.send_command(WebGLCommand::VertexAttribPointer(
            index,
            size,
            type_,
            normalized,
            stride,
            offset as u32,
        ));
        if let Some(old) = data.buffer() {
            old.decrement_attached_counter(Operation::Infallible);
        }

        *data = VertexAttribData {
            enabled_as_array: data.enabled_as_array,
            size: size as u8,
            type_,
            bytes_per_vertex: size as u8 * bytes_per_component as u8,
            normalized,
            stride: stride as u8,
            offset: offset as u32,
            buffer: buffer.map(|b| Dom::from_ref(&*b)),
            divisor: data.divisor,
        };

        Ok(())
    }

    pub fn vertex_attrib_divisor(&self, index: u32, value: u32) {
        self.vertex_attribs.borrow_mut()[index as usize].divisor = value;
    }

    pub fn enabled_vertex_attrib_array(&self, index: u32, value: bool) {
        self.vertex_attribs.borrow_mut()[index as usize].enabled_as_array = value;
    }

    pub fn unbind_buffer(&self, buffer: &WebGLBuffer) {
        for attrib in &mut **self.vertex_attribs.borrow_mut() {
            if let Some(b) = attrib.buffer() {
                if b.id() != buffer.id() {
                    continue;
                }
                b.decrement_attached_counter(Operation::Infallible);
            }
            attrib.buffer = None;
        }
        if self
            .element_array_buffer
            .get()
            .map_or(false, |b| buffer == &*b)
        {
            buffer.decrement_attached_counter(Operation::Infallible);
            self.element_array_buffer.set(None);
        }
    }

    pub fn validate_for_draw(
        &self,
        required_len: u32,
        instance_count: u32,
        active_attribs: &[ActiveAttribInfo],
    ) -> WebGLResult<()> {
        // TODO(nox): Cache limits per VAO.
        let attribs = self.vertex_attribs.borrow();
        // https://www.khronos.org/registry/webgl/specs/latest/1.0/#6.2
        if attribs
            .iter()
            .any(|data| data.enabled_as_array && data.buffer.is_none())
        {
            return Err(WebGLError::InvalidOperation);
        }
        let mut has_active_attrib = false;
        let mut has_divisor_0 = false;
        for active_info in active_attribs {
            if active_info.location < 0 {
                continue;
            }
            has_active_attrib = true;
            let attrib = &attribs[active_info.location as usize];
            if attrib.divisor == 0 {
                has_divisor_0 = true;
            }
            if !attrib.enabled_as_array {
                continue;
            }
            // https://www.khronos.org/registry/webgl/specs/latest/1.0/#6.6
            if required_len > 0 && instance_count > 0 {
                let max_vertices = attrib.max_vertices();
                if attrib.divisor == 0 {
                    if max_vertices < required_len {
                        return Err(WebGLError::InvalidOperation);
                    }
                } else if max_vertices
                    .checked_mul(attrib.divisor)
                    .map_or(false, |v| v < instance_count)
                {
                    return Err(WebGLError::InvalidOperation);
                }
            }
        }
        if has_active_attrib && !has_divisor_0 {
            return Err(WebGLError::InvalidOperation);
        }
        Ok(())
    }
}

impl Drop for VertexArrayObject {
    fn drop(&mut self) {
        if self.id.is_some() {
            self.delete(Operation::Fallible);
        }
    }
}

#[derive(Clone, JSTraceable, MallocSizeOf)]
#[crown::unrooted_must_root_lint::must_root]
pub struct VertexAttribData {
    pub enabled_as_array: bool,
    pub size: u8,
    pub type_: u32,
    bytes_per_vertex: u8,
    pub normalized: bool,
    pub stride: u8,
    pub offset: u32,
    pub buffer: Option<Dom<WebGLBuffer>>,
    pub divisor: u32,
}

impl Default for VertexAttribData {
    #[allow(crown::unrooted_must_root)]
    fn default() -> Self {
        Self {
            enabled_as_array: false,
            size: 4,
            type_: constants::FLOAT,
            bytes_per_vertex: 16,
            normalized: false,
            stride: 0,
            offset: 0,
            buffer: None,
            divisor: 0,
        }
    }
}

impl VertexAttribData {
    pub fn buffer(&self) -> Option<&WebGLBuffer> {
        self.buffer.as_deref()
    }

    pub fn max_vertices(&self) -> u32 {
        let capacity = (self.buffer().unwrap().capacity() as u32).saturating_sub(self.offset);
        if capacity < self.bytes_per_vertex as u32 {
            0
        } else if self.stride == 0 {
            capacity / self.bytes_per_vertex as u32
        } else if self.stride < self.bytes_per_vertex {
            (capacity - (self.bytes_per_vertex - self.stride) as u32) / self.stride as u32
        } else {
            let mut max = capacity / self.stride as u32;
            if capacity % self.stride as u32 >= self.bytes_per_vertex as u32 {
                max += 1;
            }
            max
        }
    }
}
