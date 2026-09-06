/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::sync::Arc;

use log::debug;
use malloc_size_of::{MallocSizeOf, MallocSizeOfOps};
use malloc_size_of_derive::MallocSizeOf;
use paint_api::CrossProcessPaintApi;
use pixels::{CorsStatus, ImageMetadata, RasterImage};
use profile_traits::mem::Report;
use resvg::usvg::{Font, fontdb};
use serde::{Deserialize, Serialize};
use servo_base::id::{PipelineId, WebViewId};
use servo_url::{ImmutableOrigin, ServoUrl};
use uuid::Uuid;
use webrender_api::ImageKey;
use webrender_api::units::DeviceIntSize;

use crate::FetchResponseMsg;
use crate::request::CorsSettings;

// ======================================================================
// Aux structs and enums.
// ======================================================================

/// An interface for resolving font families and styles for SVG images.
pub trait FontResolver: Sync + Send + MallocSizeOf {
    /// Attempt to resolve a font reference using the provided database of fonts.
    /// Adding new fonts to the database is allowed. Return an index into the database
    /// if the font resolves to an entry, otherwise return None.
    fn resolve(&self, font: &Font, database: &mut Arc<fontdb::Database>) -> Option<fontdb::ID>;
    /// Backup resolve. Find a font that can represent `char` and is not in `excluded`.
    fn resolve_fallback(
        &self,
        char: char,
        excluded: &[fontdb::ID],
        database: &mut Arc<fontdb::Database>,
    ) -> Option<fontdb::ID>;
}

pub type VectorImageId = PendingImageId;

// Images with available pixels, or validated sources that require a display
// decode/rasterization at the size selected by layout.
#[derive(Clone, Debug, MallocSizeOf)]
pub enum Image {
    Raster(#[conditional_malloc_size_of] Arc<RasterImage>),
    Vector(VectorImage),
    StaticRaster(#[conditional_malloc_size_of] Arc<StaticRasterImage>),
}

/// A validated static raster source. Display pixels are owned by the cache, not
/// by DOM/layout references, so replacing a decode releases its old buffer.
#[derive(MallocSizeOf)]
pub struct StaticRasterImage {
    pub id: PendingImageId,
    pub metadata: ImageMetadata,
    pub cors_status: CorsStatus,
    #[conditional_malloc_size_of]
    pub bytes: Arc<Vec<u8>>,
}

impl std::fmt::Debug for StaticRasterImage {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("StaticRasterImage")
            .field("id", &self.id)
            .field("metadata", &self.metadata)
            .finish_non_exhaustive()
    }
}

impl StaticRasterImage {
    /// Obtain original pixels for consumers such as canvas. Do not use for layout.
    pub fn decode(&self) -> Option<Arc<RasterImage>> {
        pixels::load_from_memory(&self.bytes, self.cors_status).map(Arc::new)
    }
}

#[derive(Clone, Debug, Deserialize, MallocSizeOf, Serialize)]
pub struct VectorImage {
    pub id: VectorImageId,
    pub svg_id: Option<Uuid>,
    pub metadata: ImageMetadata,
    pub cors_status: CorsStatus,
}

impl Image {
    pub fn metadata(&self) -> ImageMetadata {
        match self {
            Image::Vector(image, ..) => image.metadata,
            Image::Raster(image) => image.metadata,
            Image::StaticRaster(image) => image.metadata,
        }
    }

    pub fn cors_status(&self) -> CorsStatus {
        match self {
            Image::Vector(image) => image.cors_status,
            Image::Raster(image) => image.cors_status,
            Image::StaticRaster(image) => image.cors_status,
        }
    }

