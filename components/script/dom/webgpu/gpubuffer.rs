/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ops::Range;
use std::rc::Rc;
use std::string::String;

use dom_struct::dom_struct;
use js::context::JSContext;
use js::realm::CurrentRealm;
use js::typedarray::HeapArrayBuffer;
use script_bindings::cell::DomRefCell;
use script_bindings::reflector::{Reflector, reflect_dom_object_with_cx};
use script_bindings::trace::RootedTraceableBox;
use servo_base::generic_channel::GenericSharedMemory;
use webgpu_traits::{Mapping, WebGPU, WebGPUBuffer, WebGPURequest};
use wgpu_core::device::HostMap;
use wgpu_core::resource::BufferAccessError;

use crate::conversions::Convert;
use crate::dom::bindings::buffer_source::DataBlock;
use crate::dom::bindings::codegen::Bindings::WebGPUBinding::{
    GPUBufferDescriptor, GPUBufferMapState, GPUBufferMethods, GPUFlagsConstant,
    GPUMapModeConstants, GPUMapModeFlags, GPUSize64,
};
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::USVString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::dom::webgpu::gpudevice::GPUDevice;
use crate::routed_promise::{RoutedPromiseListener, callback_promise};

#[derive(JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) struct ActiveBufferMapping {
    // TODO(sagudev): Use GenericSharedMemory when https://github.com/servo/ipc-channel/pull/356 lands
    /// <https://gpuweb.github.io/gpuweb/#active-buffer-mapping-data>
    /// <https://gpuweb.github.io/gpuweb/#active-buffer-mapping-views>
    pub(crate) data: DataBlock,
    /// <https://gpuweb.github.io/gpuweb/#active-buffer-mapping-mode>
    mode: GPUMapModeFlags,
    /// <https://gpuweb.github.io/gpuweb/#active-buffer-mapping-range>
    range: Range<u64>,
}

impl ActiveBufferMapping {
    /// <https://gpuweb.github.io/gpuweb/#abstract-opdef-initialize-an-active-buffer-mapping>
    pub(crate) fn new(
        mode: GPUMapModeFlags,
        range: Range<u64>,
    ) -> Fallible<RootedTraceableBox<Self>> {
        // Step 1
        let size = range.end - range.start;
        // Step 2
        if size > (1 << 53) - 1 {
            return Err(Error::Range(c"Over MAX_SAFE_INTEGER".to_owned()));
        }
        let size: usize = size
            .try_into()
            .map_err(|_| Error::Range(c"Over usize".to_owned()))?;
        Ok(RootedTraceableBox::new(Self {
            data: DataBlock::new_zeroed(size),
            mode,
            range,
        }))
    }
}

#[dom_struct]
pub(crate) struct GPUBuffer {
    reflector_: Reflector,
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
    #[conditional_malloc_size_of]
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
        mapping: Option<RootedTraceableBox<ActiveBufferMapping>>,
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
            mapping: DomRefCell::new(mapping.map(|mapping| *mapping.into_box())),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        cx: &mut js::context::JSContext,
        global: &GlobalScope,
        channel: WebGPU,
        buffer: WebGPUBuffer,
        device: &GPUDevice,
        size: GPUSize64,
        usage: GPUFlagsConstant,
        mapping: Option<RootedTraceableBox<ActiveBufferMapping>>,
        label: USVString,
    ) -> DomRoot<Self> {
        reflect_dom_object_with_cx(
            Box::new(GPUBuffer::new_inherited(
                channel, buffer, device, size, usage, mapping, label,
            )),
            global,
            cx,
        )
    }
}

