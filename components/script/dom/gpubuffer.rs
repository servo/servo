/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ffi::c_void;
use std::ops::Range;
use std::rc::Rc;
use std::string::String;
use std::sync::Arc;

use dom_struct::dom_struct;
use ipc_channel::ipc::IpcSharedMemory;
use js::jsapi::{DetachArrayBuffer, NewExternalArrayBuffer};
use js::typedarray::{ArrayBuffer, HeapArrayBuffer};
use webgpu::wgc::device::HostMap;
use webgpu::{wgt, Mapping, WebGPU, WebGPUBuffer, WebGPURequest, WebGPUResponse};

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

#[derive(JSTraceable, MallocSizeOf)]
pub struct MappingView {
    #[no_trace]
    range: Range<usize>,
    #[ignore_malloc_size_of = "defined in mozjs"]
    buffer: HeapArrayBuffer,
}

impl Drop for MappingView {
    #[allow(unsafe_code)]
    fn drop(&mut self) {
        let cx = GlobalScope::get_cx();
        assert!(unsafe { DetachArrayBuffer(*cx, self.buffer.underlying_object().handle()) })
    }
}

#[derive(JSTraceable, MallocSizeOf)]
pub struct ActiveBufferMapping {
    // TODO(sagudev): Use IpcSharedMemory when https://github.com/servo/ipc-channel/pull/356 lands
    /// <https://gpuweb.github.io/gpuweb/#active-buffer-mapping-data>
    #[ignore_malloc_size_of = "Rc"]
    data: Arc<Box<[u8]>>,
    /// <https://gpuweb.github.io/gpuweb/#active-buffer-mapping-mode>
    mode: GPUMapModeFlags,
    /// <https://gpuweb.github.io/gpuweb/#active-buffer-mapping-range>
    range: Range<u64>,
    /// <https://gpuweb.github.io/gpuweb/#active-buffer-mapping-views>
    views: Vec<MappingView>,
}

impl ActiveBufferMapping {
    /// <https://gpuweb.github.io/gpuweb/#abstract-opdef-initialize-an-active-buffer-mapping>
    pub fn new(mode: GPUMapModeFlags, range: Range<u64>) -> Fallible<Self> {
        // Step 1
        let size: usize = (range.end - range.start).try_into().unwrap();
        // Step 2
        if size > (1 << 53) - 1 {
            return Err(Error::Range("Over MAX_SAFE_INTEGER".to_string()));
        }
        let data = vec![0; size];
        Ok(Self {
            data: Arc::new(data.into_boxed_slice()),
            mode,
            range,
            views: Vec::new(),
        })
    }

    /// Panics if there is any active view
    pub fn load_data(&mut self, src: &[u8]) {
        // `Arc::get_mut` ensures there are no views
        Arc::get_mut(&mut self.data).unwrap().clone_from_slice(src)
    }

    #[allow(unsafe_code)]
    pub fn view(&mut self, range: Range<usize>) -> Result<&MappingView, ()> {
        if self
            .views
            .iter()
            .any(|view| range_overlap(&view.range, &range))
        {
            return Err(());
        }
        let cx = GlobalScope::get_cx();
        /// `freeFunc()` must be threadsafe, should be safely callable from any thread
        /// without causing conflicts, unexpected behavior.
        unsafe extern "C" fn free_func(_contents: *mut c_void, free_user_data: *mut c_void) {
            // Clippy warns about "creating a `Arc` from a void raw pointer" here, but suggests
            // the exact same line to fix it. Doing the cast is tricky because of the use of
            // a generic type in this parameter.
            #[allow(clippy::from_raw_with_void_ptr)]
            drop(Arc::from_raw(free_user_data as *const _));
        }
        let raw: *mut Box<[u8]> = Arc::into_raw(Arc::clone(&self.data)) as _;
        rooted!(in(*cx) let object = unsafe {
            NewExternalArrayBuffer(
                *cx,
                range.end - range.start,
                // SAFETY: This is safe because we have checked there is no overlapping view
                (*raw)[range.clone()].as_mut_ptr() as _,
                Some(free_func),
                raw as _,
            )
        });
        self.views.push(MappingView {
            range,
            buffer: HeapArrayBuffer::from(*object).unwrap(),
        });
        Ok(self.views.last().unwrap())
    }
}

