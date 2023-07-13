/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![deny(missing_docs)]

use std::cell::Cell;
use std::fmt;
use std::marker::PhantomData;
use std::path::PathBuf;
use std::sync::Arc;
use std::u32;
use time::precise_time_ns;
//use crate::api::peek_poke::PeekPoke;
use crate::api::channel::{Sender, single_msg_channel, unbounded_channel};
use crate::api::{ColorF, BuiltDisplayList, IdNamespace, ExternalScrollId};
use crate::api::{SharedFontInstanceMap, FontKey, FontInstanceKey, NativeFontHandle, ZoomFactor};
use crate::api::{BlobImageData, BlobImageKey, ImageData, ImageDescriptor, ImageKey, Epoch, QualitySettings};
use crate::api::{BlobImageParams, BlobImageRequest, BlobImageResult, AsyncBlobImageRasterizer, BlobImageHandler};
use crate::api::{DocumentId, PipelineId, PropertyBindingId, PropertyBindingKey, ExternalEvent};
use crate::api::{HitTestResult, HitTesterRequest, ApiHitTester, PropertyValue, DynamicProperties};
use crate::api::{ScrollClamping, TileSize, NotificationRequest, DebugFlags, ScrollNodeState};
use crate::api::{GlyphDimensionRequest, GlyphIndexRequest, GlyphIndex, GlyphDimensions};
use crate::api::{FontInstanceOptions, FontInstancePlatformOptions, FontVariation};
use crate::api::DEFAULT_TILE_SIZE;
use crate::api::units::*;
use crate::api_resources::ApiResources;
use crate::scene_builder_thread::{SceneBuilderRequest, SceneBuilderResult};
use crate::intern::InterningMemoryReport;
use crate::profiler::{self, TransactionProfile};

#[repr(C)]
#[derive(Clone, Copy, Debug)]
#[cfg_attr(any(feature = "serde"), derive(Deserialize, Serialize))]
struct ResourceId(pub u32);

/// Update of a persistent resource in WebRender.
///
/// ResourceUpdate changes keep theirs effect across display list changes.
#[derive(Clone)]
#[cfg_attr(any(feature = "serde"), derive(Deserialize, Serialize))]
pub enum ResourceUpdate {
    /// See `AddImage`.
    AddImage(AddImage),
    /// See `UpdateImage`.
    UpdateImage(UpdateImage),
    /// Delete an existing image resource.
    ///
    /// It is invalid to continue referring to the image key in any display list
    /// in the transaction that contains the `DeleteImage` message and subsequent
    /// transactions.
    DeleteImage(ImageKey),
    /// See `AddBlobImage`.
    AddBlobImage(AddBlobImage),
    /// See `UpdateBlobImage`.
    UpdateBlobImage(UpdateBlobImage),
    /// Delete existing blob image resource.
    DeleteBlobImage(BlobImageKey),
    /// See `AddBlobImage::visible_area`.
    SetBlobImageVisibleArea(BlobImageKey, DeviceIntRect),
    /// See `AddFont`.
    AddFont(AddFont),
    /// Deletes an already existing font resource.
    ///
    /// It is invalid to continue referring to the font key in any display list
    /// in the transaction that contains the `DeleteImage` message and subsequent
    /// transactions.
    DeleteFont(FontKey),
    /// See `AddFontInstance`.
    AddFontInstance(AddFontInstance),
    /// Deletes an already existing font instance resource.
    ///
    /// It is invalid to continue referring to the font instance in any display
    /// list in the transaction that contains the `DeleteImage` message and
    /// subsequent transactions.
    DeleteFontInstance(FontInstanceKey),
}

impl fmt::Debug for ResourceUpdate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ResourceUpdate::AddImage(ref i) => f.write_fmt(format_args!(
                "ResourceUpdate::AddImage size({:?})",
                &i.descriptor.size
            )),
            ResourceUpdate::UpdateImage(ref i) => f.write_fmt(format_args!(
                "ResourceUpdate::UpdateImage size({:?})",
                &i.descriptor.size
            )),
            ResourceUpdate::AddBlobImage(ref i) => f.write_fmt(format_args!(
                "ResourceUFpdate::AddBlobImage size({:?})",
                &i.descriptor.size
            )),
            ResourceUpdate::UpdateBlobImage(i) => f.write_fmt(format_args!(
                "ResourceUpdate::UpdateBlobImage size({:?})",
                &i.descriptor.size
            )),
            ResourceUpdate::DeleteImage(..) => f.write_str("ResourceUpdate::DeleteImage"),
            ResourceUpdate::DeleteBlobImage(..) => f.write_str("ResourceUpdate::DeleteBlobImage"),
            ResourceUpdate::SetBlobImageVisibleArea(..) => f.write_str("ResourceUpdate::SetBlobImageVisibleArea"),
            ResourceUpdate::AddFont(..) => f.write_str("ResourceUpdate::AddFont"),
            ResourceUpdate::DeleteFont(..) => f.write_str("ResourceUpdate::DeleteFont"),
            ResourceUpdate::AddFontInstance(..) => f.write_str("ResourceUpdate::AddFontInstance"),
            ResourceUpdate::DeleteFontInstance(..) => f.write_str("ResourceUpdate::DeleteFontInstance"),
        }
    }
}

/// Whether to generate a frame, and if so, an id that allows tracking this
/// transaction through the various frame stages.
#[derive(Clone, Debug)]
pub enum GenerateFrame {
    /// Generate a frame if something changed.
    Yes {
        /// An id that allows tracking the frame transaction through the various
        /// frame stages. Specified by the caller of generate_frame().
        id: u64,
    },
    /// Don't generate a frame even if something has changed.
    No,
}

impl GenerateFrame {
    ///
    pub fn as_bool(&self) -> bool {
        match self {
            GenerateFrame::Yes { .. } => true,
            GenerateFrame::No => false,
        }
    }

    /// Return the frame ID, if a frame is generated.
    pub fn id(&self) -> Option<u64> {
        match self {
            GenerateFrame::Yes { id } => Some(*id),
            GenerateFrame::No => None,
        }
    }
}

/// A Transaction is a group of commands to apply atomically to a document.
///
/// This mechanism ensures that:
///  - no other message can be interleaved between two commands that need to be applied together.
///  - no redundant work is performed if two commands in the same transaction cause the scene or
///    the frame to be rebuilt.
pub struct Transaction {
    /// Operations affecting the scene (applied before scene building).
    scene_ops: Vec<SceneMsg>,
    /// Operations affecting the generation of frames (applied after scene building).
    frame_ops: Vec<FrameMsg>,

    notifications: Vec<NotificationRequest>,

    /// Persistent resource updates to apply as part of this transaction.
    pub resource_updates: Vec<ResourceUpdate>,

