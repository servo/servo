/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::GPUCanvasContextBinding::{
    GPUCanvasContextMethods, GPUSwapChainDescriptor,
};
use crate::dom::bindings::codegen::Bindings::GPUDeviceBinding::GPUDeviceBinding::GPUDeviceMethods;
use crate::dom::bindings::codegen::Bindings::GPUObjectBaseBinding::GPUObjectDescriptorBase;
use crate::dom::bindings::codegen::Bindings::GPUTextureBinding::{
    GPUExtent3D, GPUExtent3DDict, GPUTextureDescriptor, GPUTextureDimension, GPUTextureFormat,
};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot, LayoutDom};
use crate::dom::globalscope::GlobalScope;
use crate::dom::gpuswapchain::GPUSwapChain;
use crate::dom::htmlcanvaselement::{HTMLCanvasElement, LayoutCanvasRenderingContextHelpers};
use crate::dom::node::{document_from_node, Node, NodeDamage};
use arrayvec::ArrayVec;
use dom_struct::dom_struct;
use euclid::default::Size2D;
use ipc_channel::ipc;
use script_layout_interface::HTMLCanvasDataSource;
use std::cell::Cell;
use webgpu::{wgpu::id, wgt, WebGPU, WebGPURequest, PRESENTATION_BUFFER_COUNT};

#[derive(Clone, Copy, Debug, Eq, Hash, MallocSizeOf, Ord, PartialEq, PartialOrd)]
pub struct WebGPUContextId(pub u64);

#[dom_struct]
pub struct GPUCanvasContext {
    reflector_: Reflector,
    #[ignore_malloc_size_of = "channels are hard"]
    channel: WebGPU,
    canvas: Dom<HTMLCanvasElement>,
    size: Cell<Size2D<u32>>,
    swap_chain: DomRefCell<Option<Dom<GPUSwapChain>>>,
    #[ignore_malloc_size_of = "Defined in webrender"]
    webrender_image: Cell<Option<webrender_api::ImageKey>>,
    context_id: WebGPUContextId,
}

impl GPUCanvasContext {
    fn new_inherited(canvas: &HTMLCanvasElement, size: Size2D<u32>, channel: WebGPU) -> Self {
        let (sender, receiver) = ipc::channel().unwrap();
        if let Err(e) = channel.0.send((None, WebGPURequest::CreateContext(sender))) {
            warn!("Failed to send CreateContext ({:?})", e);
        }
        let external_id = receiver.recv().unwrap();
        Self {
            reflector_: Reflector::new(),
            channel,
            canvas: Dom::from_ref(canvas),
            size: Cell::new(size),
            swap_chain: DomRefCell::new(None),
            webrender_image: Cell::new(None),
            context_id: WebGPUContextId(external_id.0),
        }
    }

    pub fn new(
        global: &GlobalScope,
        canvas: &HTMLCanvasElement,
        size: Size2D<u32>,
        channel: WebGPU,
    ) -> DomRoot<Self> {
        reflect_dom_object(
            Box::new(GPUCanvasContext::new_inherited(canvas, size, channel)),
            global,
        )
    }
}

impl GPUCanvasContext {
    fn layout_handle(&self) -> HTMLCanvasDataSource {
        let image_key = if self.webrender_image.get().is_some() {
            self.webrender_image.get().unwrap()
        } else {
            webrender_api::ImageKey::DUMMY
        };
        HTMLCanvasDataSource::WebGPU(image_key)
    }

    pub fn send_swap_chain_present(&self) {
        let texture_id = self.swap_chain.borrow().as_ref().unwrap().texture_id().0;
        let encoder_id = self
            .global()
            .wgpu_id_hub()
            .lock()
            .create_command_encoder_id(texture_id.backend());
        if let Err(e) = self.channel.0.send((
            None,
            WebGPURequest::SwapChainPresent {
                external_id: self.context_id.0,
                texture_id,
                encoder_id,
            },
        )) {
            warn!(
                "Failed to send UpdateWebrenderData({:?}) ({})",
                self.context_id, e
            );
        }
    }

