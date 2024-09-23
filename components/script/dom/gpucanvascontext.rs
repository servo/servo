/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::{Cell, RefCell};

use arrayvec::ArrayVec;
use dom_struct::dom_struct;
use euclid::default::Size2D;
use ipc_channel::ipc;
use script_layout_interface::HTMLCanvasDataSource;
use webgpu::swapchain::WebGPUContextId;
use webgpu::wgc::id;
use webgpu::{WebGPU, WebGPURequest, WebGPUTexture, PRESENTATION_BUFFER_COUNT};
use webrender_api::ImageKey;

use super::bindings::codegen::Bindings::WebGPUBinding::GPUTextureUsageConstants;
use super::bindings::codegen::UnionTypes::HTMLCanvasElementOrOffscreenCanvas;
use super::bindings::error::{Error, Fallible};
use super::bindings::root::MutNullableDom;
use super::bindings::str::USVString;
use super::gpuconvert::convert_texture_descriptor;
use super::gputexture::GPUTexture;
use crate::dom::bindings::codegen::Bindings::HTMLCanvasElementBinding::HTMLCanvasElement_Binding::HTMLCanvasElementMethods;
use crate::dom::bindings::codegen::Bindings::WebGPUBinding::GPUTexture_Binding::GPUTextureMethods;
use crate::dom::bindings::codegen::Bindings::WebGPUBinding::{
    GPUCanvasConfiguration, GPUCanvasContextMethods, GPUDeviceMethods, GPUExtent3D,
    GPUExtent3DDict, GPUObjectDescriptorBase, GPUTextureDescriptor, GPUTextureDimension,
};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject, Reflector};
use crate::dom::bindings::root::{DomRoot, LayoutDom};
use crate::dom::globalscope::GlobalScope;
use crate::dom::htmlcanvaselement::{HTMLCanvasElement, LayoutCanvasRenderingContextHelpers};
use crate::dom::node::{document_from_node, Node, NodeDamage};

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

#[derive(Clone, Copy, Debug, JSTraceable, MallocSizeOf)]
enum DrawingBuffer {
    TransparentAlpha,
    Wr,
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
    webrender_image: ImageKey,
    #[no_trace]
    context_id: WebGPUContextId,
    #[ignore_malloc_size_of = "manual writing is hard"]
    /// <https://gpuweb.github.io/gpuweb/#dom-gpucanvascontext-configuration-slot>
    configuration: RefCell<Option<GPUCanvasConfiguration>>,
    /// <https://gpuweb.github.io/gpuweb/#dom-gpucanvascontext-texturedescriptor-slot>
    texture_descriptor: RefCell<Option<GPUTextureDescriptor>>,
    /// Conceptually <https://gpuweb.github.io/gpuweb/#dom-gpucanvascontext-drawingbuffer-slot>
    drawing_buffer: Cell<DrawingBuffer>,
    /// <https://gpuweb.github.io/gpuweb/#dom-gpucanvascontext-currenttexture-slot>
    current_texture: MutNullableDom<GPUTexture>,
}