    /// True if the transaction needs the scene building thread's attention.
    /// False for things that can skip the scene builder, like APZ changes and
    /// async images.
    ///
    /// Before this `Transaction` is converted to a `TransactionMsg`, we look
    /// over its contents and set this if we're doing anything the scene builder
    /// needs to know about, so this is only a default.
    use_scene_builder_thread: bool,

    /// Whether to generate a frame, and if so, an id that allows tracking this
    /// transaction through the various frame stages. Specified by the caller of
    /// generate_frame().
    generate_frame: GenerateFrame,

    /// Set to true in order to force re-rendering even if WebRender can't internally
    /// detect that something has changed.
    pub invalidate_rendered_frame: bool,

    low_priority: bool,
}

impl Transaction {
    /// Constructor.
    pub fn new() -> Self {
        Transaction {
            scene_ops: Vec::new(),
            frame_ops: Vec::new(),
            resource_updates: Vec::new(),
            notifications: Vec::new(),
            use_scene_builder_thread: true,
            generate_frame: GenerateFrame::No,
            invalidate_rendered_frame: false,
            low_priority: false,
        }
    }

    /// Marks this transaction to allow it to skip going through the scene builder
    /// thread.
    ///
    /// This is useful to avoid jank in transaction associated with animated
    /// property updates, panning and zooming.
    ///
    /// Note that transactions that skip the scene builder thread can race ahead of
    /// transactions that don't skip it.
    pub fn skip_scene_builder(&mut self) {
        self.use_scene_builder_thread = false;
    }

    /// Marks this transaction to enforce going through the scene builder thread.
    pub fn use_scene_builder_thread(&mut self) {
        self.use_scene_builder_thread = true;
    }

    /// Returns true if the transaction has no effect.
    pub fn is_empty(&self) -> bool {
        !self.generate_frame.as_bool() &&
            !self.invalidate_rendered_frame &&
            self.scene_ops.is_empty() &&
            self.frame_ops.is_empty() &&
            self.resource_updates.is_empty() &&
            self.notifications.is_empty()
    }

    /// Update a pipeline's epoch.
    pub fn update_epoch(&mut self, pipeline_id: PipelineId, epoch: Epoch) {
        // We track epochs before and after scene building.
        // This one will be applied to the pending scene right away:
        self.scene_ops.push(SceneMsg::UpdateEpoch(pipeline_id, epoch));
        // And this one will be applied to the currently built scene at the end
        // of the transaction (potentially long after the scene_ops one).
        self.frame_ops.push(FrameMsg::UpdateEpoch(pipeline_id, epoch));
        // We could avoid the duplication here by storing the epoch updates in a
        // separate array and let the render backend schedule the updates at the
        // proper times, but it wouldn't make things simpler.
    }

    /// Sets the root pipeline.
    ///
    /// # Examples
    ///
    /// ```
    /// # use webrender::api::{PipelineId};
    /// # use webrender::api::units::{DeviceIntSize};
    /// # use webrender::render_api::{RenderApiSender, Transaction};
    /// # fn example() {
    /// let pipeline_id = PipelineId(0, 0);
    /// let mut txn = Transaction::new();
    /// txn.set_root_pipeline(pipeline_id);
    /// # }
    /// ```
    pub fn set_root_pipeline(&mut self, pipeline_id: PipelineId) {
        self.scene_ops.push(SceneMsg::SetRootPipeline(pipeline_id));
    }

    /// Removes data associated with a pipeline from the internal data structures.
    /// If the specified `pipeline_id` is for the root pipeline, the root pipeline
    /// is reset back to `None`.
    pub fn remove_pipeline(&mut self, pipeline_id: PipelineId) {
        self.scene_ops.push(SceneMsg::RemovePipeline(pipeline_id));
    }

    /// Supplies a new frame to WebRender.
    ///
    /// Non-blocking, it notifies a worker process which processes the display list.
    ///
    /// Note: Scrolling doesn't require an own Frame.
    ///
    /// Arguments:
    ///
    /// * `epoch`: The unique Frame ID, monotonically increasing.
    /// * `background`: The background color of this pipeline.
    /// * `viewport_size`: The size of the viewport for this frame.
    /// * `pipeline_id`: The ID of the pipeline that is supplying this display list.
    /// * `display_list`: The root Display list used in this frame.
    /// * `preserve_frame_state`: If a previous frame exists which matches this pipeline
    ///                           id, this setting determines if frame state (such as scrolling
    ///                           position) should be preserved for this new display list.
    pub fn set_display_list(
        &mut self,
        epoch: Epoch,
        background: Option<ColorF>,
        viewport_size: LayoutSize,
        (pipeline_id, mut display_list): (PipelineId, BuiltDisplayList),
        preserve_frame_state: bool,
    ) {
        display_list.set_send_time_ns(precise_time_ns());
        self.scene_ops.push(
            SceneMsg::SetDisplayList {
                display_list,
                epoch,
                pipeline_id,
                background,
                viewport_size,
                preserve_frame_state,
            }
        );
    }

    /// Add a set of persistent resource updates to apply as part of this transaction.
    pub fn update_resources(&mut self, mut resources: Vec<ResourceUpdate>) {
        self.resource_updates.append(&mut resources);
    }

    // Note: Gecko uses this to get notified when a transaction that contains
    // potentially long blob rasterization or scene build is ready to be rendered.
    // so that the tab-switching integration can react adequately when tab
    // switching takes too long. For this use case when matters is that the
    // notification doesn't fire before scene building and blob rasterization.

    /// Trigger a notification at a certain stage of the rendering pipeline.
    ///
    /// Not that notification requests are skipped during serialization, so is is
    /// best to use them for synchronization purposes and not for things that could
    /// affect the WebRender's state.
    pub fn notify(&mut self, event: NotificationRequest) {
        self.notifications.push(event);
    }

    /// Setup the output region in the framebuffer for a given document.
    pub fn set_document_view(
        &mut self,
        device_rect: DeviceIntRect,
        device_pixel_ratio: f32,
    ) {
        assert!(device_pixel_ratio > 0.0);
        window_size_sanity_check(device_rect.size);
        self.scene_ops.push(
            SceneMsg::SetDocumentView {
                device_rect,
                device_pixel_ratio,
            },
        );
    }

    /// Scrolls the node identified by the given external scroll id to the
    /// given scroll position, relative to the pre-scrolled offset for the
    /// scrolling layer. That is, providing an origin of (0,0) will reset
    /// any WR-side scrolling and just render the display items at the
    /// pre-scrolled offsets as provided in the display list. Larger `origin`
    /// values will cause the layer to be scrolled further towards the end of
    /// the scroll range.
    /// If the ScrollClamping argument is set to clamp, the scroll position
    /// is clamped to what WebRender understands to be the bounds of the
    /// scroll range, based on the sizes of the scrollable content and the
    /// scroll port.
    pub fn scroll_node_with_id(
        &mut self,
        origin: LayoutPoint,
        id: ExternalScrollId,
        clamp: ScrollClamping,
    ) {
        self.frame_ops.push(FrameMsg::ScrollNodeWithId(origin, id, clamp));
    }

