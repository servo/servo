/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::Cow;
use std::cell::{Cell, RefCell};

use arrayvec::ArrayVec;
use base::Epoch;
use dom_struct::dom_struct;
use ipc_channel::ipc::{self};
use pixels::Snapshot;
use script_bindings::codegen::GenericBindings::WebGPUBinding::GPUTextureFormat;
use script_bindings::inheritance::Castable;
use webgpu_traits::{
    ContextConfiguration, PRESENTATION_BUFFER_COUNT, WebGPU, WebGPUContextId, WebGPURequest,
};
use webrender_api::{ImageFormat, ImageKey};
use wgpu_core::id;

use super::gpuconvert::convert_texture_descriptor;
use super::gputexture::GPUTexture;
use crate::canvas_context::{
    CanvasContext, CanvasHelpers, HTMLCanvasElementOrOffscreenCanvas,
    LayoutCanvasRenderingContextHelpers,
};
use crate::conversions::Convert;
use crate::dom::bindings::codegen::Bindings::GPUCanvasContextBinding::GPUCanvasContextMethods;
use crate::dom::bindings::codegen::Bindings::WebGPUBinding::GPUTexture_Binding::GPUTextureMethods;
use crate::dom::bindings::codegen::Bindings::WebGPUBinding::{
    GPUCanvasAlphaMode, GPUCanvasConfiguration, GPUDeviceMethods, GPUExtent3D, GPUExtent3DDict,
    GPUObjectDescriptorBase, GPUTextureDescriptor, GPUTextureDimension, GPUTextureUsageConstants,
};
use crate::dom::bindings::codegen::UnionTypes::HTMLCanvasElementOrOffscreenCanvas as RootedHTMLCanvasElementOrOffscreenCanvas;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::reflector::{DomGlobal, Reflector, reflect_dom_object};
use crate::dom::bindings::root::{Dom, DomRoot, LayoutDom, MutNullableDom};
use crate::dom::bindings::str::USVString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::html::htmlcanvaselement::HTMLCanvasElement;
use crate::dom::node::NodeTraits;
use crate::script_runtime::CanGc;

/// <https://gpuweb.github.io/gpuweb/#supported-context-formats>
fn supported_context_format(format: GPUTextureFormat) -> bool {
    // TODO: GPUTextureFormat::Rgba16float
    matches!(
        format,
        GPUTextureFormat::Bgra8unorm | GPUTextureFormat::Rgba8unorm
    )
}

#[dom_struct]
pub(crate) struct GPUCanvasContext {
    reflector_: Reflector,
    #[ignore_malloc_size_of = "channels are hard"]
    #[no_trace]
    channel: WebGPU,
    /// <https://gpuweb.github.io/gpuweb/#dom-gpucanvascontext-canvas>
    canvas: HTMLCanvasElementOrOffscreenCanvas,
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
    /// <https://gpuweb.github.io/gpuweb/#dom-gpucanvascontext-currenttexture-slot>
    current_texture: MutNullableDom<GPUTexture>,
    /// Set if image is cleared
    /// (usually done by [`GPUCanvasContext::replace_drawing_buffer`])
    cleared: Cell<bool>,
}

impl GPUCanvasContext {
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    fn new_inherited(
        global: &GlobalScope,
        canvas: HTMLCanvasElementOrOffscreenCanvas,
        channel: WebGPU,
    ) -> Self {
        let (sender, receiver) = ipc::channel().unwrap();
        let size = canvas.size().cast().cast_unit();
        let mut buffer_ids = ArrayVec::<id::BufferId, PRESENTATION_BUFFER_COUNT>::new();
        for _ in 0..PRESENTATION_BUFFER_COUNT {
            buffer_ids.push(global.wgpu_id_hub().create_buffer_id());
        }
        if let Err(e) = channel.0.send(WebGPURequest::CreateContext {
            buffer_ids,
            size,
            sender,
        }) {
            warn!("Failed to send CreateContext ({:?})", e);
        }
        let (external_id, webrender_image) = receiver.recv().unwrap();
        Self {
            reflector_: Reflector::new(),
            channel,
            canvas,
            webrender_image,
            context_id: WebGPUContextId(external_id.0),
            configuration: RefCell::new(None),
            texture_descriptor: RefCell::new(None),
            current_texture: MutNullableDom::default(),
            cleared: Cell::new(true),
        }
    }

