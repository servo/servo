/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::Cow;
use std::marker::PhantomData;

use dom_struct::dom_struct;
use js::context::{JSContext, NoGC};
use log::warn;
use malloc_size_of_derive::MallocSizeOf;
use script_bindings::DomTypes;
use script_bindings::cell::DomRefCell;
use script_bindings::codegen::GenericBindings::WebGPUBinding::{
    GPUPipelineLayoutDescriptor, GPUPipelineLayoutMethods, GPUPipelineLayoutWrap,
};
use script_bindings::reflector::{DomGlobalGeneric, Reflector, reflect_dom_object_with_wrap};
use webgpu_traits::{WebGPU, WebGPUBindGroupLayout, WebGPUPipelineLayout, WebGPURequest};
use wgpu_core::binding_model::PipelineLayoutDescriptor;

use crate::JSTraceable;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::USVString;
use crate::gpuconvert::WebGPUConvert;
use crate::traits::{Equivalence, GPUDeviceTrait, WebGPUGlobalTrait};

#[derive(MallocSizeOf)]
struct DroppableGPUPipelineLayout {
    channel: WebGPU,
    pipeline_layout: WebGPUPipelineLayout,
}

impl Drop for DroppableGPUPipelineLayout {
    fn drop(&mut self) {
        if let Err(e) = self
            .channel
            .0
            .send(WebGPURequest::DropPipelineLayout(self.pipeline_layout.0))
        {
            warn!(
                "Failed to send DropPipelineLayout ({:?}) ({})",
                self.pipeline_layout.0, e
            );
        }
    }
}

#[dom_struct]
pub struct GPUPipelineLayout<D: DomTypes> {
    reflector_: Reflector,
    label: DomRefCell<USVString>,
    #[no_trace]
    bind_group_layouts: Vec<WebGPUBindGroupLayout>,
    #[no_trace]
    droppable: DroppableGPUPipelineLayout,
    #[no_trace = "PhantomData does not exist"]
    phantom: PhantomData<D>,
}

impl<D> GPUPipelineLayout<D>
where
    D: Equivalence,
{
    fn new_inherited(
        channel: WebGPU,
        pipeline_layout: WebGPUPipelineLayout,
        label: USVString,
        bgls: Vec<WebGPUBindGroupLayout>,
    ) -> Self {
        Self {
            reflector_: Reflector::new(),
            label: DomRefCell::new(label),
            bind_group_layouts: bgls,
            droppable: DroppableGPUPipelineLayout {
                channel,
                pipeline_layout,
            },
            phantom: PhantomData,
        }
    }

    pub(crate) fn new(
        cx: &mut JSContext,
        global: &D::GlobalScope,
        channel: WebGPU,
        pipeline_layout: WebGPUPipelineLayout,
        label: USVString,
        bgls: Vec<WebGPUBindGroupLayout>,
    ) -> DomRoot<Self> {
        reflect_dom_object_with_wrap::<D, _, _>(
            Box::new(GPUPipelineLayout::new_inherited(
                channel,
                pipeline_layout,
                label,
                bgls,
            )),
            global,
            cx,
            GPUPipelineLayoutWrap::<D>,
        )
    }
}

impl<D> GPUPipelineLayout<D>
where
    D: Equivalence,
    D::GPUDevice: GPUDeviceTrait<D>,
    D::GlobalScope: WebGPUGlobalTrait,
{
    pub fn id(&self) -> WebGPUPipelineLayout {
        self.droppable.pipeline_layout
    }

    #[expect(unused)]
    fn bind_group_layouts(&self) -> Vec<WebGPUBindGroupLayout> {
        self.bind_group_layouts.clone()
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevice-createpipelinelayout>
    pub fn create(
        cx: &mut JSContext,
        device: &D::GPUDevice,
        descriptor: &GPUPipelineLayoutDescriptor<D>,
    ) -> DomRoot<GPUPipelineLayout<D>> {
        let bgls = descriptor
            .bindGroupLayouts
            .iter()
            .map(|each| each.id())
            .collect::<Vec<_>>();

        let desc = PipelineLayoutDescriptor {
            label: (&descriptor.parent).convert(),
            // TODO(sagudev): this needs webidl sync
            bind_group_layouts: Cow::Owned(bgls.iter().map(|l| Some(l.0)).collect::<Vec<_>>()),
            immediate_size: 0,
        };

        let pipeline_layout_id = device
            .global_from_reflector()
            .global_wgpu_id_hub()
            .create_pipeline_layout_id();
        device
            .channel()
            .0
            .send(WebGPURequest::CreatePipelineLayout {
                device_id: device.id().0,
                pipeline_layout_id,
                descriptor: desc,
            })
            .expect("Failed to create WebGPU PipelineLayout");

        let pipeline_layout = WebGPUPipelineLayout(pipeline_layout_id);
        GPUPipelineLayout::new(
            cx,
            &*device.global_from_reflector(),
            device.channel(),
            pipeline_layout,
            descriptor.parent.label.clone(),
            bgls,
        )
    }
}

impl<D: DomTypes> GPUPipelineLayoutMethods<D> for GPUPipelineLayout<D> {
    /// <https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label>
    fn Label(&self) -> USVString {
        self.label.borrow().clone()
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label>
    fn SetLabel(&self, no_gc: &NoGC, value: USVString) {
        *self.label.safe_borrow_mut(no_gc) = value;
    }
}