    /// Set the current quality / performance settings for this document.
    pub fn set_quality_settings(&mut self, settings: QualitySettings) {
        self.scene_ops.push(SceneMsg::SetQualitySettings { settings });
    }

    ///
    pub fn set_page_zoom(&mut self, page_zoom: ZoomFactor) {
        self.scene_ops.push(SceneMsg::SetPageZoom(page_zoom));
    }

    ///
    pub fn set_pinch_zoom(&mut self, pinch_zoom: ZoomFactor) {
        self.frame_ops.push(FrameMsg::SetPinchZoom(pinch_zoom));
    }

    ///
    pub fn set_is_transform_async_zooming(&mut self, is_zooming: bool, animation_id: PropertyBindingId) {
        self.frame_ops.push(FrameMsg::SetIsTransformAsyncZooming(is_zooming, animation_id));
    }

    ///
    pub fn set_pan(&mut self, pan: DeviceIntPoint) {
        self.frame_ops.push(FrameMsg::SetPan(pan));
    }

    /// Generate a new frame. When it's done and a RenderNotifier has been set
    /// in `webrender::Renderer`, [new_frame_ready()][notifier] gets called.
    /// Note that the notifier is called even if the frame generation was a
    /// no-op; the arguments passed to `new_frame_ready` will provide information
    /// as to when happened.
    ///
    /// [notifier]: trait.RenderNotifier.html#tymethod.new_frame_ready
    pub fn generate_frame(&mut self, id: u64) {
        self.generate_frame = GenerateFrame::Yes{ id };
    }

    /// Invalidate rendered frame. It ensure that frame will be rendered during
    /// next frame generation. WebRender could skip frame rendering if there
    /// is no update.
    /// But there are cases that needs to force rendering.
    ///  - Content of image is updated by reusing same ExternalImageId.
    ///  - Platform requests it if pixels become stale (like wakeup from standby).
    pub fn invalidate_rendered_frame(&mut self) {
        self.invalidate_rendered_frame = true;
    }

    /// Supply a list of animated property bindings that should be used to resolve
    /// bindings in the current display list.
    pub fn update_dynamic_properties(&mut self, properties: DynamicProperties) {
        self.frame_ops.push(FrameMsg::UpdateDynamicProperties(properties));
    }

    /// Add to the list of animated property bindings that should be used to
    /// resolve bindings in the current display list. This is a convenience method
    /// so the caller doesn't have to figure out all the dynamic properties before
    /// setting them on the transaction but can do them incrementally.
    pub fn append_dynamic_transform_properties(&mut self, transforms: Vec<PropertyValue<LayoutTransform>>) {
        self.frame_ops.push(FrameMsg::AppendDynamicTransformProperties(transforms));
    }

    /// Consumes this object and just returns the frame ops.
    pub fn get_frame_ops(self) -> Vec<FrameMsg> {
        self.frame_ops
    }

    fn finalize(self, document_id: DocumentId) -> Box<TransactionMsg> {
        Box::new(TransactionMsg {
            document_id,
            scene_ops: self.scene_ops,
            frame_ops: self.frame_ops,
            resource_updates: self.resource_updates,
            notifications: self.notifications,
            use_scene_builder_thread: self.use_scene_builder_thread,
            generate_frame: self.generate_frame,
            invalidate_rendered_frame: self.invalidate_rendered_frame,
            low_priority: self.low_priority,
            blob_rasterizer: None,
            blob_requests: Vec::new(),
            rasterized_blobs: Vec::new(),
            profile: TransactionProfile::new(),
        })
    }

    /// See `ResourceUpdate::AddImage`.
    pub fn add_image(
        &mut self,
        key: ImageKey,
        descriptor: ImageDescriptor,
        data: ImageData,
        tiling: Option<TileSize>,
    ) {
        self.resource_updates.push(ResourceUpdate::AddImage(AddImage {
            key,
            descriptor,
            data,
            tiling,
        }));
    }

    /// See `ResourceUpdate::UpdateImage`.
    pub fn update_image(
        &mut self,
        key: ImageKey,
        descriptor: ImageDescriptor,
        data: ImageData,
        dirty_rect: &ImageDirtyRect,
    ) {
        self.resource_updates.push(ResourceUpdate::UpdateImage(UpdateImage {
            key,
            descriptor,
            data,
            dirty_rect: *dirty_rect,
        }));
    }

    /// See `ResourceUpdate::DeleteImage`.
    pub fn delete_image(&mut self, key: ImageKey) {
        self.resource_updates.push(ResourceUpdate::DeleteImage(key));
    }

    /// See `ResourceUpdate::AddBlobImage`.
    pub fn add_blob_image(
        &mut self,
        key: BlobImageKey,
        descriptor: ImageDescriptor,
        data: Arc<BlobImageData>,
        visible_rect: DeviceIntRect,
        tile_size: Option<TileSize>,
    ) {
        self.resource_updates.push(
            ResourceUpdate::AddBlobImage(AddBlobImage {
                key,
                descriptor,
                data,
                visible_rect,
                tile_size: tile_size.unwrap_or(DEFAULT_TILE_SIZE),
            })
        );
    }

    /// See `ResourceUpdate::UpdateBlobImage`.
    pub fn update_blob_image(
        &mut self,
        key: BlobImageKey,
        descriptor: ImageDescriptor,
        data: Arc<BlobImageData>,
        visible_rect: DeviceIntRect,
        dirty_rect: &BlobDirtyRect,
    ) {
        self.resource_updates.push(
            ResourceUpdate::UpdateBlobImage(UpdateBlobImage {
                key,
                descriptor,
                data,
                visible_rect,
                dirty_rect: *dirty_rect,
            })
        );
    }

    /// See `ResourceUpdate::DeleteBlobImage`.
    pub fn delete_blob_image(&mut self, key: BlobImageKey) {
        self.resource_updates.push(ResourceUpdate::DeleteBlobImage(key));
    }

    /// See `ResourceUpdate::SetBlobImageVisibleArea`.
    pub fn set_blob_image_visible_area(&mut self, key: BlobImageKey, area: DeviceIntRect) {
        self.resource_updates.push(ResourceUpdate::SetBlobImageVisibleArea(key, area));
    }

    /// See `ResourceUpdate::AddFont`.
    pub fn add_raw_font(&mut self, key: FontKey, bytes: Vec<u8>, index: u32) {
        self.resource_updates
            .push(ResourceUpdate::AddFont(AddFont::Raw(key, Arc::new(bytes), index)));
    }