    pub(crate) fn new(
        global: &GlobalScope,
        canvas: &HTMLCanvasElement,
        channel: WebGPU,
        can_gc: CanGc,
    ) -> DomRoot<Self> {
        reflect_dom_object(
            Box::new(GPUCanvasContext::new_inherited(
                global,
                HTMLCanvasElementOrOffscreenCanvas::HTMLCanvasElement(Dom::from_ref(canvas)),
                channel,
            )),
            global,
            can_gc,
        )
    }
}

// Abstract ops from spec
impl GPUCanvasContext {
    /// <https://gpuweb.github.io/gpuweb/#abstract-opdef-gputexturedescriptor-for-the-canvas-and-configuration>
    fn texture_descriptor_for_canvas_and_configuration(
        &self,
        configuration: &GPUCanvasConfiguration,
    ) -> GPUTextureDescriptor {
        let size = self.size();
        GPUTextureDescriptor {
            size: GPUExtent3D::GPUExtent3DDict(GPUExtent3DDict {
                width: size.width,
                height: size.height,
                depthOrArrayLayers: 1,
            }),
            format: configuration.format,
            // We need to add `COPY_SRC` so we can copy texture to presentation buffer
            // causes FAIL on webgpu:web_platform,canvas,configure:usage:*
            usage: configuration.usage | GPUTextureUsageConstants::COPY_SRC,
            viewFormats: configuration.viewFormats.clone(),
            // All other members set to their defaults.
            mipLevelCount: 1,
            sampleCount: 1,
            parent: GPUObjectDescriptorBase {
                label: USVString::default(),
            },
            dimension: GPUTextureDimension::_2d,
        }
    }

    /// <https://gpuweb.github.io/gpuweb/#abstract-opdef-expire-the-current-texture>
    fn expire_current_texture(&self, skip_dirty: bool) {
        // 1. If context.[[currentTexture]] is not null:

        if let Some(current_texture) = self.current_texture.take() {
            // 1.2 Set context.[[currentTexture]] to null.

            // 1.1 Call context.[[currentTexture]].destroy()
            // (without destroying context.[[drawingBuffer]])
            // to terminate write access to the image.
            current_texture.Destroy()
            // we can safely destroy content here,
            // because we already copied content when doing present
            // or current texture is getting cleared
        }
        if !skip_dirty {
            // texture is either cleared or applied to canvas
            self.mark_as_dirty();
        }
    }

    /// <https://gpuweb.github.io/gpuweb/#abstract-opdef-replace-the-drawing-buffer>
    fn replace_drawing_buffer(&self) {
        // 1. Expire the current texture of context.
        self.expire_current_texture(false);
        // 2. Let configuration be context.[[configuration]].
        // 3. Set context.[[drawingBuffer]] to
        // a transparent black image of the same size as context.canvas
        self.cleared.set(true);
    }
}

// Internal helper methods
impl GPUCanvasContext {
    fn context_configuration(&self) -> Option<ContextConfiguration> {
        let configuration = self.configuration.borrow();
        let configuration = configuration.as_ref()?;
        Some(ContextConfiguration {
            device_id: configuration.device.id().0,
            queue_id: configuration.device.queue_id().0,
            format: match configuration.format {
                GPUTextureFormat::Bgra8unorm => ImageFormat::BGRA8,
                GPUTextureFormat::Rgba8unorm => ImageFormat::RGBA8,
                _ => unreachable!("Not be reachable due to configure"),
            },
            is_opaque: matches!(configuration.alphaMode, GPUCanvasAlphaMode::Opaque),
            size: self.size(),
        })
    }
}

