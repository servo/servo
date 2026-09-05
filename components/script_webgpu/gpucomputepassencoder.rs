/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::{JSContext, NoGC};
use log::warn;
use malloc_size_of_derive::MallocSizeOf;
use script_bindings::DomTypes;
use script_bindings::cell::DomRefCell;
use script_bindings::codegen::GenericBindings::WebGPUBinding::{
    GPUComputePassEncoderMethods, GPUComputePassEncoderWrap,
};
use script_bindings::interfaces::PromiseHelpers;
use script_bindings::reflector::{Reflector, reflect_dom_object_with_wrap};
use webgpu_traits::{WebGPU, WebGPUComputePass, WebGPURequest};

use crate::JSTraceable;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::USVString;
use crate::gpubindgroup::GPUBindGroup;
use crate::gpubuffer::GPUBuffer;
use crate::gpucommandencoder::GPUCommandEncoder;
use crate::gpucomputepipeline::GPUComputePipeline;
use crate::traits::{
    Equivalence, GPUDeviceTrait, GPUExternalTextureTrait, WebGPUGlobalTrait, WebGPUPromiseTrait,
};

#[derive(MallocSizeOf)]
struct DroppableGPUComputePassEncoder {
    channel: WebGPU,
    compute_pass: WebGPUComputePass,
}

impl Drop for DroppableGPUComputePassEncoder {
    fn drop(&mut self) {
        if let Err(e) = self
            .channel
            .0
            .send(WebGPURequest::DropComputePass(self.compute_pass.0))
        {
            warn!("Failed to send WebGPURequest::DropComputePass with {e:?}");
        }
    }
}

#[dom_struct]
pub struct GPUComputePassEncoder<D: DomTypes> {
    reflector_: Reflector,
    label: DomRefCell<USVString>,
    command_encoder: Dom<GPUCommandEncoder<D>>,
    #[no_trace]
    droppable: DroppableGPUComputePassEncoder,
}

impl<D> GPUComputePassEncoder<D>
where
    D: Equivalence,
    D::GPUDevice: GPUDeviceTrait<D>,
{
    fn new_inherited(
        channel: WebGPU,
        parent: &GPUCommandEncoder<D>,
        compute_pass: WebGPUComputePass,
        label: USVString,
    ) -> Self {
        Self {
            reflector_: Reflector::new(),
            label: DomRefCell::new(label),
            command_encoder: Dom::from_ref(parent),
            droppable: DroppableGPUComputePassEncoder {
                channel,
                compute_pass,
            },
        }
    }

    pub(crate) fn new(
        cx: &mut JSContext,
        global: &D::GlobalScope,
        channel: WebGPU,
        parent: &GPUCommandEncoder<D>,
        compute_pass: WebGPUComputePass,
        label: USVString,
    ) -> DomRoot<Self> {
        reflect_dom_object_with_wrap::<D, _, _>(
            Box::new(GPUComputePassEncoder::new_inherited(
                channel,
                parent,
                compute_pass,
                label,
            )),
            global,
            cx,
            GPUComputePassEncoderWrap::<D>,
        )
    }
}