    /// See `ResourceUpdate::AddFont`.
    pub fn add_native_font(&mut self, key: FontKey, native_handle: NativeFontHandle) {
        self.resource_updates
            .push(ResourceUpdate::AddFont(AddFont::Native(key, native_handle)));
    }

    /// See `ResourceUpdate::DeleteFont`.
    pub fn delete_font(&mut self, key: FontKey) {
        self.resource_updates.push(ResourceUpdate::DeleteFont(key));
    }

    /// See `ResourceUpdate::AddFontInstance`.
    pub fn add_font_instance(
        &mut self,
        key: FontInstanceKey,
        font_key: FontKey,
        glyph_size: f32,
        options: Option<FontInstanceOptions>,
        platform_options: Option<FontInstancePlatformOptions>,
        variations: Vec<FontVariation>,
    ) {
        self.resource_updates
            .push(ResourceUpdate::AddFontInstance(AddFontInstance {
                key,
                font_key,
                glyph_size,
                options,
                platform_options,
                variations,
            }));
    }

    /// See `ResourceUpdate::DeleteFontInstance`.
    pub fn delete_font_instance(&mut self, key: FontInstanceKey) {
        self.resource_updates.push(ResourceUpdate::DeleteFontInstance(key));
    }

    /// A hint that this transaction can be processed at a lower priority. High-
    /// priority transactions can jump ahead of regular-priority transactions,
    /// but both high- and regular-priority transactions are processed in order
    /// relative to other transactions of the same priority.
    pub fn set_low_priority(&mut self, low_priority: bool) {
        self.low_priority = low_priority;
    }

    /// Returns whether this transaction is marked as low priority.
    pub fn is_low_priority(&self) -> bool {
        self.low_priority
    }
}

///
pub struct DocumentTransaction {
    ///
    pub document_id: DocumentId,
    ///
    pub transaction: Transaction,
}

/// Represents a transaction in the format sent through the channel.
pub struct TransactionMsg {
    ///
    pub document_id: DocumentId,
    /// Changes that require re-building the scene.
    pub scene_ops: Vec<SceneMsg>,
    /// Changes to animated properties that do not require re-building the scene.
    pub frame_ops: Vec<FrameMsg>,
    /// Updates to resources that persist across display lists.
    pub resource_updates: Vec<ResourceUpdate>,
    /// Whether to trigger frame building and rendering if something has changed.
    pub generate_frame: GenerateFrame,
    /// Whether to force frame building and rendering even if no changes are internally
    /// observed.
    pub invalidate_rendered_frame: bool,
    /// Whether to enforce that this transaction go through the scene builder.
    pub use_scene_builder_thread: bool,
    ///
    pub low_priority: bool,

    /// Handlers to notify at certain points of the pipeline.
    pub notifications: Vec<NotificationRequest>,
    ///
    pub blob_rasterizer: Option<Box<dyn AsyncBlobImageRasterizer>>,
    ///
    pub blob_requests: Vec<BlobImageParams>,
    ///
    pub rasterized_blobs: Vec<(BlobImageRequest, BlobImageResult)>,
    /// Collect various data along the rendering pipeline to display it in the embedded profiler.
    pub profile: TransactionProfile,
}

impl fmt::Debug for TransactionMsg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "threaded={}, genframe={:?}, invalidate={}, low_priority={}",
                        self.use_scene_builder_thread,
                        self.generate_frame,
                        self.invalidate_rendered_frame,
                        self.low_priority,
                    ).unwrap();
        for scene_op in &self.scene_ops {
            writeln!(f, "\t\t{:?}", scene_op).unwrap();
        }

        for frame_op in &self.frame_ops {
            writeln!(f, "\t\t{:?}", frame_op).unwrap();
        }

        for resource_update in &self.resource_updates {
            writeln!(f, "\t\t{:?}", resource_update).unwrap();
        }
        Ok(())
    }
}

impl TransactionMsg {
    /// Returns true if this transaction has no effect.
    pub fn is_empty(&self) -> bool {
        !self.generate_frame.as_bool() &&
            !self.invalidate_rendered_frame &&
            self.scene_ops.is_empty() &&
            self.frame_ops.is_empty() &&
            self.resource_updates.is_empty() &&
            self.notifications.is_empty()
    }
}

/// Creates an image resource with provided parameters.
///
/// Must be matched with a `DeleteImage` at some point to prevent memory leaks.
#[derive(Clone)]
#[cfg_attr(any(feature = "serde"), derive(Deserialize, Serialize))]
pub struct AddImage {
    /// A key to identify the image resource.
    pub key: ImageKey,
    /// Properties of the image.
    pub descriptor: ImageDescriptor,
    /// The pixels of the image.
    pub data: ImageData,
    /// An optional tiling scheme to apply when storing the image's data
    /// on the GPU. Applies to both width and heights of the tiles.
    ///
    /// Note that WebRender may internally chose to tile large images
    /// even if this member is set to `None`.
    pub tiling: Option<TileSize>,
}

/// Updates an already existing image resource.
#[derive(Clone)]
#[cfg_attr(any(feature = "serde"), derive(Deserialize, Serialize))]
pub struct UpdateImage {
    /// The key identfying the image resource to update.
    pub key: ImageKey,
    /// Properties of the image.
    pub descriptor: ImageDescriptor,
    /// The pixels of the image.
    pub data: ImageData,
    /// An optional dirty rect that lets WebRender optimize the amount of
    /// data to transfer to the GPU.
    ///
    /// The data provided must still represent the entire image.
    pub dirty_rect: ImageDirtyRect,
}

/// Creates a blob-image resource with provided parameters.
///
/// Must be matched with a `DeleteImage` at some point to prevent memory leaks.
#[derive(Clone)]
#[cfg_attr(any(feature = "serde"), derive(Deserialize, Serialize))]
pub struct AddBlobImage {
    /// A key to identify the blob-image resource.
    pub key: BlobImageKey,
    /// Properties of the image.
    pub descriptor: ImageDescriptor,
    /// The blob-image's serialized commands.
    pub data: Arc<BlobImageData>,
    /// The portion of the plane in the blob-image's internal coordinate
    /// system that is stretched to fill the image display item.
    ///
    /// Unlike regular images, blob images are not limited in size. The
    /// top-left corner of their internal coordinate system is also not
    /// necessary at (0, 0).
    /// This means that blob images can be updated to insert/remove content
    /// in any direction to support panning and zooming.
    pub visible_rect: DeviceIntRect,
    /// The blob image's tile size to apply when rasterizing the blob-image
    /// and when storing its rasterized data on the GPU.
    /// Applies to both width and heights of the tiles.
    ///
    /// All blob images are tiled.
    pub tile_size: TileSize,
}

