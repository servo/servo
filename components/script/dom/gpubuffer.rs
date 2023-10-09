/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::{Cell, RefCell};
use std::ffi::c_void;
use std::ops::Range;
use std::ptr::NonNull;
use std::rc::Rc;
use std::string::String;

use dom_struct::dom_struct;
use ipc_channel::ipc::IpcSharedMemory;
use js::jsapi::{DetachArrayBuffer, Heap, JSObject, NewExternalArrayBuffer};
use webgpu::identity::WebGPUOpResult;
use webgpu::wgpu::device::HostMap;
use webgpu::{wgt, WebGPU, WebGPUBuffer, WebGPURequest, WebGPUResponse, WebGPUResponseResult};

use super::bindings::codegen::Bindings::WebGPUBinding::{
    GPUBufferMapState, GPUFlagsConstant, GPUMapModeFlags,
};
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::WebGPUBinding::{
    GPUBufferMethods, GPUMapModeConstants, GPUSize64,
};
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::USVString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::gpu::{response_async, AsyncWGPUListener};
use crate::dom::gpudevice::GPUDevice;
use crate::dom::promise::Promise;
use crate::realms::InRealm;
use crate::script_runtime::JSContext;

const RANGE_OFFSET_ALIGN_MASK: u64 = 8;
const RANGE_SIZE_ALIGN_MASK: u64 = 4;

// https://gpuweb.github.io/gpuweb/#buffer-internals-state
#[derive(Clone, Copy, JSTraceable, MallocSizeOf, PartialEq)]
pub enum GPUBufferState {
    Available,
    Unavailable,
    Destroyed,
}

#[derive(JSTraceable, MallocSizeOf)]
pub struct GPUArrayBuffer {
    pub range: Range<u64>,
    #[ignore_malloc_size_of = "defined in mozjs"]
    pub js_buffer: Box<Heap<*mut JSObject>>,
}

#[derive(JSTraceable, MallocSizeOf)]
pub struct GPUBufferMapInfo {
    /// <https://gpuweb.github.io/gpuweb/#active-buffer-mapping-data>
    #[ignore_malloc_size_of = "Rc"]
    pub data: Rc<RefCell<Vec<u8>>>,
    /// <https://gpuweb.github.io/gpuweb/#active-buffer-mapping-range>
    pub range: Range<u64>,
    /// <https://gpuweb.github.io/gpuweb/#active-buffer-mapping-views>
    pub views: Vec<GPUArrayBuffer>,
    /// <https://gpuweb.github.io/gpuweb/#active-buffer-mapping-mode>
    pub mode: GPUMapModeFlags,
}

impl GPUBufferMapInfo {
    // https://gpuweb.github.io/gpuweb/#abstract-opdef-initialize-an-active-buffer-mapping
    pub fn new(mode: GPUMapModeFlags, range: Range<u64>) -> Self {
        Self {
            data: Rc::new(RefCell::new(Vec::with_capacity(0))),
            range,
            views: Vec::new(),
            mode,
        }
    }
}

#[dom_struct]
pub struct GPUBuffer {
    reflector_: Reflector,
    #[ignore_malloc_size_of = "defined in webgpu"]
    #[no_trace]
    channel: WebGPU,
    label: DomRefCell<USVString>,
    state: Cell<GPUBufferState>,
    #[no_trace]
    buffer: WebGPUBuffer,
    device: Dom<GPUDevice>,
    size: GPUSize64,
    usage: GPUFlagsConstant,
    /// <https://gpuweb.github.io/gpuweb/#dom-gpubuffer-pending_map-slot>
    #[ignore_malloc_size_of = "promises are hard"]
    pending_map: DomRefCell<Option<Rc<Promise>>>,
    /// <https://gpuweb.github.io/gpuweb/#dom-gpubuffer-mapping-slot>
    mapping: DomRefCell<Option<GPUBufferMapInfo>>,
}