impl GPUBuffer {
    pub(crate) fn id(&self) -> WebGPUBuffer {
        self.buffer
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevice-createbuffer>
    pub(crate) fn create(
        cx: &mut js::context::JSContext,
        device: &GPUDevice,
        descriptor: &GPUBufferDescriptor,
    ) -> Fallible<DomRoot<GPUBuffer>> {
        let desc = wgpu_types::BufferDescriptor {
            label: (&descriptor.parent).convert(),
            size: descriptor.size as wgpu_types::BufferAddress,
            usage: wgpu_types::BufferUsages::from_bits_retain(descriptor.usage),
            mapped_at_creation: descriptor.mappedAtCreation,
        };
        let id = device.global().wgpu_id_hub().create_buffer_id();

        device
            .channel()
            .0
            .send(WebGPURequest::CreateBuffer {
                device_id: device.id().0,
                buffer_id: id,
                descriptor: desc,
            })
            .expect("Failed to create WebGPU buffer");

        let buffer = WebGPUBuffer(id);
        let mapping = if descriptor.mappedAtCreation {
            Some(ActiveBufferMapping::new(
                GPUMapModeConstants::WRITE,
                0..descriptor.size,
            )?)
        } else {
            None
        };

        Ok(GPUBuffer::new(
            cx,
            &device.global(),
            device.channel(),
            buffer,
            device,
            descriptor.size,
            descriptor.usage,
            mapping,
            descriptor.parent.label.clone(),
        ))
    }
}

impl Drop for GPUBuffer {
    fn drop(&mut self) {
        if let Err(e) = self
            .channel
            .0
            .send(WebGPURequest::DropBuffer(self.buffer.0))
        {
            error!(
                "Failed to send WebGPURequest::DropBuffer({:?}) ({}) - Potential leak",
                self.buffer.0, e
            );
        }
    }
}

impl GPUBufferMethods<crate::DomTypeHolder> for GPUBuffer {
    /// <https://gpuweb.github.io/gpuweb/#dom-gpubuffer-unmap>
    fn Unmap(&self, cx: &mut js::context::JSContext) {
        // Step 1
        let promise = self.pending_map.borrow_mut().take();
        if let Some(promise) = promise {
            promise.reject_error(cx, Error::Abort(None));
        }
        // Step 2
        let mut mapping = RootedTraceableBox::new(self.mapping.borrow_mut().take());
        let mapping = if let Some(mapping) = mapping.as_mut() {
            mapping
        } else {
            return;
        };

        // Step 3
        mapping.data.clear_views();
        // Step 5&7
        if let Err(e) = self.channel.0.send(WebGPURequest::UnmapBuffer {
            buffer_id: self.id().0,
            mapping: if mapping.mode >= GPUMapModeConstants::WRITE {
                Some(Mapping {
                    data: GenericSharedMemory::from_bytes(mapping.data.data()),
                    range: mapping.range.clone(),
                    mode: HostMap::Write,
                })
            } else {
                None
            },
        }) {
            warn!("Failed to send Buffer unmap ({:?}) ({})", self.buffer.0, e);
        }
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpubuffer-destroy>
    fn Destroy(&self, cx: &mut JSContext) {
        // Step 1
        self.Unmap(cx);
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
        cx: &mut CurrentRealm<'_>,
        mode: u32,
        offset: GPUSize64,
        size: Option<GPUSize64>,
    ) -> Rc<Promise> {
        let promise = Promise::new_in_realm(cx);
        // Step 2
        if self.pending_map.borrow().is_some() {
            promise.reject_error(cx, Error::Operation(None));
            return promise;
        }
        // Step 4
        *self.pending_map.borrow_mut() = Some(promise.clone());
        // Step 5
        let host_map = match mode {
            GPUMapModeConstants::READ => HostMap::Read,
            GPUMapModeConstants::WRITE => HostMap::Write,
            _ => {
                self.device
                    .dispatch_error(webgpu_traits::Error::Validation(String::from(
                        "Invalid MapModeFlags",
                    )));
                self.map_failure(cx, &promise);
                return promise;
            },
        };

        let callback = callback_promise(
            &promise,
            self,
            self.global().task_manager().dom_manipulation_task_source(),
        );
        if let Err(e) = self.channel.0.send(WebGPURequest::BufferMapAsync {
            callback,
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
            self.map_failure(cx, &promise);
            return promise;
        }
        // Step 6
        promise
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpubuffer-getmappedrange>
    fn GetMappedRange(
        &self,
        cx: &mut js::context::JSContext,
        offset: GPUSize64,
        size: Option<GPUSize64>,
    ) -> Fallible<RootedTraceableBox<HeapArrayBuffer>> {
        let range_size = if let Some(s) = size {
            s
        } else {
            self.size.saturating_sub(offset)
        };
        // Step 2: validation
        let mut mapping = self
            .mapping
            .borrow_mut()
            .take()
            .map(RootedTraceableBox::new)
            .ok_or(Error::Operation(None))?;

        let valid = offset.is_multiple_of(wgpu_types::MAP_ALIGNMENT) &&
            range_size % wgpu_types::COPY_BUFFER_ALIGNMENT == 0 &&
            offset >= mapping.range.start &&
            offset + range_size <= mapping.range.end;
        if !valid {
            self.mapping.borrow_mut().replace(*mapping.into_box());
            return Err(Error::Operation(None));
        }

        // Step 4
        // only mapping.range is mapped with mapping.range.start at 0
        // so we need to rebase range to mapped.range
        let rebased_offset = (offset - mapping.range.start) as usize;
        let result = mapping
            .data
            .view(cx, rebased_offset..rebased_offset + range_size as usize)
            .map(|view| view.array_buffer())
            .map_err(|()| Error::Operation(None));

        self.mapping.borrow_mut().replace(*mapping.into_box());
        result
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

impl GPUBuffer {
    fn map_failure(&self, cx: &mut JSContext, p: &Rc<Promise>) {
        // Step 1
        if self.pending_map.borrow().as_ref() != Some(p) {
            assert!(p.is_rejected());
            return;
        }
        // Step 2
        assert!(p.is_pending());
        // Step 3
        self.pending_map.borrow_mut().take();
        // Step 4
        let is_lost = self.device.is_lost();
        if is_lost {
            p.reject_error(cx, Error::Abort(None));
        } else {
            p.reject_error(cx, Error::Operation(None));
        }
    }

    fn map_success(&self, cx: &mut js::context::JSContext, p: &Rc<Promise>, wgpu_mapping: Mapping) {
        // Step 1
        if self.pending_map.borrow().as_ref() != Some(p) {
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
                *self.pending_map.borrow_mut() = None;
                p.reject_error(cx, error);
            },
            Ok(mut mapping) => {
                // Step 5
                mapping.data.load(&wgpu_mapping.data);
                // Step 6
                self.mapping.borrow_mut().replace(*mapping.into_box());
                // Step 7
                self.pending_map.borrow_mut().take();
                p.resolve_native_with_cx(cx, &());
            },
        }
    }
}

impl RoutedPromiseListener<Result<Mapping, BufferAccessError>> for GPUBuffer {
    fn handle_response(
        &self,
        cx: &mut js::context::JSContext,
        response: Result<Mapping, BufferAccessError>,
        promise: &Rc<Promise>,
    ) {
        match response {
            Ok(mapping) => self.map_success(cx, promise, mapping),
            Err(_) => self.map_failure(cx, promise),
        }
    }
}