impl CanvasContext for GPUCanvasContext {
    type ID = WebGPUContextId;

    fn context_id(&self) -> WebGPUContextId {
        self.context_id
    }

    fn image_key(&self) -> Option<ImageKey> {
        if self.cleared.get() {
            None
        } else {
            Some(self.webrender_image)
        }
    }

    /// <https://gpuweb.github.io/gpuweb/#abstract-opdef-updating-the-rendering-of-a-webgpu-canvas>
    fn update_rendering(&self, canvas_epoch: Epoch) -> bool {
        let mut updated_image = false;
        if let Some(texture) = self.current_texture.get() {
            // Copy the image into the presentation buffer before destroying it.
            let encoder_id = self.global().wgpu_id_hub().create_command_encoder_id();
            if let Err(e) = self.channel.0.send(WebGPURequest::Present {
                context_id: self.context_id,
                texture_id: texture.id().0,
                encoder_id,
                configuration: self
                    .context_configuration()
                    .expect("Context should be configured if there is a texture."),
                canvas_epoch,
            }) {
                warn!(
                    "Failed to send WebGPURequest::Present({:?}) ({})",
                    self.context_id, e
                );
            } else {
                updated_image = true;
            }
        }
        // 1. Expire the current texture of context.
        self.expire_current_texture(true);

        updated_image
    }

    /// <https://gpuweb.github.io/gpuweb/#abstract-opdef-update-the-canvas-size>
    fn resize(&self) {
        // 1. Replace the drawing buffer of context.
        self.replace_drawing_buffer();
        // 2. Let configuration be context.[[configuration]]
        let configuration = self.configuration.borrow();
        // 3. If configuration is not null:
        if let Some(configuration) = configuration.as_ref() {
            // 3.1. Set context.[[textureDescriptor]] to the
            // GPUTextureDescriptor for the canvas and configuration(canvas, configuration).
            self.texture_descriptor.replace(Some(
                self.texture_descriptor_for_canvas_and_configuration(configuration),
            ));
        }
    }

    fn reset_bitmap(&self) {
        warn!("The GPUCanvasContext 'reset_bitmap' is not implemented yet");
    }

    /// <https://gpuweb.github.io/gpuweb/#ref-for-abstract-opdef-get-a-copy-of-the-image-contents-of-a-context%E2%91%A5>
    fn get_image_data(&self) -> Option<Snapshot> {
        // 1. Return a copy of the image contents of context.
        Some(if self.cleared.get() {
            Snapshot::cleared(self.size())
        } else {
            let (sender, receiver) = ipc::channel().unwrap();
            self.channel
                .0
                .send(WebGPURequest::GetImage {
                    context_id: self.context_id,
                    // We need to read from the pending texture, if one exists.
                    pending_texture: self.current_texture.get().map(|texture| {
                        (
                            texture.id().0,
                            self.global().wgpu_id_hub().create_command_encoder_id(),
                            self.context_configuration()
                                .expect("Context should be configured if there is a texture."),
                        )
                    }),
                    sender,
                })
                .ok()?;
            receiver.recv().ok()?.to_owned()
        })
    }

    fn canvas(&self) -> Option<RootedHTMLCanvasElementOrOffscreenCanvas> {
        Some(RootedHTMLCanvasElementOrOffscreenCanvas::from(&self.canvas))
    }

    fn mark_as_dirty(&self) {
        if let HTMLCanvasElementOrOffscreenCanvas::HTMLCanvasElement(ref canvas) = self.canvas {
            canvas.upcast::<Node>().dirty(NodeDamage::Other);
            canvas.owner_document().add_dirty_webgpu_context(self);
        }
    }
}

impl LayoutCanvasRenderingContextHelpers for LayoutDom<'_, GPUCanvasContext> {
    fn canvas_data_source(self) -> Option<ImageKey> {
        (*self.unsafe_get()).image_key()
    }
}

