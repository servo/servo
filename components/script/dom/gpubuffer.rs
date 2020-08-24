/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::GPUBufferBinding::{GPUBufferMethods, GPUSize64};
use crate::dom::bindings::codegen::Bindings::GPUMapModeBinding::GPUMapModeConstants;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::reflector::DomObject;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::USVString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::gpu::{response_async, AsyncWGPUListener};
use crate::dom::gpudevice::GPUDevice;
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
use std::string::String;
use webgpu::{
    identity::WebGPUOpResult, wgpu::device::HostMap, WebGPU, WebGPUBuffer, WebGPURequest,
    WebGPUResponse, WebGPUResponseResult,
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
    label: DomRefCell<Option<USVString>>,
    state: Cell<GPUBufferState>,
    buffer: WebGPUBuffer,
    device: Dom<GPUDevice>,
    size: GPUSize64,
    #[ignore_malloc_size_of = "promises are hard"]
    map_promise: DomRefCell<Option<Rc<Promise>>>,
    map_info: DomRefCell<Option<GPUBufferMapInfo>>,
}

impl GPUBuffer {
    fn new_inherited(
        channel: WebGPU,
        buffer: WebGPUBuffer,
        device: &GPUDevice,
        state: GPUBufferState,
        size: GPUSize64,
        map_info: DomRefCell<Option<GPUBufferMapInfo>>,
        label: Option<USVString>,
    ) -> Self {
        Self {
            reflector_: Reflector::new(),
            channel,
            label: DomRefCell::new(label),
            state: Cell::new(state),
            device: Dom::from_ref(device),
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
        device: &GPUDevice,
        state: GPUBufferState,
        size: GPUSize64,
        map_info: DomRefCell<Option<GPUBufferMapInfo>>,
        label: Option<USVString>,
    ) -> DomRoot<Self> {
        reflect_dom_object(
            Box::new(GPUBuffer::new_inherited(
                channel, buffer, device, state, size, map_info, label,
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
                if let Err(e) = self.channel.0.send((
                    self.device.use_current_scope(),
                    WebGPURequest::UnmapBuffer {
                        buffer_id: self.id().0,
                        device_id: self.device.id().0,
                        array_buffer: IpcSharedMemory::from_bytes(
                            m_info.mapping.borrow().as_slice(),
                        ),
                        is_map_read: m_info.map_mode == Some(GPUMapModeConstants::READ),
                        offset: m_range.start,
                        size: m_range.end - m_range.start,
                    },
                )) {
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
            GPUBufferState::Destroyed => return,
            _ => {},
        };
        if let Err(e) = self
            .channel
            .0
            .send((None, WebGPURequest::DestroyBuffer(self.buffer.0)))
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
        size: Option<GPUSize64>,
        comp: InRealm,
    ) -> Rc<Promise> {
        let promise = Promise::new_in_current_realm(&self.global(), comp);
        let range_size = if let Some(s) = size {
            s
        } else if offset >= self.size {
            promise.reject_error(Error::Operation);
            return promise;
        } else {
            self.size - offset
        };
        let scope_id = self.device.use_current_scope();
        if self.state.get() != GPUBufferState::Unmapped {
            self.device.handle_server_msg(
                scope_id,
                WebGPUOpResult::ValidationError(String::from("Buffer is not Unmapped")),
            );
            promise.reject_error(Error::Abort);
            return promise;
        }
        let host_map = match mode {
            GPUMapModeConstants::READ => HostMap::Read,
            GPUMapModeConstants::WRITE => HostMap::Write,
            _ => {
                self.device.handle_server_msg(
                    scope_id,
                    WebGPUOpResult::ValidationError(String::from("Invalid MapModeFlags")),
                );
                promise.reject_error(Error::Abort);
                return promise;
            },
        };

        let map_range = offset..offset + range_size;

        let sender = response_async(&promise, self);
        if let Err(e) = self.channel.0.send((
            scope_id,
            WebGPURequest::BufferMapAsync {
                sender,
                buffer_id: self.buffer.0,
                device_id: self.device.id().0,
                host_map,
                map_range: map_range.clone(),
            },
        )) {
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
        size: Option<GPUSize64>,
    ) -> Fallible<NonNull<JSObject>> {
        let range_size = if let Some(s) = size {
            s
        } else if offset >= self.size {
            return Err(Error::Operation);
        } else {
            self.size - offset
        };
        let m_end = offset + range_size;
        let mut info = self.map_info.borrow_mut();
        let m_info = info.as_mut().unwrap();

        let mut valid = match self.state.get() {
            GPUBufferState::Mapped | GPUBufferState::MappedAtCreation => true,
            _ => false,
        };
        valid &= offset % RANGE_OFFSET_ALIGN_MASK == 0 &&
            range_size % RANGE_SIZE_ALIGN_MASK == 0 &&
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
                range_size as usize,
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
    fn GetLabel(&self) -> Option<USVString> {
        self.label.borrow().clone()
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label
    fn SetLabel(&self, value: Option<USVString>) {
        *self.label.borrow_mut() = value;
    }
}

impl AsyncWGPUListener for GPUBuffer {
    #[allow(unsafe_code)]
    fn handle_response(&self, response: WebGPUResponseResult, promise: &Rc<Promise>) {
        match response {
            Ok(WebGPUResponse::BufferMapAsync(bytes)) => {
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
            Err(e) => {
                warn!("Could not map buffer({:?})", e);
                promise.reject_error(Error::Abort);
            },
            _ => {
                warn!("GPUBuffer received wrong WebGPUResponse");
                promise.reject_error(Error::Operation);
            },
        }
        *self.map_promise.borrow_mut() = None;
        if let Err(e) = self
            .channel
            .0
            .send((None, WebGPURequest::BufferMapComplete(self.buffer.0)))
        {
            warn!(
                "Failed to send BufferMapComplete({:?}) ({})",
                self.buffer.0, e
            );
        }
    }
}
