/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![deny(unsafe_code)]

pub mod display_list;
pub mod rendering_context;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use base::id::PipelineId;
use crossbeam_channel::Sender;
use display_list::{CompositorDisplayListInfo, ScrollTreeNodeId};
use embedder_traits::Cursor;
use euclid::default::Size2D;
use ipc_channel::ipc::{self, IpcBytesReceiver, IpcSender};
use libc::c_void;
use log::warn;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use webrender_api::units::{DevicePoint, LayoutPoint, TexelRect};
use webrender_api::{
    BuiltDisplayList, BuiltDisplayListDescriptor, ExternalImage, ExternalImageData,
    ExternalImageHandler, ExternalImageId, ExternalImageSource, ExternalScrollId,
    FontInstanceFlags, FontInstanceKey, FontKey, HitTestFlags, ImageData, ImageDescriptor,
    ImageKey, NativeFontHandle, PipelineId as WebRenderPipelineId,
};

pub use crate::rendering_context::RenderingContext;

/// This trait is used as a bridge between the different GL clients
/// in Servo that handles WebRender ExternalImages and the WebRender
/// ExternalImageHandler API.
//
/// This trait is used to notify lock/unlock messages and get the
/// required info that WR needs.
pub trait WebrenderExternalImageApi {
    fn lock(&mut self, id: u64) -> (WebrenderImageSource, Size2D<i32>);
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

pub trait WebRenderFontApi {
    fn add_font_instance(
        &self,
        font_key: FontKey,
        size: f32,
        flags: FontInstanceFlags,
    ) -> FontInstanceKey;
    fn add_font(&self, data: Arc<Vec<u8>>, index: u32) -> FontKey;
    fn add_system_font(&self, handle: NativeFontHandle) -> FontKey;

    /// Forward a `AddFont` message, sending it on to the compositor. This is used to get WebRender
    /// [`FontKey`]s for web fonts in the per-layout `FontContext`.
    fn forward_add_font_message(
        &self,
        bytes_receiver: IpcBytesReceiver,
        font_index: u32,
        result_sender: IpcSender<FontKey>,
    );
    /// Forward a `AddFontInstance` message, sending it on to the compositor. This is used to get
    /// WebRender [`FontInstanceKey`]s for web fonts in the per-layout `FontContext`.
    fn forward_add_font_instance_message(
        &self,
        font_key: FontKey,
        size: f32,
        flags: FontInstanceFlags,
        result_receiver: IpcSender<FontInstanceKey>,
    );
}

pub enum CanvasToCompositorMsg {
    GenerateKey(Sender<ImageKey>),
    UpdateImages(Vec<ImageUpdate>),
}

pub enum FontToCompositorMsg {
    AddFontInstance(FontKey, f32, FontInstanceFlags, Sender<FontInstanceKey>),
    AddFont(Sender<FontKey>, u32, IpcBytesReceiver),
    AddSystemFont(Sender<FontKey>, NativeFontHandle),
}

#[derive(Deserialize, Serialize)]
pub enum NetToCompositorMsg {
    AddImage(ImageKey, ImageDescriptor, ImageData),
    GenerateImageKey(IpcSender<ImageKey>),
}

/// The set of WebRender operations that can be initiated by the content process.
#[derive(Deserialize, Serialize)]
pub enum ScriptToCompositorMsg {
    /// Inform WebRender of the existence of this pipeline.
    SendInitialTransaction(WebRenderPipelineId),
    /// Perform a scroll operation.
    SendScrollNode(WebRenderPipelineId, LayoutPoint, ExternalScrollId),
    /// Inform WebRender of a new display list for the given pipeline.
    SendDisplayList {
        /// The [CompositorDisplayListInfo] that describes the display list being sent.
        display_list_info: CompositorDisplayListInfo,
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
    /// Perform a resource update operation.
    UpdateImages(Vec<SerializedImageUpdate>),
    /// Remove the given font resources from our WebRender instance.
    RemoveFonts(Vec<FontKey>, Vec<FontInstanceKey>),
}

/// A mechanism to send messages from networking to the WebRender instance.
#[derive(Clone, Deserialize, Serialize)]
pub struct WebRenderNetApi(IpcSender<NetToCompositorMsg>);

impl WebRenderNetApi {
    pub fn new(sender: IpcSender<NetToCompositorMsg>) -> Self {
        Self(sender)
    }

    pub fn generate_image_key(&self) -> ImageKey {
        let (sender, receiver) = ipc::channel().unwrap();
        self.0
            .send(NetToCompositorMsg::GenerateImageKey(sender))
            .expect("error sending image key generation");
        receiver.recv().expect("error receiving image key result")
    }

    pub fn add_image(&self, key: ImageKey, descriptor: ImageDescriptor, data: ImageData) {
        if let Err(e) = self
            .0
            .send(NetToCompositorMsg::AddImage(key, descriptor, data))
        {
            warn!("Error sending image update: {}", e);
        }
    }
}

/// A mechanism to send messages from ScriptThread to the parent process' WebRender instance.
#[derive(Clone, Deserialize, Serialize)]
pub struct WebRenderScriptApi(IpcSender<ScriptToCompositorMsg>);

impl WebRenderScriptApi {
    /// Create a new WebrenderIpcSender object that wraps the provided channel sender.
    pub fn new(sender: IpcSender<ScriptToCompositorMsg>) -> Self {
        Self(sender)
    }