impl<D> GPUComputePassEncoderMethods<D> for GPUComputePassEncoder<D>
where
    D: Equivalence,
    D::GlobalScope: WebGPUGlobalTrait,
    D::GPUDevice: GPUDeviceTrait<D>,
    D::GPUExternalTexture: GPUExternalTextureTrait<D>,
    D::Promise: PromiseHelpers<D>,
    <D::Promise as PromiseHelpers<D>>::StackRoot: WebGPUPromiseTrait<D>,
{
    /// <https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label>
    fn Label(&self) -> USVString {
        self.label.borrow().clone()
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label>
    fn SetLabel(&self, no_gc: &NoGC, value: USVString) {
        *self.label.safe_borrow_mut(no_gc) = value;
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpucomputepassencoder-dispatchworkgroups>
    fn DispatchWorkgroups(&self, x: u32, y: u32, z: u32) {
        if let Err(e) =
            self.droppable
                .channel
                .0
                .send(WebGPURequest::ComputePassDispatchWorkgroups {
                    compute_pass_id: self.droppable.compute_pass.0,
                    x,
                    y,
                    z,
                    device_id: self.command_encoder.device_id().0,
                })
        {
            warn!("Error sending WebGPURequest::ComputePassDispatchWorkgroups: {e:?}")
        }
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpucomputepassencoder-dispatchworkgroupsindirect>
    fn DispatchWorkgroupsIndirect(&self, buffer: &GPUBuffer<D>, offset: u64) {
        if let Err(e) =
            self.droppable
                .channel
                .0
                .send(WebGPURequest::ComputePassDispatchWorkgroupsIndirect {
                    compute_pass_id: self.droppable.compute_pass.0,
                    buffer_id: buffer.id().0,
                    offset,
                    device_id: self.command_encoder.device_id().0,
                })
        {
            warn!("Error sending WebGPURequest::ComputePassDispatchWorkgroupsIndirect: {e:?}")
        }
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpurenderpassencoder-endpass>
    fn End(&self) {
        if let Err(e) = self
            .droppable
            .channel
            .0
            .send(WebGPURequest::EndComputePass {
                compute_pass_id: self.droppable.compute_pass.0,
                device_id: self.command_encoder.device_id().0,
            })
        {
            warn!("Failed to send WebGPURequest::EndComputePass: {e:?}");
        }
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpuprogrammablepassencoder-setbindgroup>
    fn SetBindGroup(&self, index: u32, bind_group: &GPUBindGroup<D>, offsets: Vec<u32>) {
        if let Err(e) = self
            .droppable
            .channel
            .0
            .send(WebGPURequest::ComputePassSetBindGroup {
                compute_pass_id: self.droppable.compute_pass.0,
                index,
                bind_group_id: bind_group.id().0,
                offsets,
                device_id: self.command_encoder.device_id().0,
            })
        {
            warn!("Error sending WebGPURequest::ComputePassSetBindGroup: {e:?}")
        }
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpucomputepassencoder-setpipeline>
    fn SetPipeline(&self, pipeline: &GPUComputePipeline<D>) {
        if let Err(e) = self
            .droppable
            .channel
            .0
            .send(WebGPURequest::ComputePassSetPipeline {
                compute_pass_id: self.droppable.compute_pass.0,
                pipeline_id: pipeline.id().0,
                device_id: self.command_encoder.device_id().0,
            })
        {
            warn!("Error sending WebGPURequest::ComputePassSetPipeline: {e:?}")
        }
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudebugcommandsmixin-pushdebuggroup>
    fn PushDebugGroup(&self, group_label: USVString) {
        if let Err(e) = self
            .droppable
            .channel
            .0
            .send(WebGPURequest::ComputePassPushDebugGroup {
                compute_pass_id: self.droppable.compute_pass.0,
                label: group_label.to_string(),
                device_id: self.command_encoder.device_id().0,
            })
        {
            warn!("Error sending WebGPURequest::ComputePassPushDebugGroup: {e:?}")
        }
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudebugcommandsmixin-popdebuggroup>
    fn PopDebugGroup(&self) {
        if let Err(e) = self
            .droppable
            .channel
            .0
            .send(WebGPURequest::ComputePassPopDebugGroup {
                compute_pass_id: self.droppable.compute_pass.0,
                device_id: self.command_encoder.device_id().0,
            })
        {
            warn!("Error sending WebGPURequest::ComputePassPopDebugGroup: {e:?}")
        }
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudebugcommandsmixin-insertdebugmarker>
    fn InsertDebugMarker(&self, marker_label: USVString) {
        if let Err(e) = self
            .droppable
            .channel
            .0
            .send(WebGPURequest::ComputePassInsertDebugMarker {
                compute_pass_id: self.droppable.compute_pass.0,
                label: marker_label.to_string(),
                device_id: self.command_encoder.device_id().0,
            })
        {
            warn!("Error sending WebGPURequest::ComputePassInsertDebugMarker: {e:?}")
        }
    }
}