    /// Get original pixels, decoding static sources synchronously if necessary.
    /// Layout must use display-cache lookup instead.
    pub fn as_raster_image(&self) -> Option<Arc<RasterImage>> {
        match self {
            Image::Raster(image) => Some(image.clone()),
            Image::StaticRaster(image) => image.decode(),
            Image::Vector(..) => None,
        }
    }
}

/// Indicating either entire image or just metadata availability
#[derive(Clone, Debug, MallocSizeOf)]
pub enum ImageOrMetadataAvailable {
    ImageAvailable { image: Image, url: ServoUrl },
    MetadataAvailable(ImageMetadata, PendingImageId),
}

pub type ImageCacheResponseCallback = Box<dyn Fn(ImageCacheResponseMessage) + Send + 'static>;

/// This is optionally passed to the image cache when requesting
/// and image, and returned to the specified event loop when the
/// image load completes. It is typically used to trigger a reflow
/// and/or repaint.
#[derive(MallocSizeOf)]
pub struct ImageLoadListener {
    pipeline_id: PipelineId,
    pub id: PendingImageId,
    #[ignore_malloc_size_of = "Difficult to measure FnOnce"]
    callback: ImageCacheResponseCallback,
}

impl ImageLoadListener {
    pub fn new(
        callback: ImageCacheResponseCallback,
        pipeline_id: PipelineId,
        id: PendingImageId,
    ) -> ImageLoadListener {
        ImageLoadListener {
            pipeline_id,
            callback,
            id,
        }
    }

    pub fn respond(&self, response: ImageResponse) {
        debug!("Notifying listener");
        (self.callback)(ImageCacheResponseMessage::NotifyPendingImageLoadStatus(
            PendingImageResponse {
                pipeline_id: self.pipeline_id,
                response,
                id: self.id,
            },
        ));
    }
}

/// The returned image.
#[derive(Clone, Debug, MallocSizeOf)]
pub enum ImageResponse {
    /// The requested image was loaded.
    Loaded(Image, ServoUrl),
    /// The request image metadata was loaded.
    MetadataLoaded(ImageMetadata),
    /// The requested image failed to load or decode.
    FailedToLoadOrDecode,
}

/// The unique id for an image that has previously been requested.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, MallocSizeOf, PartialEq, Serialize)]
pub struct PendingImageId(pub u64);

#[derive(Clone, Debug)]
pub struct PendingImageResponse {
    pub pipeline_id: PipelineId,
    pub response: ImageResponse,
    pub id: PendingImageId,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RasterizationCompleteResponse {
    pub pipeline_id: PipelineId,
    pub image_id: PendingImageId,
    pub requested_size: DeviceIntSize,
}

#[derive(Clone, Debug)]
pub enum ImageCacheResponseMessage {
    NotifyPendingImageLoadStatus(PendingImageResponse),
    VectorImageRasterizationComplete(RasterizationCompleteResponse),
    StaticRasterImageReady(PipelineId, PendingImageId, u64),
}

// ======================================================================
// ImageCache public API.
// ======================================================================

pub enum ImageCacheResult {
    Available(ImageOrMetadataAvailable),
    FailedToLoadOrDecode,
    Pending(PendingImageId),
    ReadyForRequest(PendingImageId),
}

/// Status of an active display demand. The generation distinguishes callbacks
/// already queued for an obsolete resize from the current request.
pub struct StaticRasterDemandStatus {
    pub id: PendingImageId,
    pub generation: u64,
    pub pending: bool,
}

/// A shared [`ImageCacheFactory`] is a per-process data structure used to create an [`ImageCache`]
/// inside that process in any `ScriptThread`. This allows sharing the same font database (for
/// SVGs) and also decoding thread pool among all [`ImageCache`]s in the same process.
pub trait ImageCacheFactory: Sync + Send {
    fn create(
        &self,
        webview_id: WebViewId,
        pipeline_id: PipelineId,
        paint_api: &CrossProcessPaintApi,
        font_resolver: Arc<dyn FontResolver>,
    ) -> Arc<dyn ImageCache>;
}

/// An [`ImageCache`] manages fetching and decoding images for a single `Pipeline` for its
/// `Document` and all of its associated `Worker`s.
pub trait ImageCache: Sync + Send {
    fn memory_reports(&self, prefix: &str, ops: &mut MallocSizeOfOps) -> Vec<Report>;

