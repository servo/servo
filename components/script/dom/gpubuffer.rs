/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::cell::{DomRefCell, RefCell};
use crate::dom::bindings::codegen::Bindings::GPUBufferBinding::{GPUBufferMethods, GPUSize64};
use crate::dom::bindings::codegen::Bindings::GPUMapModeBinding::GPUMapModeConstants;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::reflector::DomObject;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::gpu::{response_async, AsyncWGPUListener};
use crate::dom::promise::Promise;
use crate::realms::InRealm;
use crate::script_runtime::JSContext;
use dom_struct::dom_struct;
use js::jsapi::DetachArrayBuffer;
use js::jsapi::NewExternalArrayBuffer;
use js::jsapi::{Heap, JSObject};
use std::cell::Cell;
use std::ffi::c_void;
use std::ops::Range;
use std::ptr::NonNull;
use std::rc::Rc;
use webgpu::{
    wgpu::device::HostMap, WebGPU, WebGPUBuffer, WebGPUDevice, WebGPURequest, WebGPUResponse,
};

const RANGE_OFFSET_ALIGN_MASK: u64 = 8;
const RANGE_SIZE_ALIGN_MASK: u64 = 4;

// https://gpuweb.github.io/gpuweb/#buffer-state
#[derive(Clone, Copy, MallocSizeOf, PartialEq)]
pub enum GPUBufferState {
    Mapped,
    MappedAtCreation,
    MappingPending,
    Unmapped,
    Destroyed,
}

#[dom_struct]
pub struct GPUBuffer {
    reflector_: Reflector,
    #[ignore_malloc_size_of = "defined in webgpu"]
    channel: WebGPU,
    label: DomRefCell<Option<DOMString>>,
    state: Cell<GPUBufferState>,
    buffer: WebGPUBuffer,
    device: WebGPUDevice,
    valid: Cell<bool>,
    #[ignore_malloc_size_of = "defined in mozjs"]
    mapping: Rc<RefCell<Option<Vec<u8>>>>,
    mapping_range: DomRefCell<Option<Range<u64>>>,
    mapped_ranges: DomRefCell<Option<Vec<Range<u64>>>>,
    #[ignore_malloc_size_of = "defined in mozjs"]
    js_buffers: DomRefCell<Option<Vec<Box<Heap<*mut JSObject>>>>>,
    #[ignore_malloc_size_of = "defined in mozjs"]
    map_promise: DomRefCell<Option<Rc<Promise>>>,
    size: GPUSize64,
    map_mode: Cell<Option<u32>>,
}

impl GPUBuffer {
    fn new_inherited(
        channel: WebGPU,
        buffer: WebGPUBuffer,
        device: WebGPUDevice,
        state: GPUBufferState,
        size: GPUSize64,
        valid: bool,
        mapping: Rc<RefCell<Option<Vec<u8>>>>,
        mapping_range: Range<u64>,
    ) -> Self {
        Self {
            reflector_: Reflector::new(),
            channel,
            label: DomRefCell::new(None),
            state: Cell::new(state),
            valid: Cell::new(valid),
            device,
            buffer,
            mapping,
            mapped_ranges: DomRefCell::new(None),
            js_buffers: DomRefCell::new(None),
            map_promise: DomRefCell::new(None),
            size,
            mapping_range: DomRefCell::new(Some(mapping_range)),
            map_mode: Cell::new(None),
        }
    }

    #[allow(unsafe_code)]
    pub fn new(
        global: &GlobalScope,
        channel: WebGPU,
        buffer: WebGPUBuffer,
        device: WebGPUDevice,
        state: GPUBufferState,
        size: GPUSize64,
        valid: bool,
        mapping: Rc<RefCell<Option<Vec<u8>>>>,
        mapping_range: Range<u64>,
    ) -> DomRoot<Self> {
        reflect_dom_object(
            Box::new(GPUBuffer::new_inherited(
                channel,
                buffer,
                device,
                state,
                size,
                valid,
                mapping,
                mapping_range,
            )),
            global,
        )
    }
}

