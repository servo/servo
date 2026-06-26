/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use dom_struct::dom_struct;
use js::context::JSContext;
use pixels::{SnapshotAlphaMode, SnapshotPixelFormat};
use script_bindings::cell::DomRefCell;
use script_bindings::codegen::GenericBindings::CanvasRenderingContext2DBinding::ImageDataMethods;
use script_bindings::codegen::GenericBindings::HTMLCanvasElementBinding::HTMLCanvasElementMethods;
use script_bindings::codegen::GenericBindings::HTMLImageElementBinding::HTMLImageElementMethods;
use script_bindings::codegen::GenericBindings::HTMLVideoElementBinding::HTMLVideoElementMethods;
use script_bindings::codegen::GenericBindings::ImageBitmapBinding::ImageBitmapMethods;
use script_bindings::codegen::GenericBindings::OffscreenCanvasBinding::OffscreenCanvasMethods;
use script_bindings::reflector::{Reflector, reflect_dom_object_with_cx};
use servo_base::generic_channel::GenericSharedMemory;
use webgpu_traits::{WebGPU, WebGPUQueue, WebGPURequest};

use crate::conversions::{Convert, TryConvert};
use crate::dom::bindings::codegen::Bindings::WebGPUBinding::{
    GPUCopyExternalImageDestInfo, GPUCopyExternalImageSourceInfo, GPUExtent3D, GPUQueueMethods,
    GPUSize64, GPUTexelCopyBufferLayout, GPUTexelCopyTextureInfo,
};
use crate::dom::bindings::codegen::UnionTypes::{
    ArrayBufferViewOrArrayBuffer as BufferSource,
    ImageBitmapOrImageDataOrHTMLImageElementOrHTMLVideoElementOrHTMLCanvasElementOrOffscreenCanvas as GPUCopyExternalImageSource,
};
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::USVString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::dom::webgpu::gpubuffer::GPUBuffer;
use crate::dom::webgpu::gpucommandbuffer::GPUCommandBuffer;
use crate::dom::webgpu::gpudevice::GPUDevice;
use crate::routed_promise::{RoutedPromiseListener, callback_promise};

#[dom_struct]
pub(crate) struct GPUQueue {
    reflector_: Reflector,
    #[ignore_malloc_size_of = "defined in webgpu"]
    #[no_trace]
    channel: WebGPU,
    device: DomRefCell<Option<Dom<GPUDevice>>>,
    label: DomRefCell<USVString>,
    #[no_trace]
    queue: WebGPUQueue,
}

impl GPUQueue {
    fn new_inherited(channel: WebGPU, queue: WebGPUQueue) -> Self {
        GPUQueue {
            channel,
            reflector_: Reflector::new(),
            device: DomRefCell::new(None),
            label: DomRefCell::new(USVString::default()),
            queue,
        }
    }

    pub(crate) fn new(
        cx: &mut JSContext,
        global: &GlobalScope,
        channel: WebGPU,
        queue: WebGPUQueue,
    ) -> DomRoot<Self> {
        reflect_dom_object_with_cx(
            Box::new(GPUQueue::new_inherited(channel, queue)),
            global,
            cx,
        )
    }
}

impl GPUQueue {
    pub(crate) fn set_device(&self, device: &GPUDevice) {
        *self.device.borrow_mut() = Some(Dom::from_ref(device));
    }

    pub(crate) fn id(&self) -> WebGPUQueue {
        self.queue
    }
}

