/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::cell::DomRefCell;
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
use ipc_channel::ipc::IpcSharedMemory;
use js::jsapi::DetachArrayBuffer;
use js::jsapi::NewExternalArrayBuffer;
use js::jsapi::{Heap, JSObject};
use std::cell::{Cell, RefCell};
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

#[derive(JSTraceable, MallocSizeOf)]
pub struct GPUBufferMapInfo {
    #[ignore_malloc_size_of = "Rc"]
    pub mapping: Rc<RefCell<Vec<u8>>>,
    pub mapping_range: Range<u64>,
    pub mapped_ranges: Vec<Range<u64>>,
    #[ignore_malloc_size_of = "defined in mozjs"]
    pub js_buffers: Vec<Box<Heap<*mut JSObject>>>,
    pub map_mode: Option<u32>,
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
    size: GPUSize64,
    #[ignore_malloc_size_of = "promises are hard"]
    map_promise: DomRefCell<Option<Rc<Promise>>>,
    map_info: DomRefCell<Option<GPUBufferMapInfo>>,
}

impl GPUBuffer {
    fn new_inherited(
        channel: WebGPU,
        buffer: WebGPUBuffer,
        device: WebGPUDevice,
        state: GPUBufferState,
        size: GPUSize64,
        valid: bool,
        map_info: DomRefCell<Option<GPUBufferMapInfo>>,
    ) -> Self {
        Self {
            reflector_: Reflector::new(),
            channel,
            label: DomRefCell::new(None),
            state: Cell::new(state),
            valid: Cell::new(valid),
            device,
            buffer,
            map_promise: DomRefCell::new(None),
            size,
            map_info,
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
        map_info: DomRefCell<Option<GPUBufferMapInfo>>,
    ) -> DomRoot<Self> {
        reflect_dom_object(
            Box::new(GPUBuffer::new_inherited(
                channel, buffer, device, state, size, valid, map_info,
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
                let mut info = self.map_info.borrow_mut();
                let m_info = info.as_mut().unwrap();
                let m_range = m_info.mapping_range.clone();
                if let Err(e) = self.channel.0.send(WebGPURequest::UnmapBuffer {
                    buffer_id: self.id().0,
                    array_buffer: IpcSharedMemory::from_bytes(m_info.mapping.borrow().as_slice()),
                    is_map_read: m_info.map_mode == Some(GPUMapModeConstants::READ),
                    offset: m_range.start,
                    size: m_range.end - m_range.start,
                }) {
                    warn!("Failed to send Buffer unmap ({:?}) ({})", self.buffer.0, e);
                }
                // Step 3.3
                m_info.js_buffers.drain(..).for_each(|obj| unsafe {
                    DetachArrayBuffer(*cx, obj.handle());
                });
            },
            // Step 2
            GPUBufferState::MappingPending => {
                let promise = self.map_promise.borrow_mut().take().unwrap();
                promise.reject_error(Error::Operation);
            },
        };
        // Step 4
        self.state.set(GPUBufferState::Unmapped);
        *self.map_info.borrow_mut() = None;
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
        *self.map_info.borrow_mut() = Some(GPUBufferMapInfo {
            mapping: Rc::new(RefCell::new(Vec::with_capacity(0))),
            mapping_range: map_range,
            mapped_ranges: Vec::new(),
            js_buffers: Vec::new(),
            map_mode: Some(mode),
        });
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
        let m_end = if size == 0 { self.size } else { offset + size };
        let mut info = self.map_info.borrow_mut();
        let m_info = info.as_mut().unwrap();

        let mut valid = match self.state.get() {
            GPUBufferState::Mapped | GPUBufferState::MappedAtCreation => true,
            _ => false,
        };
        valid &= offset % RANGE_OFFSET_ALIGN_MASK == 0 &&
            (m_end - offset) % RANGE_SIZE_ALIGN_MASK == 0 &&
            offset >= m_info.mapping_range.start &&
            m_end <= m_info.mapping_range.end;
        valid &= m_info
            .mapped_ranges
            .iter()
            .all(|range| range.start >= m_end || range.end <= offset);
        if !valid {
            return Err(Error::Operation);
        }

        unsafe extern "C" fn free_func(_contents: *mut c_void, free_user_data: *mut c_void) {
            let _ = Rc::from_raw(free_user_data as _);
        }

        let array_buffer = unsafe {
            NewExternalArrayBuffer(
                *cx,
                (m_end - offset) as usize,
                m_info.mapping.borrow_mut()[offset as usize..m_end as usize].as_mut_ptr() as _,
                Some(free_func),
                Rc::into_raw(m_info.mapping.clone()) as _,
            )
        };

        m_info.mapped_ranges.push(offset..m_end);
        m_info.js_buffers.push(Heap::boxed(array_buffer));

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
                *self
                    .map_info
                    .borrow_mut()
                    .as_mut()
                    .unwrap()
                    .mapping
                    .borrow_mut() = bytes.to_vec();
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
