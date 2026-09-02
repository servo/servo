/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::string::String;

use dom_struct::dom_struct;
use js::context::{JSContext, NoGC};
use log::warn;
use malloc_size_of_derive::MallocSizeOf;
use script_bindings::DomTypes;
use script_bindings::cell::DomRefCell;
use script_bindings::codegen::GenericBindings::WebGPUBinding::{
    GPUTextureAspect, GPUTextureDescriptor, GPUTextureDimension, GPUTextureFormat,
    GPUTextureMethods, GPUTextureViewDescriptor, GPUTextureWrap,
};
use script_bindings::dom::MutNullableDom;
use script_bindings::reflector::{DomGlobalGeneric, Reflector, reflect_dom_object_with_wrap};
use webgpu_traits::{WebGPU, WebGPURequest, WebGPUTexture, WebGPUTextureView};
use wgpu_core::resource::{self, TextureDescriptor};

use crate::JSTraceable;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::USVString;
use crate::gpuconvert::{WebGPUConvert, convert_texture_descriptor};
use crate::gputextureview::GPUTextureView;
use crate::traits::{Equivalence, GPUDeviceTrait, WebGPUGlobalTrait};

#[derive(JSTraceable, MallocSizeOf)]
struct DroppableGPUTexture {
    #[no_trace]
    channel: WebGPU,
    #[no_trace]
    texture: WebGPUTexture,
}

impl Drop for DroppableGPUTexture {
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

#[dom_struct]
pub struct GPUTexture<D: DomTypes> {
    reflector_: Reflector,
    label: DomRefCell<USVString>,
    device: Dom<D::GPUDevice>,
    #[no_trace]
    #[ignore_malloc_size_of = "External type"]
    texture_size: wgpu_types::Extent3d,
    mip_level_count: u32,
    sample_count: u32,
    dimension: GPUTextureDimension,
    format: GPUTextureFormat,
    texture_usage: u32,
    droppable: DroppableGPUTexture,
    default_view: MutNullableDom<GPUTextureView<D>>,
}

impl<D: Equivalence> GPUTexture<D> {
    #[expect(clippy::too_many_arguments)]
    fn new_inherited(
        texture: WebGPUTexture,
        device: &D::GPUDevice,
        channel: WebGPU,
        texture_size: wgpu_types::Extent3d,
        mip_level_count: u32,
        sample_count: u32,
        dimension: GPUTextureDimension,
        format: GPUTextureFormat,
        texture_usage: u32,
        label: USVString,
    ) -> Self {
        Self {
            reflector_: Reflector::new(),
            label: DomRefCell::new(label),
            device: Dom::from_ref(device),
            texture_size,
            mip_level_count,
            sample_count,
            dimension,
            format,
            texture_usage,
            droppable: DroppableGPUTexture { channel, texture },
            default_view: MutNullableDom::new(None),
        }
    }

