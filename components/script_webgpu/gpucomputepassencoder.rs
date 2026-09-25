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
use webgpu_traits::{
    ComputePassEncoderCommand, DebugCommand, WebGPU, WebGPUComputePass, WebGPURequest,
};

use crate::JSTraceable;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::USVString;
use crate::gpubindgroup::GPUBindGroup;
use crate::gpubuffer::GPUBuffer;
use crate::gpucommandencoder::GPUCommandEncoder;
use crate::gpucomputepipeline::GPUComputePipeline;
use crate::traits::{Equivalence, WebGPUPromise};

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
    <D::Promise as PromiseHelpers<D>>::StackRoot: WebGPUPromise<D>,
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
            cx,
            Box::new(GPUComputePassEncoder::new_inherited(
                channel,
                parent,
                compute_pass,
                label,
            )),
            global,
            GPUComputePassEncoderWrap::<D>,
        )
    }

    fn send_command(&self, command: ComputePassEncoderCommand) {
        if let Err(e) = self
            .droppable
            .channel
            .0
            .send(WebGPURequest::ComputePassCommand {
                compute_pass_id: self.droppable.compute_pass.0,
                compute_command: command,
                device_id: self.command_encoder.device_id().0,
            })
        {
            warn!("Error sending WebGPURequest::ComputePassCommand: {e:?}")
        }
    }
}

impl<D> GPUComputePassEncoderMethods<D> for GPUComputePassEncoder<D>
where
    D: Equivalence,
    <D::Promise as PromiseHelpers<D>>::StackRoot: WebGPUPromise<D>,
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
        self.send_command(ComputePassEncoderCommand::DispatchWorkgroups {
            workgroup_count_x: x,
            workgroup_count_y: y,
            workgroup_count_z: z,
        });
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpucomputepassencoder-dispatchworkgroupsindirect>
    fn DispatchWorkgroupsIndirect(&self, buffer: &GPUBuffer<D>, offset: u64) {
        self.send_command(ComputePassEncoderCommand::DispatchWorkgroupsIndirect {
            indirect_buffer: buffer.id().0,
            indirect_offset: offset,
        });
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
        self.send_command(ComputePassEncoderCommand::BindingCommand(
            webgpu_traits::BindingCommand::SetBindGroup {
                index,
                bind_group: Some(bind_group.id().0),
                dynamic_offsets: offsets,
            },
        ));
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpucomputepassencoder-setpipeline>
    fn SetPipeline(&self, pipeline: &GPUComputePipeline<D>) {
        self.send_command(ComputePassEncoderCommand::SetPipeline(pipeline.id().0));
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudebugcommandsmixin-pushdebuggroup>
    fn PushDebugGroup(&self, group_label: USVString) {
        self.send_command(ComputePassEncoderCommand::DebugCommand(
            DebugCommand::PushDebugGroup(group_label.to_string()),
        ));
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudebugcommandsmixin-popdebuggroup>
    fn PopDebugGroup(&self) {
        self.send_command(ComputePassEncoderCommand::DebugCommand(
            DebugCommand::PopDebugGroup,
        ));
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudebugcommandsmixin-insertdebugmarker>
    fn InsertDebugMarker(&self, marker_label: USVString) {
        self.send_command(ComputePassEncoderCommand::DebugCommand(
            DebugCommand::InsertDebugMarker(marker_label.to_string()),
        ));
    }
}