    pub fn context_id(&self) -> WebGPUContextId {
        self.context_id
    }

    pub fn mark_as_dirty(&self) {
        self.canvas
            .upcast::<Node>()
            .dirty(NodeDamage::OtherNodeDamage);

        let document = document_from_node(&*self.canvas);
        document.add_dirty_webgpu_canvas(self);
    }
}

impl LayoutCanvasRenderingContextHelpers for LayoutDom<'_, GPUCanvasContext> {
    #[allow(unsafe_code)]
    unsafe fn canvas_data_source(self) -> HTMLCanvasDataSource {
        (*self.unsafe_get()).layout_handle()
    }
}

impl GPUCanvasContextMethods for GPUCanvasContext {
    /// https://gpuweb.github.io/gpuweb/#dom-gpucanvascontext-configureswapchain
    fn ConfigureSwapChain(&self, descriptor: &GPUSwapChainDescriptor) -> DomRoot<GPUSwapChain> {
        if let Some(chain) = &*self.swap_chain.borrow() {
            chain.destroy(self.context_id.0, self.webrender_image.get().unwrap());
            self.webrender_image.set(None);
        }
        *self.swap_chain.borrow_mut() = None;

        let mut buffer_ids = ArrayVec::<[id::BufferId; PRESENTATION_BUFFER_COUNT]>::new();
        for _ in 0..PRESENTATION_BUFFER_COUNT {
            buffer_ids.push(
                self.global()
                    .wgpu_id_hub()
                    .lock()
                    .create_buffer_id(descriptor.device.id().0.backend()),
            );
        }

        let image_desc = webrender_api::ImageDescriptor {
            format: match descriptor.format {
                GPUTextureFormat::Rgba8unorm => webrender_api::ImageFormat::RGBA8,
                GPUTextureFormat::Bgra8unorm => webrender_api::ImageFormat::BGRA8,
                _ => panic!("SwapChain format({:?}) not supported", descriptor.format),
            },
            size: webrender_api::units::DeviceIntSize::new(
                self.size.get().width as i32,
                self.size.get().height as i32,
            ),
            stride: Some(
                (((self.size.get().width * 4) | (wgt::COPY_BYTES_PER_ROW_ALIGNMENT - 1)) + 1)
                    as i32,
            ),
            offset: 0,
            flags: webrender_api::ImageDescriptorFlags::from_bits(1).unwrap(),
        };

        let image_data = webrender_api::ImageData::External(webrender_api::ExternalImageData {
            id: webrender_api::ExternalImageId(self.context_id.0),
            channel_index: 0,
            image_type: webrender_api::ExternalImageType::Buffer,
        });

        let (sender, receiver) = ipc::channel().unwrap();

        self.channel
            .0
            .send((
                None,
                WebGPURequest::CreateSwapChain {
                    device_id: descriptor.device.id().0,
                    buffer_ids,
                    external_id: self.context_id.0,
                    sender,
                    image_desc,
                    image_data,
                },
            ))
            .expect("Failed to create WebGPU SwapChain");

        let usage = if descriptor.usage % 2 == 0 {
            descriptor.usage + 1
        } else {
            descriptor.usage
        };
        let text_desc = GPUTextureDescriptor {
            parent: GPUObjectDescriptorBase { label: None },
            dimension: GPUTextureDimension::_2d,
            format: descriptor.format,
            mipLevelCount: 1,
            sampleCount: 1,
            usage,
            size: GPUExtent3D::GPUExtent3DDict(GPUExtent3DDict {
                width: self.size.get().width,
                height: self.size.get().height,
                depth: 1,
            }),
        };

        let texture = descriptor.device.CreateTexture(&text_desc);

        self.webrender_image.set(Some(receiver.recv().unwrap()));

        let swap_chain = GPUSwapChain::new(
            &self.global(),
            self.channel.clone(),
            &self,
            &*texture,
            descriptor.parent.label.as_ref().cloned(),
        );
        *self.swap_chain.borrow_mut() = Some(Dom::from_ref(&*swap_chain));
        swap_chain
    }
}