/// Updates an already existing blob-image resource.
#[derive(Clone)]
#[cfg_attr(any(feature = "serde"), derive(Deserialize, Serialize))]
pub struct UpdateBlobImage {
    /// The key identfying the blob-image resource to update.
    pub key: BlobImageKey,
    /// Properties of the image.
    pub descriptor: ImageDescriptor,
    /// The blob-image's serialized commands.
    pub data: Arc<BlobImageData>,
    /// See `AddBlobImage::visible_rect`.
    pub visible_rect: DeviceIntRect,
    /// An optional dirty rect that lets WebRender optimize the amount of
    /// data to to rasterize and transfer to the GPU.
    pub dirty_rect: BlobDirtyRect,
}

/// Creates a font resource.
///
/// Must be matched with a corresponding `ResourceUpdate::DeleteFont` at some point to prevent
/// memory leaks.
#[derive(Clone)]
#[cfg_attr(any(feature = "serde"), derive(Deserialize, Serialize))]
pub enum AddFont {
    ///
    Raw(FontKey, Arc<Vec<u8>>, u32),
    ///
    Native(FontKey, NativeFontHandle),
}

/// Creates a font instance resource.
///
/// Must be matched with a corresponding `DeleteFontInstance` at some point
/// to prevent memory leaks.
#[derive(Clone)]
#[cfg_attr(any(feature = "serde"), derive(Deserialize, Serialize))]
pub struct AddFontInstance {
    /// A key to identify the font instance.
    pub key: FontInstanceKey,
    /// The font resource's key.
    pub font_key: FontKey,
    /// Glyph size in app units.
    pub glyph_size: f32,
    ///
    pub options: Option<FontInstanceOptions>,
    ///
    pub platform_options: Option<FontInstancePlatformOptions>,
    ///
    pub variations: Vec<FontVariation>,
}

/// Frame messages affect building the scene.
pub enum SceneMsg {
    ///
    UpdateEpoch(PipelineId, Epoch),
    ///
    SetPageZoom(ZoomFactor),
    ///
    SetRootPipeline(PipelineId),
    ///
    RemovePipeline(PipelineId),
    ///
    SetDisplayList {
        ///
        display_list: BuiltDisplayList,
        ///
        epoch: Epoch,
        ///
        pipeline_id: PipelineId,
        ///
        background: Option<ColorF>,
        ///
        viewport_size: LayoutSize,
        ///
        preserve_frame_state: bool,
    },
    ///
    SetDocumentView {
        ///
        device_rect: DeviceIntRect,
        ///
        device_pixel_ratio: f32,
    },
    /// Set the current quality / performance configuration for this document.
    SetQualitySettings {
        /// The set of available quality / performance config values.
        settings: QualitySettings,
    },
}

/// Frame messages affect frame generation (applied after building the scene).
pub enum FrameMsg {
    ///
    UpdateEpoch(PipelineId, Epoch),
    ///
    HitTest(Option<PipelineId>, WorldPoint, Sender<HitTestResult>),
    ///
    RequestHitTester(Sender<Arc<dyn ApiHitTester>>),
    ///
    SetPan(DeviceIntPoint),
    ///
    ScrollNodeWithId(LayoutPoint, ExternalScrollId, ScrollClamping),
    ///
    GetScrollNodeState(Sender<Vec<ScrollNodeState>>),
    ///
    UpdateDynamicProperties(DynamicProperties),
    ///
    AppendDynamicTransformProperties(Vec<PropertyValue<LayoutTransform>>),
    ///
    SetPinchZoom(ZoomFactor),
    ///
    SetIsTransformAsyncZooming(bool, PropertyBindingId),
}

impl fmt::Debug for SceneMsg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match *self {
            SceneMsg::UpdateEpoch(..) => "SceneMsg::UpdateEpoch",
            SceneMsg::SetDisplayList { .. } => "SceneMsg::SetDisplayList",
            SceneMsg::SetPageZoom(..) => "SceneMsg::SetPageZoom",
            SceneMsg::RemovePipeline(..) => "SceneMsg::RemovePipeline",
            SceneMsg::SetDocumentView { .. } => "SceneMsg::SetDocumentView",
            SceneMsg::SetRootPipeline(..) => "SceneMsg::SetRootPipeline",
            SceneMsg::SetQualitySettings { .. } => "SceneMsg::SetQualitySettings",
        })
    }
}

impl fmt::Debug for FrameMsg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match *self {
            FrameMsg::UpdateEpoch(..) => "FrameMsg::UpdateEpoch",
            FrameMsg::HitTest(..) => "FrameMsg::HitTest",
            FrameMsg::RequestHitTester(..) => "FrameMsg::RequestHitTester",
            FrameMsg::SetPan(..) => "FrameMsg::SetPan",
            FrameMsg::ScrollNodeWithId(..) => "FrameMsg::ScrollNodeWithId",
            FrameMsg::GetScrollNodeState(..) => "FrameMsg::GetScrollNodeState",
            FrameMsg::UpdateDynamicProperties(..) => "FrameMsg::UpdateDynamicProperties",
            FrameMsg::AppendDynamicTransformProperties(..) => "FrameMsg::AppendDynamicTransformProperties",
            FrameMsg::SetPinchZoom(..) => "FrameMsg::SetPinchZoom",
            FrameMsg::SetIsTransformAsyncZooming(..) => "FrameMsg::SetIsTransformAsyncZooming",
        })
    }
}

bitflags!{
    /// Bit flags for WR stages to store in a capture.
    // Note: capturing `FRAME` without `SCENE` is not currently supported.
    pub struct CaptureBits: u8 {
        ///
        const SCENE = 0x1;
        ///
        const FRAME = 0x2;
        ///
        const TILE_CACHE = 0x4;
        ///
        const EXTERNAL_RESOURCES = 0x8;
    }
}

bitflags!{
    /// Mask for clearing caches in debug commands.
    pub struct ClearCache: u8 {
        ///
        const IMAGES = 0b1;
        ///
        const GLYPHS = 0b10;
        ///
        const GLYPH_DIMENSIONS = 0b100;
        ///
        const RENDER_TASKS = 0b1000;
        ///
        const TEXTURE_CACHE = 0b10000;
        /// Clear render target pool
        const RENDER_TARGETS = 0b100000;
    }
}

/// Information about a loaded capture of each document
/// that is returned by `RenderBackend`.
#[derive(Clone, Debug)]
pub struct CapturedDocument {
    ///
    pub document_id: DocumentId,
    ///
    pub root_pipeline_id: Option<PipelineId>,
}

