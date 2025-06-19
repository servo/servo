/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The interface to the `compositing` crate.

use std::fmt::{Debug, Error, Formatter};

use base::id::{PipelineId, WebViewId};
use crossbeam_channel::Sender;
use embedder_traits::{
    AnimationState, EventLoopWaker, MouseButton, MouseButtonAction, TouchEventResult,
    WebDriverMessageId,
};
use euclid::Rect;
use ipc_channel::ipc::IpcSender;
use log::warn;
use malloc_size_of_derive::MallocSizeOf;
use pixels::RasterImage;
use strum_macros::IntoStaticStr;
use style_traits::CSSPixel;
use webrender_api::DocumentId;

pub mod display_list;
pub mod rendering_context;
pub mod viewport_description;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use bitflags::bitflags;
use display_list::CompositorDisplayListInfo;
use embedder_traits::{CompositorHitTestResult, ScreenGeometry};
use euclid::default::Size2D as UntypedSize2D;
use ipc_channel::ipc::{self, IpcSharedMemory};
use profile_traits::mem::{OpaqueSender, ReportsChan};
use serde::{Deserialize, Serialize};
use servo_geometry::{DeviceIndependentIntRect, DeviceIndependentIntSize};
use webrender_api::units::{DevicePoint, LayoutVector2D, TexelRect};
use webrender_api::{
    BuiltDisplayList, BuiltDisplayListDescriptor, ExternalImage, ExternalImageData,
    ExternalImageHandler, ExternalImageId, ExternalImageSource, ExternalScrollId,
    FontInstanceFlags, FontInstanceKey, FontKey, HitTestFlags, ImageData, ImageDescriptor,
    ImageKey, NativeFontHandle, PipelineId as WebRenderPipelineId,
};

use crate::viewport_description::ViewportDescription;

/// Sends messages to the compositor.
#[derive(Clone)]
pub struct CompositorProxy {
    pub sender: Sender<CompositorMsg>,
    /// Access to [`Self::sender`] that is possible to send across an IPC
    /// channel. These messages are routed via the router thread to
    /// [`Self::sender`].
    pub cross_process_compositor_api: CrossProcessCompositorApi,
    pub event_loop_waker: Box<dyn EventLoopWaker>,
}

impl OpaqueSender<CompositorMsg> for CompositorProxy {
    fn send(&self, message: CompositorMsg) {
        CompositorProxy::send(self, message)
    }
}

impl CompositorProxy {
    pub fn send(&self, msg: CompositorMsg) {
        if let Err(err) = self.sender.send(msg) {
            warn!("Failed to send response ({:?}).", err);
        }
        self.event_loop_waker.wake();
    }
}

/// Messages from (or via) the constellation thread to the compositor.
#[derive(Deserialize, IntoStaticStr, Serialize)]
pub enum CompositorMsg {
    /// Alerts the compositor that the given pipeline has changed whether it is running animations.
    ChangeRunningAnimationsState(WebViewId, PipelineId, AnimationState),
    /// Create or update a webview, given its frame tree.
    CreateOrUpdateWebView(SendableFrameTree),
    /// Remove a webview.
    RemoveWebView(WebViewId),
    /// Script has handled a touch event, and either prevented or allowed default actions.
    TouchEventProcessed(WebViewId, TouchEventResult),
    /// Composite to a PNG file and return the Image over a passed channel.
    CreatePng(
        WebViewId,
        Option<Rect<f32, CSSPixel>>,
        IpcSender<Option<RasterImage>>,
    ),
    /// A reply to the compositor asking if the output image is stable.
    IsReadyToSaveImageReply(bool),
    /// Set whether to use less resources by stopping animations.
    SetThrottled(WebViewId, PipelineId, bool),
    /// WebRender has produced a new frame. This message informs the compositor that
    /// the frame is ready. It contains a bool to indicate if it needs to composite and the
    /// `DocumentId` of the new frame.
    NewWebRenderFrameReady(DocumentId, bool),
    /// Script or the Constellation is notifying the renderer that a Pipeline has finished
    /// shutting down. The renderer will not discard the Pipeline until both report that
    /// they have fully shut it down, to avoid recreating it due to any subsequent
    /// messages.
    PipelineExited(WebViewId, PipelineId, PipelineExitSource),
    /// The load of a page has completed
    LoadComplete(WebViewId),
    /// WebDriver mouse button event
    WebDriverMouseButtonEvent(
        WebViewId,
        MouseButtonAction,
        MouseButton,
        f32,
        f32,
        Option<WebDriverMessageId>,
    ),
    /// WebDriver mouse move event
    WebDriverMouseMoveEvent(WebViewId, f32, f32, Option<WebDriverMessageId>),
    // Webdriver wheel scroll event
    WebDriverWheelScrollEvent(WebViewId, f32, f32, f64, f64, Option<WebDriverMessageId>),

