/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![deny(unsafe_code)]

pub mod display_list;
pub mod rendering_context;

use core::fmt;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use base::id::PipelineId;
use display_list::{CompositorDisplayListInfo, ScrollTreeNodeId};
use embedder_traits::Cursor;
use euclid::default::Size2D as UntypedSize2D;
use ipc_channel::ipc::{self, IpcSender, IpcSharedMemory};
use libc::c_void;
use log::warn;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use servo_geometry::{DeviceIndependentIntRect, DeviceIndependentIntSize};
use webrender_api::units::{DevicePoint, LayoutPoint, TexelRect};
use webrender_api::{
    BuiltDisplayList, BuiltDisplayListDescriptor, ExternalImage, ExternalImageData,
    ExternalImageHandler, ExternalImageId, ExternalImageSource, ExternalScrollId,
    FontInstanceFlags, FontInstanceKey, FontKey, HitTestFlags, ImageData, ImageDescriptor,
    ImageKey, NativeFontHandle, PipelineId as WebRenderPipelineId,
};

#[derive(Deserialize, Serialize)]
pub enum CrossProcessCompositorMessage {
    /// Inform WebRender of the existence of this pipeline.
    SendInitialTransaction(WebRenderPipelineId),
    /// Perform a scroll operation.
    SendScrollNode(WebRenderPipelineId, LayoutPoint, ExternalScrollId),
    /// Inform WebRender of a new display list for the given pipeline.
    SendDisplayList {
        /// The [CompositorDisplayListInfo] that describes the display list being sent.
        display_list_info: Box<CompositorDisplayListInfo>,
        /// A descriptor of this display list used to construct this display list from raw data.
        display_list_descriptor: BuiltDisplayListDescriptor,
        /// An [ipc::IpcBytesReceiver] used to send the raw data of the display list.
        display_list_receiver: ipc::IpcBytesReceiver,
    },
    /// Perform a hit test operation. The result will be returned via
    /// the provided channel sender.
    HitTest(
        Option<WebRenderPipelineId>,
        DevicePoint,
        HitTestFlags,
        IpcSender<Vec<CompositorHitTestResult>>,
    ),
    /// Create a new image key. The result will be returned via the
    /// provided channel sender.
    GenerateImageKey(IpcSender<ImageKey>),
    /// Add an image with the given data and `ImageKey`.
    AddImage(ImageKey, ImageDescriptor, SerializableImageData),
    /// Perform a resource update operation.
    UpdateImages(Vec<ImageUpdate>),

    /// Generate a new batch of font keys which can be used to allocate
    /// keys asynchronously.
    GenerateFontKeys(
        usize,
        usize,
        IpcSender<(Vec<FontKey>, Vec<FontInstanceKey>)>,
    ),
    /// Add a font with the given data and font key.
    AddFont(FontKey, Arc<IpcSharedMemory>, u32),
    /// Add a system font with the given font key and handle.
    AddSystemFont(FontKey, NativeFontHandle),
    /// Add an instance of a font with the given instance key.
    AddFontInstance(FontInstanceKey, FontKey, f32, FontInstanceFlags),
    /// Remove the given font resources from our WebRender instance.
    RemoveFonts(Vec<FontKey>, Vec<FontInstanceKey>),

    /// Get the client window size and position.
    GetClientWindowRect(IpcSender<DeviceIndependentIntRect>),
    /// Get the size of the screen that the client window inhabits.
    GetScreenSize(IpcSender<DeviceIndependentIntSize>),
    /// Get the available screen size (without toolbars and docks) for the screen
    /// the client window inhabits.
    GetAvailableScreenSize(IpcSender<DeviceIndependentIntSize>),
}

impl fmt::Debug for CrossProcessCompositorMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AddImage(..) => f.write_str("AddImage"),
            Self::GenerateFontKeys(..) => f.write_str("GenerateFontKeys"),
            Self::AddSystemFont(..) => f.write_str("AddSystemFont"),
            Self::SendInitialTransaction(..) => f.write_str("SendInitialTransaction"),
            Self::SendScrollNode(..) => f.write_str("SendScrollNode"),
            Self::SendDisplayList { .. } => f.write_str("SendDisplayList"),
            Self::HitTest(..) => f.write_str("HitTest"),
            Self::GenerateImageKey(..) => f.write_str("GenerateImageKey"),
            Self::UpdateImages(..) => f.write_str("UpdateImages"),
            Self::RemoveFonts(..) => f.write_str("RemoveFonts"),
            Self::AddFontInstance(..) => f.write_str("AddFontInstance"),
            Self::AddFont(..) => f.write_str("AddFont"),
            Self::GetClientWindowRect(..) => f.write_str("GetClientWindowRect"),
            Self::GetScreenSize(..) => f.write_str("GetScreenSize"),
            Self::GetAvailableScreenSize(..) => f.write_str("GetAvailableScreenSize"),
        }
    }
}