#[dom_struct]
pub struct GPUBuffer {
    reflector_: Reflector,
    #[ignore_malloc_size_of = "defined in webgpu"]
    #[no_trace]
    channel: WebGPU,
    label: DomRefCell<USVString>,
    #[no_trace]
    buffer: WebGPUBuffer,
    device: Dom<GPUDevice>,
    /// <https://gpuweb.github.io/gpuweb/#dom-gpubuffer-size>
    size: GPUSize64,
    /// <https://gpuweb.github.io/gpuweb/#dom-gpubuffer-usage>
    usage: GPUFlagsConstant,
    /// <https://gpuweb.github.io/gpuweb/#dom-gpubuffer-pending_map-slot>
    #[ignore_malloc_size_of = "promises are hard"]
    pending_map: DomRefCell<Option<Rc<Promise>>>,
    /// <https://gpuweb.github.io/gpuweb/#dom-gpubuffer-mapping-slot>
    mapping: DomRefCell<Option<ActiveBufferMapping>>,
}

impl GPUBuffer {
    fn new_inherited(
        channel: WebGPU,
        buffer: WebGPUBuffer,
        device: &GPUDevice,
        size: GPUSize64,
        usage: GPUFlagsConstant,
        mapping: Option<ActiveBufferMapping>,
        label: USVString,
    ) -> Self {
        Self {
            reflector_: Reflector::new(),
            channel,
            label: DomRefCell::new(label),
            device: Dom::from_ref(device),
            buffer,
            pending_map: DomRefCell::new(None),
            size,
            usage,
            mapping: DomRefCell::new(mapping),
        }
    }

    pub fn new(
        global: &GlobalScope,
        channel: WebGPU,
        buffer: WebGPUBuffer,
        device: &GPUDevice,
        size: GPUSize64,
        usage: GPUFlagsConstant,
        mapping: Option<ActiveBufferMapping>,
        label: USVString,
    ) -> DomRoot<Self> {
        reflect_dom_object(
            Box::new(GPUBuffer::new_inherited(
                channel, buffer, device, size, usage, mapping, label,
            )),
            global,
        )
    }
}

impl GPUBuffer {
    pub fn id(&self) -> WebGPUBuffer {
        self.buffer
    }
}

impl Drop for GPUBuffer {
    fn drop(&mut self) {
        self.Destroy()
    }
}