/// Update of the state of built-in debugging facilities.
#[derive(Clone)]
pub enum DebugCommand {
    /// Sets the provided debug flags.
    SetFlags(DebugFlags),
    /// Configure if dual-source blending is used, if available.
    EnableDualSourceBlending(bool),
    /// Save a capture of all the documents state.
    SaveCapture(PathBuf, CaptureBits),
    /// Load a capture of all the documents state.
    LoadCapture(PathBuf, Option<(u32, u32)>, Sender<CapturedDocument>),
    /// Start capturing a sequence of scene/frame changes.
    StartCaptureSequence(PathBuf, CaptureBits),
    /// Stop capturing a sequence of scene/frame changes.
    StopCaptureSequence,
    /// Clear cached resources, forcing them to be re-uploaded from templates.
    ClearCaches(ClearCache),
    /// Enable/disable native compositor usage
    EnableNativeCompositor(bool),
    /// Enable/disable parallel job execution with rayon.
    EnableMultithreading(bool),
    /// Sets the maximum amount of existing batches to visit before creating a new one.
    SetBatchingLookback(u32),
    /// Invalidate GPU cache, forcing the update from the CPU mirror.
    InvalidateGpuCache,
    /// Causes the scene builder to pause for a given amount of milliseconds each time it
    /// processes a transaction.
    SimulateLongSceneBuild(u32),
    /// Set an override tile size to use for picture caches
    SetPictureTileSize(Option<DeviceIntSize>),
}

/// Message sent by the `RenderApi` to the render backend thread.
pub enum ApiMsg {
    /// Adds a new document namespace.
    CloneApi(Sender<IdNamespace>),
    /// Adds a new document namespace.
    CloneApiByClient(IdNamespace),
    /// Adds a new document with given initial size.
    AddDocument(DocumentId, DeviceIntSize),
    /// A message targeted at a particular document.
    UpdateDocuments(Vec<Box<TransactionMsg>>),
    /// Flush from the caches anything that isn't necessary, to free some memory.
    MemoryPressure,
    /// Collects a memory report.
    ReportMemory(Sender<Box<MemoryReport>>),
    /// Change debugging options.
    DebugCommand(DebugCommand),
    /// Message from the scene builder thread.
    SceneBuilderResult(SceneBuilderResult),
}

impl fmt::Debug for ApiMsg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match *self {
            ApiMsg::CloneApi(..) => "ApiMsg::CloneApi",
            ApiMsg::CloneApiByClient(..) => "ApiMsg::CloneApiByClient",
            ApiMsg::AddDocument(..) => "ApiMsg::AddDocument",
            ApiMsg::UpdateDocuments(..) => "ApiMsg::UpdateDocuments",
            ApiMsg::MemoryPressure => "ApiMsg::MemoryPressure",
            ApiMsg::ReportMemory(..) => "ApiMsg::ReportMemory",
            ApiMsg::DebugCommand(..) => "ApiMsg::DebugCommand",
            ApiMsg::SceneBuilderResult(..) => "ApiMsg::SceneBuilderResult",
        })
    }
}

/// Allows the API to communicate with WebRender.
///
/// This object is created along with the `Renderer` and it's main use from a
/// user perspective is to create one or several `RenderApi` objects.
pub struct RenderApiSender {
    api_sender: Sender<ApiMsg>,
    scene_sender: Sender<SceneBuilderRequest>,
    low_priority_scene_sender: Sender<SceneBuilderRequest>,
    blob_image_handler: Option<Box<dyn BlobImageHandler>>,
    shared_font_instances: SharedFontInstanceMap,
}

impl RenderApiSender {
    /// Used internally by the `Renderer`.
    pub fn new(
        api_sender: Sender<ApiMsg>,
        scene_sender: Sender<SceneBuilderRequest>,
        low_priority_scene_sender: Sender<SceneBuilderRequest>,
        blob_image_handler: Option<Box<dyn BlobImageHandler>>,
        shared_font_instances: SharedFontInstanceMap,
    ) -> Self {
        RenderApiSender {
            api_sender,
            scene_sender,
            low_priority_scene_sender,
            blob_image_handler,
            shared_font_instances,
        }
    }

    /// Creates a new resource API object with a dedicated namespace.
    pub fn create_api(&self) -> RenderApi {
        let (sync_tx, sync_rx) = single_msg_channel();
        let msg = ApiMsg::CloneApi(sync_tx);
        self.api_sender.send(msg).expect("Failed to send CloneApi message");
        let namespace_id = sync_rx.recv().expect("Failed to receive CloneApi reply");
        RenderApi {
            api_sender: self.api_sender.clone(),
            scene_sender: self.scene_sender.clone(),
            low_priority_scene_sender: self.low_priority_scene_sender.clone(),
            namespace_id,
            next_id: Cell::new(ResourceId(0)),
            resources: ApiResources::new(
                self.blob_image_handler.as_ref().map(|handler| handler.create_similar()),
                self.shared_font_instances.clone(),
            ),
        }
    }

    /// Creates a new resource API object with a dedicated namespace.
    /// Namespace id is allocated by client.
    ///
    /// The function could be used only when RendererOptions::namespace_alloc_by_client is true.
    /// When the option is true, create_api() could not be used to prevent namespace id conflict.
    pub fn create_api_by_client(&self, namespace_id: IdNamespace) -> RenderApi {
        let msg = ApiMsg::CloneApiByClient(namespace_id);
        self.api_sender.send(msg).expect("Failed to send CloneApiByClient message");
        RenderApi {
            api_sender: self.api_sender.clone(),
            scene_sender: self.scene_sender.clone(),
            low_priority_scene_sender: self.low_priority_scene_sender.clone(),
            namespace_id,
            next_id: Cell::new(ResourceId(0)),
            resources: ApiResources::new(
                self.blob_image_handler.as_ref().map(|handler| handler.create_similar()),
                self.shared_font_instances.clone(),
            ),
        }
    }
}

/// The main entry point to interact with WebRender.
pub struct RenderApi {
    api_sender: Sender<ApiMsg>,
    scene_sender: Sender<SceneBuilderRequest>,
    low_priority_scene_sender: Sender<SceneBuilderRequest>,
    namespace_id: IdNamespace,
    next_id: Cell<ResourceId>,
    resources: ApiResources,
}

impl RenderApi {
    /// Returns the namespace ID used by this API object.
    pub fn get_namespace_id(&self) -> IdNamespace {
        self.namespace_id
    }

    ///
    pub fn create_sender(&self) -> RenderApiSender {
        RenderApiSender::new(
            self.api_sender.clone(),
            self.scene_sender.clone(),
            self.low_priority_scene_sender.clone(),
            self.resources.blob_image_handler.as_ref().map(|handler| handler.create_similar()),
            self.resources.get_shared_font_instances(),
        )
    }

