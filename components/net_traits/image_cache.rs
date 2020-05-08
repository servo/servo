/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::image::base::{Image, ImageMetadata};
use crate::request::CorsSettings;
use crate::FetchResponseMsg;
use crate::WebrenderIpcSender;
use ipc_channel::ipc::IpcSender;
use servo_url::{ImmutableOrigin, ServoUrl};
use std::sync::Arc;

// ======================================================================
// Aux structs and enums.
// ======================================================================

/// Indicating either entire image or just metadata availability
#[derive(Clone, Debug, Deserialize, MallocSizeOf, Serialize)]
pub enum ImageOrMetadataAvailable {
    ImageAvailable {
        #[ignore_malloc_size_of = "Arc"]
        image: Arc<Image>,
        url: ServoUrl,
        is_placeholder: bool,
    },
    MetadataAvailable(ImageMetadata),
}

/// This is optionally passed to the image cache when requesting
/// and image, and returned to the specified event loop when the
/// image load completes. It is typically used to trigger a reflow
/// and/or repaint.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ImageResponder {
    id: PendingImageId,
    sender: IpcSender<PendingImageResponse>,
}

impl ImageResponder {
    pub fn new(sender: IpcSender<PendingImageResponse>, id: PendingImageId) -> ImageResponder {
        ImageResponder {
            sender: sender,
            id: id,
        }
    }

    pub fn respond(&self, response: ImageResponse) {
        debug!("Notifying listener");
        // This send can fail if thread waiting for this notification has panicked.
        // That's not a case that's worth warning about.
        // TODO(#15501): are there cases in which we should perform cleanup?
        let _ = self.sender.send(PendingImageResponse {
            response: response,
            id: self.id,
        });
    }
}

/// The returned image.
#[derive(Clone, Debug, Deserialize, MallocSizeOf, Serialize)]
pub enum ImageResponse {
    /// The requested image was loaded.
    Loaded(#[ignore_malloc_size_of = "Arc"] Arc<Image>, ServoUrl),
    /// The request image metadata was loaded.
    MetadataLoaded(ImageMetadata),
    /// The requested image failed to load, so a placeholder was loaded instead.
    PlaceholderLoaded(#[ignore_malloc_size_of = "Arc"] Arc<Image>, ServoUrl),
    /// Neither the requested image nor the placeholder could be loaded.
    None,
}

/// The unique id for an image that has previously been requested.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, MallocSizeOf, PartialEq, Serialize)]
pub struct PendingImageId(pub u64);

#[derive(Debug, Deserialize, Serialize)]
pub struct PendingImageResponse {
    pub response: ImageResponse,
    pub id: PendingImageId,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum UsePlaceholder {
    No,
    Yes,
}

// ======================================================================
// ImageCache public API.
// ======================================================================

pub enum ImageCacheResult {
    Available(ImageOrMetadataAvailable),
    LoadError,
    Pending(PendingImageId),
    ReadyForRequest(PendingImageId),
}

pub trait ImageCache: Sync + Send {
    fn new(webrender_api: WebrenderIpcSender) -> Self
    where
        Self: Sized;

    /// Definitively check whether there is a cached, fully loaded image available.
    fn get_image(
        &self,
        url: ServoUrl,
        origin: ImmutableOrigin,
        cors_setting: Option<CorsSettings>,
    ) -> Option<Arc<Image>>;

    fn get_cached_image_status(
        &self,
        url: ServoUrl,
        origin: ImmutableOrigin,
        cors_setting: Option<CorsSettings>,
        use_placeholder: UsePlaceholder,
    ) -> ImageCacheResult;

    /// Add a listener for the provided pending image id, eventually called by
    /// ImageCacheStore::complete_load.
    /// If only metadata is available, Available(ImageOrMetadataAvailable) will
    /// be returned.
    /// If Available(ImageOrMetadataAvailable::Image) or LoadError is the final value,
    /// the provided listener will be dropped (consumed & not added to PendingLoad).
    fn track_image(
        &self,
        url: ServoUrl,
        origin: ImmutableOrigin,
        cors_setting: Option<CorsSettings>,
        sender: IpcSender<PendingImageResponse>,
        use_placeholder: UsePlaceholder,
    ) -> ImageCacheResult;

    /// Add a new listener for the given pending image id. If the image is already present,
    /// the responder will still receive the expected response.
    fn add_listener(&self, id: PendingImageId, listener: ImageResponder);

    /// Inform the image cache about a response for a pending request.
    fn notify_pending_response(&self, id: PendingImageId, action: FetchResponseMsg);
}

/// Whether this response passed any CORS checks, and is thus safe to read from
/// in cross-origin environments.
#[derive(Clone, Copy, Debug, Deserialize, MallocSizeOf, PartialEq, Serialize)]
pub enum CorsStatus {
    /// The response is either same-origin or cross-origin but passed CORS checks.
    Safe,
    /// The response is cross-origin and did not pass CORS checks. It is unsafe
    /// to expose pixel data to the requesting environment.
    Unsafe,
}