impl GPUBuffer {
    pub fn id(&self) -> WebGPUBuffer {
        self.buffer
    }

    pub fn state(&self) -> GPUBufferState {
        self.state.get()
    }

    pub fn is_valid(&self) -> bool {
        self.valid.get()
    }
}

impl Drop for GPUBuffer {
    fn drop(&mut self) {
        self.Destroy()
    }
}

impl GPUBufferMethods for GPUBuffer {
    #[allow(unsafe_code)]
    /// https://gpuweb.github.io/gpuweb/#dom-gpubuffer-unmap
    fn Unmap(&self) {
        let cx = self.global().get_cx();
        // Step 1
        match self.state.get() {
            GPUBufferState::Unmapped | GPUBufferState::Destroyed => {
                // TODO: Record validation error on the current scope
                return;
            },
            // Step 3
            GPUBufferState::Mapped | GPUBufferState::MappedAtCreation => {
                if let Err(e) = self.channel.0.send(WebGPURequest::UnmapBuffer {
                    buffer_id: self.id().0,
                    array_buffer: self.mapping.borrow().as_ref().unwrap().clone(),
                    is_map_read: self.map_mode.get() == Some(GPUMapModeConstants::READ),
                    offset: self.mapping_range.borrow().as_ref().unwrap().start,
                    size: self.mapping_range.borrow().as_ref().unwrap().end -
                        self.mapping_range.borrow().as_ref().unwrap().start,
                }) {
                    warn!("Failed to send Buffer unmap ({:?}) ({})", self.buffer.0, e);
                }
                // Step 3.3
                let mut bufs = self.js_buffers.borrow_mut().take().unwrap();
                bufs.drain(..).for_each(|obj| unsafe {
                    DetachArrayBuffer(*cx, obj.handle());
                });
                *self.mapped_ranges.borrow_mut() = None;
                *self.mapping.borrow_mut() = None;
            },
            // Step 2
            GPUBufferState::MappingPending => {
                let promise = self.map_promise.borrow_mut().take().unwrap();
                promise.reject_error(Error::Operation);
            },
        };
        // Step 4
        self.state.set(GPUBufferState::Unmapped);
        self.map_mode.set(None);
        *self.mapping_range.borrow_mut() = None;
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpubuffer-destroy
    fn Destroy(&self) {
        let state = self.state.get();
        match state {
            GPUBufferState::Mapped | GPUBufferState::MappedAtCreation => {
                self.Unmap();
            },
            _ => {},
        };
        if let Err(e) = self
            .channel
            .0
            .send(WebGPURequest::DestroyBuffer(self.buffer.0))
        {
            warn!(
                "Failed to send WebGPURequest::DestroyBuffer({:?}) ({})",
                self.buffer.0, e
            );
        };
        self.state.set(GPUBufferState::Destroyed);
    }

    #[allow(unsafe_code)]
    /// https://gpuweb.github.io/gpuweb/#dom-gpubuffer-mapasync-offset-size
    fn MapAsync(
        &self,
        mode: u32,
        offset: GPUSize64,
        size: GPUSize64,
        comp: InRealm,
    ) -> Rc<Promise> {
        let promise = Promise::new_in_current_realm(&self.global(), comp);
        let map_range = if size == 0 {
            offset..self.size
        } else {
            if offset + size > self.size {
                warn!("Requested mapping size is greated than buffer size");
                promise.reject_error(Error::Abort);
                return promise;
            }
            offset..offset + size
        };
        let host_map = match mode {
            GPUMapModeConstants::READ => HostMap::Read,
            GPUMapModeConstants::WRITE => HostMap::Write,
            _ => {
                promise.reject_error(Error::Abort);
                return promise;
            },
        };
        if self.state.get() != GPUBufferState::Unmapped {
            promise.reject_error(Error::Abort);
            return promise;
        }

        let sender = response_async(&promise, self);
        if let Err(e) = self.channel.0.send(WebGPURequest::BufferMapAsync {
            sender,
            buffer_id: self.buffer.0,
            host_map,
            map_range: map_range.clone(),
        }) {
            warn!(
                "Failed to send BufferMapAsync ({:?}) ({})",
                self.buffer.0, e
            );
            promise.reject_error(Error::Operation);
            return promise;
        }

        self.state.set(GPUBufferState::MappingPending);
        self.map_mode.set(Some(mode));
        *self.mapping_range.borrow_mut() = Some(map_range);
        *self.map_promise.borrow_mut() = Some(promise.clone());
        promise
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpubuffer-getmappedrange
    #[allow(unsafe_code)]
    fn GetMappedRange(
        &self,
        cx: JSContext,
        offset: GPUSize64,
        size: GPUSize64,
    ) -> Fallible<NonNull<JSObject>> {
        if self.mapped_ranges.borrow().is_none() {
            *self.mapped_ranges.borrow_mut() = Some(Vec::new());
        }
        if self.js_buffers.borrow().is_none() {
            *self.js_buffers.borrow_mut() = Some(Vec::new());
        }
        let act_size = if size == 0 { self.size - offset } else { size };
        let mut valid = match self.state.get() {
            GPUBufferState::Mapped | GPUBufferState::MappedAtCreation => true,
            _ => false,
        };
        valid &= offset % RANGE_OFFSET_ALIGN_MASK == 0 &&
            act_size % RANGE_SIZE_ALIGN_MASK == 0 &&
            offset >= self.mapping_range.borrow().as_ref().unwrap().start &&
            offset + act_size <= self.mapping_range.borrow().as_ref().unwrap().end;
        valid &= self
            .mapped_ranges
            .borrow()
            .as_ref()
            .unwrap()
            .iter()
            .all(|range| range.start > offset + act_size || range.end < offset);
        if !valid {
            return Err(Error::Operation);
        }

        unsafe extern "C" fn free_func(_contents: *mut c_void, free_user_data: *mut c_void) {
            let _ = Rc::from_raw(free_user_data as _);
        }

        let array_buffer = unsafe {
            NewExternalArrayBuffer(
                *cx,
                act_size as usize,
                self.mapping.borrow_mut().as_mut().unwrap()
                    [offset as usize..(offset + act_size) as usize]
                    .as_mut_ptr() as _,
                Some(free_func),
                Rc::into_raw(self.mapping.clone()) as _,
            )
        };
        self.mapped_ranges
            .borrow_mut()
            .as_mut()
            .map(|v| v.push(offset..offset + act_size));
        self.js_buffers
            .borrow_mut()
            .as_mut()
            .map(|a| a.push(Heap::boxed(array_buffer)));

        Ok(NonNull::new(array_buffer).unwrap())
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label
    fn GetLabel(&self) -> Option<DOMString> {
        self.label.borrow().clone()
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label
    fn SetLabel(&self, value: Option<DOMString>) {
        *self.label.borrow_mut() = value;
    }
}

impl AsyncWGPUListener for GPUBuffer {
    #[allow(unsafe_code)]
    fn handle_response(&self, response: WebGPUResponse, promise: &Rc<Promise>) {
        match response {
            WebGPUResponse::BufferMapAsync(bytes) => {
                *self.mapping.borrow_mut() = Some(bytes);
                promise.resolve_native(&());
                self.state.set(GPUBufferState::Mapped);
            },
            _ => {
                warn!("Wrong WebGPUResponse received");
                promise.reject_error(Error::Operation);
            },
        }
        *self.map_promise.borrow_mut() = None;
        if let Err(e) = self
            .channel
            .0
            .send(WebGPURequest::BufferMapComplete(self.buffer.0))
        {
            warn!(
                "Failed to send BufferMapComplete({:?}) ({})",
                self.buffer.0, e
            );
        }
    }
}
