/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![deny(missing_docs)]

extern crate serde_bytes;

use peek_poke::PeekPoke;
use std::cell::Cell;
use std::fmt;
use std::marker::PhantomData;
use std::os::raw::c_void;
use std::path::PathBuf;
use std::sync::Arc;
use std::u32;
use std::sync::mpsc::{Sender, Receiver, channel};
use time::precise_time_ns;
// local imports
use crate::{display_item as di, font};
use crate::color::{ColorU, ColorF};
use crate::display_list::BuiltDisplayList;
use crate::font::SharedFontInstanceMap;
use crate::image::{BlobImageData, BlobImageKey, ImageData, ImageDescriptor, ImageKey};
use crate::image::{BlobImageParams, BlobImageRequest, BlobImageResult, AsyncBlobImageRasterizer, BlobImageHandler};
use crate::image::DEFAULT_TILE_SIZE;
use crate::resources::ApiResources;
use crate::units::*;

/// Width and height in device pixels of image tiles.
pub type TileSize = u16;

/// Documents are rendered in the ascending order of their associated layer values.
pub type DocumentLayer = i8;

/// Various settings that the caller can select based on desired tradeoffs
/// between rendering quality and performance / power usage.
#[derive(Copy, Clone, Deserialize, Serialize)]
pub struct QualitySettings {
    /// If true, disable creating separate picture cache slices when the
    /// scroll root changes. This gives maximum opportunity to find an
    /// opaque background, which enables subpixel AA. However, it is
    /// usually significantly more expensive to render when scrolling.
    pub force_subpixel_aa_where_possible: bool,
}

impl Default for QualitySettings {
    fn default() -> Self {
        QualitySettings {
            // Prefer performance over maximum subpixel AA quality, since WR
            // already enables subpixel AA in more situations than other browsers.
            force_subpixel_aa_where_possible: false,
        }
    }
}

/// Update of a persistent resource in WebRender.
///
/// ResourceUpdate changes keep theirs effect across display list changes.
#[derive(Clone, Deserialize, Serialize)]
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
    DeleteFont(font::FontKey),
    /// See `AddFontInstance`.
    AddFontInstance(AddFontInstance),
    /// Deletes an already existing font instance resource.
    ///
    /// It is invalid to continue referring to the font instance in any display
    /// list in the transaction that contains the `DeleteImage` message and
    /// subsequent transactions.
    DeleteFontInstance(font::FontInstanceKey),
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

    /// If true the transaction is piped through the scene building thread, if false
    /// it will be applied directly on the render backend.
    use_scene_builder_thread: bool,

    generate_frame: bool,

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
            generate_frame: false,
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
        !self.generate_frame &&
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
    /// # use webrender_api::{PipelineId, RenderApiSender, Transaction};
    /// # use webrender_api::units::{DeviceIntSize};
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
    /// * `content_size`: The total screen space size of this display list's display items.
    /// * `display_list`: The root Display list used in this frame.
    /// * `preserve_frame_state`: If a previous frame exists which matches this pipeline
    ///                           id, this setting determines if frame state (such as scrolling
    ///                           position) should be preserved for this new display list.
    pub fn set_display_list(
        &mut self,
        epoch: Epoch,
        background: Option<ColorF>,
        viewport_size: LayoutSize,
        (pipeline_id, content_size, mut display_list): (PipelineId, LayoutSize, BuiltDisplayList),
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
                content_size,
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
        self.scene_ops.push(
            SceneMsg::SetDocumentView {
                device_rect,
                device_pixel_ratio,
            },
        );
    }

    /// Enable copying of the output of this pipeline id to
    /// an external texture for callers to consume.
    pub fn enable_frame_output(&mut self, pipeline_id: PipelineId, enable: bool) {
        self.scene_ops.push(SceneMsg::EnableFrameOutput(pipeline_id, enable));
    }

    /// Scrolls the scrolling layer under the `cursor`
    ///
    /// WebRender looks for the layer closest to the user
    /// which has `ScrollPolicy::Scrollable` set.
    pub fn scroll(&mut self, scroll_location: ScrollLocation, cursor: WorldPoint) {
        self.frame_ops.push(FrameMsg::Scroll(scroll_location, cursor));
    }

    ///
    pub fn scroll_node_with_id(
        &mut self,
        origin: LayoutPoint,
        id: di::ExternalScrollId,
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
    pub fn generate_frame(&mut self) {
        self.generate_frame = true;
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
        self.resource_updates.push(ResourceUpdate::SetBlobImageVisibleArea(key, area))
    }

    /// See `ResourceUpdate::AddFont`.
    pub fn add_raw_font(&mut self, key: font::FontKey, bytes: Vec<u8>, index: u32) {
        self.resource_updates
            .push(ResourceUpdate::AddFont(AddFont::Raw(key, Arc::new(bytes), index)));
    }

    /// See `ResourceUpdate::AddFont`.
    pub fn add_native_font(&mut self, key: font::FontKey, native_handle: font::NativeFontHandle) {
        self.resource_updates
            .push(ResourceUpdate::AddFont(AddFont::Native(key, native_handle)));
    }

    /// See `ResourceUpdate::DeleteFont`.
    pub fn delete_font(&mut self, key: font::FontKey) {
        self.resource_updates.push(ResourceUpdate::DeleteFont(key));
    }

    /// See `ResourceUpdate::AddFontInstance`.
    pub fn add_font_instance(
        &mut self,
        key: font::FontInstanceKey,
        font_key: font::FontKey,
        glyph_size: f32,
        options: Option<font::FontInstanceOptions>,
        platform_options: Option<font::FontInstancePlatformOptions>,
        variations: Vec<font::FontVariation>,
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
    pub fn delete_font_instance(&mut self, key: font::FontInstanceKey) {
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
    pub generate_frame: bool,
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
}

impl fmt::Debug for TransactionMsg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "threaded={}, genframe={}, invalidate={}, low_priority={}",
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
        !self.generate_frame &&
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
#[derive(Clone, Deserialize, Serialize)]
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
#[derive(Clone, Deserialize, Serialize)]
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
#[derive(Clone, Deserialize, Serialize)]
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
#[derive(Clone, Deserialize, Serialize)]
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
#[derive(Clone, Deserialize, Serialize)]
pub enum AddFont {
    ///
    Raw(font::FontKey, Arc<Vec<u8>>, u32),
    ///
    Native(font::FontKey, font::NativeFontHandle),
}

/// Describe an item that matched a hit-test query.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct HitTestItem {
    /// The pipeline that the display item that was hit belongs to.
    pub pipeline: PipelineId,

    /// The tag of the hit display item.
    pub tag: di::ItemTag,

    /// The hit point in the coordinate space of the "viewport" of the display item. The
    /// viewport is the scroll node formed by the root reference frame of the display item's
    /// pipeline.
    pub point_in_viewport: LayoutPoint,

    /// The coordinates of the original hit test point relative to the origin of this item.
    /// This is useful for calculating things like text offsets in the client.
    pub point_relative_to_item: LayoutPoint,
}

/// Returned by `RenderApi::hit_test`.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct HitTestResult {
    /// List of items that are match the hit-test query.
    pub items: Vec<HitTestItem>,
}