impl GPUBuffer {
    fn new_inherited(
        channel: WebGPU,
        buffer: WebGPUBuffer,
        device: &GPUDevice,
        state: GPUBufferState,
        size: GPUSize64,
        usage: GPUFlagsConstant,
        map_info: DomRefCell<Option<GPUBufferMapInfo>>,
        label: USVString,
    ) -> Self {
        Self {
            reflector_: Reflector::new(),
            channel,
            label: DomRefCell::new(label),
            state: Cell::new(state),
            device: Dom::from_ref(device),
            buffer,
            pending_map: DomRefCell::new(None),
            size,
            usage,
            mapping: map_info,
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
        usage: GPUFlagsConstant,
        map_info: DomRefCell<Option<GPUBufferMapInfo>>,
        label: USVString,
    ) -> DomRoot<Self> {
        reflect_dom_object(
            Box::new(GPUBuffer::new_inherited(
                channel, buffer, device, state, size, usage, map_info, label,
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
        let cx = GlobalScope::get_cx();
        // Step 1
        if let Some(promise) = self.pending_map.borrow_mut().take() {
            promise.reject_error(Error::Abort);
        }
        let mut info = self.mapping.borrow_mut().take();
        // Step 2
        if let Some(m_info) = info.as_mut() {
            // Step 3
            m_info.views.drain(..).for_each(|obj| unsafe {
                DetachArrayBuffer(*cx, obj.js_buffer.handle());
            });
            // Step 5&7
            let m_range = m_info.range.clone();
            if let Err(e) = self.channel.0.send((
                self.device.use_current_scope(),
                WebGPURequest::UnmapBuffer {
                    buffer_id: self.id().0,
                    device_id: self.device.id().0,
                    array_buffer: IpcSharedMemory::from_bytes(m_info.data.borrow().as_slice()),
                    is_write: m_info.mode >= GPUMapModeConstants::WRITE,
                    offset: m_range.start,
                    size: m_range.end - m_range.start,
                },
            )) {
                warn!("Failed to send Buffer unmap ({:?}) ({})", self.buffer.0, e);
            } else {
                // TODO(wpu): is this after response???
                self.state.set(GPUBufferState::Available);
            }
        } else {
            return;
        }
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpubuffer-destroy
    fn Destroy(&self) {
        // Step 1
        self.Unmap();
        // Step 2
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
        // Step 2
        self.state.set(GPUBufferState::Destroyed);
    }

    #[allow(unsafe_code)]
    /// https://gpuweb.github.io/gpuweb/#dom-gpubuffer-mapasync
    fn MapAsync(
        &self,
        mode: u32,
        offset: GPUSize64,
        size: Option<GPUSize64>,
        comp: InRealm,
    ) -> Rc<Promise> {
        let promise = Promise::new_in_current_realm(comp);
        // Step 2
        if self.pending_map.borrow().is_some() {
            promise.reject_error(Error::Operation);
            return promise;
        }
        // This checks should be part of device timeline (webgpu thread), but we do them here
        {
            // Step 1
            let range_size = if let Some(s) = size {
                s
            } else if offset >= self.size {
                promise.reject_error(Error::Operation);
                return promise;
            } else {
                self.size - offset
            };
            let scope_id = self.device.use_current_scope();
            if self.state.get() != GPUBufferState::Available {
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

            let range = offset..offset + range_size;

            let sender = response_async(&promise, self);
            if let Err(e) = self.channel.0.send((
                scope_id,
                WebGPURequest::BufferMapAsync {
                    sender,
                    buffer_id: self.buffer.0,
                    device_id: self.device.id().0,
                    host_map,
                    map_range: range.clone(),
                },
            )) {
                warn!(
                    "Failed to send BufferMapAsync ({:?}) ({})",
                    self.buffer.0, e
                );
                promise.reject_error(Error::Operation);
                return promise;
            }

            self.state.set(GPUBufferState::Unavailable);
            //*self.mapping.borrow_mut() = Some(GPUBufferMapInfo::new(mode, range));
            // content timeline continues down in async wgpu handler
        }
        // Step 4
        *self.pending_map.borrow_mut() = Some(promise.clone());
        // Step 6
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
        // Step 1
        let range_size = if let Some(s) = size {
            s
        } else if offset >= self.size {
            return Err(Error::Operation);
        } else {
            self.size - offset
        };
        let range = offset..offset + range_size;
        // Step 2: validation
        let mut info = self.mapping.borrow_mut();
        if let Some(info) = info.as_mut() {
            let mut valid = offset % wgt::MAP_ALIGNMENT == 0 &&
                range_size % wgt::COPY_BUFFER_ALIGNMENT == 0 &&
                range.start >= info.range.start &&
                range.end <= info.range.end;
            // does not overlap
            valid &= info.views.iter().all(|arr_buf| {
                arr_buf.range.start <= range.end && range.start <= arr_buf.range.end
            });
            if !valid {
                return Err(Error::Operation);
            }

            // Step 4
            unsafe extern "C" fn free_func(_contents: *mut c_void, free_user_data: *mut c_void) {
                drop(Rc::from_raw(free_user_data as _));
            }

            let array_buffer = unsafe {
                NewExternalArrayBuffer(
                    *cx,
                    range_size as usize,
                    info.data.borrow_mut()[range.start as usize..range.end as usize].as_mut_ptr()
                        as _,
                    Some(free_func),
                    Rc::into_raw(info.data.clone()) as _,
                )
            };

            // Step 6
            info.views.push(GPUArrayBuffer {
                range,
                js_buffer: Heap::boxed(array_buffer),
            });

            // Step 7
            Ok(NonNull::new(array_buffer).unwrap())
        } else {
            return Err(Error::Operation);
        }
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label
    fn Label(&self) -> USVString {
        self.label.borrow().clone()
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label
    fn SetLabel(&self, value: USVString) {
        *self.label.borrow_mut() = value;
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpubuffer-size
    fn Size(&self) -> GPUSize64 {
        self.size
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpubuffer-usage
    fn Usage(&self) -> GPUFlagsConstant {
        self.usage
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpubuffer-mapstate
    fn MapState(&self) -> GPUBufferMapState {
        /// Step 1&2&3
        if self.mapping.borrow().is_some() {
            GPUBufferMapState::Mapped
        } else if self.pending_map.borrow().is_some() {
            GPUBufferMapState::Pending
        } else {
            GPUBufferMapState::Unmapped
        }
    }
}

impl AsyncWGPUListener for GPUBuffer {
    #[allow(unsafe_code)]
    fn handle_response(&self, response: WebGPUResponseResult, promise: &Rc<Promise>) {
        // Step 1
        if let Some(pending_map) = self.pending_map.borrow().as_ref() {
            if Rc::ptr_eq(promise, pending_map) {
                // Step 2
                debug_assert!(!promise.is_fulfilled());
                match response {
                    Ok(WebGPUResponse::BufferMapAsync { data, range, mode }) => {
                        let mode = match mode {
                            HostMap::Read => GPUMapModeConstants::READ,
                            HostMap::Write => GPUMapModeConstants::WRITE,
                        };
                        // Step 4
                        let mapping = GPUBufferMapInfo::new(mode, range);
                        // Step 5
                        *mapping.data.borrow_mut() = data.to_vec();
                        // Step 6
                        *self.mapping.borrow_mut() = Some(mapping);
                        // Step 7
                        promise.resolve_native(&());
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
        self.pending_map.borrow_mut().take();
        // Step 1.1
        assert!(promise.is_fulfilled());
    }
}