impl GPUQueueMethods<crate::DomTypeHolder> for GPUQueue {
    /// <https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label>
    fn Label(&self) -> USVString {
        self.label.borrow().clone()
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label>
    fn SetLabel(&self, value: USVString) {
        *self.label.borrow_mut() = value;
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpuqueue-submit>
    fn Submit(&self, command_buffers: Vec<DomRoot<GPUCommandBuffer>>) {
        let command_buffers = command_buffers.iter().map(|cb| cb.id().0).collect();
        self.channel
            .0
            .send(WebGPURequest::Submit {
                device_id: self.device.borrow().as_ref().unwrap().id().0,
                queue_id: self.queue.0,
                command_buffers,
            })
            .unwrap();
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpuqueue-writebuffer>
    #[expect(unsafe_code)]
    fn WriteBuffer(
        &self,
        buffer: &GPUBuffer,
        buffer_offset: GPUSize64,
        data: BufferSource,
        data_offset: GPUSize64,
        size: Option<GPUSize64>,
    ) -> Fallible<()> {
        // Step 1
        let (sizeof_element, data_len): (usize, usize) = match &data {
            BufferSource::ArrayBufferView(d) => {
                (d.get_array_type().byte_size().unwrap_or(1), d.len())
            },
            BufferSource::ArrayBuffer(d) => (1, d.len()),
        };
        // Step 2
        let data_size: usize = data_len / sizeof_element;
        debug_assert_eq!(data_len % sizeof_element, 0);
        // Step 3
        let content_size = if let Some(s) = size {
            s
        } else {
            (data_size as GPUSize64)
                .checked_sub(data_offset)
                .ok_or(Error::Operation(None))?
        };

        // Step 4
        let valid = data_offset + content_size <= data_size as u64 &&
            (content_size * sizeof_element as u64)
                .is_multiple_of(wgpu_types::COPY_BUFFER_ALIGNMENT);
        if !valid {
            return Err(Error::Operation(None));
        }

        // Step 5&6
        let byte_start = (data_offset as usize) * sizeof_element;
        let byte_end = ((data_offset + content_size) as usize) * sizeof_element;
        let contents = match &data {
            BufferSource::ArrayBufferView(data) => {
                // SAFETY: The subslice is immediately copied into GenericSharedMemory,
                // hence there is no opportunity for the slice to invalidated.
                GenericSharedMemory::from_bytes(unsafe { &data.as_slice()[byte_start..byte_end] })
            },
            BufferSource::ArrayBuffer(data) => {
                // SAFETY: The subslice is immediately copied into GenericSharedMemory,
                // hence there is no opportunity for the slice to invalidated.
                GenericSharedMemory::from_bytes(unsafe { &data.as_slice()[byte_start..byte_end] })
            },
        };
        if let Err(e) = self.channel.0.send(WebGPURequest::WriteBuffer {
            device_id: self.device.borrow().as_ref().unwrap().id().0,
            queue_id: self.queue.0,
            buffer_id: buffer.id().0,
            buffer_offset,
            data: contents,
        }) {
            warn!("Failed to send WriteBuffer({:?}) ({})", buffer.id(), e);
            return Err(Error::Operation(None));
        }

        Ok(())
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpuqueue-writetexture>
    fn WriteTexture(
        &self,
        destination: &GPUTexelCopyTextureInfo,
        data: BufferSource,
        data_layout: &GPUTexelCopyBufferLayout,
        size: GPUExtent3D,
    ) -> Fallible<()> {
        let (bytes, len) = match data {
            BufferSource::ArrayBufferView(d) => (d.to_vec(), d.len() as u64),
            BufferSource::ArrayBuffer(d) => (d.to_vec(), d.len() as u64),
        };
        let valid = data_layout.offset <= len;

        if !valid {
            return Err(Error::Operation(None));
        }

        let texture_cv = destination.try_convert()?;
        let texture_layout = data_layout.convert();
        let write_size = (&size).try_convert()?;
        let final_data = GenericSharedMemory::from_bytes(&bytes);

        if let Err(e) = self.channel.0.send(WebGPURequest::WriteTexture {
            device_id: self.device.borrow().as_ref().unwrap().id().0,
            queue_id: self.queue.0,
            texture_cv,
            data_layout: texture_layout,
            size: write_size,
            data: final_data,
        }) {
            warn!(
                "Failed to send WriteTexture({:?}) ({})",
                destination.texture.id().0,
                e
            );
            return Err(Error::Operation(None));
        }

        Ok(())
    }

    #[expect(
        clippy::nonminimal_bool,
        reason = "Following the spec steps more closely"
    )]
    /// <https://gpuweb.github.io/gpuweb/#dom-gpuqueue-copyexternalimagetotexture>
    fn CopyExternalImageToTexture(
        &self,
        cx: &mut JSContext,
        source: &GPUCopyExternalImageSourceInfo,
        destination: &GPUCopyExternalImageDestInfo,
        copy_size: GPUExtent3D,
    ) -> Fallible<()> {
        // 1. ? validate GPUOrigin2D shape(source.origin).
        let source_origin = source.origin.try_convert()?;
        // 2. ? validate GPUOrigin3D shape(destination.origin).
        let destination_tex_info = destination.parent.try_convert()?;
        // 3. ? validate GPUExtent3D shape(copySize).
        let copy_size = copy_size.try_convert()?;
        // 4. Let sourceImage be source.source.
        let source_image = &source.source;
        // 5. If sourceImage is not origin-clean, throw a SecurityError and return.
        let is_origin_clean = match source_image {
            GPUCopyExternalImageSource::ImageBitmap(inner) => inner.origin_is_clean(),
            GPUCopyExternalImageSource::ImageData(_) => true,
            GPUCopyExternalImageSource::HTMLImageElement(inner) => {
                inner.same_origin(&GlobalScope::entry().origin())
            },
            GPUCopyExternalImageSource::HTMLVideoElement(inner) => inner.origin_is_clean(),
            GPUCopyExternalImageSource::HTMLCanvasElement(inner) => inner.origin_is_clean(),
            GPUCopyExternalImageSource::OffscreenCanvas(inner) => inner.origin_is_clean(),
        };
        if !is_origin_clean {
            return Err(Error::Security(Some(
                "Image source is not origin clean!".to_string(),
            )));
        }
        // 6. If any of the following requirements are unmet, throw an OperationError and return.
        let (source_image_width, source_image_height) = match source_image {
            GPUCopyExternalImageSource::ImageBitmap(inner) => (inner.Width(), inner.Height()),
            GPUCopyExternalImageSource::ImageData(inner) => (inner.Width(), inner.Height()),
            GPUCopyExternalImageSource::HTMLImageElement(inner) => (inner.Width(), inner.Height()),
            GPUCopyExternalImageSource::HTMLVideoElement(inner) => (inner.Width(), inner.Height()),
            GPUCopyExternalImageSource::HTMLCanvasElement(inner) => (inner.Width(), inner.Height()),
            GPUCopyExternalImageSource::OffscreenCanvas(inner) => {
                (inner.Width() as u32, inner.Height() as u32)
            },
        };
        // source.origin.x + copySize.width must be ≤ the width of sourceImage.
        if !(source_origin.x + copy_size.width <= source_image_width) {
            return Err(Error::Operation(Some(
                "Source origin x + copy width exceeds source image width".to_string(),
            )));
        }
        // source.origin.y + copySize.height must be ≤ the height of sourceImage.
        if !(source_origin.y + copy_size.height <= source_image_height) {
            return Err(Error::Operation(Some(
                "Source origin y + copy height exceeds source image height".to_string(),
            )));
        }
        // copySize.depthOrArrayLayers must be ≤ 1.
        if !(copy_size.depth_or_array_layers <= 1) {
            return Err(Error::Operation(Some(
                "Copy depth or array layers must be less than or equal to 1".to_string(),
            )));
        }
        // 7. Let usability be ? check the usability of the image argument(source).
        // with usable variant we also send the snapshot
        let usable_snapshot = match source_image {
            GPUCopyExternalImageSource::ImageBitmap(bitmap) => {
                // If image's [[Detached]] internal slot value is set to true, then throw an "InvalidStateError" DOMException.
                Some(bitmap.bitmap_data().clone().ok_or_else(|| {
                    Error::InvalidState(Some("ImageBitmap is detached".to_string()))
                })?)
            },
            GPUCopyExternalImageSource::ImageData(data) => {
                // If image's [[Detached]] internal slot value is set to true, then throw an "InvalidStateError" DOMException.
                if data.is_detached(cx) {
                    return Err(Error::InvalidState(Some(
                        "ImageData is detached".to_string(),
                    )));
                }
                Some(data.get_snapshot())
            },
            GPUCopyExternalImageSource::HTMLImageElement(inner) => {
                if inner.is_usable()? {
                    inner.get_raster_image_data()
                } else {
                    None
                }
            },
            GPUCopyExternalImageSource::HTMLVideoElement(inner) => {
                if inner.is_usable() {
                    inner.get_current_frame_data()
                } else {
                    None
                }
            },
            GPUCopyExternalImageSource::HTMLCanvasElement(inner) => {
                // If image has either a horizontal dimension or a vertical dimension equal to zero, then throw an "InvalidStateError" DOMException.
                if inner.is_valid() {
                    inner.get_image_data()
                } else {
                    return Err(Error::InvalidState(Some(
                        "Canvas has zero area".to_string(),
                    )));
                }
            },
            GPUCopyExternalImageSource::OffscreenCanvas(inner) => {
                // If image has either a horizontal dimension or a vertical dimension equal to zero, then throw an "InvalidStateError" DOMException.
                if inner.Width() == 0 || inner.Height() == 0 {
                    return Err(Error::InvalidState(Some(
                        "Canvas has zero area".to_string(),
                    )));
                } else {
                    inner.get_image_data()
                }
            },
        };
        // this is out ouf spec, but we currently do not support more
        let texture_descriptor = destination.parent.texture.wgpu_texture_descriptor();
        let target_snapshot_format =
            match texture_descriptor.format {
                wgpu_types::TextureFormat::Bgra8Unorm |
                wgpu_types::TextureFormat::Bgra8UnormSrgb => SnapshotPixelFormat::BGRA,
                wgpu_types::TextureFormat::Rgba8Unorm |
                wgpu_types::TextureFormat::Rgba8UnormSrgb => SnapshotPixelFormat::RGBA,
                _ => {
                    return Err(Error::Operation(Some(
                        "Unsupported texture format for copy".to_string(),
                    )));
                },
            };
        let usable_snapshot = usable_snapshot.map(|mut snapshot| {
            if source.flipY {
                pixels::flip_y_rgba8_image_inplace(snapshot.size(), snapshot.as_raw_bytes_mut());
            }
            snapshot.transform(
                SnapshotAlphaMode::Transparent {
                    premultiplied: destination.premultipliedAlpha,
                },
                target_snapshot_format,
            );
            snapshot.to_shared()
        });
        // 8. Issue the subsequent steps on the Device timeline of this.
        if let Err(e) = self
            .channel
            .0
            .send(WebGPURequest::CopyExternalImageToTexture {
                device_id: self.device.borrow().as_ref().unwrap().id().0,
                queue_id: self.queue.0,
                usable_source: usable_snapshot,
                destination: destination_tex_info,
                dest_tex_descriptor: texture_descriptor,
                copy_size,
            })
        {
            warn!(
                "Failed to send CopyExternalImageToTexture({:?}) ({e})",
                destination.parent.texture.id().0
            );
            return Err(Error::Operation(None));
        }
        Ok(())
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpuqueue-onsubmittedworkdone>
    fn OnSubmittedWorkDone(&self, cx: &mut JSContext) -> Rc<Promise> {
        let global = self.global();
        let promise = Promise::new(cx, &global);
        let task_manager = global.task_manager();
        let task_source = task_manager.dom_manipulation_task_source();
        let callback = callback_promise(&promise, self, task_source);

        if let Err(e) = self
            .channel
            .0
            .send(WebGPURequest::QueueOnSubmittedWorkDone {
                sender: callback,
                queue_id: self.queue.0,
            })
        {
            warn!("QueueOnSubmittedWorkDone failed with {e}")
        }
        promise
    }
}

impl RoutedPromiseListener<()> for GPUQueue {
    fn handle_response(
        &self,
        cx: &mut js::context::JSContext,
        _response: (),
        promise: &Rc<Promise>,
    ) {
        promise.resolve_native(cx, &());
    }
}
