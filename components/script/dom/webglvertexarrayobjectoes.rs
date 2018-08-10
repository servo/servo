/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use canvas_traits::webgl::{ActiveAttribInfo, WebGLCommand, WebGLError, WebGLResult, WebGLVertexArrayId};
use dom::bindings::cell::DomRefCell;
use dom::bindings::codegen::Bindings::WebGLRenderingContextBinding::WebGLRenderingContextConstants as constants;
use dom::bindings::codegen::Bindings::WebGLVertexArrayObjectOESBinding;
use dom::bindings::inheritance::Castable;
use dom::bindings::reflector::{DomObject, reflect_dom_object};
use dom::bindings::root::{Dom, DomRoot, MutNullableDom};
use dom::webglbuffer::WebGLBuffer;
use dom::webglobject::WebGLObject;
use dom::webglrenderingcontext::WebGLRenderingContext;
use dom_struct::dom_struct;
use ref_filter_map::ref_filter_map;
use std::cell::{Cell, Ref};
use typeholder::TypeHolderTrait;

#[dom_struct]
pub struct WebGLVertexArrayObjectOES<TH: TypeHolderTrait> {
    webgl_object_: WebGLObject<TH>,
    id: Option<WebGLVertexArrayId>,
    ever_bound: Cell<bool>,
    is_deleted: Cell<bool>,
    vertex_attribs: DomRefCell<Box<[VertexAttribData<TH>]>>,
    element_array_buffer: MutNullableDom<WebGLBuffer<TH>>,
}

impl<TH: TypeHolderTrait> WebGLVertexArrayObjectOES<TH> {
    fn new_inherited(context: &WebGLRenderingContext<TH>, id: Option<WebGLVertexArrayId>) -> Self {
    let max_vertex_attribs = context.limits().max_vertex_attribs as usize;    
    Self {
            webgl_object_: WebGLObject::new_inherited(context),
            id,
            ever_bound: Default::default(),
            is_deleted: Default::default(),
            vertex_attribs: DomRefCell::new(vec![Default::default(); max_vertex_attribs].into()),
            element_array_buffer: Default::default(),
        }
    }

    pub fn new(context: &WebGLRenderingContext<TH>, id: Option<WebGLVertexArrayId>) -> DomRoot<Self> {
        reflect_dom_object(
            Box::new(WebGLVertexArrayObjectOES::new_inherited(context, id)),
            &*context.global(),
            WebGLVertexArrayObjectOESBinding::Wrap,
        )
    }

    pub fn id(&self) -> Option<WebGLVertexArrayId> {
        self.id
    }

    pub fn is_deleted(&self) -> bool {
        self.is_deleted.get()
    }

    pub fn delete(&self) {
        assert!(self.id.is_some());
        if self.is_deleted.get() {
            return;
        }
        self.is_deleted.set(true);

        self.upcast::<WebGLObject<TH>>()
            .context()
            .send_command(WebGLCommand::DeleteVertexArray(self.id.unwrap()));

        for attrib_data in &**self.vertex_attribs.borrow() {
            if let Some(buffer) = attrib_data.buffer() {
                buffer.decrement_attached_counter();
            }
        }
        if let Some(buffer) = self.element_array_buffer.get() {
            buffer.decrement_attached_counter();
        }
    }

    pub fn ever_bound(&self) -> bool {
        return self.ever_bound.get()
    }

    pub fn set_ever_bound(&self) {
        self.ever_bound.set(true);
    }

    pub fn element_array_buffer(&self) -> &MutNullableDom<WebGLBuffer<TH>> {
        &self.element_array_buffer
    }

    pub fn get_vertex_attrib(&self, index: u32) -> Option<Ref<VertexAttribData<TH>>> {
        ref_filter_map(self.vertex_attribs.borrow(), |attribs| attribs.get(index as usize))
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
        let data = attribs.get_mut(index as usize).ok_or(WebGLError::InvalidValue)?;

        if size < 1 || size > 4 {
            return Err(WebGLError::InvalidValue);
        }

        // https://www.khronos.org/registry/webgl/specs/latest/1.0/#BUFFER_OFFSET_AND_STRIDE
        // https://www.khronos.org/registry/webgl/specs/latest/1.0/#VERTEX_STRIDE
        if stride < 0 || stride > 255 || offset < 0 {
            return Err(WebGLError::InvalidValue);
        }
        let bytes_per_component: i32 = match type_ {
            constants::BYTE | constants::UNSIGNED_BYTE => 1,
            constants::SHORT | constants::UNSIGNED_SHORT => 2,
            constants::FLOAT => 4,
            _ => return Err(WebGLError::InvalidEnum),
        };
        if offset % bytes_per_component as i64 > 0 || stride % bytes_per_component > 0 {
            return Err(WebGLError::InvalidOperation);
        }

        let context = self.upcast::<WebGLObject<TH>>().context();
        let buffer = context.array_buffer().ok_or(WebGLError::InvalidOperation)?;
        buffer.increment_attached_counter();
        context.send_command(WebGLCommand::VertexAttribPointer(
            index,
            size,
            type_,
            normalized,
            stride,
            offset as u32,
        ));
        if let Some(old) = data.buffer() {
            old.decrement_attached_counter();
        }

        *data = VertexAttribData {
            enabled_as_array: data.enabled_as_array,
            size: size as u8,
            type_,
            bytes_per_vertex: size as u8 * bytes_per_component as u8,
            normalized,
            stride: stride as u8,
            offset: offset as u32,
            buffer: Some(Dom::from_ref(&*buffer)),
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

    pub fn unbind_buffer(&self, buffer: &WebGLBuffer<TH>) {
        for attrib in &mut **self.vertex_attribs.borrow_mut() {
            if let Some(b) = attrib.buffer() {
                if b.id() != buffer.id() {
                    continue;
                }
                b.decrement_attached_counter();
            }
            attrib.buffer = None;
        }
        if self.element_array_buffer.get().map_or(false, |b| buffer == &*b) {
            buffer.decrement_attached_counter();
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
        if attribs.iter().any(|data| data.enabled_as_array && data.buffer.is_none()) {
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
                } else if max_vertices.checked_mul(attrib.divisor).map_or(false, |v| v < instance_count) {
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

impl<TH: TypeHolderTrait> Drop for WebGLVertexArrayObjectOES<TH> {
    fn drop(&mut self) {
        if self.id.is_some() {
            self.delete();
        }
    }
}

#[derive(Clone, JSTraceable, MallocSizeOf)]
#[must_root]
pub struct VertexAttribData<TH: TypeHolderTrait> {
    pub enabled_as_array: bool,
    pub size: u8,
    pub type_: u32,
    bytes_per_vertex: u8,
    pub normalized: bool,
    pub stride: u8,
    pub offset: u32,
    pub buffer: Option<Dom<WebGLBuffer<TH>>>,
    pub divisor: u32,
}

impl<TH: TypeHolderTrait> Default for VertexAttribData<TH> {
    #[allow(unrooted_must_root)]
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

impl<TH: TypeHolderTrait> VertexAttribData<TH> {
    pub fn buffer(&self) -> Option<&WebGLBuffer<TH>> {
        self.buffer.as_ref().map(|b| &**b)
    }

    fn max_vertices(&self) -> u32 {
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
