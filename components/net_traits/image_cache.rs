/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use FetchResponseMsg;
use image::base::{Image, ImageMetadata};
use ipc_channel::ipc::IpcSender;
use servo_url::ServoUrl;
use std::sync::Arc;
use webrender_api;

// ======================================================================
// Aux structs and enums.
// ======================================================================

/// Whether a consumer is in a position to request images or not. This can occur
/// when animations are being processed by the layout thread while the script
/// thread is executing in parallel.
#[derive(Copy, Clone, PartialEq, Deserialize, Serialize)]
pub enum CanRequestImages {
    No,
    Yes,
}

/// Indicating either entire image or just metadata availability
#[derive(Clone, Deserialize, Serialize, HeapSizeOf)]
pub enum ImageOrMetadataAvailable {
    ImageAvailable(Arc<Image>, ServoUrl),
    MetadataAvailable(ImageMetadata),
}

/// This is optionally passed to the image cache when requesting
/// and image, and returned to the specified event loop when the
/// image load completes. It is typically used to trigger a reflow
/// and/or repaint.
#[derive(Clone, Deserialize, Serialize)]
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
#[derive(Clone, Debug, Deserialize, Serialize, HeapSizeOf)]
pub enum ImageResponse {
    /// The requested image was loaded.
    Loaded(Arc<Image>, ServoUrl),
    /// The request image metadata was loaded.
    MetadataLoaded(ImageMetadata),
    /// The requested image failed to load, so a placeholder was loaded instead.
    PlaceholderLoaded(Arc<Image>, ServoUrl),
    /// Neither the requested image nor the placeholder could be loaded.
    None,
}

/// The current state of an image in the cache.
#[derive(PartialEq, Copy, Clone, Deserialize, Serialize)]
pub enum ImageState {
    Pending(PendingImageId),
    LoadError,
    NotRequested(PendingImageId),
}

/// The unique id for an image that has previously been requested.
#[derive(Copy, Clone, PartialEq, Eq, Deserialize, Serialize, HeapSizeOf, Hash, Debug)]
pub struct PendingImageId(pub u64);

#[derive(Debug, Deserialize, Serialize)]
pub struct PendingImageResponse {
    pub response: ImageResponse,
    pub id: PendingImageId,
}

#[derive(Copy, Clone, PartialEq, Hash, Eq, Deserialize, Serialize)]
pub enum UsePlaceholder {
    No,
    Yes,
}

// ======================================================================
// ImageCache public API.
// ======================================================================

pub trait ImageCache: Sync + Send {
    fn new(webrender_api: webrender_api::RenderApi) -> Self where Self: Sized;

    /// Return any available metadata or image for the given URL,
    /// or an indication that the image is not yet available if it is in progress,
    /// or else reserve a slot in the cache for the URL if the consumer can request images.
    fn find_image_or_metadata(&self,
                              url: ServoUrl,
                              use_placeholder: UsePlaceholder,
                              can_request: CanRequestImages)
                              -> Result<ImageOrMetadataAvailable, ImageState>;

    /// Add a new listener for the given pending image id. If the image is already present,
    /// the responder will still receive the expected response.
    fn add_listener(&self, id: PendingImageId, listener: ImageResponder);

    /// Inform the image cache about a response for a pending request.
    fn notify_pending_response(&self, id: PendingImageId, action: FetchResponseMsg);

    /// Ensure an image has a webrender key.
    fn set_webrender_image_key(&self, image: &mut Image);
}
