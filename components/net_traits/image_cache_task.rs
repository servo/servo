/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use image::base::Image;
use ipc_channel::ipc::{self, IpcSender};
use std::sync::Arc;
use url::Url;
use util::mem::HeapSizeOf;

/// This is optionally passed to the image cache when requesting
/// and image, and returned to the specified event loop when the
/// image load completes. It is typically used to trigger a reflow
/// and/or repaint.
#[derive(Deserialize, Serialize)]
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
    /// The requested image failed to load, so a placeholder was loaded instead.
    PlaceholderLoaded(Arc<Image>),
    /// Neither the requested image nor the placeholder could be loaded.
    None
}

/// Channel for sending commands to the image cache.
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
    RequestImage(Url, ImageCacheChan, Option<ImageResponder>),

    /// Synchronously check the state of an image in the cache.
    /// TODO(gw): Profile this on some real world sites and see
    /// if it's worth caching the results of this locally in each
    /// layout / paint task.
    GetImageIfAvailable(Url, UsePlaceholder, IpcSender<Result<Arc<Image>, ImageState>>),

    /// Clients must wait for a response before shutting down the ResourceTask
    Exit(IpcSender<()>),
}

#[derive(Copy, Clone, PartialEq, Deserialize, Serialize)]
pub enum UsePlaceholder {
    No,
    Yes,
}

/// The client side of the image cache task. This can be safely cloned
/// and passed to different tasks.
#[derive(Clone, Deserialize, Serialize)]
pub struct ImageCacheTask {
    chan: IpcSender<ImageCacheCommand>,
}

/// The public API for the image cache task.
impl ImageCacheTask {

    /// Construct a new image cache
    pub fn new(chan: IpcSender<ImageCacheCommand>) -> ImageCacheTask {
        ImageCacheTask {
            chan: chan,
        }
    }

    /// Asynchronously request and image. See ImageCacheCommand::RequestImage.
    pub fn request_image(&self,
                         url: Url,
                         result_chan: ImageCacheChan,
                         responder: Option<ImageResponder>) {
        let msg = ImageCacheCommand::RequestImage(url, result_chan, responder);
        self.chan.send(msg).unwrap();
    }

    /// Get the current state of an image. See ImageCacheCommand::GetImageIfAvailable.
    pub fn get_image_if_available(&self, url: Url, use_placeholder: UsePlaceholder)
                                  -> Result<Arc<Image>, ImageState> {
        let (sender, receiver) = ipc::channel().unwrap();
        let msg = ImageCacheCommand::GetImageIfAvailable(url, use_placeholder, sender);
        self.chan.send(msg).unwrap();
        receiver.recv().unwrap()
    }

    /// Shutdown the image cache task.
    pub fn exit(&self) {
        let (response_chan, response_port) = ipc::channel().unwrap();
        self.chan.send(ImageCacheCommand::Exit(response_chan)).unwrap();
        response_port.recv().unwrap();
    }
}