    #[cfg(feature = "test-util")]
    /// Returns the number of rasterization tasks
    fn number_of_rasterize_tasks(&self) -> usize;

    /// Get an [`ImageKey`] to be used for external WebRender image management for
    /// things like canvas rendering. Returns `None` when an [`ImageKey`] cannot
    /// be generated properly.
    fn get_image_key(&self) -> Option<ImageKey>;

    /// Definitively check whether there is a cached, fully loaded image available.
    fn get_image(
        &self,
        url: ServoUrl,
        origin: ImmutableOrigin,
        cors_setting: Option<CorsSettings>,
    ) -> Option<Image>;

    /// Returns if the Image is already in the cache or not. If the Image is not yet completely decoded, we return [`ImageCacheResult::Pending`] or [`ImageCacheResult::Available`].
    fn get_cached_image_status(
        &self,
        url: ServoUrl,
        origin: ImmutableOrigin,
        cors_setting: Option<CorsSettings>,
    ) -> ImageCacheResult;

    /// Current display pixels, if available. This never decodes synchronously.
    fn static_raster_image_key(&self, image_id: PendingImageId) -> Option<ImageKey>;

    /// Replace the complete set of static raster demands after building a display
    /// list. Duplicate ids are combined by their largest aspect-preserving scale.
    /// Completion notifications request another paint, without repeating load events.
    /// Returns each active generation and whether its completion is still pending.
    fn set_static_raster_demands(
        &self,
        demands: Vec<(PendingImageId, DeviceIntSize)>,
        callback: ImageCacheResponseCallback,
    ) -> Vec<StaticRasterDemandStatus>;

    /// Returns `Some` if the given `image_id` has already been rasterized at the given `size`.
    /// Otherwise, triggers a new job to perform the rasterization. If a notification
    /// is needed after rasterization is completed, the `add_rasterization_complete_listener`
    /// API below can be used to add a listener.
    fn rasterize_vector_image(
        &self,
        image_id: VectorImageId,
        size: DeviceIntSize,
        svg_id: Option<Uuid>,
    ) -> Option<RasterImage>;

    /// Adds a new listener to be notified once the given `image_id` has been rasterized at
    /// the given `size`. The listener will receive a `VectorImageRasterizationComplete`
    /// message on the given `sender`, even if the listener is called after rasterization
    /// at has already completed.
    fn add_rasterization_complete_listener(
        &self,
        pipeline_id: PipelineId,
        image_id: VectorImageId,
        size: DeviceIntSize,
        callback: ImageCacheResponseCallback,
    );

    /// Removes the rasterized image from the image_cache, identified by the id of the SVG
    fn evict_rasterized_image(&self, svg_id: &Uuid);

    /// Removes the completed image from the image_cache, identified by url, origin, and cors
    fn evict_completed_image(
        &self,
        url: &ServoUrl,
        origin: &ImmutableOrigin,
        cors_setting: &Option<CorsSettings>,
    );

    /// Synchronously get the broken image icon for this [`ImageCache`]. This will
    /// allocate space for this icon and upload it to WebRender.
    fn get_broken_image_icon(&self) -> Option<Arc<RasterImage>>;

    /// Add a new listener for the given pending image id. If the image is already present,
    /// the responder will still receive the expected response.
    fn add_listener(&self, listener: ImageLoadListener);

    /// Inform the image cache about a response for a pending request.
    fn notify_pending_response(&self, id: PendingImageId, action: FetchResponseMsg);

    /// Fills the image cache with a batch of keys.
    fn dispatch_fill_key_cache_with_batch_of_keys(&self, image_keys: Vec<ImageKey>);

    /// Clear the image cache.
    fn clear(&self);
}