/// A mechanism to send messages from ScriptThread to the parent process' WebRender instance.
#[derive(Clone, Deserialize, Serialize)]
pub struct CrossProcessCompositorApi(pub IpcSender<CrossProcessCompositorMessage>);

impl CrossProcessCompositorApi {
    /// Create a new [`CrossProcessCompositorApi`] struct that does not have a listener on the other
    /// end to use for unit testing.
    pub fn dummy() -> Self {
        let (sender, _) = ipc::channel().unwrap();
        Self(sender)
    }

    /// Get the sender for this proxy.
    pub fn sender(&self) -> &IpcSender<CrossProcessCompositorMessage> {
        &self.0
    }

    /// Inform WebRender of the existence of this pipeline.
    pub fn send_initial_transaction(&self, pipeline: WebRenderPipelineId) {
        if let Err(e) = self
            .0
            .send(CrossProcessCompositorMessage::SendInitialTransaction(
                pipeline,
            ))
        {
            warn!("Error sending initial transaction: {}", e);
        }
    }

    /// Perform a scroll operation.
    pub fn send_scroll_node(
        &self,
        pipeline_id: WebRenderPipelineId,
        point: LayoutPoint,
        scroll_id: ExternalScrollId,
    ) {
        if let Err(e) = self.0.send(CrossProcessCompositorMessage::SendScrollNode(
            pipeline_id,
            point,
            scroll_id,
        )) {
            warn!("Error sending scroll node: {}", e);
        }
    }

    /// Inform WebRender of a new display list for the given pipeline.
    pub fn send_display_list(
        &self,
        display_list_info: CompositorDisplayListInfo,
        list: BuiltDisplayList,
    ) {
        let (display_list_data, display_list_descriptor) = list.into_data();
        let (display_list_sender, display_list_receiver) = ipc::bytes_channel().unwrap();
        if let Err(e) = self.0.send(CrossProcessCompositorMessage::SendDisplayList {
            display_list_info: Box::new(display_list_info),
            display_list_descriptor,
            display_list_receiver,
        }) {
            warn!("Error sending display list: {}", e);
        }

        if let Err(error) = display_list_sender.send(&display_list_data.items_data) {
            warn!("Error sending display list items: {}", error);
        }
        if let Err(error) = display_list_sender.send(&display_list_data.cache_data) {
            warn!("Error sending display list cache data: {}", error);
        }
        if let Err(error) = display_list_sender.send(&display_list_data.spatial_tree) {
            warn!("Error sending display spatial tree: {}", error);
        }
    }

    /// Perform a hit test operation. Blocks until the operation is complete and
    /// and a result is available.
    pub fn hit_test(
        &self,
        pipeline: Option<WebRenderPipelineId>,
        point: DevicePoint,
        flags: HitTestFlags,
    ) -> Vec<CompositorHitTestResult> {
        let (sender, receiver) = ipc::channel().unwrap();
        self.0
            .send(CrossProcessCompositorMessage::HitTest(
                pipeline, point, flags, sender,
            ))
            .expect("error sending hit test");
        receiver.recv().expect("error receiving hit test result")
    }

    /// Create a new image key. Blocks until the key is available.
    pub fn generate_image_key(&self) -> Option<ImageKey> {
        let (sender, receiver) = ipc::channel().unwrap();
        self.0
            .send(CrossProcessCompositorMessage::GenerateImageKey(sender))
            .ok()?;
        receiver.recv().ok()
    }

    pub fn add_image(
        &self,
        key: ImageKey,
        descriptor: ImageDescriptor,
        data: SerializableImageData,
    ) {
        if let Err(e) = self.0.send(CrossProcessCompositorMessage::AddImage(
            key, descriptor, data,
        )) {
            warn!("Error sending image update: {}", e);
        }
    }

    /// Perform an image resource update operation.
    pub fn update_images(&self, updates: Vec<ImageUpdate>) {
        if let Err(e) = self
            .0
            .send(CrossProcessCompositorMessage::UpdateImages(updates))
        {
            warn!("error sending image updates: {}", e);
        }
    }

    pub fn remove_unused_font_resources(
        &self,
        keys: Vec<FontKey>,
        instance_keys: Vec<FontInstanceKey>,
    ) {
        if keys.is_empty() && instance_keys.is_empty() {
            return;
        }
        let _ = self.0.send(CrossProcessCompositorMessage::RemoveFonts(
            keys,
            instance_keys,
        ));
    }

    pub fn add_font_instance(
        &self,
        font_instance_key: FontInstanceKey,
        font_key: FontKey,
        size: f32,
        flags: FontInstanceFlags,
    ) {
        let _x = self.0.send(CrossProcessCompositorMessage::AddFontInstance(
            font_instance_key,
            font_key,
            size,
            flags,
        ));
    }

