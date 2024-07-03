/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use arrayvec::ArrayVec;
use dom_struct::dom_struct;
use euclid::default::Size2D;
use ipc_channel::ipc;
use script_layout_interface::HTMLCanvasDataSource;
use webgpu::wgc::id;
use webgpu::{wgt, WebGPU, WebGPURequest, WebGPUTexture, PRESENTATION_BUFFER_COUNT};
use webrender_api::{
    units, ExternalImageData, ExternalImageId, ExternalImageType, ImageData, ImageDescriptor,
    ImageDescriptorFlags, ImageFormat, ImageKey,
};

use super::bindings::codegen::Bindings::WebGPUBinding::GPUTextureUsageConstants;
use super::bindings::codegen::UnionTypes::HTMLCanvasElementOrOffscreenCanvas;
use super::bindings::error::{Error, Fallible};
use super::bindings::root::MutNullableDom;
use super::gputexture::GPUTexture;
use crate::dom::bindings::codegen::Bindings::HTMLCanvasElementBinding::HTMLCanvasElement_Binding::HTMLCanvasElementMethods;
use crate::dom::bindings::codegen::Bindings::WebGPUBinding::{
    GPUCanvasConfiguration, GPUCanvasContextMethods, GPUDeviceMethods, GPUExtent3D,
    GPUExtent3DDict, GPUObjectDescriptorBase, GPUTextureDescriptor, GPUTextureDimension,
    GPUTextureFormat,
};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject, Reflector};
use crate::dom::bindings::root::{DomRoot, LayoutDom};
use crate::dom::globalscope::GlobalScope;
use crate::dom::htmlcanvaselement::{HTMLCanvasElement, LayoutCanvasRenderingContextHelpers};
use crate::dom::node::{document_from_node, Node, NodeDamage};

#[derive(Clone, Copy, Debug, Eq, Hash, MallocSizeOf, Ord, PartialEq, PartialOrd)]
pub struct WebGPUContextId(pub u64);

// TODO: make all this derivables available via new Bindings.conf option
impl Clone for GPUCanvasConfiguration {
    fn clone(&self) -> Self {
        Self {
            alphaMode: self.alphaMode,
            device: self.device.clone(),
            format: self.format,
            usage: self.usage,
            viewFormats: self.viewFormats.clone(),
        }
    }
}

impl Clone for HTMLCanvasElementOrOffscreenCanvas {
    fn clone(&self) -> Self {
        match self {
            Self::HTMLCanvasElement(arg0) => Self::HTMLCanvasElement(arg0.clone()),
            Self::OffscreenCanvas(arg0) => Self::OffscreenCanvas(arg0.clone()),
        }
    }
}

impl malloc_size_of::MallocSizeOf for GPUTextureDescriptor {
    fn size_of(&self, ops: &mut malloc_size_of::MallocSizeOfOps) -> usize {
        let Self {
            parent,
            dimension,
            format,
            mipLevelCount,
            sampleCount,
            size,
            usage,
            viewFormats,
        } = self;
        parent.size_of(ops) +
            dimension.size_of(ops) +
            format.size_of(ops) +
            mipLevelCount.size_of(ops) +
            sampleCount.size_of(ops) +
            size.size_of(ops) +
            usage.size_of(ops) +
            viewFormats.size_of(ops)
    }
}

impl malloc_size_of::MallocSizeOf for HTMLCanvasElementOrOffscreenCanvas {
    fn size_of(&self, ops: &mut malloc_size_of::MallocSizeOfOps) -> usize {
        match self {
            HTMLCanvasElementOrOffscreenCanvas::HTMLCanvasElement(canvas) => canvas.size_of(ops),
            HTMLCanvasElementOrOffscreenCanvas::OffscreenCanvas(canvas) => canvas.size_of(ops),
        }
    }
}