    /// Inform WebRender of the existence of this pipeline.
    pub fn send_initial_transaction(&self, pipeline: WebRenderPipelineId) {
        if let Err(e) = self
            .0
            .send(ScriptToCompositorMsg::SendInitialTransaction(pipeline))
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
        if let Err(e) = self.0.send(ScriptToCompositorMsg::SendScrollNode(
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
        if let Err(e) = self.0.send(ScriptToCompositorMsg::SendDisplayList {
            display_list_info,
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
            .send(ScriptToCompositorMsg::HitTest(
                pipeline, point, flags, sender,
            ))
            .expect("error sending hit test");
        receiver.recv().expect("error receiving hit test result")
    }

    /// Create a new image key. Blocks until the key is available.
    pub fn generate_image_key(&self) -> Option<ImageKey> {
        let (sender, receiver) = ipc::channel().unwrap();
        self.0
            .send(ScriptToCompositorMsg::GenerateImageKey(sender))
            .ok()?;
        receiver.recv().ok()
    }

    /// Perform a resource update operation.
    pub fn update_images(&self, updates: Vec<ImageUpdate>) {
        let mut senders = Vec::new();
        // Convert `ImageUpdate` to `SerializedImageUpdate` because `ImageData` may contain large
        // byes. With this conversion, we send `IpcBytesReceiver` instead and use it to send the
        // actual bytes.
        let updates = updates
            .into_iter()
            .map(|update| match update {
                ImageUpdate::AddImage(k, d, data) => {
                    let data = match data {
                        ImageData::Raw(r) => {
                            let (sender, receiver) = ipc::bytes_channel().unwrap();
                            senders.push((sender, r));
                            SerializedImageData::Raw(receiver)
                        },
                        ImageData::External(e) => SerializedImageData::External(e),
                    };
                    SerializedImageUpdate::AddImage(k, d, data)
                },
                ImageUpdate::DeleteImage(k) => SerializedImageUpdate::DeleteImage(k),
                ImageUpdate::UpdateImage(k, d, data) => {
                    let data = match data {
                        ImageData::Raw(r) => {
                            let (sender, receiver) = ipc::bytes_channel().unwrap();
                            senders.push((sender, r));
                            SerializedImageData::Raw(receiver)
                        },
                        ImageData::External(e) => SerializedImageData::External(e),
                    };
                    SerializedImageUpdate::UpdateImage(k, d, data)
                },
            })
            .collect();

        if let Err(e) = self.0.send(ScriptToCompositorMsg::UpdateImages(updates)) {
            warn!("error sending image updates: {}", e);
        }

        senders.into_iter().for_each(|(tx, data)| {
            if let Err(e) = tx.send(&data) {
                warn!("error sending image data: {}", e);
            }
        });
    }

    pub fn remove_unused_font_resources(
        &self,
        keys: Vec<FontKey>,
        instance_keys: Vec<FontInstanceKey>,
    ) {
        if keys.is_empty() && instance_keys.is_empty() {
            return;
        }
        let _ = self
            .0
            .send(ScriptToCompositorMsg::RemoveFonts(keys, instance_keys));
    }
}

#[derive(Deserialize, Serialize)]
/// Serializable image updates that must be performed by WebRender.
pub enum ImageUpdate {
    /// Register a new image.
    AddImage(ImageKey, ImageDescriptor, ImageData),
    /// Delete a previously registered image registration.
    DeleteImage(ImageKey),
    /// Update an existing image registration.
    UpdateImage(ImageKey, ImageDescriptor, ImageData),
}

#[derive(Deserialize, Serialize)]
/// Serialized `ImageUpdate`.
pub enum SerializedImageUpdate {
    /// Register a new image.
    AddImage(ImageKey, ImageDescriptor, SerializedImageData),
    /// Delete a previously registered image registration.
    DeleteImage(ImageKey),
    /// Update an existing image registration.
    UpdateImage(ImageKey, ImageDescriptor, SerializedImageData),
}

#[derive(Debug, Deserialize, Serialize)]
/// Serialized `ImageData`. It contains IPC byte channel receiver to prevent from loading bytes too
/// slow.
pub enum SerializedImageData {
    /// A simple series of bytes, provided by the embedding and owned by WebRender.
    /// The format is stored out-of-band, currently in ImageDescriptor.
    Raw(ipc::IpcBytesReceiver),
    /// An image owned by the embedding, and referenced by WebRender. This may
    /// take the form of a texture or a heap-allocated buffer.
    External(ExternalImageData),
}

impl SerializedImageData {
    /// Convert to ``ImageData`.
    pub fn to_image_data(&self) -> Result<ImageData, ipc::IpcError> {
        match self {
            SerializedImageData::Raw(rx) => rx.recv().map(ImageData::new),
            SerializedImageData::External(image) => Ok(ImageData::External(*image)),
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