    pub fn add_font(&self, font_key: FontKey, data: Arc<IpcSharedMemory>, index: u32) {
        let _ = self.0.send(CrossProcessCompositorMessage::AddFont(
            font_key, data, index,
        ));
    }

    pub fn add_system_font(&self, font_key: FontKey, handle: NativeFontHandle) {
        let _ = self.0.send(CrossProcessCompositorMessage::AddSystemFont(
            font_key, handle,
        ));
    }

    pub fn fetch_font_keys(
        &self,
        number_of_font_keys: usize,
        number_of_font_instance_keys: usize,
    ) -> (Vec<FontKey>, Vec<FontInstanceKey>) {
        let (sender, receiver) = ipc_channel::ipc::channel().expect("Could not create IPC channel");
        let _ = self.0.send(CrossProcessCompositorMessage::GenerateFontKeys(
            number_of_font_keys,
            number_of_font_instance_keys,
            sender,
        ));
        receiver.recv().unwrap()
    }
}

/// This trait is used as a bridge between the different GL clients
/// in Servo that handles WebRender ExternalImages and the WebRender
/// ExternalImageHandler API.
//
/// This trait is used to notify lock/unlock messages and get the
/// required info that WR needs.
pub trait WebrenderExternalImageApi {
    fn lock(&mut self, id: u64) -> (WebrenderImageSource, UntypedSize2D<i32>);
    fn unlock(&mut self, id: u64);
}

pub enum WebrenderImageSource<'a> {
    TextureHandle(u32),
    Raw(&'a [u8]),
}

/// Type of Webrender External Image Handler.
pub enum WebrenderImageHandlerType {
    WebGL,
    Media,
    WebGPU,
}

/// List of Webrender external images to be shared among all external image
/// consumers (WebGL, Media, WebGPU).
/// It ensures that external image identifiers are unique.
#[derive(Default)]
pub struct WebrenderExternalImageRegistry {
    /// Map of all generated external images.
    external_images: HashMap<ExternalImageId, WebrenderImageHandlerType>,
    /// Id generator for the next external image identifier.
    next_image_id: u64,
}

impl WebrenderExternalImageRegistry {
    pub fn next_id(&mut self, handler_type: WebrenderImageHandlerType) -> ExternalImageId {
        self.next_image_id += 1;
        let key = ExternalImageId(self.next_image_id);
        self.external_images.insert(key, handler_type);
        key
    }

    pub fn remove(&mut self, key: &ExternalImageId) {
        self.external_images.remove(key);
    }

    pub fn get(&self, key: &ExternalImageId) -> Option<&WebrenderImageHandlerType> {
        self.external_images.get(key)
    }
}

/// WebRender External Image Handler implementation.
pub struct WebrenderExternalImageHandlers {
    /// WebGL handler.
    webgl_handler: Option<Box<dyn WebrenderExternalImageApi>>,
    /// Media player handler.
    media_handler: Option<Box<dyn WebrenderExternalImageApi>>,
    /// WebGPU handler.
    webgpu_handler: Option<Box<dyn WebrenderExternalImageApi>>,
    /// Webrender external images.
    external_images: Arc<Mutex<WebrenderExternalImageRegistry>>,
}

impl WebrenderExternalImageHandlers {
    pub fn new() -> (Self, Arc<Mutex<WebrenderExternalImageRegistry>>) {
        let external_images = Arc::new(Mutex::new(WebrenderExternalImageRegistry::default()));
        (
            Self {
                webgl_handler: None,
                media_handler: None,
                webgpu_handler: None,
                external_images: external_images.clone(),
            },
            external_images,
        )
    }

    pub fn set_handler(
        &mut self,
        handler: Box<dyn WebrenderExternalImageApi>,
        handler_type: WebrenderImageHandlerType,
    ) {
        match handler_type {
            WebrenderImageHandlerType::WebGL => self.webgl_handler = Some(handler),
            WebrenderImageHandlerType::Media => self.media_handler = Some(handler),
            WebrenderImageHandlerType::WebGPU => self.webgpu_handler = Some(handler),
        }
    }
}

