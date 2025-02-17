/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::Cow;

use dom_struct::dom_struct;
use webgpu::wgc::command::{
    bundle_ffi as wgpu_bundle, RenderBundleEncoder, RenderBundleEncoderDescriptor,
};
use webgpu::{wgt, WebGPU, WebGPURenderBundle, WebGPURequest};

use crate::conversions::Convert;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::WebGPUBinding::{
    GPUIndexFormat, GPURenderBundleDescriptor, GPURenderBundleEncoderDescriptor,
    GPURenderBundleEncoderMethods,
};
use crate::dom::bindings::import::module::Fallible;
use crate::dom::bindings::reflector::{reflect_dom_object, DomGlobal, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::USVString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::webgpu::gpubindgroup::GPUBindGroup;
use crate::dom::webgpu::gpubuffer::GPUBuffer;
use crate::dom::webgpu::gpudevice::GPUDevice;
use crate::dom::webgpu::gpurenderbundle::GPURenderBundle;
use crate::dom::webgpu::gpurenderpipeline::GPURenderPipeline;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct GPURenderBundleEncoder {
    reflector_: Reflector,
    #[ignore_malloc_size_of = "channels are hard"]
    #[no_trace]
    channel: WebGPU,
    device: Dom<GPUDevice>,
    #[ignore_malloc_size_of = "defined in wgpu-core"]
    #[no_trace]
    render_bundle_encoder: DomRefCell<Option<RenderBundleEncoder>>,
    label: DomRefCell<USVString>,
}

impl GPURenderBundleEncoder {
    fn new_inherited(
        render_bundle_encoder: RenderBundleEncoder,
        device: &GPUDevice,
        channel: WebGPU,
        label: USVString,
    ) -> Self {
        Self {
            reflector_: Reflector::new(),
            render_bundle_encoder: DomRefCell::new(Some(render_bundle_encoder)),
            device: Dom::from_ref(device),
            channel,
            label: DomRefCell::new(label),
        }
    }

    pub(crate) fn new(
        global: &GlobalScope,
        render_bundle_encoder: RenderBundleEncoder,
        device: &GPUDevice,
        channel: WebGPU,
        label: USVString,
        can_gc: CanGc,
    ) -> DomRoot<Self> {
        reflect_dom_object(
            Box::new(GPURenderBundleEncoder::new_inherited(
                render_bundle_encoder,
                device,
                channel,
                label,
            )),
            global,
            can_gc,
        )
    }
}

impl GPURenderBundleEncoder {
    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevice-createrenderbundleencoder>
    pub(crate) fn create(
        device: &GPUDevice,
        descriptor: &GPURenderBundleEncoderDescriptor,
    ) -> Fallible<DomRoot<GPURenderBundleEncoder>> {
        let desc = RenderBundleEncoderDescriptor {
            label: (&descriptor.parent.parent).convert(),
            color_formats: Cow::Owned(
                descriptor
                    .parent
                    .colorFormats
                    .iter()
                    .map(|format| {
                        device
                            .validate_texture_format_required_features(format)
                            .map(Some)
                    })
                    .collect::<Fallible<Vec<_>>>()?,
            ),
            depth_stencil: descriptor
                .parent
                .depthStencilFormat
                .map(|dsf| {
                    device
                        .validate_texture_format_required_features(&dsf)
                        .map(|format| wgt::RenderBundleDepthStencil {
                            format,
                            depth_read_only: descriptor.depthReadOnly,
                            stencil_read_only: descriptor.stencilReadOnly,
                        })
                })
                .transpose()?,
            sample_count: descriptor.parent.sampleCount,
            multiview: None,
        };

        // Handle error gracefully
        let render_bundle_encoder = RenderBundleEncoder::new(&desc, device.id().0, None).unwrap();

        Ok(GPURenderBundleEncoder::new(
            &device.global(),
            render_bundle_encoder,
            device,
            device.channel().clone(),
            descriptor.parent.parent.label.clone(),
            CanGc::note(),
        ))
    }
}

impl GPURenderBundleEncoderMethods<crate::DomTypeHolder> for GPURenderBundleEncoder {
    /// <https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label>
    fn Label(&self) -> USVString {
        self.label.borrow().clone()
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label>
    fn SetLabel(&self, value: USVString) {
        *self.label.borrow_mut() = value;
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpuprogrammablepassencoder-setbindgroup>
    #[allow(unsafe_code)]
    fn SetBindGroup(&self, index: u32, bind_group: &GPUBindGroup, dynamic_offsets: Vec<u32>) {
        if let Some(encoder) = self.render_bundle_encoder.borrow_mut().as_mut() {
            unsafe {
                wgpu_bundle::wgpu_render_bundle_set_bind_group(
                    encoder,
                    index,
                    Some(bind_group.id().0),
                    dynamic_offsets.as_ptr(),
                    dynamic_offsets.len(),
                )
            };
        }
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpurenderencoderbase-setpipeline>
    fn SetPipeline(&self, pipeline: &GPURenderPipeline) {
        if let Some(encoder) = self.render_bundle_encoder.borrow_mut().as_mut() {
            wgpu_bundle::wgpu_render_bundle_set_pipeline(encoder, pipeline.id().0);
        }
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpurenderencoderbase-setindexbuffer>
    fn SetIndexBuffer(
        &self,
        buffer: &GPUBuffer,
        index_format: GPUIndexFormat,
        offset: u64,
        size: u64,
    ) {
        if let Some(encoder) = self.render_bundle_encoder.borrow_mut().as_mut() {
            wgpu_bundle::wgpu_render_bundle_set_index_buffer(
                encoder,
                buffer.id().0,
                match index_format {
                    GPUIndexFormat::Uint16 => wgt::IndexFormat::Uint16,
                    GPUIndexFormat::Uint32 => wgt::IndexFormat::Uint32,
                },
                offset,
                wgt::BufferSize::new(size),
            );
        }
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpurenderencoderbase-setvertexbuffer>
    fn SetVertexBuffer(&self, slot: u32, buffer: &GPUBuffer, offset: u64, size: u64) {
        if let Some(encoder) = self.render_bundle_encoder.borrow_mut().as_mut() {
            wgpu_bundle::wgpu_render_bundle_set_vertex_buffer(
                encoder,
                slot,
                buffer.id().0,
                offset,
                wgt::BufferSize::new(size),
            );
        }
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpurenderencoderbase-draw>
    fn Draw(&self, vertex_count: u32, instance_count: u32, first_vertex: u32, first_instance: u32) {
        if let Some(encoder) = self.render_bundle_encoder.borrow_mut().as_mut() {
            wgpu_bundle::wgpu_render_bundle_draw(
                encoder,
                vertex_count,
                instance_count,
                first_vertex,
                first_instance,
            );
        }
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpurenderencoderbase-drawindexed>
    fn DrawIndexed(
        &self,
        index_count: u32,
        instance_count: u32,
        first_index: u32,
        base_vertex: i32,
        first_instance: u32,
    ) {
        if let Some(encoder) = self.render_bundle_encoder.borrow_mut().as_mut() {
            wgpu_bundle::wgpu_render_bundle_draw_indexed(
                encoder,
                index_count,
                instance_count,
                first_index,
                base_vertex,
                first_instance,
            );
        }
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpurenderencoderbase-drawindirect>
    fn DrawIndirect(&self, indirect_buffer: &GPUBuffer, indirect_offset: u64) {
        if let Some(encoder) = self.render_bundle_encoder.borrow_mut().as_mut() {
            wgpu_bundle::wgpu_render_bundle_draw_indirect(
                encoder,
                indirect_buffer.id().0,
                indirect_offset,
            );
        }
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpurenderencoderbase-drawindexedindirect>
    fn DrawIndexedIndirect(&self, indirect_buffer: &GPUBuffer, indirect_offset: u64) {
        if let Some(encoder) = self.render_bundle_encoder.borrow_mut().as_mut() {
            wgpu_bundle::wgpu_render_bundle_draw_indexed_indirect(
                encoder,
                indirect_buffer.id().0,
                indirect_offset,
            );
        }
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpurenderbundleencoder-finish>
    fn Finish(&self, descriptor: &GPURenderBundleDescriptor) -> DomRoot<GPURenderBundle> {
        let desc = wgt::RenderBundleDescriptor {
            label: (&descriptor.parent).convert(),
        };
        let encoder = self.render_bundle_encoder.borrow_mut().take().unwrap();
        let render_bundle_id = self.global().wgpu_id_hub().create_render_bundle_id();

        self.channel
            .0
            .send(WebGPURequest::RenderBundleEncoderFinish {
                render_bundle_encoder: encoder,
                descriptor: desc,
                render_bundle_id,
                device_id: self.device.id().0,
            })
            .expect("Failed to send RenderBundleEncoderFinish");

        let render_bundle = WebGPURenderBundle(render_bundle_id);
        GPURenderBundle::new(
            &self.global(),
            render_bundle,
            self.device.id(),
            self.channel.clone(),
            descriptor.parent.label.clone(),
            CanGc::note(),
        )
    }
}