bitflags! {
    #[derive(Deserialize, MallocSizeOf, Serialize)]
    ///
    pub struct HitTestFlags: u8 {
        ///
        const FIND_ALL = 0b00000001;
        ///
        const POINT_RELATIVE_TO_PIPELINE_VIEWPORT = 0b00000010;
    }
}

/// Creates a font instance resource.
///
/// Must be matched with a corresponding `DeleteFontInstance` at some point
/// to prevent memory leaks.
#[derive(Clone, Deserialize, Serialize)]
pub struct AddFontInstance {
    /// A key to identify the font instance.
    pub key: font::FontInstanceKey,
    /// The font resource's key.
    pub font_key: font::FontKey,
    /// Glyph size in app units.
    pub glyph_size: f32,
    ///
    pub options: Option<font::FontInstanceOptions>,
    ///
    pub platform_options: Option<font::FontInstancePlatformOptions>,
    ///
    pub variations: Vec<font::FontVariation>,
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
    EnableFrameOutput(PipelineId, bool),
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
        content_size: LayoutSize,
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
    HitTest(Option<PipelineId>, WorldPoint, HitTestFlags, Sender<HitTestResult>),
    ///
    RequestHitTester(Sender<Arc<dyn ApiHitTester>>),
    ///
    SetPan(DeviceIntPoint),
    ///
    Scroll(ScrollLocation, WorldPoint),
    ///
    ScrollNodeWithId(LayoutPoint, di::ExternalScrollId, ScrollClamping),
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
            SceneMsg::EnableFrameOutput(..) => "SceneMsg::EnableFrameOutput",
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
            FrameMsg::Scroll(..) => "FrameMsg::Scroll",
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
        const GLYPHS = 0b01;
        ///
        const GLYPH_DIMENSIONS = 0b001;
        ///
        const RENDER_TASKS = 0b0001;
        ///
        const TEXTURE_CACHE = 0b00001;
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
    /// Fetch current documents and display lists.
    FetchDocuments,
    /// Fetch current passes and batches.
    FetchPasses,
    // TODO: This should be called FetchClipScrollTree. However, that requires making
    // changes to webrender's web debugger ui, touching a 4Mb minified file that
    // is too big to submit through the conventional means.
    /// Fetch the spatial tree.
    FetchClipScrollTree,
    /// Fetch render tasks.
    FetchRenderTasks,
    /// Fetch screenshot.
    FetchScreenshot,
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
    /// Causes the low priority scene builder to pause for a given amount of milliseconds
    /// each time it processes a transaction.
    SimulateLongLowPrioritySceneBuild(u32),
    /// Set an override tile size to use for picture caches
    SetPictureTileSize(Option<DeviceIntSize>),
}

/// Message sent by the `RenderApi` to the render backend thread.
pub enum ApiMsg {
    /// Gets the glyph dimensions
    GetGlyphDimensions(font::GlyphDimensionRequest),
    /// Gets the glyph indices from a string
    GetGlyphIndices(font::GlyphIndexRequest),
    /// Adds a new document namespace.
    CloneApi(Sender<IdNamespace>),
    /// Adds a new document namespace.
    CloneApiByClient(IdNamespace),
    /// Adds a new document with given initial size.
    AddDocument(DocumentId, DeviceIntSize, DocumentLayer),
    /// A message targeted at a particular document.
    UpdateDocuments(Vec<Box<TransactionMsg>>),
    /// Deletes an existing document.
    DeleteDocument(DocumentId),
    /// An opaque handle that must be passed to the render notifier. It is used by Gecko
    /// to forward gecko-specific messages to the render thread preserving the ordering
    /// within the other messages.
    ExternalEvent(ExternalEvent),
    /// Removes all resources associated with a namespace.
    ClearNamespace(IdNamespace),
    /// Flush from the caches anything that isn't necessary, to free some memory.
    MemoryPressure,
    /// Collects a memory report.
    ReportMemory(Sender<Box<MemoryReport>>),
    /// Change debugging options.
    DebugCommand(DebugCommand),
    /// Wakes the render backend's event loop up. Needed when an event is communicated
    /// through another channel.
    WakeUp,
    /// See `RenderApi::wake_scene_builder`.
    WakeSceneBuilder,
    /// Block until a round-trip to the scene builder thread has completed. This
    /// ensures that any transactions (including ones deferred to the scene
    /// builder thread) have been processed.
    FlushSceneBuilder(Sender<()>),
    /// Shut the WebRender instance down.
    ShutDown(Option<Sender<()>>),
}

