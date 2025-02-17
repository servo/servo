/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::string::String;

use dom_struct::dom_struct;
use webgpu::wgc::resource;
use webgpu::{wgt, WebGPU, WebGPURequest, WebGPUTexture, WebGPUTextureView};

use super::gpuconvert::convert_texture_descriptor;
use crate::conversions::Convert;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::WebGPUBinding::{
    GPUTextureAspect, GPUTextureDescriptor, GPUTextureDimension, GPUTextureFormat,
    GPUTextureMethods, GPUTextureViewDescriptor,
};
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::reflector::{reflect_dom_object, DomGlobal, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::USVString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::webgpu::gpudevice::GPUDevice;
use crate::dom::webgpu::gputextureview::GPUTextureView;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct GPUTexture {
    reflector_: Reflector,
    #[no_trace]
    texture: WebGPUTexture,
    label: DomRefCell<USVString>,
    device: Dom<GPUDevice>,
    #[ignore_malloc_size_of = "channels are hard"]
    #[no_trace]
    channel: WebGPU,
    #[ignore_malloc_size_of = "defined in wgpu"]
    #[no_trace]
    texture_size: wgt::Extent3d,
    mip_level_count: u32,
    sample_count: u32,
    dimension: GPUTextureDimension,
    format: GPUTextureFormat,
    texture_usage: u32,
}

impl GPUTexture {
    #[allow(clippy::too_many_arguments)]
    fn new_inherited(
        texture: WebGPUTexture,
        device: &GPUDevice,
        channel: WebGPU,
        texture_size: wgt::Extent3d,
        mip_level_count: u32,
        sample_count: u32,
        dimension: GPUTextureDimension,
        format: GPUTextureFormat,
        texture_usage: u32,
        label: USVString,
    ) -> Self {
        Self {
            reflector_: Reflector::new(),
            texture,
            label: DomRefCell::new(label),
            device: Dom::from_ref(device),
            channel,
            texture_size,
            mip_level_count,
            sample_count,
            dimension,
            format,
            texture_usage,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        global: &GlobalScope,
        texture: WebGPUTexture,
        device: &GPUDevice,
        channel: WebGPU,
        texture_size: wgt::Extent3d,
        mip_level_count: u32,
        sample_count: u32,
        dimension: GPUTextureDimension,
        format: GPUTextureFormat,
        texture_usage: u32,
        label: USVString,
        can_gc: CanGc,
    ) -> DomRoot<Self> {
        reflect_dom_object(
            Box::new(GPUTexture::new_inherited(
                texture,
                device,
                channel,
                texture_size,
                mip_level_count,
                sample_count,
                dimension,
                format,
                texture_usage,
                label,
            )),
            global,
            can_gc,
        )
    }
}

impl Drop for GPUTexture {
    fn drop(&mut self) {
        if let Err(e) = self
            .channel
            .0
            .send(WebGPURequest::DropTexture(self.texture.0))
        {
            warn!(
                "Failed to send WebGPURequest::DropTexture({:?}) ({})",
                self.texture.0, e
            );
        };
    }
}

impl GPUTexture {
    pub(crate) fn id(&self) -> WebGPUTexture {
        self.texture
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevice-createtexture>
    pub(crate) fn create(
        device: &GPUDevice,
        descriptor: &GPUTextureDescriptor,
    ) -> Fallible<DomRoot<GPUTexture>> {
        let (desc, size) = convert_texture_descriptor(descriptor, device)?;

        let texture_id = device.global().wgpu_id_hub().create_texture_id();

        device
            .channel()
            .0
            .send(WebGPURequest::CreateTexture {
                device_id: device.id().0,
                texture_id,
                descriptor: desc,
            })
            .expect("Failed to create WebGPU Texture");

        let texture = WebGPUTexture(texture_id);

        Ok(GPUTexture::new(
            &device.global(),
            texture,
            device,
            device.channel().clone(),
            size,
            descriptor.mipLevelCount,
            descriptor.sampleCount,
            descriptor.dimension,
            descriptor.format,
            descriptor.usage,
            descriptor.parent.label.clone(),
            CanGc::note(),
        ))
    }
}

impl GPUTextureMethods<crate::DomTypeHolder> for GPUTexture {
    /// <https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label>
    fn Label(&self) -> USVString {
        self.label.borrow().clone()
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label>
    fn SetLabel(&self, value: USVString) {
        *self.label.borrow_mut() = value;
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gputexture-createview>
    fn CreateView(
        &self,
        descriptor: &GPUTextureViewDescriptor,
    ) -> Fallible<DomRoot<GPUTextureView>> {
        let desc = if !matches!(descriptor.mipLevelCount, Some(0)) &&
            !matches!(descriptor.arrayLayerCount, Some(0))
        {
            Some(resource::TextureViewDescriptor {
                label: (&descriptor.parent).convert(),
                format: descriptor
                    .format
                    .map(|f| self.device.validate_texture_format_required_features(&f))
                    .transpose()?,
                dimension: descriptor.dimension.map(|dimension| dimension.convert()),
                usage: Some(wgt::TextureUsages::from_bits_retain(descriptor.usage)),
                range: wgt::ImageSubresourceRange {
                    aspect: match descriptor.aspect {
                        GPUTextureAspect::All => wgt::TextureAspect::All,
                        GPUTextureAspect::Stencil_only => wgt::TextureAspect::StencilOnly,
                        GPUTextureAspect::Depth_only => wgt::TextureAspect::DepthOnly,
                    },
                    base_mip_level: descriptor.baseMipLevel,
                    mip_level_count: descriptor.mipLevelCount,
                    base_array_layer: descriptor.baseArrayLayer,
                    array_layer_count: descriptor.arrayLayerCount,
                },
            })
        } else {
            self.device
                .dispatch_error(webgpu::Error::Validation(String::from(
                    "arrayLayerCount and mipLevelCount cannot be 0",
                )));
            None
        };

        let texture_view_id = self.global().wgpu_id_hub().create_texture_view_id();

        self.channel
            .0
            .send(WebGPURequest::CreateTextureView {
                texture_id: self.texture.0,
                texture_view_id,
                device_id: self.device.id().0,
                descriptor: desc,
            })
            .expect("Failed to create WebGPU texture view");

        let texture_view = WebGPUTextureView(texture_view_id);

        Ok(GPUTextureView::new(
            &self.global(),
            self.channel.clone(),
            texture_view,
            self,
            descriptor.parent.label.clone(),
            CanGc::note(),
        ))
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gputexture-destroy>
    fn Destroy(&self) {
        if let Err(e) = self
            .channel
            .0
            .send(WebGPURequest::DestroyTexture(self.texture.0))
        {
            warn!(
                "Failed to send WebGPURequest::DestroyTexture({:?}) ({})",
                self.texture.0, e
            );
        };
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gputexture-width>
    fn Width(&self) -> u32 {
        self.texture_size.width
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gputexture-height>
    fn Height(&self) -> u32 {
        self.texture_size.height
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gputexture-depthorarraylayers>
    fn DepthOrArrayLayers(&self) -> u32 {
        self.texture_size.depth_or_array_layers
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gputexture-miplevelcount>
    fn MipLevelCount(&self) -> u32 {
        self.mip_level_count
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gputexture-samplecount>
    fn SampleCount(&self) -> u32 {
        self.sample_count
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gputexture-dimension>
    fn Dimension(&self) -> GPUTextureDimension {
        self.dimension
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gputexture-format>
    fn Format(&self) -> GPUTextureFormat {
        self.format
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gputexture-usage>
    fn Usage(&self) -> u32 {
        self.texture_usage
    }
}
