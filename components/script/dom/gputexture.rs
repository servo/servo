/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::string::String;

use dom_struct::dom_struct;
use webgpu::wgc::resource;
use webgpu::{wgt, WebGPU, WebGPURequest, WebGPUTexture, WebGPUTextureView};

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::WebGPUBinding::{
    GPUExtent3DDict, GPUTextureAspect, GPUTextureDimension, GPUTextureFormat, GPUTextureMethods,
    GPUTextureViewDescriptor,
};
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::USVString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::gpuconvert::{
    convert_label, convert_texture_format, convert_texture_view_dimension,
};
use crate::dom::gpudevice::GPUDevice;
use crate::dom::gputextureview::GPUTextureView;

#[dom_struct]
pub struct GPUTexture {
    reflector_: Reflector,
    #[no_trace]
    texture: WebGPUTexture,
    label: DomRefCell<USVString>,
    device: Dom<GPUDevice>,
    #[ignore_malloc_size_of = "channels are hard"]
    #[no_trace]
    channel: WebGPU,
    #[ignore_malloc_size_of = "defined in webgpu"]
    texture_size: GPUExtent3DDict,
    mip_level_count: u32,
    sample_count: u32,
    dimension: GPUTextureDimension,
    format: GPUTextureFormat,
    texture_usage: u32,
    destroyed: Cell<bool>,
}

impl GPUTexture {
    #[allow(clippy::too_many_arguments)]
    fn new_inherited(
        texture: WebGPUTexture,
        device: &GPUDevice,
        channel: WebGPU,
        texture_size: GPUExtent3DDict,
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
            destroyed: Cell::new(false),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new(
        global: &GlobalScope,
        texture: WebGPUTexture,
        device: &GPUDevice,
        channel: WebGPU,
        texture_size: GPUExtent3DDict,
        mip_level_count: u32,
        sample_count: u32,
        dimension: GPUTextureDimension,
        format: GPUTextureFormat,
        texture_usage: u32,
        label: USVString,
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
        )
    }
}

impl Drop for GPUTexture {
    fn drop(&mut self) {
        if self.destroyed.get() {
            return;
        }
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
    pub fn id(&self) -> WebGPUTexture {
        self.texture
    }
}

impl GPUTextureMethods for GPUTexture {
    /// <https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label>
    fn Label(&self) -> USVString {
        self.label.borrow().clone()
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label>
    fn SetLabel(&self, value: USVString) {
        *self.label.borrow_mut() = value;
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gputexture-createview>
    fn CreateView(&self, descriptor: &GPUTextureViewDescriptor) -> DomRoot<GPUTextureView> {
        let desc = if !matches!(descriptor.mipLevelCount, Some(0)) &&
            !matches!(descriptor.arrayLayerCount, Some(0))
        {
            Some(resource::TextureViewDescriptor {
                label: convert_label(&descriptor.parent),
                format: descriptor.format.map(convert_texture_format),
                dimension: descriptor.dimension.map(convert_texture_view_dimension),
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

        let texture_view_id = self
            .global()
            .wgpu_id_hub()
            .create_texture_view_id(self.device.id().0.backend());

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

        GPUTextureView::new(
            &self.global(),
            self.channel.clone(),
            texture_view,
            self,
            descriptor.parent.label.clone().unwrap_or_default(),
        )
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gputexture-destroy>
    fn Destroy(&self) {
        if self.destroyed.get() {
            return;
        }
        if let Err(e) = self.channel.0.send(WebGPURequest::DestroyTexture {
            device_id: self.device.id().0,
            texture_id: self.texture.0,
        }) {
            warn!(
                "Failed to send WebGPURequest::DestroyTexture({:?}) ({})",
                self.texture.0, e
            );
        };
        self.destroyed.set(true);
    }
}