impl ExternalImageHandler for WebrenderExternalImageHandlers {
    /// Lock the external image. Then, WR could start to read the
    /// image content.
    /// The WR client should not change the image content until the
    /// unlock() call.
    fn lock(&mut self, key: ExternalImageId, _channel_index: u8) -> ExternalImage {
        let external_images = self.external_images.lock().unwrap();
        let handler_type = external_images
            .get(&key)
            .expect("Tried to get unknown external image");
        match handler_type {
            WebrenderImageHandlerType::WebGL => {
                let (source, size) = self.webgl_handler.as_mut().unwrap().lock(key.0);
                let texture_id = match source {
                    WebrenderImageSource::TextureHandle(b) => b,
                    _ => panic!("Wrong type"),
                };
                ExternalImage {
                    uv: TexelRect::new(0.0, size.height as f32, size.width as f32, 0.0),
                    source: ExternalImageSource::NativeTexture(texture_id),
                }
            },
            WebrenderImageHandlerType::Media => {
                let (source, size) = self.media_handler.as_mut().unwrap().lock(key.0);
                let texture_id = match source {
                    WebrenderImageSource::TextureHandle(b) => b,
                    _ => panic!("Wrong type"),
                };
                ExternalImage {
                    uv: TexelRect::new(0.0, size.height as f32, size.width as f32, 0.0),
                    source: ExternalImageSource::NativeTexture(texture_id),
                }
            },
            WebrenderImageHandlerType::WebGPU => {
                let (source, size) = self.webgpu_handler.as_mut().unwrap().lock(key.0);
                let buffer = match source {
                    WebrenderImageSource::Raw(b) => b,
                    _ => panic!("Wrong type"),
                };
                ExternalImage {
                    uv: TexelRect::new(0.0, size.height as f32, size.width as f32, 0.0),
                    source: ExternalImageSource::RawData(buffer),
                }
            },
        }
    }

    /// Unlock the external image. The WR should not read the image
    /// content after this call.
    fn unlock(&mut self, key: ExternalImageId, _channel_index: u8) {
        let external_images = self.external_images.lock().unwrap();
        let handler_type = external_images
            .get(&key)
            .expect("Tried to get unknown external image");
        match handler_type {
            WebrenderImageHandlerType::WebGL => self.webgl_handler.as_mut().unwrap().unlock(key.0),
            WebrenderImageHandlerType::Media => self.media_handler.as_mut().unwrap().unlock(key.0),
            WebrenderImageHandlerType::WebGPU => {
                self.webgpu_handler.as_mut().unwrap().unlock(key.0)
            },
        };
    }
}

#[derive(Deserialize, Serialize)]
/// Serializable image updates that must be performed by WebRender.
pub enum ImageUpdate {
    /// Register a new image.
    AddImage(ImageKey, ImageDescriptor, SerializableImageData),
    /// Delete a previously registered image registration.
    DeleteImage(ImageKey),
    /// Update an existing image registration.
    UpdateImage(ImageKey, ImageDescriptor, SerializableImageData),
}

#[derive(Debug, Deserialize, Serialize)]
/// Serialized `ImageData`. It contains IPC byte channel receiver to prevent from loading bytes too
/// slow.
pub enum SerializableImageData {
    /// A simple series of bytes, provided by the embedding and owned by WebRender.
    /// The format is stored out-of-band, currently in ImageDescriptor.
    Raw(IpcSharedMemory),
    /// An image owned by the embedding, and referenced by WebRender. This may
    /// take the form of a texture or a heap-allocated buffer.
    External(ExternalImageData),
}

impl From<SerializableImageData> for ImageData {
    fn from(value: SerializableImageData) -> Self {
        match value {
            SerializableImageData::Raw(shared_memory) => ImageData::new(shared_memory.to_vec()),
            SerializableImageData::External(image) => ImageData::External(image),
        }
    }
}

/// The address of a node. Layout sends these back. They must be validated via
/// `from_untrusted_node_address` before they can be used, because we do not trust layout.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct UntrustedNodeAddress(pub *const c_void);

#[allow(unsafe_code)]
unsafe impl Send for UntrustedNodeAddress {}

impl Serialize for UntrustedNodeAddress {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        (self.0 as usize).serialize(s)
    }
}

impl<'de> Deserialize<'de> for UntrustedNodeAddress {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<UntrustedNodeAddress, D::Error> {
        let value: usize = Deserialize::deserialize(d)?;
        Ok(UntrustedNodeAddress::from_id(value))
    }
}

impl UntrustedNodeAddress {
    /// Creates an `UntrustedNodeAddress` from the given pointer address value.
    #[inline]
    pub fn from_id(id: usize) -> UntrustedNodeAddress {
        UntrustedNodeAddress(id as *const c_void)
    }
}

/// The result of a hit test in the compositor.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CompositorHitTestResult {
    /// The pipeline id of the resulting item.
    pub pipeline_id: PipelineId,

    /// The hit test point in the item's viewport.
    pub point_in_viewport: euclid::default::Point2D<f32>,

    /// The hit test point relative to the item itself.
    pub point_relative_to_item: euclid::default::Point2D<f32>,

    /// The node address of the hit test result.
    pub node: UntrustedNodeAddress,

    /// The cursor that should be used when hovering the item hit by the hit test.
    pub cursor: Option<Cursor>,

    /// The scroll tree node associated with this hit test item.
    pub scroll_tree_node: ScrollTreeNodeId,
}
