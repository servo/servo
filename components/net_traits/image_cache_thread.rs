/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

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
    sender: IpcSender<ImageResponse>,
}

impl ImageResponder {
    pub fn new(sender: IpcSender<ImageResponse>) -> ImageResponder {
        ImageResponder {
            sender: sender,
        }
    }

    pub fn respond(&self, response: ImageResponse) {
        self.sender.send(response).unwrap()
    }
}

/// The current state of an image in the cache.
#[derive(PartialEq, Copy, Clone, Deserialize, Serialize)]
pub enum ImageState {
    Pending,
    LoadError,
    NotRequested,
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
    None
}

/// Indicating either entire image or just metadata availability
#[derive(Clone, Deserialize, Serialize, HeapSizeOf)]
pub enum ImageOrMetadataAvailable {
    ImageAvailable(Arc<Image>),
    MetadataAvailable(ImageMetadata),
}

/// Channel used by the image cache to send results.
#[derive(Clone, Deserialize, Serialize)]
pub struct ImageCacheChan(pub IpcSender<ImageCacheResult>);

/// The result of an image cache command that is returned to the
/// caller.
#[derive(Deserialize, Serialize)]
pub struct ImageCacheResult {
    pub responder: Option<ImageResponder>,
    pub image_response: ImageResponse,
}

/// Commands that the image cache understands.
#[derive(Deserialize, Serialize)]
pub enum ImageCacheCommand {
    /// Request an image asynchronously from the cache. Supply a channel
    /// to receive the result, and optionally an image responder
    /// that is passed to the result channel.
    RequestImage(ServoUrl, ImageCacheChan, Option<ImageResponder>),

    /// Requests an image and a "metadata-ready" notification message asynchronously from the
    /// cache. The cache will make an effort to send metadata before the image is completely
    /// loaded. Supply a channel to receive the results, and optionally an image responder
    /// that is passed to the result channel.
    RequestImageAndMetadata(ServoUrl, ImageCacheChan, Option<ImageResponder>),

    /// Synchronously check the state of an image in the cache.
    /// TODO(gw): Profile this on some real world sites and see
    /// if it's worth caching the results of this locally in each
    /// layout / paint thread.
    GetImageIfAvailable(ServoUrl, UsePlaceholder, IpcSender<Result<Arc<Image>, ImageState>>),

    /// Synchronously check the state of an image in the cache. If the image is in a loading
    /// state and but its metadata has been made available, it will be sent as a response.
    GetImageOrMetadataIfAvailable(ServoUrl, UsePlaceholder, IpcSender<Result<ImageOrMetadataAvailable, ImageState>>),

    /// Instruct the cache to store this data as a newly-complete network request and continue
    /// decoding the result into pixel data
    StoreDecodeImage(ServoUrl, Vec<u8>),

    /// Clients must wait for a response before shutting down the ResourceThread
    Exit(IpcSender<()>),
}

#[derive(Copy, Clone, PartialEq, Hash, Eq, Deserialize, Serialize)]
pub enum UsePlaceholder {
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

    /// Asynchronously request an image. See ImageCacheCommand::RequestImage.
    pub fn request_image(&self,
                         url: ServoUrl,
                         result_chan: ImageCacheChan,
                         responder: Option<ImageResponder>) {
        let msg = ImageCacheCommand::RequestImage(url, result_chan, responder);
        let _ = self.chan.send(msg);
    }

    /// Asynchronously request an image and metadata.
    /// See ImageCacheCommand::RequestImageAndMetadata
    pub fn request_image_and_metadata(&self,
                                      url: ServoUrl,
                                      result_chan: ImageCacheChan,
                                      responder: Option<ImageResponder>) {
        let msg = ImageCacheCommand::RequestImageAndMetadata(url, result_chan, responder);
        let _ = self.chan.send(msg);
    }

    /// Get the current state of an image. See ImageCacheCommand::GetImageIfAvailable.
    pub fn find_image(&self, url: ServoUrl, use_placeholder: UsePlaceholder)
                                  -> Result<Arc<Image>, ImageState> {
        let (sender, receiver) = ipc::channel().unwrap();
        let msg = ImageCacheCommand::GetImageIfAvailable(url, use_placeholder, sender);
        let _ = self.chan.send(msg);
        try!(receiver.recv().map_err(|_| ImageState::LoadError))
    }

    /// Get the current state of an image, returning its metadata if available.
    /// See ImageCacheCommand::GetImageOrMetadataIfAvailable.
    ///
    /// FIXME: We shouldn't do IPC for data uris!
    pub fn find_image_or_metadata(&self, url: ServoUrl, use_placeholder: UsePlaceholder)
                                  -> Result<ImageOrMetadataAvailable, ImageState> {
        let (sender, receiver) = ipc::channel().unwrap();
        let msg = ImageCacheCommand::GetImageOrMetadataIfAvailable(url, use_placeholder, sender);
        let _ = self.chan.send(msg);
        try!(receiver.recv().map_err(|_| ImageState::LoadError))
    }

    /// Decode the given image bytes and cache the result for the given URL.
    pub fn store_complete_image_bytes(&self,
                                      url: ServoUrl,
                                      image_data: Vec<u8>) {
        let msg = ImageCacheCommand::StoreDecodeImage(url, image_data);
        let _ = self.chan.send(msg);
    }

    /// Shutdown the image cache thread.
    pub fn exit(&self) {
        let (response_chan, response_port) = ipc::channel().unwrap();
        let _ = self.chan.send(ImageCacheCommand::Exit(response_chan));
        let _ = response_port.recv();
    }
}