    /// Add a document to the WebRender instance.
    ///
    /// Instances can manage one or several documents (using the same render backend thread).
    /// Each document will internally correspond to a single scene, and scenes are made of
    /// one or several pipelines.
    pub fn add_document(&self, initial_size: DeviceIntSize) -> DocumentId {
        let new_id = self.next_unique_id();
        self.add_document_with_id(initial_size, new_id)
    }

    /// See `add_document`
    pub fn add_document_with_id(&self,
                                initial_size: DeviceIntSize,
                                id: u32) -> DocumentId {
        window_size_sanity_check(initial_size);

        let document_id = DocumentId::new(self.namespace_id, id);

        // We send this message to both the render backend and the scene builder instead of having
        // the scene builder thread forward it to the render backend as we do elswhere. This is because
        // some transactions can skip the scene builder thread and we want to avoid them arriving before
        // the render backend knows about the existence of the corresponding document id.
        // It may not be necessary, though.
        self.api_sender.send(
            ApiMsg::AddDocument(document_id, initial_size)
        ).unwrap();
        self.scene_sender.send(
            SceneBuilderRequest::AddDocument(document_id, initial_size)
        ).unwrap();

        document_id
    }

    /// Delete a document.
    pub fn delete_document(&self, document_id: DocumentId) {
        self.low_priority_scene_sender.send(
            SceneBuilderRequest::DeleteDocument(document_id)
        ).unwrap();
    }

    /// Generate a new font key
    pub fn generate_font_key(&self) -> FontKey {
        let new_id = self.next_unique_id();
        FontKey::new(self.namespace_id, new_id)
    }

    /// Generate a new font instance key
    pub fn generate_font_instance_key(&self) -> FontInstanceKey {
        let new_id = self.next_unique_id();
        FontInstanceKey::new(self.namespace_id, new_id)
    }

    /// Gets the dimensions for the supplied glyph keys
    ///
    /// Note: Internally, the internal texture cache doesn't store
    /// 'empty' textures (height or width = 0)
    /// This means that glyph dimensions e.g. for spaces (' ') will mostly be None.
    pub fn get_glyph_dimensions(
        &self,
        key: FontInstanceKey,
        glyph_indices: Vec<GlyphIndex>,
    ) -> Vec<Option<GlyphDimensions>> {
        let (sender, rx) = single_msg_channel();
        let msg = SceneBuilderRequest::GetGlyphDimensions(GlyphDimensionRequest {
            key,
            glyph_indices,
            sender
        });
        self.low_priority_scene_sender.send(msg).unwrap();
        rx.recv().unwrap()
    }

    /// Gets the glyph indices for the supplied string. These
    /// can be used to construct GlyphKeys.
    pub fn get_glyph_indices(&self, key: FontKey, text: &str) -> Vec<Option<u32>> {
        let (sender, rx) = single_msg_channel();
        let msg = SceneBuilderRequest::GetGlyphIndices(GlyphIndexRequest {
            key,
            text: text.to_string(),
            sender,
        });
        self.low_priority_scene_sender.send(msg).unwrap();
        rx.recv().unwrap()
    }

    /// Creates an `ImageKey`.
    pub fn generate_image_key(&self) -> ImageKey {
        let new_id = self.next_unique_id();
        ImageKey::new(self.namespace_id, new_id)
    }

    /// Creates a `BlobImageKey`.
    pub fn generate_blob_image_key(&self) -> BlobImageKey {
        BlobImageKey(self.generate_image_key())
    }

    /// A Gecko-specific notification mechanism to get some code executed on the
    /// `Renderer`'s thread, mostly replaced by `NotificationHandler`. You should
    /// probably use the latter instead.
    pub fn send_external_event(&self, evt: ExternalEvent) {
        let msg = SceneBuilderRequest::ExternalEvent(evt);
        self.low_priority_scene_sender.send(msg).unwrap();
    }

    /// Notify WebRender that now is a good time to flush caches and release
    /// as much memory as possible.
    pub fn notify_memory_pressure(&self) {
        self.api_sender.send(ApiMsg::MemoryPressure).unwrap();
    }

    /// Synchronously requests memory report.
    pub fn report_memory(&self, _ops: malloc_size_of::MallocSizeOfOps) -> MemoryReport {
        let (tx, rx) = single_msg_channel();
        self.api_sender.send(ApiMsg::ReportMemory(tx)).unwrap();
        *rx.recv().unwrap()
    }

    /// Update debugging flags.
    pub fn set_debug_flags(&self, flags: DebugFlags) {
        let cmd = DebugCommand::SetFlags(flags);
        self.api_sender.send(ApiMsg::DebugCommand(cmd)).unwrap();
    }

    /// Stop RenderBackend's task until shut down
    pub fn stop_render_backend(&self) {
        self.low_priority_scene_sender.send(SceneBuilderRequest::StopRenderBackend).unwrap();
    }

    /// Shut the WebRender instance down.
    pub fn shut_down(&self, synchronously: bool) {
        if synchronously {
            let (tx, rx) = single_msg_channel();
            self.low_priority_scene_sender.send(SceneBuilderRequest::ShutDown(Some(tx))).unwrap();
            rx.recv().unwrap();
        } else {
            self.low_priority_scene_sender.send(SceneBuilderRequest::ShutDown(None)).unwrap();
        }
    }

    /// Create a new unique key that can be used for
    /// animated property bindings.
    pub fn generate_property_binding_key<T: Copy>(&self) -> PropertyBindingKey<T> {
        let new_id = self.next_unique_id();
        PropertyBindingKey {
            id: PropertyBindingId {
                namespace: self.namespace_id,
                uid: new_id,
            },
            _phantom: PhantomData,
        }
    }

    #[inline]
    fn next_unique_id(&self) -> u32 {
        let ResourceId(id) = self.next_id.get();
        self.next_id.set(ResourceId(id + 1));
        id
    }

    // For use in Wrench only
    #[doc(hidden)]
    pub fn send_message(&self, msg: ApiMsg) {
        self.api_sender.send(msg).unwrap();
    }

    /// Creates a transaction message from a single frame message.
    fn frame_message(&self, msg: FrameMsg, document_id: DocumentId) -> Box<TransactionMsg> {
        Box::new(TransactionMsg {
            document_id,
            scene_ops: Vec::new(),
            frame_ops: vec![msg],
            resource_updates: Vec::new(),
            notifications: Vec::new(),
            generate_frame: GenerateFrame::No,
            invalidate_rendered_frame: false,
            use_scene_builder_thread: false,
            low_priority: false,
            blob_rasterizer: None,
            blob_requests: Vec::new(),
            rasterized_blobs: Vec::new(),
            profile: TransactionProfile::new(),
        })
    }