#[dom_struct]
pub struct GPUCanvasContext {
    reflector_: Reflector,
    #[ignore_malloc_size_of = "channels are hard"]
    #[no_trace]
    channel: WebGPU,
    /// <https://gpuweb.github.io/gpuweb/#dom-gpucanvascontext-canvas>
    canvas: HTMLCanvasElementOrOffscreenCanvas,
    // TODO: can we have wgpu surface that is hw accelerated inside wr ...
    #[ignore_malloc_size_of = "Defined in webrender"]
    #[no_trace]
    webrender_image: Cell<Option<webrender_api::ImageKey>>,
    context_id: WebGPUContextId,
    /// <https://gpuweb.github.io/gpuweb/#dom-gpucanvascontext-currenttexture-slot>
    texture: MutNullableDom<GPUTexture>,
}

impl GPUCanvasContext {
    fn new_inherited(canvas: HTMLCanvasElementOrOffscreenCanvas, channel: WebGPU) -> Self {
        let (sender, receiver) = ipc::channel().unwrap();
        if let Err(e) = channel.0.send(WebGPURequest::CreateContext(sender)) {
            warn!("Failed to send CreateContext ({:?})", e);
        }
        let external_id = receiver.recv().unwrap();
        Self {
            reflector_: Reflector::new(),
            channel,
            canvas,
            webrender_image: Cell::new(None),
            context_id: WebGPUContextId(external_id.0),
            texture: MutNullableDom::default(),
        }
    }

    pub fn new(global: &GlobalScope, canvas: &HTMLCanvasElement, channel: WebGPU) -> DomRoot<Self> {
        reflect_dom_object(
            Box::new(GPUCanvasContext::new_inherited(
                HTMLCanvasElementOrOffscreenCanvas::HTMLCanvasElement(DomRoot::from_ref(canvas)),
                channel,
            )),
            global,
        )
    }
}

impl GPUCanvasContext {
    fn layout_handle(&self) -> HTMLCanvasDataSource {
        let image_key = if self.webrender_image.get().is_some() {
            self.webrender_image.get().unwrap()
        } else {
            ImageKey::DUMMY
        };
        HTMLCanvasDataSource::WebGPU(image_key)
    }

    pub fn send_swap_chain_present(&self) {
        let texture_id = self.texture_id().unwrap().0;
        let encoder_id = self
            .global()
            .wgpu_id_hub()
            .create_command_encoder_id(texture_id.backend());
        if let Err(e) = self.channel.0.send(WebGPURequest::SwapChainPresent {
            external_id: self.context_id.0,
            texture_id,
            encoder_id,
        }) {
            warn!(
                "Failed to send UpdateWebrenderData({:?}) ({})",
                self.context_id, e
            );
        }
    }

    pub fn context_id(&self) -> WebGPUContextId {
        self.context_id
    }

    pub fn texture_id(&self) -> Option<WebGPUTexture> {
        self.texture.get().map(|t| t.id())
    }

    pub fn mark_as_dirty(&self) {
        if let HTMLCanvasElementOrOffscreenCanvas::HTMLCanvasElement(canvas) = &self.canvas {
            canvas.upcast::<Node>().dirty(NodeDamage::OtherNodeDamage);
            let document = document_from_node(&**canvas);
            document.add_dirty_webgpu_canvas(self);
        }
        // TODO(sagudev): offscreen canvas also dirty?
    }

    fn size(&self) -> Size2D<u64> {
        match &self.canvas {
            HTMLCanvasElementOrOffscreenCanvas::HTMLCanvasElement(canvas) => {
                Size2D::new(canvas.Width() as u64, canvas.Height() as u64)
            },
            HTMLCanvasElementOrOffscreenCanvas::OffscreenCanvas(canvas) => canvas.get_size(),
        }
    }
}

impl LayoutCanvasRenderingContextHelpers for LayoutDom<'_, GPUCanvasContext> {
    fn canvas_data_source(self) -> HTMLCanvasDataSource {
        (*self.unsafe_get()).layout_handle()
    }
}

