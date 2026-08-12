/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ops::Range;
use std::rc::Rc;

use dom_struct::dom_struct;
use js::context::{JSContext, NoGC};
use js::realm::CurrentRealm;
use js::typedarray::HeapArrayBuffer;
use jstraceable_derive::JSTraceable;
use log::{error, warn};
use malloc_size_of_derive::MallocSizeOf;
use script_bindings::DomTypes;
use script_bindings::cell::DomRefCell;
use script_bindings::codegen::GenericBindings::WebGPUBinding::{
    GPUBufferDescriptor, GPUBufferMapState, GPUBufferMethods, GPUBufferWrap, GPUFlagsConstant,
    GPUMapModeConstants, GPUMapModeFlags, GPUSize64,
};
use script_bindings::error::{Error, Fallible};
use script_bindings::interfaces::PromiseHelpers;
use script_bindings::reflector::{DomGlobalGeneric, Reflector, reflect_dom_object_with_wrap};
use script_bindings::trace::RootedTraceableBox;
use servo_base::generic_channel::GenericSharedMemory;
use webgpu_traits::{Mapping, WebGPU, WebGPUBuffer, WebGPURequest};
use wgpu_core::device::HostMap;

use crate::datablock::DataBlock;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::USVString;
use crate::gpuconvert::WebGPUConvert;
use crate::traits::{GPUDeviceTrait, WebGPUGlobalTrait, WebGPUPromiseTrait};

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

#[derive(JSTraceable, MallocSizeOf)]
pub struct DroppableGPUBuffer {
    #[no_trace]
    channel: WebGPU,
    #[no_trace]
    buffer: WebGPUBuffer,
}