    /// A helper method to send document messages.
    fn send_frame_msg(&self, document_id: DocumentId, msg: FrameMsg) {
        // This assertion fails on Servo use-cases, because it creates different
        // `RenderApi` instances for layout and compositor.
        //assert_eq!(document_id.0, self.namespace_id);
        self.api_sender
            .send(ApiMsg::UpdateDocuments(vec![self.frame_message(msg, document_id)]))
            .unwrap()
    }

    /// Send a transaction to WebRender.
    pub fn send_transaction(&mut self, document_id: DocumentId, transaction: Transaction) {
        let mut transaction = transaction.finalize(document_id);

        self.resources.update(&mut transaction);

        if transaction.generate_frame.as_bool() {
            transaction.profile.start_time(profiler::API_SEND_TIME);
            transaction.profile.start_time(profiler::TOTAL_FRAME_CPU_TIME);
        }

        if transaction.use_scene_builder_thread {
            let sender = if transaction.low_priority {
                &mut self.low_priority_scene_sender
            } else {
                &mut self.scene_sender
            };

            sender.send(SceneBuilderRequest::Transactions(vec![transaction])).unwrap();
        } else {
            self.api_sender.send(ApiMsg::UpdateDocuments(vec![transaction])).unwrap();
        }
    }

    /// Does a hit test on display items in the specified document, at the given
    /// point. If a pipeline_id is specified, it is used to further restrict the
    /// hit results so that only items inside that pipeline are matched. The vector
    /// of hit results will contain all display items that match, ordered from
    /// front to back.
    pub fn hit_test(&self,
        document_id: DocumentId,
        pipeline_id: Option<PipelineId>,
        point: WorldPoint,
    ) -> HitTestResult {
        let (tx, rx) = single_msg_channel();

        self.send_frame_msg(
            document_id,
            FrameMsg::HitTest(pipeline_id, point, tx)
        );
        rx.recv().unwrap()
    }

    /// Synchronously request an object that can perform fast hit testing queries.
    pub fn request_hit_tester(&self, document_id: DocumentId) -> HitTesterRequest {
        let (tx, rx) = single_msg_channel();
        self.send_frame_msg(
            document_id,
            FrameMsg::RequestHitTester(tx)
        );

        HitTesterRequest { rx }
    }

    ///
    pub fn get_scroll_node_state(&self, document_id: DocumentId) -> Vec<ScrollNodeState> {
        let (tx, rx) = single_msg_channel();
        self.send_frame_msg(document_id, FrameMsg::GetScrollNodeState(tx));
        rx.recv().unwrap()
    }

    // Some internal scheduling magic that leaked into the API.
    // Buckle up and see APZUpdater.cpp for more info about what this is about.
    #[doc(hidden)]
    pub fn wake_scene_builder(&self) {
        self.scene_sender.send(SceneBuilderRequest::WakeUp).unwrap();
    }

    /// Block until a round-trip to the scene builder thread has completed. This
    /// ensures that any transactions (including ones deferred to the scene
    /// builder thread) have been processed.
    pub fn flush_scene_builder(&self) {
        let (tx, rx) = single_msg_channel();
        self.low_priority_scene_sender.send(SceneBuilderRequest::Flush(tx)).unwrap();
        rx.recv().unwrap(); // Block until done.
    }

    /// Save a capture of the current frame state for debugging.
    pub fn save_capture(&self, path: PathBuf, bits: CaptureBits) {
        let msg = ApiMsg::DebugCommand(DebugCommand::SaveCapture(path, bits));
        self.send_message(msg);
    }

    /// Load a capture of the current frame state for debugging.
    pub fn load_capture(&self, path: PathBuf, ids: Option<(u32, u32)>) -> Vec<CapturedDocument> {
        // First flush the scene builder otherwise async scenes might clobber
        // the capture we are about to load.
        self.flush_scene_builder();

        let (tx, rx) = unbounded_channel();
        let msg = ApiMsg::DebugCommand(DebugCommand::LoadCapture(path, ids, tx));
        self.send_message(msg);

        let mut documents = Vec::new();
        while let Ok(captured_doc) = rx.recv() {
            documents.push(captured_doc);
        }
        documents
    }

    /// Start capturing a sequence of frames.
    pub fn start_capture_sequence(&self, path: PathBuf, bits: CaptureBits) {
        let msg = ApiMsg::DebugCommand(DebugCommand::StartCaptureSequence(path, bits));
        self.send_message(msg);
    }

    /// Stop capturing sequences of frames.
    pub fn stop_capture_sequence(&self) {
        let msg = ApiMsg::DebugCommand(DebugCommand::StopCaptureSequence);
        self.send_message(msg);
    }

    /// Update the state of builtin debugging facilities.
    pub fn send_debug_cmd(&mut self, cmd: DebugCommand) {
        if let DebugCommand::EnableMultithreading(enable) = cmd {
            // TODO(nical) we should enable it for all RenderApis.
            self.resources.enable_multithreading(enable);
        }
        let msg = ApiMsg::DebugCommand(cmd);
        self.send_message(msg);
    }
}

impl Drop for RenderApi {
    fn drop(&mut self) {
        let msg = SceneBuilderRequest::ClearNamespace(self.namespace_id);
        let _ = self.low_priority_scene_sender.send(msg);
    }
}


fn window_size_sanity_check(size: DeviceIntSize) {
    // Anything bigger than this will crash later when attempting to create
    // a render task.
    use crate::render_task::MAX_RENDER_TASK_SIZE;
    if size.width > MAX_RENDER_TASK_SIZE || size.height > MAX_RENDER_TASK_SIZE {
        panic!("Attempting to create a {}x{} window/document", size.width, size.height);
    }
}

/// Collection of heap sizes, in bytes.
/// cbindgen:derive-eq=false
/// cbindgen:derive-ostream=false
#[repr(C)]
#[allow(missing_docs)]
#[derive(AddAssign, Clone, Debug, Default)]
pub struct MemoryReport {
    //
    // CPU Memory.
    //
    pub clip_stores: usize,
    pub gpu_cache_metadata: usize,
    pub gpu_cache_cpu_mirror: usize,
    pub render_tasks: usize,
    pub hit_testers: usize,
    pub fonts: usize,
    pub weak_fonts: usize,
    pub images: usize,
    pub rasterized_blobs: usize,
    pub shader_cache: usize,
    pub interning: InterningMemoryReport,
    pub display_list: usize,
    pub upload_staging_memory: usize,
    pub swgl: usize,

    //
    // GPU memory.
    //
    pub gpu_cache_textures: usize,
    pub vertex_data_textures: usize,
    pub render_target_textures: usize,
    pub texture_cache_textures: usize,
    pub texture_cache_structures: usize,
    pub depth_target_textures: usize,
    pub texture_upload_pbos: usize,
    pub swap_chain: usize,
    pub render_texture_hosts: usize,
    pub upload_staging_textures: usize,
}