impl fmt::Debug for ApiMsg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match *self {
            ApiMsg::GetGlyphDimensions(..) => "ApiMsg::GetGlyphDimensions",
            ApiMsg::GetGlyphIndices(..) => "ApiMsg::GetGlyphIndices",
            ApiMsg::CloneApi(..) => "ApiMsg::CloneApi",
            ApiMsg::CloneApiByClient(..) => "ApiMsg::CloneApiByClient",
            ApiMsg::AddDocument(..) => "ApiMsg::AddDocument",
            ApiMsg::UpdateDocuments(..) => "ApiMsg::UpdateDocuments",
            ApiMsg::DeleteDocument(..) => "ApiMsg::DeleteDocument",
            ApiMsg::ExternalEvent(..) => "ApiMsg::ExternalEvent",
            ApiMsg::ClearNamespace(..) => "ApiMsg::ClearNamespace",
            ApiMsg::MemoryPressure => "ApiMsg::MemoryPressure",
            ApiMsg::ReportMemory(..) => "ApiMsg::ReportMemory",
            ApiMsg::DebugCommand(..) => "ApiMsg::DebugCommand",
            ApiMsg::ShutDown(..) => "ApiMsg::ShutDown",
            ApiMsg::WakeUp => "ApiMsg::WakeUp",
            ApiMsg::WakeSceneBuilder => "ApiMsg::WakeSceneBuilder",
            ApiMsg::FlushSceneBuilder(..) => "ApiMsg::FlushSceneBuilder",
        })
    }
}

/// An epoch identifies the state of a pipeline in time.
///
/// This is mostly used as a synchronization mechanism to observe how/when particular pipeline
/// updates propagate through WebRender and are applied at various stages.
#[repr(C)]
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Epoch(pub u32);

impl Epoch {
    /// Magic invalid epoch value.
    pub fn invalid() -> Epoch {
        Epoch(u32::MAX)
    }
}

/// ID namespaces uniquely identify different users of WebRender's API.
///
/// For example in Gecko each content process uses a separate id namespace.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, MallocSizeOf, PartialEq, Hash, Ord, PartialOrd, PeekPoke)]
#[derive(Deserialize, Serialize)]
pub struct IdNamespace(pub u32);

/// A key uniquely identifying a WebRender document.
///
/// Instances can manage one or several documents (using the same render backend thread).
/// Each document will internally correspond to a single scene, and scenes are made of
/// one or several pipelines.
#[repr(C)]
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, MallocSizeOf, PartialEq, Serialize, PeekPoke)]
pub struct DocumentId {
    ///
    pub namespace_id: IdNamespace,
    ///
    pub id: u32,
}

impl DocumentId {
    ///
    pub fn new(namespace_id: IdNamespace, id: u32) -> Self {
        DocumentId {
            namespace_id,
            id,
        }
    }

    ///
    pub const INVALID: DocumentId = DocumentId { namespace_id: IdNamespace(0), id: 0 };
}

/// This type carries no valuable semantics for WR. However, it reflects the fact that
/// clients (Servo) may generate pipelines by different semi-independent sources.
/// These pipelines still belong to the same `IdNamespace` and the same `DocumentId`.
/// Having this extra Id field enables them to generate `PipelineId` without collision.
pub type PipelineSourceId = u32;

/// From the point of view of WR, `PipelineId` is completely opaque and generic as long as
/// it's clonable, serializable, comparable, and hashable.
#[repr(C)]
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, MallocSizeOf, PartialEq, Serialize, PeekPoke)]
pub struct PipelineId(pub PipelineSourceId, pub u32);

impl Default for PipelineId {
    fn default() -> Self {
        PipelineId::dummy()
    }
}

impl PipelineId {
    ///
    pub fn dummy() -> Self {
        PipelineId(0, 0)
    }
}

///
#[derive(Copy, Clone, Debug, MallocSizeOf, Serialize, Deserialize)]
pub enum ClipIntern {}

///
#[derive(Copy, Clone, Debug, MallocSizeOf, Serialize, Deserialize)]
pub enum FilterDataIntern {}

/// Information specific to a primitive type that
/// uniquely identifies a primitive template by key.
#[derive(Debug, Clone, Eq, MallocSizeOf, PartialEq, Hash, Serialize, Deserialize)]
pub enum PrimitiveKeyKind {
    /// Clear an existing rect, used for special effects on some platforms.
    Clear,
    ///
    Rectangle {
        ///
        color: PropertyBinding<ColorU>,
    },
}

/// Meta-macro to enumerate the various interner identifiers and types.
///
/// IMPORTANT: Keep this synchronized with the list in mozilla-central located at
/// gfx/webrender_bindings/webrender_ffi.h
///
/// Note that this could be a lot less verbose if concat_idents! were stable. :-(
#[macro_export]
macro_rules! enumerate_interners {
    ($macro_name: ident) => {
        $macro_name! {
            clip: ClipIntern,
            prim: PrimitiveKeyKind,
            normal_border: NormalBorderPrim,
            image_border: ImageBorder,
            image: Image,
            yuv_image: YuvImage,
            line_decoration: LineDecoration,
            linear_grad: LinearGradient,
            radial_grad: RadialGradient,
            conic_grad: ConicGradient,
            picture: Picture,
            text_run: TextRun,
            filter_data: FilterDataIntern,
            backdrop: Backdrop,
        }
    }
}