impl GPUBufferMethods for GPUBuffer {
    #[allow(unsafe_code)]
    /// <https://gpuweb.github.io/gpuweb/#dom-gpubuffer-unmap>
    fn Unmap(&self) {
        // Step 1
        if let Some(promise) = self.pending_map.borrow_mut().take() {
            promise.reject_error(Error::Abort);
        }
        // Step 2
        let mut mapping = self.mapping.borrow_mut().take();
        let mapping = if let Some(mapping) = mapping.as_mut() {
            mapping
        } else {
            return;
        };

        // Step 3
        mapping.views.clear();
        // Step 5&7
        if let Err(e) = self.channel.0.send(WebGPURequest::UnmapBuffer {
            buffer_id: self.id().0,
            array_buffer: IpcSharedMemory::from_bytes(&mapping.data),
            write_back: mapping.mode >= GPUMapModeConstants::WRITE,
            offset: mapping.range.start,
            size: mapping.range.end - mapping.range.start,
        }) {
            warn!("Failed to send Buffer unmap ({:?}) ({})", self.buffer.0, e);
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
            .send(WebGPURequest::DestroyBuffer(self.buffer.0))
        {
            warn!(
                "Failed to send WebGPURequest::DestroyBuffer({:?}) ({})",
                self.buffer.0, e
            );
        };
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpubuffer-mapasync>
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
        // Step 4
        *self.pending_map.borrow_mut() = Some(promise.clone());
        // Step 5

        // This should be bitflags in wgpu-core
        let host_map = match mode {
            GPUMapModeConstants::READ => HostMap::Read,
            GPUMapModeConstants::WRITE => HostMap::Write,
            _ => {
                self.device
                    .dispatch_error(webgpu::Error::Validation(String::from(
                        "Invalid MapModeFlags",
                    )));
                self.map_failure(&promise);
                return promise;
            },
        };

        let sender = response_async(&promise, self);
        if let Err(e) = self.channel.0.send(WebGPURequest::BufferMapAsync {
            sender,
            buffer_id: self.buffer.0,
            device_id: self.device.id().0,
            host_map,
            offset,
            size,
        }) {
            warn!(
                "Failed to send BufferMapAsync ({:?}) ({})",
                self.buffer.0, e
            );
            self.map_failure(&promise);
            return promise;
        }
        // Step 6
        promise
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpubuffer-getmappedrange>
    #[allow(unsafe_code)]
    fn GetMappedRange(
        &self,
        _cx: JSContext,
        offset: GPUSize64,
        size: Option<GPUSize64>,
    ) -> Fallible<ArrayBuffer> {
        let range_size = if let Some(s) = size {
            s
        } else {
            self.size.checked_sub(offset).unwrap_or(0)
        };
        // Step 2: validation
        let mut mapping = self.mapping.borrow_mut();
        let mapping = mapping.as_mut().ok_or(Error::Operation)?;

        let valid = offset % wgt::MAP_ALIGNMENT == 0 &&
            range_size % wgt::COPY_BUFFER_ALIGNMENT == 0 &&
            offset >= mapping.range.start &&
            offset + range_size <= mapping.range.end;
        if !valid {
            return Err(Error::Operation);
        }

        // Step 4
        // only mapping.range is mapped with mapping.range.start at 0
        // so we need to rebase range to mapped.range
        let rebased_offset = (offset - mapping.range.start) as usize;
        mapping
            .view(rebased_offset..rebased_offset + range_size as usize)
            .map(|mapping_view| unsafe {
                ArrayBuffer::from(mapping_view.buffer.underlying_object().get()).unwrap()
            })
            .map_err(|()| Error::Operation)
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label>
    fn Label(&self) -> USVString {
        self.label.borrow().clone()
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label>
    fn SetLabel(&self, value: USVString) {
        *self.label.borrow_mut() = value;
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpubuffer-size>
    fn Size(&self) -> GPUSize64 {
        self.size
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpubuffer-usage>
    fn Usage(&self) -> GPUFlagsConstant {
        self.usage
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpubuffer-mapstate>
    fn MapState(&self) -> GPUBufferMapState {
        // Step 1&2&3
        if self.mapping.borrow().is_some() {
            GPUBufferMapState::Mapped
        } else if self.pending_map.borrow().is_some() {
            GPUBufferMapState::Pending
        } else {
            GPUBufferMapState::Unmapped
        }
    }
}

/// Returns true if two non-inclusive ranges overlap
// https://stackoverflow.com/questions/3269434/whats-the-most-efficient-way-to-test-if-two-ranges-overlap
fn range_overlap<T: std::cmp::PartialOrd>(range1: &Range<T>, range2: &Range<T>) -> bool {
    range1.start < range2.end && range2.start < range1.end
}

impl GPUBuffer {
    fn map_failure(&self, p: &Rc<Promise>) {
        let mut pending_map = self.pending_map.borrow_mut();
        // Step 1
        if pending_map.as_ref() != Some(p) {
            assert!(p.is_rejected());
            return;
        }
        // Step 2
        assert!(p.is_pending());
        // Step 3
        pending_map.take();
        // Step 4
        if self.device.is_lost() {
            p.reject_error(Error::Abort);
        } else {
            p.reject_error(Error::Operation);
        }
    }

    fn map_success(&self, p: &Rc<Promise>, wgpu_mapping: Mapping) {
        let mut pending_map = self.pending_map.borrow_mut();

        // Step 1
        if pending_map.as_ref() != Some(p) {
            assert!(p.is_rejected());
            return;
        }

        // Step 2
        assert!(p.is_pending());

        // Step 4
        let mapping = ActiveBufferMapping::new(
            match wgpu_mapping.mode {
                HostMap::Read => GPUMapModeConstants::READ,
                HostMap::Write => GPUMapModeConstants::WRITE,
            },
            wgpu_mapping.range,
        );

        match mapping {
            Err(error) => {
                *pending_map = None;
                p.reject_error(error.clone());
            },
            Ok(mut mapping) => {
                // Step 5
                mapping.load_data(&wgpu_mapping.data);
                // Step 6
                self.mapping.borrow_mut().replace(mapping);
                // Step 7
                pending_map.take();
                p.resolve_native(&());
            },
        }
    }
}

impl AsyncWGPUListener for GPUBuffer {
    #[allow(unsafe_code)]
    fn handle_response(&self, response: WebGPUResponse, promise: &Rc<Promise>) {
        match response {
            WebGPUResponse::BufferMapAsync(Ok(mapping)) => self.map_success(promise, mapping),
            WebGPUResponse::BufferMapAsync(Err(_)) => self.map_failure(promise),
            _ => unreachable!("Wrong response received on AsyncWGPUListener for GPUBuffer"),
        }
    }
}
