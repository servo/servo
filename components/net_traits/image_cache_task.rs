/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use image::base::Image;
use url::Url;
use std::sync::Arc;
use std::sync::mpsc::{channel, Sender};

/// This is optionally passed to the image cache when requesting
/// and image, and returned to the specified event loop when the
/// image load completes. It is typically used to trigger a reflow
/// and/or repaint.
pub trait ImageResponder : Send {
    fn respond(&self, Option<Arc<Image>>);
}

/// The current state of an image in the cache.
#[derive(PartialEq, Copy, Clone)]
pub enum ImageState {
    Pending,
    LoadError,
    NotRequested,
}

/// Channel for sending commands to the image cache.
#[derive(Clone)]
pub struct ImageCacheChan(pub Sender<ImageCacheResult>);

/// The result of an image cache command that is returned to the
/// caller.
pub struct ImageCacheResult {
    pub responder: Option<Box<ImageResponder>>,
    pub image: Option<Arc<Image>>,
}

/// Commands that the image cache understands.
pub enum ImageCacheCommand {
    /// Request an image asynchronously from the cache. Supply a channel
    /// to receive the result, and optionally an image responder
    /// that is passed to the result channel.
    RequestImage(Url, ImageCacheChan, Option<Box<ImageResponder>>),

    /// Synchronously check the state of an image in the cache.
    /// TODO(gw): Profile this on some real world sites and see
    /// if it's worth caching the results of this locally in each
    /// layout / paint task.
    GetImageIfAvailable(Url, Sender<Result<Arc<Image>, ImageState>>),

    /// Clients must wait for a response before shutting down the ResourceTask
    Exit(Sender<()>),
}

/// The client side of the image cache task. This can be safely cloned
/// and passed to different tasks.
#[derive(Clone)]
pub struct ImageCacheTask {
    chan: Sender<ImageCacheCommand>,
}

/// The public API for the image cache task.
impl ImageCacheTask {

    /// Construct a new image cache
    pub fn new(chan: Sender<ImageCacheCommand>) -> ImageCacheTask {
        ImageCacheTask {
            chan: chan,
        }
    }

    /// Asynchronously request and image. See ImageCacheCommand::RequestImage.
    pub fn request_image(&self,
                         url: Url,
                         result_chan: ImageCacheChan,
                         responder: Option<Box<ImageResponder>>) {
        let msg = ImageCacheCommand::RequestImage(url, result_chan, responder);
        self.chan.send(msg).unwrap();
    }

    /// Get the current state of an image. See ImageCacheCommand::GetImageIfAvailable.
    pub fn get_image_if_available(&self, url: Url) -> Result<Arc<Image>, ImageState> {
        let (sender, receiver) = channel();
        let msg = ImageCacheCommand::GetImageIfAvailable(url, sender);
        self.chan.send(msg).unwrap();
        receiver.recv().unwrap()
    }

    /// Shutdown the image cache task.
    pub fn exit(&self) {
        let (response_chan, response_port) = channel();
        self.chan.send(ImageCacheCommand::Exit(response_chan)).unwrap();
        response_port.recv().unwrap();
    }
}