macro_rules! declare_interning_memory_report {
    ( $( $name:ident: $ty:ident, )+ ) => {
        ///
        #[repr(C)]
        #[derive(AddAssign, Clone, Debug, Default)]
        pub struct InternerSubReport {
            $(
                ///
                pub $name: usize,
            )+
        }
    }
}

enumerate_interners!(declare_interning_memory_report);

/// Memory report for interning-related data structures.
/// cbindgen:derive-eq=false
#[repr(C)]
#[derive(Clone, Debug, Default)]
pub struct InterningMemoryReport {
    ///
    pub interners: InternerSubReport,
    ///
    pub data_stores: InternerSubReport,
}

impl ::std::ops::AddAssign for InterningMemoryReport {
    fn add_assign(&mut self, other: InterningMemoryReport) {
        self.interners += other.interners;
        self.data_stores += other.data_stores;
    }
}

/// Collection of heap sizes, in bytes.
/// cbindgen:derive-eq=false
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
    pub images: usize,
    pub rasterized_blobs: usize,
    pub shader_cache: usize,
    pub interning: InterningMemoryReport,
    pub display_list: usize,

    //
    // GPU memory.
    //
    pub gpu_cache_textures: usize,
    pub vertex_data_textures: usize,
    pub render_target_textures: usize,
    pub texture_cache_textures: usize,
    pub depth_target_textures: usize,
    pub swap_chain: usize,
}

/// A C function that takes a pointer to a heap allocation and returns its size.
///
/// This is borrowed from the malloc_size_of crate, upon which we want to avoid
/// a dependency from WebRender.
pub type VoidPtrToSizeFn = unsafe extern "C" fn(ptr: *const c_void) -> usize;

#[repr(C)]
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
struct ResourceId(pub u32);

/// An opaque pointer-sized value.
#[repr(C)]
#[derive(Clone)]
pub struct ExternalEvent {
    raw: usize,
}

unsafe impl Send for ExternalEvent {}

impl ExternalEvent {
    /// Creates the event from an opaque pointer-sized value.
    pub fn from_raw(raw: usize) -> Self {
        ExternalEvent { raw }
    }
    /// Consumes self to make it obvious that the event should be forwarded only once.
    pub fn unwrap(self) -> usize {
        self.raw
    }
}

/// Describe whether or not scrolling should be clamped by the content bounds.
#[derive(Clone, Deserialize, Serialize)]
pub enum ScrollClamping {
    ///
    ToContentBounds,
    ///
    NoClamping,
}

/// Allows the API to communicate with WebRender.
///
/// This object is created along with the `Renderer` and it's main use from a
/// user perspective is to create one or several `RenderApi` objects.
pub struct RenderApiSender {
    api_sender: Sender<ApiMsg>,
    blob_image_handler: Option<Box<dyn BlobImageHandler>>,
    shared_font_instances: SharedFontInstanceMap,
}

impl RenderApiSender {
    /// Used internally by the `Renderer`.
    pub fn new(
        api_sender: Sender<ApiMsg>,
        blob_image_handler: Option<Box<dyn BlobImageHandler>>,
        shared_font_instances: SharedFontInstanceMap,
    ) -> Self {
        RenderApiSender {
            api_sender,
            blob_image_handler,
            shared_font_instances,
        }
    }

    /// Creates a new resource API object with a dedicated namespace.
    pub fn create_api(&self) -> RenderApi {
        let (sync_tx, sync_rx) = channel();
        let msg = ApiMsg::CloneApi(sync_tx);
        self.api_sender.send(msg).expect("Failed to send CloneApi message");
        let namespace_id = match sync_rx.recv() {
            Ok(id) => id,
            Err(e) => {
                // This is used to discover the underlying cause of https://github.com/servo/servo/issues/13480.
                let webrender_is_alive = self.api_sender.send(ApiMsg::WakeUp);
                if webrender_is_alive.is_err() {
                    panic!("WebRender was shut down before processing CloneApi: {}", e);
                } else {
                    panic!("CloneApi message response was dropped while WebRender was still alive: {}", e);
                }
            }
        };
        RenderApi {
            api_sender: self.api_sender.clone(),
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
            namespace_id,
            next_id: Cell::new(ResourceId(0)),
            resources: ApiResources::new(
                self.blob_image_handler.as_ref().map(|handler| handler.create_similar()),
                self.shared_font_instances.clone(),
            ),
        }
    }
}

