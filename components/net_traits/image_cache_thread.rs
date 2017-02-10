/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use FetchResponseMsg;
use image::base::{Image, ImageMetadata};
use ipc_channel::ipc::{self, IpcSender};
use servo_url::ServoUrl;
use std::sync::Arc;

/// This is optionally passed to the image cache when requesting
/// and image, and returned to the specified event loop when the
/// image load completes. It is typically used to trigger a reflow
/// and/or repaint.
#[derive(Clone, Deserialize, Serialize)]
pub struct ImageResponder {
    id: PendingImageId,
    sender: IpcSender<PendingImageResponse>,
}

#[derive(Deserialize, Serialize)]
pub struct PendingImageResponse {
    pub response: ImageResponse,
    pub id: PendingImageId,
}

impl ImageResponder {
    pub fn new(sender: IpcSender<PendingImageResponse>, id: PendingImageId) -> ImageResponder {
        ImageResponder {
            sender: sender,
            id: id,
        }
    }

    pub fn respond(&self, response: ImageResponse) {
        // This send can fail if thread waiting for this notification has panicked.
        // That's not a case that's worth warning about.
        // TODO(#15501): are there cases in which we should perform cleanup?
        let _ = self.sender.send(PendingImageResponse {
            response: response,
            id: self.id,
        });
    }
}

/// The unique id for an image that has previously been requested.
#[derive(Copy, Clone, PartialEq, Eq, Deserialize, Serialize, HeapSizeOf, Hash, Debug)]
pub struct PendingImageId(pub u64);

/// The current state of an image in the cache.
#[derive(PartialEq, Copy, Clone, Deserialize, Serialize)]
pub enum ImageState {
    Pending(PendingImageId),
    LoadError,
    NotRequested(PendingImageId),
}

/// The returned image.
#[derive(Clone, Deserialize, Serialize, HeapSizeOf)]
pub enum ImageResponse {
    /// The requested image was loaded.
    Loaded(Arc<Image>),
    /// The request image metadata was loaded.
    MetadataLoaded(ImageMetadata),
    /// The requested image failed to load, so a placeholder was loaded instead.
    PlaceholderLoaded(Arc<Image>),
    /// Neither the requested image nor the placeholder could be loaded.
    None,
}

/// Indicating either entire image or just metadata availability
#[derive(Clone, Deserialize, Serialize, HeapSizeOf)]
pub enum ImageOrMetadataAvailable {
    ImageAvailable(Arc<Image>),
    MetadataAvailable(ImageMetadata),
}

/// Commands that the image cache understands.
#[derive(Deserialize, Serialize)]
pub enum ImageCacheCommand {
    /// Synchronously check the state of an image in the cache. If the image is in a loading
    /// state and but its metadata has been made available, it will be sent as a response.
    GetImageOrMetadataIfAvailable(ServoUrl,
                                  UsePlaceholder,
                                  CanRequestImages,
                                  IpcSender<Result<ImageOrMetadataAvailable, ImageState>>),

    /// Add a new listener for the given pending image.
    AddListener(PendingImageId, ImageResponder),

    /// Instruct the cache to store this data as a newly-complete network request and continue
    /// decoding the result into pixel data
    StoreDecodeImage(PendingImageId, FetchResponseMsg),

    /// Clients must wait for a response before shutting down the ResourceThread
    Exit(IpcSender<()>),
}

#[derive(Copy, Clone, PartialEq, Hash, Eq, Deserialize, Serialize)]
pub enum UsePlaceholder {
    No,
    Yes,
}

/// Whether a consumer is in a position to request images or not. This can occur when
/// animations are being processed by the layout thread while the script thread is executing
/// in parallel.
#[derive(Copy, Clone, PartialEq, Deserialize, Serialize)]
pub enum CanRequestImages {
    No,
    Yes,
}

/// The client side of the image cache thread. This can be safely cloned
/// and passed to different threads.
#[derive(Clone, Deserialize, Serialize)]
pub struct ImageCacheThread {
    chan: IpcSender<ImageCacheCommand>,
}

/// The public API for the image cache thread.
impl ImageCacheThread {
    /// Construct a new image cache
    pub fn new(chan: IpcSender<ImageCacheCommand>) -> ImageCacheThread {
        ImageCacheThread {
            chan: chan,
        }
    }

    /// Get the current state of an image, returning its metadata if available.
    /// See ImageCacheCommand::GetImageOrMetadataIfAvailable.
    ///
    /// FIXME: We shouldn't do IPC for data uris!
    pub fn find_image_or_metadata(&self,
                                  url: ServoUrl,
                                  use_placeholder: UsePlaceholder,
                                  can_request: CanRequestImages)
                                  -> Result<ImageOrMetadataAvailable, ImageState> {
        let (sender, receiver) = ipc::channel().unwrap();
        let msg = ImageCacheCommand::GetImageOrMetadataIfAvailable(url,
                                                                   use_placeholder,
                                                                   can_request,
                                                                   sender);
        let _ = self.chan.send(msg);
        try!(receiver.recv().map_err(|_| ImageState::LoadError))
    }

    /// Add a new listener for the given pending image id. If the image is already present,
    /// the responder will still receive the expected response.
    pub fn add_listener(&self, id: PendingImageId, responder: ImageResponder) {
        let msg = ImageCacheCommand::AddListener(id, responder);
        self.chan.send(msg).expect("Image cache thread is not available");
    }

    /// Inform the image cache about a response for a pending request.
    pub fn notify_pending_response(&self, id: PendingImageId, data: FetchResponseMsg) {
        let msg = ImageCacheCommand::StoreDecodeImage(id, data);
        self.chan.send(msg).expect("Image cache thread is not available");
    }

    /// Shutdown the image cache thread.
    pub fn exit(&self) {
        // If the image cache is not available when we're trying to shut it down,
        // that is not worth warning about.
        let (response_chan, response_port) = ipc::channel().unwrap();
        let _ = self.chan.send(ImageCacheCommand::Exit(response_chan));
        let _ = response_port.recv();
    }
}
