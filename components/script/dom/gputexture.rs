/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::GPUTextureBinding::{
    GPUExtent3DDict, GPUTextureDimension, GPUTextureFormat, GPUTextureMethods,
};
use crate::dom::bindings::codegen::Bindings::GPUTextureViewBinding::{
    GPUTextureAspect, GPUTextureViewDescriptor,
};
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::gpudevice::{convert_texture_format, convert_texture_view_dimension};
use crate::dom::gputextureview::GPUTextureView;
use dom_struct::dom_struct;
use std::cell::Cell;
use webgpu::{wgt, WebGPU, WebGPUDevice, WebGPURequest, WebGPUTexture, WebGPUTextureView};

#[dom_struct]
pub struct GPUTexture {
    reflector_: Reflector,
    texture: WebGPUTexture,
    label: DomRefCell<Option<DOMString>>,
    device: WebGPUDevice,
    #[ignore_malloc_size_of = "channels are hard"]
    channel: WebGPU,
    #[ignore_malloc_size_of = "defined in webgpu"]
    texture_size: GPUExtent3DDict,
    mip_level_count: u32,
    sample_count: u32,
    dimension: GPUTextureDimension,
    format: GPUTextureFormat,
    texture_usage: u32,
    valid: Cell<bool>,
}

impl GPUTexture {
    fn new_inherited(
        texture: WebGPUTexture,
        device: WebGPUDevice,
        channel: WebGPU,
        texture_size: GPUExtent3DDict,
        mip_level_count: u32,
        sample_count: u32,
        dimension: GPUTextureDimension,
        format: GPUTextureFormat,
        texture_usage: u32,
        valid: bool,
    ) -> Self {
        Self {
            reflector_: Reflector::new(),
            texture,
            label: DomRefCell::new(None),
            device,
            channel,
            texture_size,
            mip_level_count,
            sample_count,
            dimension,
            format,
            texture_usage,
            valid: Cell::new(valid),
        }
    }

    pub fn new(
        global: &GlobalScope,
        texture: WebGPUTexture,
        device: WebGPUDevice,
        channel: WebGPU,
        texture_size: GPUExtent3DDict,
        mip_level_count: u32,
        sample_count: u32,
        dimension: GPUTextureDimension,
        format: GPUTextureFormat,
        texture_usage: u32,
        valid: bool,
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
                valid,
            )),
            global,
        )
    }
}

impl Drop for GPUTexture {
    fn drop(&mut self) {
        self.Destroy()
    }
}

impl GPUTexture {
    pub fn id(&self) -> WebGPUTexture {
        self.texture
    }
}

impl GPUTextureMethods for GPUTexture {
    /// https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label
    fn GetLabel(&self) -> Option<DOMString> {
        self.label.borrow().clone()
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label
    fn SetLabel(&self, value: Option<DOMString>) {
        *self.label.borrow_mut() = value;
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gputexture-createview
    fn CreateView(&self, descriptor: &GPUTextureViewDescriptor) -> DomRoot<GPUTextureView> {
        let desc = wgt::TextureViewDescriptor {
            label: Default::default(),
            format: convert_texture_format(descriptor.format.unwrap_or(self.format)),
            dimension: match descriptor.dimension {
                Some(d) => convert_texture_view_dimension(d),
                None => match self.dimension {
                    GPUTextureDimension::_1d => wgt::TextureViewDimension::D1,
                    GPUTextureDimension::_2d => {
                        if self.texture_size.depth > 1 && descriptor.arrayLayerCount == 0 {
                            wgt::TextureViewDimension::D2Array
                        } else {
                            wgt::TextureViewDimension::D2
                        }
                    },
                    GPUTextureDimension::_3d => wgt::TextureViewDimension::D3,
                },
            },
            aspect: match descriptor.aspect {
                GPUTextureAspect::All => wgt::TextureAspect::All,
                GPUTextureAspect::Stencil_only => wgt::TextureAspect::StencilOnly,
                GPUTextureAspect::Depth_only => wgt::TextureAspect::DepthOnly,
            },
            base_mip_level: descriptor.baseMipLevel,
            level_count: if descriptor.mipLevelCount == 0 {
                self.mip_level_count - descriptor.baseMipLevel
            } else {
                descriptor.mipLevelCount
            },
            base_array_layer: descriptor.baseArrayLayer,
            array_layer_count: if descriptor.arrayLayerCount == 0 {
                self.texture_size.depth - descriptor.baseArrayLayer
            } else {
                descriptor.arrayLayerCount
            },
        };

        let texture_view_id = self
            .global()
            .wgpu_id_hub()
            .lock()
            .create_texture_view_id(self.device.0.backend());

        self.channel
            .0
            .send(WebGPURequest::CreateTextureView {
                texture_id: self.texture.0,
                texture_view_id,
                descriptor: desc,
            })
            .expect("Failed to create WebGPU texture view");

        let texture_view = WebGPUTextureView(texture_view_id);

        GPUTextureView::new(&self.global(), texture_view, self.device, true)
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gputexture-destroy
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
}