bitflags! {
    /// Flags to enable/disable various builtin debugging tools.
    #[repr(C)]
    #[derive(Default, Deserialize, MallocSizeOf, Serialize)]
    pub struct DebugFlags: u32 {
        /// Display the frame profiler on screen.
        const PROFILER_DBG          = 1 << 0;
        /// Display intermediate render targets on screen.
        const RENDER_TARGET_DBG     = 1 << 1;
        /// Display all texture cache pages on screen.
        const TEXTURE_CACHE_DBG     = 1 << 2;
        /// Display GPU timing results.
        const GPU_TIME_QUERIES      = 1 << 3;
        /// Query the number of pixels that pass the depth test divided and show it
        /// in the profiler as a percentage of the number of pixels in the screen
        /// (window width times height).
        const GPU_SAMPLE_QUERIES    = 1 << 4;
        /// Render each quad with their own draw call.
        ///
        /// Terrible for performance but can help with understanding the drawing
        /// order when inspecting renderdoc or apitrace recordings.
        const DISABLE_BATCHING      = 1 << 5;
        /// Display the pipeline epochs.
        const EPOCHS                = 1 << 6;
        /// Reduce the amount of information displayed by the profiler so that
        /// it occupies less screen real-estate.
        const COMPACT_PROFILER      = 1 << 7;
        /// Print driver messages to stdout.
        const ECHO_DRIVER_MESSAGES  = 1 << 8;
        /// Show an indicator that moves every time a frame is rendered.
        const NEW_FRAME_INDICATOR   = 1 << 9;
        /// Show an indicator that moves every time a scene is built.
        const NEW_SCENE_INDICATOR   = 1 << 10;
        /// Show an overlay displaying overdraw amount.
        const SHOW_OVERDRAW         = 1 << 11;
        /// Display the contents of GPU cache.
        const GPU_CACHE_DBG         = 1 << 12;
        /// Show a red bar that moves each time a slow frame is detected.
        const SLOW_FRAME_INDICATOR  = 1 << 13;
        /// Clear evicted parts of the texture cache for debugging purposes.
        const TEXTURE_CACHE_DBG_CLEAR_EVICTED = 1 << 14;
        /// Show picture caching debug overlay
        const PICTURE_CACHING_DBG   = 1 << 15;
        /// Highlight all primitives with colors based on kind.
        const PRIMITIVE_DBG = 1 << 16;
        /// Draw a zoom widget showing part of the framebuffer zoomed in.
        const ZOOM_DBG = 1 << 17;
        /// Scale the debug renderer down for a smaller screen. This will disrupt
        /// any mapping between debug display items and page content, so shouldn't
        /// be used with overlays like the picture caching or primitive display.
        const SMALL_SCREEN = 1 << 18;
        /// Disable various bits of the WebRender pipeline, to help narrow
        /// down where slowness might be coming from.
        const DISABLE_OPAQUE_PASS = 1 << 19;
        ///
        const DISABLE_ALPHA_PASS = 1 << 20;
        ///
        const DISABLE_CLIP_MASKS = 1 << 21;
        ///
        const DISABLE_TEXT_PRIMS = 1 << 22;
        ///
        const DISABLE_GRADIENT_PRIMS = 1 << 23;
        ///
        const OBSCURE_IMAGES = 1 << 24;
        /// Taint the transparent area of the glyphs with a random opacity to easily
        /// see when glyphs are re-rasterized.
        const GLYPH_FLASHING = 1 << 25;
        /// The profiler only displays information that is out of the ordinary.
        const SMART_PROFILER        = 1 << 26;
        /// Dynamically control whether picture caching is enabled.
        const DISABLE_PICTURE_CACHING = 1 << 27;
        /// If set, dump picture cache invalidation debug to console.
        const INVALIDATION_DBG = 1 << 28;
        /// Log tile cache to memory for later saving as part of wr-capture
        const TILE_CACHE_LOGGING_DBG   = 1 << 29;
        /// For debugging, force-disable automatic scaling of establishes_raster_root
        /// pictures that are too large (ie go back to old behavior that prevents those
        /// large pictures from establishing a raster root).
        const DISABLE_RASTER_ROOT_SCALING = 1 << 30;
    }
}