impl Drop for DroppableGPUBuffer {
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

#[dom_struct]
pub struct GPUBuffer<D: DomTypes> {
    reflector_: Reflector,
    droppable: DroppableGPUBuffer,
    label: DomRefCell<USVString>,
    device: Dom<D::GPUDevice>,
    /// <https://gpuweb.github.io/gpuweb/#dom-gpubuffer-size>
    size: GPUSize64,
    /// <https://gpuweb.github.io/gpuweb/#dom-gpubuffer-usage>
    usage: GPUFlagsConstant,
    /// <https://gpuweb.github.io/gpuweb/#dom-gpubuffer-pending_map-slot>
    #[conditional_malloc_size_of]
    pending_map: DomRefCell<Option<Rc<D::Promise>>>,
    /// <https://gpuweb.github.io/gpuweb/#dom-gpubuffer-mapping-slot>
    mapping: DomRefCell<Option<ActiveBufferMapping>>,
}

impl<D> GPUBuffer<D>
where
    D: DomTypes<GPUBuffer = GPUBuffer<D>>,
    D::Promise: PromiseHelpers<D>,
{
    fn new_inherited(
        channel: WebGPU,
        buffer: WebGPUBuffer,
        device: &D::GPUDevice,
        size: GPUSize64,
        usage: GPUFlagsConstant,
        mapping: Option<RootedTraceableBox<ActiveBufferMapping>>,
        label: USVString,
    ) -> Self {
        Self {
            reflector_: Reflector::new(),
            droppable: DroppableGPUBuffer { channel, buffer },
            label: DomRefCell::new(label),
            device: Dom::from_ref(device),
            pending_map: DomRefCell::new(None),
            size,
            usage,
            mapping: DomRefCell::new(mapping.map(|mapping| *mapping.into_box())),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        cx: &mut js::context::JSContext,
        global: &D::GlobalScope,
        channel: WebGPU,
        buffer: WebGPUBuffer,
        device: &D::GPUDevice,
        size: GPUSize64,
        usage: GPUFlagsConstant,
        mapping: Option<RootedTraceableBox<ActiveBufferMapping>>,
        label: USVString,
    ) -> DomRoot<Self> {
        reflect_dom_object_with_wrap::<D, _, _>(
            Box::new(GPUBuffer::new_inherited(
                channel, buffer, device, size, usage, mapping, label,
            )),
            global,
            cx,
            GPUBufferWrap::<D>,
        )
    }
}

impl<D> GPUBuffer<D>
where
    D: DomTypes<GPUBuffer = GPUBuffer<D>>,
    D::GPUDevice: DomGlobalGeneric<D> + GPUDeviceTrait<D>,
    D::GlobalScope: WebGPUGlobalTrait,
    D::Promise: PromiseHelpers<D>,
{
    pub fn id(&self) -> WebGPUBuffer {
        self.droppable.buffer
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevice-createbuffer>
    pub fn create(
        cx: &mut js::context::JSContext,
        device: &D::GPUDevice,
        descriptor: &GPUBufferDescriptor,
    ) -> Fallible<DomRoot<GPUBuffer<D>>> {
        let desc = wgpu_types::BufferDescriptor {
            label: (&descriptor.parent).convert(),
            size: descriptor.size as wgpu_types::BufferAddress,
            usage: wgpu_types::BufferUsages::from_bits_retain(descriptor.usage),
            mapped_at_creation: descriptor.mappedAtCreation,
        };
        let id = <D::GPUDevice as DomGlobalGeneric<D>>::global_from_reflector(device)
            .global_wgpu_id_hub()
            .create_buffer_id();

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

        let global = <D::GPUDevice as DomGlobalGeneric<D>>::global_from_reflector(device);
        Ok(GPUBuffer::new(
            cx,
            &*global,
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

impl<D> GPUBufferMethods<D> for GPUBuffer<D>
where
    D: DomTypes<GPUBuffer = GPUBuffer<D>>,
    D::Promise: PromiseHelpers<D> + WebGPUPromiseTrait<D> + PartialEq,
    D::GPUDevice: GPUDeviceTrait<D>,
    D::GPUDevice: DomGlobalGeneric<D> + GPUDeviceTrait<D>,
    D::GlobalScope: WebGPUGlobalTrait,
    D::Promise: PromiseHelpers<D>,
{
    /// <https://gpuweb.github.io/gpuweb/#dom-gpubuffer-unmap>
    fn Unmap(&self, cx: &mut js::context::JSContext) {
        // Step 1
        let promise = self.pending_map.safe_borrow_mut(cx).take();
        if let Some(promise) = promise {
            promise.reject_error(cx, Error::Abort(Some("No pending map".into())));
        }
        // Step 2
        let mut mapping = RootedTraceableBox::new(self.mapping.safe_borrow_mut(cx).take());
        let mapping = if let Some(mapping) = mapping.as_mut() {
            mapping
        } else {
            return;
        };

        // Step 3
        mapping.data.clear_views(cx);
        // Step 5&7
        if let Err(e) = self.droppable.channel.0.send(WebGPURequest::UnmapBuffer {
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
            warn!(
                "Failed to send Buffer unmap ({:?}) ({})",
                self.droppable.buffer.0, e
            );
        }
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpubuffer-destroy>
    fn Destroy(&self, cx: &mut JSContext) {
        // Step 1
        self.Unmap(cx);
        // Step 2
        if let Err(e) = self
            .droppable
            .channel
            .0
            .send(WebGPURequest::DestroyBuffer(self.droppable.buffer.0))
        {
            warn!(
                "Failed to send WebGPURequest::DestroyBuffer({:?}) ({})",
                self.droppable.buffer.0, e
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
    ) -> Rc<D::Promise> {
        let promise = D::Promise::new_in_realm(cx);
        // Step 2
        if self.pending_map.borrow().is_some() {
            promise.reject_error(
                cx,
                Error::Operation(Some("There is already an active map".into())),
            );
            return promise;
        }
        // Step 4
        *self.pending_map.safe_borrow_mut(cx) = Some(promise.clone());
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

        let callback = D::Promise::callback_promise_gpubuffer(&promise, self);
        if let Err(e) = self
            .droppable
            .channel
            .0
            .send(WebGPURequest::BufferMapAsync {
                callback,
                buffer_id: self.droppable.buffer.0,
                device_id: self.device.id().0,
                host_map,
                offset,
                size,
            })
        {
            warn!(
                "Failed to send BufferMapAsync ({:?}) ({})",
                self.droppable.buffer.0, e
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
            .safe_borrow_mut(cx)
            .take()
            .map(RootedTraceableBox::new)
            .ok_or(Error::Operation(Some("No active buffer map".into())))?;

        let valid = offset.is_multiple_of(wgpu_types::MAP_ALIGNMENT) &&
            range_size % wgpu_types::COPY_BUFFER_ALIGNMENT == 0 &&
            offset >= mapping.range.start &&
            offset + range_size <= mapping.range.end;
        if !valid {
            self.mapping
                .safe_borrow_mut(cx)
                .replace(*mapping.into_box());
            return Err(Error::Operation(Some(
                "Buffer Mapping is not active".into(),
            )));
        }

        // Step 4
        // only mapping.range is mapped with mapping.range.start at 0
        // so we need to rebase range to mapped.range
        let rebased_offset = (offset - mapping.range.start) as usize;
        let result = mapping
            .data
            .view(cx, rebased_offset..rebased_offset + range_size as usize)
            .map(|view| view.array_buffer())
            .map_err(|()| {
                Error::Operation(Some(
                    "Mapped range overlaps with others or is out of bounds.".into(),
                ))
            });

        self.mapping
            .safe_borrow_mut(cx)
            .replace(*mapping.into_box());
        result
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label>
    fn Label(&self) -> USVString {
        self.label.borrow().clone()
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label>
    fn SetLabel(&self, no_gc: &NoGC, value: USVString) {
        *self.label.safe_borrow_mut(no_gc) = value;
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

impl<D> GPUBuffer<D>
where
    D: DomTypes,
    D::Promise: PromiseHelpers<D> + PartialEq,
    D::GPUDevice: GPUDeviceTrait<D>,
{
    pub fn map_failure(&self, cx: &mut JSContext, p: &Rc<D::Promise>) {
        // Step 1
        if self.pending_map.borrow().as_ref() != Some(p) {
            assert!(p.is_rejected());
            return;
        }
        // Step 2
        assert!(p.is_pending());
        // Step 3
        self.pending_map.safe_borrow_mut(cx).take();
        // Step 4
        let is_lost = self.device.is_lost();
        if is_lost {
            p.reject_error(cx, Error::Abort(Some("GPUDevice is lost".into())));
        } else {
            p.reject_error(cx, Error::Operation(Some("Mapping failure".into())));
        }
    }

    pub fn map_success(
        &self,
        cx: &mut js::context::JSContext,
        p: &Rc<D::Promise>,
        wgpu_mapping: Mapping,
    ) {
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
                *self.pending_map.safe_borrow_mut(cx) = None;
                p.reject_error(cx, error);
            },
            Ok(mut mapping) => {
                // Step 5
                mapping.data.load(&wgpu_mapping.data);
                // Step 6
                self.mapping
                    .safe_borrow_mut(cx)
                    .replace(*mapping.into_box());
                // Step 7
                self.pending_map.safe_borrow_mut(cx).take();
                p.resolve_native(cx, &());
            },
        }
    }
}