    #[expect(clippy::too_many_arguments)]
    pub(crate) fn new(
        cx: &mut JSContext,
        global: &D::GlobalScope,
        texture: WebGPUTexture,
        device: &D::GPUDevice,
        channel: WebGPU,
        texture_size: wgpu_types::Extent3d,
        mip_level_count: u32,
        sample_count: u32,
        dimension: GPUTextureDimension,
        format: GPUTextureFormat,
        texture_usage: u32,
        label: USVString,
    ) -> DomRoot<Self> {
        reflect_dom_object_with_wrap::<D, _, _>(
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
            cx,
            GPUTextureWrap::<D>,
        )
    }
}

impl<D> GPUTexture<D>
where
    D: Equivalence,
    D::GPUDevice: GPUDeviceTrait<D>,
    D::GlobalScope: WebGPUGlobalTrait,
{
    pub fn id(&self) -> WebGPUTexture {
        self.droppable.texture
    }

    pub fn wgpu_texture_descriptor(&self) -> TextureDescriptor<'static> {
        TextureDescriptor {
            label: Some(self.label.borrow().to_string().into()),
            size: self.texture_size,
            mip_level_count: self.mip_level_count,
            sample_count: self.sample_count,
            dimension: self.dimension.convert(),
            format: self.format.convert(),
            usage: wgpu_types::TextureUsages::from_bits_retain(self.texture_usage),
            view_formats: vec![],
        }
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevice-createtexture>
    pub fn create(
        cx: &mut JSContext,
        device: &D::GPUDevice,
        descriptor: &GPUTextureDescriptor,
    ) -> Fallible<DomRoot<GPUTexture<D>>> {
        let (desc, size) = convert_texture_descriptor::<D>(descriptor, device)?;

        let texture_id = device
            .global_from_reflector()
            .global_wgpu_id_hub()
            .create_texture_id();

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
            cx,
            &*device.global_from_reflector(),
            texture,
            device,
            device.channel(),
            size,
            descriptor.mipLevelCount,
            descriptor.sampleCount,
            descriptor.dimension,
            descriptor.format,
            descriptor.usage,
            descriptor.parent.label.clone(),
        ))
    }

    pub(crate) fn get_default_view(&self, cx: &mut JSContext) -> WebGPUTextureView {
        self.default_view
            .or_init(|| {
                self.CreateView(cx, &GPUTextureViewDescriptor::default())
                    .expect("Default descriptor should always be valid.")
            })
            .id()
    }
}

impl<D> GPUTextureMethods<D> for GPUTexture<D>
where
    D: Equivalence,
    D::GPUDevice: GPUDeviceTrait<D>,
    D::GlobalScope: WebGPUGlobalTrait,
    Self: DomGlobalGeneric<D>,
{
    /// <https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label>
    fn Label(&self) -> USVString {
        self.label.borrow().clone()
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label>
    fn SetLabel(&self, no_gc: &NoGC, value: USVString) {
        *self.label.safe_borrow_mut(no_gc) = value;
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gputexture-createview>
    fn CreateView(
        &self,
        cx: &mut JSContext,
        descriptor: &GPUTextureViewDescriptor,
    ) -> Fallible<DomRoot<GPUTextureView<D>>> {
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
                usage: Some(wgpu_types::TextureUsages::from_bits_retain(
                    descriptor.usage,
                )),
                range: wgpu_types::ImageSubresourceRange {
                    aspect: match descriptor.aspect {
                        GPUTextureAspect::All => wgpu_types::TextureAspect::All,
                        GPUTextureAspect::Stencil_only => wgpu_types::TextureAspect::StencilOnly,
                        GPUTextureAspect::Depth_only => wgpu_types::TextureAspect::DepthOnly,
                    },
                    base_mip_level: descriptor.baseMipLevel,
                    mip_level_count: descriptor.mipLevelCount,
                    base_array_layer: descriptor.baseArrayLayer,
                    array_layer_count: descriptor.arrayLayerCount,
                },
            })
        } else {
            self.device
                .dispatch_error(webgpu_traits::Error::Validation(String::from(
                    "arrayLayerCount and mipLevelCount cannot be 0",
                )));
            None
        };

        let texture_view_id = self
            .global_from_reflector()
            .global_wgpu_id_hub()
            .create_texture_view_id();

        self.droppable
            .channel
            .0
            .send(WebGPURequest::CreateTextureView {
                texture_id: self.id().0,
                texture_view_id,
                device_id: self.device.id().0,
                descriptor: desc,
            })
            .expect("Failed to create WebGPU texture view");

        let texture_view = WebGPUTextureView(texture_view_id);

        Ok(GPUTextureView::new(
            cx,
            &*self.global_from_reflector(),
            self.droppable.channel.clone(),
            texture_view,
            self,
            descriptor.parent.label.clone(),
        ))
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gputexture-destroy>
    fn Destroy(&self) {
        if let Err(e) = self
            .droppable
            .channel
            .0
            .send(WebGPURequest::DestroyTexture(self.id().0))
        {
            warn!(
                "Failed to send WebGPURequest::DestroyTexture({:?}) ({})",
                self.id().0,
                e
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