/// The main entry point to interact with WebRender.
pub struct RenderApi {
    api_sender: Sender<ApiMsg>,
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
            self.resources.blob_image_handler.as_ref().map(|handler| handler.create_similar()),
            self.resources.get_shared_font_instances(),
        )
    }

    /// Add a document to the WebRender instance.
    ///
    /// Instances can manage one or several documents (using the same render backend thread).
    /// Each document will internally correspond to a single scene, and scenes are made of
    /// one or several pipelines.
    pub fn add_document(&self, initial_size: DeviceIntSize, layer: DocumentLayer) -> DocumentId {
        let new_id = self.next_unique_id();
        self.add_document_with_id(initial_size, layer, new_id)
    }

    /// See `add_document`
    pub fn add_document_with_id(&self,
                                initial_size: DeviceIntSize,
                                layer: DocumentLayer,
                                id: u32) -> DocumentId {
        let document_id = DocumentId::new(self.namespace_id, id);

        let msg = ApiMsg::AddDocument(document_id, initial_size, layer);
        self.api_sender.send(msg).unwrap();

        document_id
    }

    /// Delete a document.
    pub fn delete_document(&self, document_id: DocumentId) {
        let msg = ApiMsg::DeleteDocument(document_id);
        self.api_sender.send(msg).unwrap();
    }

    /// Generate a new font key
    pub fn generate_font_key(&self) -> font::FontKey {
        let new_id = self.next_unique_id();
        font::FontKey::new(self.namespace_id, new_id)
    }

    /// Generate a new font instance key
    pub fn generate_font_instance_key(&self) -> font::FontInstanceKey {
        let new_id = self.next_unique_id();
        font::FontInstanceKey::new(self.namespace_id, new_id)
    }

    /// Gets the dimensions for the supplied glyph keys
    ///
    /// Note: Internally, the internal texture cache doesn't store
    /// 'empty' textures (height or width = 0)
    /// This means that glyph dimensions e.g. for spaces (' ') will mostly be None.
    pub fn get_glyph_dimensions(
        &self,
        key: font::FontInstanceKey,
        glyph_indices: Vec<font::GlyphIndex>,
    ) -> Vec<Option<font::GlyphDimensions>> {
        let (sender, rx) = channel();
        let msg = ApiMsg::GetGlyphDimensions(font::GlyphDimensionRequest {
            key,
            glyph_indices,
            sender
        });
        self.api_sender.send(msg).unwrap();
        rx.recv().unwrap()
    }

    /// Gets the glyph indices for the supplied string. These
    /// can be used to construct GlyphKeys.
    pub fn get_glyph_indices(&self, key: font::FontKey, text: &str) -> Vec<Option<u32>> {
        let (sender, rx) = channel();
        let msg = ApiMsg::GetGlyphIndices(font::GlyphIndexRequest {
            key,
            text: text.to_string(),
            sender,
        });
        self.api_sender.send(msg).unwrap();
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
        let msg = ApiMsg::ExternalEvent(evt);
        self.api_sender.send(msg).unwrap();
    }

    /// Notify WebRender that now is a good time to flush caches and release
    /// as much memory as possible.
    pub fn notify_memory_pressure(&self) {
        self.api_sender.send(ApiMsg::MemoryPressure).unwrap();
    }

    /// Synchronously requests memory report.
    pub fn report_memory(&self) -> MemoryReport {
        let (tx, rx) = channel();
        self.api_sender.send(ApiMsg::ReportMemory(tx)).unwrap();
        *rx.recv().unwrap()
    }

    /// Update debugging flags.
    pub fn set_debug_flags(&self, flags: DebugFlags) {
        let cmd = DebugCommand::SetFlags(flags);
        self.api_sender.send(ApiMsg::DebugCommand(cmd)).unwrap();
    }

    /// Shut the WebRender instance down.
    pub fn shut_down(&self, synchronously: bool) {
        if synchronously {
            let (tx, rx) = channel();
            self.api_sender.send(ApiMsg::ShutDown(Some(tx))).unwrap();
            rx.recv().unwrap();
        } else {
            self.api_sender.send(ApiMsg::ShutDown(None)).unwrap();
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
            generate_frame: false,
            invalidate_rendered_frame: false,
            use_scene_builder_thread: false,
            low_priority: false,
            blob_rasterizer: None,
            blob_requests: Vec::new(),
            rasterized_blobs: Vec::new(),
        })
    }

    /// Creates a transaction message from a single scene message.
    fn scene_message(&self, msg: SceneMsg, document_id: DocumentId) -> Box<TransactionMsg> {
        Box::new(TransactionMsg {
            document_id,
            scene_ops: vec![msg],
            frame_ops: Vec::new(),
            resource_updates: Vec::new(),
            notifications: Vec::new(),
            generate_frame: false,
            invalidate_rendered_frame: false,
            use_scene_builder_thread: false,
            low_priority: false,
            blob_rasterizer: None,
            blob_requests: Vec::new(),
            rasterized_blobs: Vec::new(),
        })
    }

    /// A helper method to send document messages.
    fn send_scene_msg(&self, document_id: DocumentId, msg: SceneMsg) {
        // This assertion fails on Servo use-cases, because it creates different
        // `RenderApi` instances for layout and compositor.
        //assert_eq!(document_id.0, self.namespace_id);
        self.api_sender
            .send(ApiMsg::UpdateDocuments(vec![self.scene_message(msg, document_id)]))
            .unwrap()
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

        self.api_sender.send(ApiMsg::UpdateDocuments(vec![transaction])).unwrap();
    }

    /// Send multiple transactions.
    pub fn send_transactions(&mut self, document_ids: Vec<DocumentId>, mut transactions: Vec<Transaction>) {
        debug_assert!(document_ids.len() == transactions.len());
        let msgs = transactions.drain(..).zip(document_ids)
            .map(|(txn, id)| {
                let mut txn = txn.finalize(id);
                self.resources.update(&mut txn);

                txn
            })
            .collect();

        self.api_sender.send(ApiMsg::UpdateDocuments(msgs)).unwrap();
    }

    /// Does a hit test on display items in the specified document, at the given
    /// point. If a pipeline_id is specified, it is used to further restrict the
    /// hit results so that only items inside that pipeline are matched. If the
    /// HitTestFlags argument contains the FIND_ALL flag, then the vector of hit
    /// results will contain all display items that match, ordered from front
    /// to back.
    pub fn hit_test(&self,
                    document_id: DocumentId,
                    pipeline_id: Option<PipelineId>,
                    point: WorldPoint,
                    flags: HitTestFlags)
                    -> HitTestResult {
        let (tx, rx) = channel();

        self.send_frame_msg(
            document_id,
            FrameMsg::HitTest(pipeline_id, point, flags, tx)
        );
        rx.recv().unwrap()
    }

    /// Synchronously request an object that can perform fast hit testing queries.
    pub fn request_hit_tester(&self, document_id: DocumentId) -> HitTesterRequest {
        let (tx, rx) = channel();
        self.send_frame_msg(
            document_id,
            FrameMsg::RequestHitTester(tx)
        );

        HitTesterRequest { rx }
    }

    /// Setup the output region in the framebuffer for a given document.
    pub fn set_document_view(
        &self,
        document_id: DocumentId,
        device_rect: DeviceIntRect,
        device_pixel_ratio: f32,
    ) {
        self.send_scene_msg(
            document_id,
            SceneMsg::SetDocumentView { device_rect, device_pixel_ratio },
        );
    }

    /// Setup the output region in the framebuffer for a given document.
    /// Enable copying of the output of this pipeline id to
    /// an external texture for callers to consume.
    pub fn enable_frame_output(
        &self,
        document_id: DocumentId,
        pipeline_id: PipelineId,
        enable: bool,
    ) {
        self.send_scene_msg(
            document_id,
            SceneMsg::EnableFrameOutput(pipeline_id, enable),
        );
    }

    ///
    pub fn get_scroll_node_state(&self, document_id: DocumentId) -> Vec<ScrollNodeState> {
        let (tx, rx) = channel();
        self.send_frame_msg(document_id, FrameMsg::GetScrollNodeState(tx));
        rx.recv().unwrap()
    }

    // Some internal scheduling magic that leaked into the API.
    // Buckle up and see APZUpdater.cpp for more info about what this is about.
    #[doc(hidden)]
    pub fn wake_scene_builder(&self) {
        self.send_message(ApiMsg::WakeSceneBuilder);
    }

    /// Block until a round-trip to the scene builder thread has completed. This
    /// ensures that any transactions (including ones deferred to the scene
    /// builder thread) have been processed.
    pub fn flush_scene_builder(&self) {
        let (tx, rx) = channel();
        self.send_message(ApiMsg::FlushSceneBuilder(tx));
        rx.recv().unwrap(); // block until done
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

        let (tx, rx) = channel();
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
        let msg = ApiMsg::ClearNamespace(self.namespace_id);
        let _ = self.api_sender.send(msg);
    }
}