impl GPUCanvasContextMethods for GPUCanvasContext {
    /// <https://gpuweb.github.io/gpuweb/#dom-gpucanvascontext-canvas>
    fn Canvas(&self) -> HTMLCanvasElementOrOffscreenCanvas {
        self.canvas.clone()
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpucanvascontext-configure>
    fn Configure(&self, descriptor: &GPUCanvasConfiguration) -> Fallible<()> {
        // Step 1 is let
        // Step 2
        // TODO: device features
        let format = match descriptor.format {
            GPUTextureFormat::Rgba8unorm | GPUTextureFormat::Rgba8unorm_srgb => ImageFormat::RGBA8,
            GPUTextureFormat::Bgra8unorm | GPUTextureFormat::Bgra8unorm_srgb => ImageFormat::BGRA8,
            _ => {
                return Err(Error::Type(format!(
                    "SwapChain format({:?}) not supported",
                    descriptor.format
                )))
            },
        };

        // Step 3
        // TODO: device features

        // Step 4
        let size = self.size();
        let text_desc = GPUTextureDescriptor {
            format: descriptor.format,
            mipLevelCount: 1,
            sampleCount: 1,
            usage: descriptor.usage | GPUTextureUsageConstants::COPY_SRC, // TODO: specs
            size: GPUExtent3D::GPUExtent3DDict(GPUExtent3DDict {
                width: size.width as u32,
                height: size.height as u32,
                depthOrArrayLayers: 1,
            }),
            viewFormats: descriptor.viewFormats.clone(),
            // other members to default
            parent: GPUObjectDescriptorBase { label: None },
            dimension: GPUTextureDimension::_2d,
        };

        // Step 8
        let image_desc = ImageDescriptor {
            format,
            size: units::DeviceIntSize::new(size.width as i32, size.height as i32),
            stride: Some(
                (((size.width as u32 * 4) | (wgt::COPY_BYTES_PER_ROW_ALIGNMENT - 1)) + 1) as i32,
            ),
            offset: 0,
            flags: ImageDescriptorFlags::from_bits(1).unwrap(),
        };

        let image_data = ImageData::External(ExternalImageData {
            id: ExternalImageId(self.context_id.0),
            channel_index: 0,
            image_type: ExternalImageType::Buffer,
        });

        let (sender, receiver) = ipc::channel().unwrap();

        let mut buffer_ids = ArrayVec::<id::BufferId, PRESENTATION_BUFFER_COUNT>::new();
        for _ in 0..PRESENTATION_BUFFER_COUNT {
            buffer_ids.push(
                self.global()
                    .wgpu_id_hub()
                    .create_buffer_id(descriptor.device.id().0.backend()),
            );
        }

        self.channel
            .0
            .send(WebGPURequest::CreateSwapChain {
                device_id: descriptor.device.id().0,
                buffer_ids,
                external_id: self.context_id.0,
                sender,
                image_desc,
                image_data,
            })
            .expect("Failed to create WebGPU SwapChain");

        self.texture
            .set(Some(&descriptor.device.CreateTexture(&text_desc).unwrap()));

        self.webrender_image.set(Some(receiver.recv().unwrap()));
        Ok(())
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpucanvascontext-unconfigure>
    fn Unconfigure(&self) {
        if let Some(image_key) = self.webrender_image.take() {
            if let Err(e) = self.channel.0.send(WebGPURequest::DestroySwapChain {
                external_id: self.context_id.0,
                image_key,
            }) {
                warn!(
                    "Failed to send DestroySwapChain-ImageKey({:?}) ({})",
                    image_key, e
                );
            }
        }
        self.texture.take();
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpucanvascontext-getcurrenttexture>
    fn GetCurrentTexture(&self) -> Fallible<DomRoot<GPUTexture>> {
        // Step 5.
        self.mark_as_dirty();
        // Step 6.
        self.texture.get().ok_or(Error::InvalidState)
    }
}

impl Drop for GPUCanvasContext {
    fn drop(&mut self) {
        self.Unconfigure()
    }
}