impl GPUCanvasContext {
    fn new_inherited(canvas: HTMLCanvasElementOrOffscreenCanvas, channel: WebGPU) -> Self {
        let (sender, receiver) = ipc::channel().unwrap();
        if let Err(e) = channel.0.send(WebGPURequest::CreateContext(sender)) {
            warn!("Failed to send CreateContext ({:?})", e);
        }
        let (external_id, webrender_image) = receiver.recv().unwrap();
        Self {
            reflector_: Reflector::new(),
            channel,
            canvas,
            webrender_image,
            context_id: WebGPUContextId(external_id.0),
            drawing_buffer: Cell::new(DrawingBuffer::TransparentAlpha),
            configuration: RefCell::new(None),
            texture_descriptor: RefCell::new(None),
            current_texture: MutNullableDom::default(),
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

// Abstract ops from spec
impl GPUCanvasContext {
    /// <https://gpuweb.github.io/gpuweb/#abstract-opdef-gputexturedescriptor-for-the-canvas-and-configuration>
    fn texture_descriptor_for_canvas(
        &self,
        configuration: &GPUCanvasConfiguration,
    ) -> GPUTextureDescriptor {
        let size = self.size();
        GPUTextureDescriptor {
            format: configuration.format,
            // We need to add `COPY_SRC` so we can copy texture to presentation buffer
            usage: configuration.usage | GPUTextureUsageConstants::COPY_SRC,
            size: GPUExtent3D::GPUExtent3DDict(GPUExtent3DDict {
                width: size.width as u32,
                height: size.height as u32,
                depthOrArrayLayers: 1,
            }),
            viewFormats: configuration.viewFormats.clone(),
            // other members to default
            mipLevelCount: 1,
            sampleCount: 1,
            parent: GPUObjectDescriptorBase {
                label: USVString::default(),
            },
            dimension: GPUTextureDimension::_2d,
        }
    }

    /// <https://gpuweb.github.io/gpuweb/#abstract-opdef-expire-the-current-texture>
    fn expire_current_texture(&self) {
        if let Some(current_texture) = self.current_texture.take() {
            // Make copy of texture content
            self.send_swap_chain_present(current_texture.id());
            // Step 1
            current_texture.Destroy()
        }
    }

    /// <https://gpuweb.github.io/gpuweb/#abstract-opdef-replace-the-drawing-buffer>
    fn replace_drawing_buffer(&self) {
        // Step 1
        self.expire_current_texture();
        // Step 2
        //let configuration = self.configuration.borrow();
        // Step 3
        self.drawing_buffer.set(DrawingBuffer::TransparentAlpha);
        // TODO: Configuration on drawing buffer
        // here we should recreate swapchain
        // expire does needless copy???
    }
}

// Internal helper methods
impl GPUCanvasContext {
    fn layout_handle(&self) -> HTMLCanvasDataSource {
        match self.drawing_buffer.get() {
            DrawingBuffer::TransparentAlpha => HTMLCanvasDataSource::Empty,
            DrawingBuffer::Wr => HTMLCanvasDataSource::WebGPU(self.webrender_image),
        }
    }

    fn send_swap_chain_present(&self, texture_id: WebGPUTexture) {
        self.drawing_buffer.set(DrawingBuffer::Wr);
        let encoder_id = self.global().wgpu_id_hub().create_command_encoder_id();
        if let Err(e) = self.channel.0.send(WebGPURequest::SwapChainPresent {
            context_id: self.context_id,
            texture_id: texture_id.0,
            encoder_id,
        }) {
            warn!(
                "Failed to send UpdateWebrenderData({:?}) ({})",
                self.context_id, e
            );
        }
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

// public methods for canvas handling
// these methods should probably be behind trait for all canvases
impl GPUCanvasContext {
    pub fn context_id(&self) -> WebGPUContextId {
        self.context_id
    }

    pub fn mark_as_dirty(&self) {
        if let HTMLCanvasElementOrOffscreenCanvas::HTMLCanvasElement(canvas) = &self.canvas {
            canvas.upcast::<Node>().dirty(NodeDamage::OtherNodeDamage);
            let document = document_from_node(&**canvas);
            document.add_dirty_webgpu_canvas(self);
        }
    }

    /// <https://gpuweb.github.io/gpuweb/#abstract-opdef-updating-the-rendering-of-a-webgpu-canvas>
    pub fn update_rendering_of_webgpu_canvas(&self) {
        // Step 1
        self.expire_current_texture();
    }

    /// <https://gpuweb.github.io/gpuweb/#abstract-opdef-update-the-canvas-size>
    pub fn resize(&self) {
        // Step 1
        self.replace_drawing_buffer();
        // Step 2
        let configuration = self.configuration.borrow();
        // Step 3
        if let Some(configuration) = configuration.as_ref() {
            self.texture_descriptor
                .replace(Some(self.texture_descriptor_for_canvas(configuration)));
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
    fn Configure(&self, configuration: &GPUCanvasConfiguration) -> Fallible<()> {
        // Step 4
        let descriptor = self.texture_descriptor_for_canvas(configuration);

        // Step 2&3
        let (desc, _) = convert_texture_descriptor(&descriptor, &configuration.device)?;

        // Step 5
        self.configuration.replace(Some(configuration.clone()));

        // Step 6
        self.texture_descriptor.replace(Some(descriptor));

        // Step 7
        self.replace_drawing_buffer();

        // Step 8
        let mut buffer_ids = ArrayVec::<id::BufferId, PRESENTATION_BUFFER_COUNT>::new();
        for _ in 0..PRESENTATION_BUFFER_COUNT {
            buffer_ids.push(self.global().wgpu_id_hub().create_buffer_id());
        }
        let texture_id = self.global().wgpu_id_hub().create_texture_id();

        self.channel
            .0
            .send(WebGPURequest::ValidateTextureDescriptorAndCreateSwapChain {
                device_id: configuration.device.id().0,
                queue_id: configuration.device.GetQueue().id().0,
                buffer_ids,
                context_id: self.context_id,
                image_key: self.webrender_image,
                size: self.size().cast().cast_unit(),
                texture_id,
                descriptor: desc,
            })
            .expect("Failed to create WebGPU SwapChain");

        Ok(())
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpucanvascontext-unconfigure>
    fn Unconfigure(&self) {
        self.current_texture.take();
        let configuration = self.configuration.take();
        if configuration.is_some() {
            if let Err(e) = self.channel.0.send(WebGPURequest::DestroySwapChain {
                context_id: self.context_id,
                image_key: self.webrender_image,
            }) {
                warn!(
                    "Failed to send DestroySwapChain-ImageKey({:?}) ({})",
                    self.webrender_image, e
                );
            }
        }
        self.replace_drawing_buffer();
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpucanvascontext-getcurrenttexture>
    fn GetCurrentTexture(&self) -> Fallible<DomRoot<GPUTexture>> {
        // Step 1
        let configuration = self.configuration.borrow();
        let Some(configuration) = configuration.as_ref() else {
            return Err(Error::InvalidState);
        };
        // Step 2
        let texture_descriptor = self.texture_descriptor.borrow();
        let texture_descriptor = texture_descriptor.as_ref().unwrap();
        // Step 6
        let current_texture = if let Some(current_texture) = self.current_texture.get() {
            current_texture
        } else {
            // Step 3&4
            self.replace_drawing_buffer();
            let current_texture = configuration.device.CreateTexture(&texture_descriptor)?;
            self.current_texture.set(Some(&current_texture));
            current_texture
        };
        // Step 5
        self.mark_as_dirty();
        // Step 6
        Ok(current_texture)
    }
}

impl Drop for GPUCanvasContext {
    fn drop(&mut self) {
        self.Unconfigure()
    }
}