/// A hit tester requested to the render backend thread but not necessarily ready yet.
///
/// The request should be resolved as late as possible to reduce the likelihood of blocking.
pub struct HitTesterRequest {
    rx: Receiver<Arc<dyn ApiHitTester>>,
}

impl HitTesterRequest {
    /// Block until the hit tester is available and return it, consuming teh request.
    pub fn resolve(self) -> Arc<dyn ApiHitTester> {
        self.rx.recv().unwrap()
    }
}

///
#[derive(Clone)]
pub struct ScrollNodeState {
    ///
    pub id: di::ExternalScrollId,
    ///
    pub scroll_offset: LayoutVector2D,
}

///
#[derive(Clone, Copy, Debug)]
pub enum ScrollLocation {
    /// Scroll by a certain amount.
    Delta(LayoutVector2D),
    /// Scroll to very top of element.
    Start,
    /// Scroll to very bottom of element.
    End,
}

/// Represents a zoom factor.
#[derive(Clone, Copy, Debug)]
pub struct ZoomFactor(f32);

impl ZoomFactor {
    /// Construct a new zoom factor.
    pub fn new(scale: f32) -> Self {
        ZoomFactor(scale)
    }

    /// Get the zoom factor as an untyped float.
    pub fn get(self) -> f32 {
        self.0
    }
}

/// A key to identify an animated property binding.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Deserialize, MallocSizeOf, PartialEq, Serialize, Eq, Hash, PeekPoke)]
pub struct PropertyBindingId {
    namespace: IdNamespace,
    uid: u32,
}

impl PropertyBindingId {
    /// Constructor.
    pub fn new(value: u64) -> Self {
        PropertyBindingId {
            namespace: IdNamespace((value >> 32) as u32),
            uid: value as u32,
        }
    }
}

/// A unique key that is used for connecting animated property
/// values to bindings in the display list.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, MallocSizeOf, PartialEq, Serialize, PeekPoke)]
pub struct PropertyBindingKey<T> {
    ///
    pub id: PropertyBindingId,
    _phantom: PhantomData<T>,
}

/// Construct a property value from a given key and value.
impl<T: Copy> PropertyBindingKey<T> {
    ///
    pub fn with(self, value: T) -> PropertyValue<T> {
        PropertyValue { key: self, value }
    }
}

impl<T> PropertyBindingKey<T> {
    /// Constructor.
    pub fn new(value: u64) -> Self {
        PropertyBindingKey {
            id: PropertyBindingId::new(value),
            _phantom: PhantomData,
        }
    }
}

/// A binding property can either be a specific value
/// (the normal, non-animated case) or point to a binding location
/// to fetch the current value from.
/// Note that Binding has also a non-animated value, the value is
/// used for the case where the animation is still in-delay phase
/// (i.e. the animation doesn't produce any animation values).
#[repr(C)]
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, MallocSizeOf, PartialEq, Serialize, PeekPoke)]
pub enum PropertyBinding<T> {
    /// Non-animated value.
    Value(T),
    /// Animated binding.
    Binding(PropertyBindingKey<T>, T),
}

impl<T: Default> Default for PropertyBinding<T> {
    fn default() -> Self {
        PropertyBinding::Value(Default::default())
    }
}

impl<T> From<T> for PropertyBinding<T> {
    fn from(value: T) -> PropertyBinding<T> {
        PropertyBinding::Value(value)
    }
}

impl From<PropertyBindingKey<ColorF>> for PropertyBindingKey<ColorU> {
    fn from(key: PropertyBindingKey<ColorF>) -> PropertyBindingKey<ColorU> {
        PropertyBindingKey {
            id: key.id.clone(),
            _phantom: PhantomData,
        }
    }
}

impl From<PropertyBindingKey<ColorU>> for PropertyBindingKey<ColorF> {
    fn from(key: PropertyBindingKey<ColorU>) -> PropertyBindingKey<ColorF> {
        PropertyBindingKey {
            id: key.id.clone(),
            _phantom: PhantomData,
        }
    }
}

impl From<PropertyBinding<ColorF>> for PropertyBinding<ColorU> {
    fn from(value: PropertyBinding<ColorF>) -> PropertyBinding<ColorU> {
        match value {
            PropertyBinding::Value(value) => PropertyBinding::Value(value.into()),
            PropertyBinding::Binding(k, v) => {
                PropertyBinding::Binding(k.into(), v.into())
            }
        }
    }
}