impl GPUCanvasContextMethods<crate::DomTypeHolder> for GPUCanvasContext {
    /// <https://gpuweb.github.io/gpuweb/#dom-gpucanvascontext-canvas>
    fn Canvas(&self) -> RootedHTMLCanvasElementOrOffscreenCanvas {
        RootedHTMLCanvasElementOrOffscreenCanvas::from(&self.canvas)
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpucanvascontext-configure>
    fn Configure(&self, configuration: &GPUCanvasConfiguration) -> Fallible<()> {
        // 1. Let device be configuration.device
        let device = &configuration.device;

        // 5. Let descriptor be the GPUTextureDescriptor for the canvas and configuration.
        let descriptor = self.texture_descriptor_for_canvas_and_configuration(configuration);

        // 2. Validate texture format required features of configuration.format with device.[[device]].
        // 3. Validate texture format required features of each element of configuration.viewFormats with device.[[device]].
        let (mut desc, _) = convert_texture_descriptor(&descriptor, device)?;
        desc.label = Some(Cow::Borrowed(
            "dummy texture for texture descriptor validation",
        ));

        // 4. If Supported context formats does not contain configuration.format, throw a TypeError
        if !supported_context_format(configuration.format) {
            return Err(Error::Type(format!(
                "Unsupported context format: {:?}",
                configuration.format
            )));
        }

        // 6. Let this.[[configuration]] to configuration.
        self.configuration.replace(Some(configuration.clone()));

        // 7. Set this.[[textureDescriptor]] to descriptor.
        self.texture_descriptor.replace(Some(descriptor));

        // 8. Replace the drawing buffer of this.
        self.replace_drawing_buffer();

        // 9. Validate texture descriptor
        let texture_id = self.global().wgpu_id_hub().create_texture_id();
        self.channel
            .0
            .send(WebGPURequest::ValidateTextureDescriptor {
                device_id: device.id().0,
                texture_id,
                descriptor: desc,
            })
            .expect("Failed to create WebGPU SwapChain");

        Ok(())
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpucanvascontext-unconfigure>
    fn Unconfigure(&self) {
        // 1. Set this.[[configuration]] to null.
        self.configuration.take();
        // 2. Set this.[[textureDescriptor]] to null.
        self.current_texture.take();
        // 3. Replace the drawing buffer of this.
        self.replace_drawing_buffer();
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpucanvascontext-getcurrenttexture>
    fn GetCurrentTexture(&self) -> Fallible<DomRoot<GPUTexture>> {
        // 1. If this.[[configuration]] is null, throw an InvalidStateError and return.
        let configuration = self.configuration.borrow();
        let Some(configuration) = configuration.as_ref() else {
            return Err(Error::InvalidState);
        };
        // 2. Assert this.[[textureDescriptor]] is not null.
        let texture_descriptor = self.texture_descriptor.borrow();
        let texture_descriptor = texture_descriptor.as_ref().unwrap();
        // 3. Let device be this.[[configuration]].device.
        let device = &configuration.device;
        let current_texture = if let Some(current_texture) = self.current_texture.get() {
            current_texture
        } else {
            // If this.[[currentTexture]] is null:
            // 4.1. Replace the drawing buffer of this.
            self.replace_drawing_buffer();
            // 4.2. Set this.[[currentTexture]] to the result of calling device.createTexture() with this.[[textureDescriptor]],
            // except with the GPUTexture’s underlying storage pointing to this.[[drawingBuffer]].
            let current_texture = device.CreateTexture(texture_descriptor)?;
            self.current_texture.set(Some(&current_texture));

            // content of texture is content of canvas
            self.cleared.set(false);

            current_texture
        };
        // 6. Return this.[[currentTexture]].
        Ok(current_texture)
    }
}

impl Drop for GPUCanvasContext {
    fn drop(&mut self) {
        if let Err(e) = self.channel.0.send(WebGPURequest::DestroyContext {
            context_id: self.context_id,
        }) {
            warn!(
                "Failed to send DestroySwapChain-ImageKey({:?}) ({})",
                self.webrender_image, e
            );
        }
    }
}