    /// Inform WebRender of the existence of this pipeline.
    SendInitialTransaction(WebRenderPipelineId),
    /// Perform a scroll operation.
    SendScrollNode(
        WebViewId,
        WebRenderPipelineId,
        LayoutVector2D,
        ExternalScrollId,
    ),
    /// Inform WebRender of a new display list for the given pipeline.
    SendDisplayList {
        /// The [`WebViewId`] that this display list belongs to.
        webview_id: WebViewId,
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
    GetClientWindowRect(WebViewId, IpcSender<DeviceIndependentIntRect>),
    /// Get the size of the screen that the client window inhabits.
    GetScreenSize(WebViewId, IpcSender<DeviceIndependentIntSize>),
    /// Get the available screen size (without toolbars and docks) for the screen
    /// the client window inhabits.
    GetAvailableScreenSize(WebViewId, IpcSender<DeviceIndependentIntSize>),

    /// Measure the current memory usage associated with the compositor.
    /// The report must be sent on the provided channel once it's complete.
    CollectMemoryReport(ReportsChan),
    /// A top-level frame has parsed a viewport metatag and is sending the new constraints.
    Viewport(WebViewId, ViewportDescription),
}

impl Debug for CompositorMsg {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), Error> {
        let string: &'static str = self.into();
        write!(formatter, "{string}")
    }
}

#[derive(Deserialize, Serialize)]
pub struct SendableFrameTree {
    pub pipeline: CompositionPipeline,
    pub children: Vec<SendableFrameTree>,
}

/// The subset of the pipeline that is needed for layer composition.
#[derive(Clone, Deserialize, Serialize)]
pub struct CompositionPipeline {
    pub id: PipelineId,
    pub webview_id: WebViewId,
}

/// A mechanism to send messages from ScriptThread to the parent process' WebRender instance.
#[derive(Clone, Deserialize, MallocSizeOf, Serialize)]
pub struct CrossProcessCompositorApi(pub IpcSender<CompositorMsg>);

impl CrossProcessCompositorApi {
    /// Create a new [`CrossProcessCompositorApi`] struct that does not have a listener on the other
    /// end to use for unit testing.
    pub fn dummy() -> Self {
        let (sender, _) = ipc::channel().unwrap();
        Self(sender)
    }

    /// Get the sender for this proxy.
    pub fn sender(&self) -> &IpcSender<CompositorMsg> {
        &self.0
    }

    /// Inform WebRender of the existence of this pipeline.
    pub fn send_initial_transaction(&self, pipeline: WebRenderPipelineId) {
        if let Err(e) = self.0.send(CompositorMsg::SendInitialTransaction(pipeline)) {
            warn!("Error sending initial transaction: {}", e);
        }
    }