impl From<PropertyBinding<ColorU>> for PropertyBinding<ColorF> {
    fn from(value: PropertyBinding<ColorU>) -> PropertyBinding<ColorF> {
        match value {
            PropertyBinding::Value(value) => PropertyBinding::Value(value.into()),
            PropertyBinding::Binding(k, v) => {
                PropertyBinding::Binding(k.into(), v.into())
            }
        }
    }
}

/// The current value of an animated property. This is
/// supplied by the calling code.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
pub struct PropertyValue<T> {
    ///
    pub key: PropertyBindingKey<T>,
    ///
    pub value: T,
}

/// When using `generate_frame()`, a list of `PropertyValue` structures
/// can optionally be supplied to provide the current value of any
/// animated properties.
#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Default)]
pub struct DynamicProperties {
    ///
    pub transforms: Vec<PropertyValue<LayoutTransform>>,
    /// opacity
    pub floats: Vec<PropertyValue<f32>>,
    /// background color
    pub colors: Vec<PropertyValue<ColorF>>,
}

/// A handler to integrate WebRender with the thread that contains the `Renderer`.
pub trait RenderNotifier: Send {
    ///
    fn clone(&self) -> Box<dyn RenderNotifier>;
    /// Wake the thread containing the `Renderer` up (after updates have been put
    /// in the renderer's queue).
    fn wake_up(&self);
    /// Notify the thread containing the `Renderer` that a new frame is ready.
    fn new_frame_ready(&self, _: DocumentId, scrolled: bool, composite_needed: bool, render_time_ns: Option<u64>);
    /// A Gecko-specific notification mechanism to get some code executed on the
    /// `Renderer`'s thread, mostly replaced by `NotificationHandler`. You should
    /// probably use the latter instead.
    fn external_event(&self, _evt: ExternalEvent) {
        unimplemented!()
    }
    /// Notify the thread containing the `Renderer` that the render backend has been
    /// shut down.
    fn shut_down(&self) {}
}

/// A stage of the rendering pipeline.
#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Checkpoint {
    ///
    SceneBuilt,
    ///
    FrameBuilt,
    ///
    FrameTexturesUpdated,
    ///
    FrameRendered,
    /// NotificationRequests get notified with this if they get dropped without having been
    /// notified. This provides the guarantee that if a request is created it will get notified.
    TransactionDropped,
}

/// A handler to notify when a transaction reaches certain stages of the rendering
/// pipeline.
pub trait NotificationHandler : Send + Sync {
    /// Entry point of the handler to implement. Invoked by WebRender.
    fn notify(&self, when: Checkpoint);
}

/// A request to notify a handler when the transaction reaches certain stages of the
/// rendering pipeline.
///
/// The request is guaranteed to be notified once and only once, even if the transaction
/// is dropped before the requested check-point.
pub struct NotificationRequest {
    handler: Option<Box<dyn NotificationHandler>>,
    when: Checkpoint,
}

impl NotificationRequest {
    /// Constructor.
    pub fn new(when: Checkpoint, handler: Box<dyn NotificationHandler>) -> Self {
        NotificationRequest {
            handler: Some(handler),
            when,
        }
    }

    /// The specified stage at which point the handler should be notified.
    pub fn when(&self) -> Checkpoint { self.when }

    /// Called by WebRender at specified stages to notify the registered handler.
    pub fn notify(mut self) {
        if let Some(handler) = self.handler.take() {
            handler.notify(self.when);
        }
    }
}

/// An object that can perform hit-testing without doing synchronous queries to
/// the RenderBackendThread.
pub trait ApiHitTester: Send + Sync {
    /// Does a hit test on display items in the specified document, at the given
    /// point. If a pipeline_id is specified, it is used to further restrict the
    /// hit results so that only items inside that pipeline are matched. If the
    /// HitTestFlags argument contains the FIND_ALL flag, then the vector of hit
    /// results will contain all display items that match, ordered from front
    /// to back.
    fn hit_test(&self, pipeline_id: Option<PipelineId>, point: WorldPoint, flags: HitTestFlags) -> HitTestResult;
}

impl Drop for NotificationRequest {
    fn drop(&mut self) {
        if let Some(ref mut handler) = self.handler {
            handler.notify(Checkpoint::TransactionDropped);
        }
    }
}

// This Clone impl yields an "empty" request because we don't want the requests
// to be notified twice so the request is owned by only one of the API messages
// (the original one) after the clone.
// This works in practice because the notifications requests are used for
// synchronization so we don't need to include them in the recording mechanism
// in wrench that clones the messages.
impl Clone for NotificationRequest {
    fn clone(&self) -> Self {
        NotificationRequest {
            when: self.when,
            handler: None,
        }
    }
}


bitflags! {
    /// Each bit of the edge AA mask is:
    /// 0, when the edge of the primitive needs to be considered for AA
    /// 1, when the edge of the segment needs to be considered for AA
    ///
    /// *Note*: the bit values have to match the shader logic in
    /// `write_transform_vertex()` function.
    #[cfg_attr(feature = "serialize", derive(Serialize))]
    #[cfg_attr(feature = "deserialize", derive(Deserialize))]
    #[derive(MallocSizeOf)]
    pub struct EdgeAaSegmentMask: u8 {
        ///
        const LEFT = 0x1;
        ///
        const TOP = 0x2;
        ///
        const RIGHT = 0x4;
        ///
        const BOTTOM = 0x8;
    }
}