    /// Perform a scroll operation.
    pub fn send_scroll_node(
        &self,
        webview_id: WebViewId,
        pipeline_id: WebRenderPipelineId,
        point: LayoutVector2D,
        scroll_id: ExternalScrollId,
    ) {
        if let Err(e) = self.0.send(CompositorMsg::SendScrollNode(
            webview_id,
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
        webview_id: WebViewId,
        display_list_info: &CompositorDisplayListInfo,
        list: BuiltDisplayList,
    ) {
        let (display_list_data, display_list_descriptor) = list.into_data();
        let (display_list_sender, display_list_receiver) = ipc::bytes_channel().unwrap();
        if let Err(e) = self.0.send(CompositorMsg::SendDisplayList {
            webview_id,
            display_list_descriptor,
            display_list_receiver,
        }) {
            warn!("Error sending display list: {}", e);
        }

        let display_list_info_serialized =
            bincode::serialize(&display_list_info).unwrap_or_default();
        if let Err(error) = display_list_sender.send(&display_list_info_serialized) {
            warn!("Error sending display list info: {error}");
        }

        if let Err(error) = display_list_sender.send(&display_list_data.items_data) {
            warn!("Error sending display list items: {error}");
        }
        if let Err(error) = display_list_sender.send(&display_list_data.cache_data) {
            warn!("Error sending display list cache data: {error}");
        }
        if let Err(error) = display_list_sender.send(&display_list_data.spatial_tree) {
            warn!("Error sending display spatial tree: {error}");
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
            .send(CompositorMsg::HitTest(pipeline, point, flags, sender))
            .expect("error sending hit test");
        receiver.recv().expect("error receiving hit test result")
    }

    /// Create a new image key. Blocks until the key is available.
    pub fn generate_image_key(&self) -> Option<ImageKey> {
        let (sender, receiver) = ipc::channel().unwrap();
        self.0.send(CompositorMsg::GenerateImageKey(sender)).ok()?;
        receiver.recv().ok()
    }

    pub fn add_image(
        &self,
        key: ImageKey,
        descriptor: ImageDescriptor,
        data: SerializableImageData,
    ) {
        if let Err(e) = self.0.send(CompositorMsg::AddImage(key, descriptor, data)) {
            warn!("Error sending image update: {}", e);
        }
    }

    /// Perform an image resource update operation.
    pub fn update_images(&self, updates: Vec<ImageUpdate>) {
        if let Err(e) = self.0.send(CompositorMsg::UpdateImages(updates)) {
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
        let _ = self.0.send(CompositorMsg::RemoveFonts(keys, instance_keys));
    }

    pub fn add_font_instance(
        &self,
        font_instance_key: FontInstanceKey,
        font_key: FontKey,
        size: f32,
        flags: FontInstanceFlags,
    ) {
        let _x = self.0.send(CompositorMsg::AddFontInstance(
            font_instance_key,
            font_key,
            size,
            flags,
        ));
    }

    pub fn add_font(&self, font_key: FontKey, data: Arc<IpcSharedMemory>, index: u32) {
        let _ = self.0.send(CompositorMsg::AddFont(font_key, data, index));
    }

    pub fn add_system_font(&self, font_key: FontKey, handle: NativeFontHandle) {
        let _ = self.0.send(CompositorMsg::AddSystemFont(font_key, handle));
    }

    pub fn fetch_font_keys(
        &self,
        number_of_font_keys: usize,
        number_of_font_instance_keys: usize,
    ) -> (Vec<FontKey>, Vec<FontInstanceKey>) {
        let (sender, receiver) = ipc_channel::ipc::channel().expect("Could not create IPC channel");
        let _ = self.0.send(CompositorMsg::GenerateFontKeys(
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

/// A trait that exposes the embedding layer's `WebView` to the Servo renderer.
/// This is to prevent a dependency cycle between the renderer and the embedding
/// layer.
pub trait WebViewTrait {
    fn id(&self) -> WebViewId;
    fn screen_geometry(&self) -> Option<ScreenGeometry>;
    fn set_animating(&self, new_value: bool);
}

/// What entity is reporting that a `Pipeline` has exited. Only when all have
/// done this will the renderer discard its details.
#[derive(Clone, Copy, Default, Deserialize, PartialEq, Serialize)]
pub struct PipelineExitSource(u8);

bitflags! {
    impl PipelineExitSource: u8 {
        const Script = 1 << 0;
        const Constellation = 1 << 1;
    }
}
