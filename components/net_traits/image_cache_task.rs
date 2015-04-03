/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use image::base::Image;
use {ControlMsg, LoadData, ProgressMsg, ResourceTask};
use url::Url;

use std::sync::Arc;
use std::sync::mpsc::{channel, Sender};

pub enum Msg {
    /// Tell the cache that we may need a particular image soon. Must be posted
    /// before Decode
    Prefetch(Url),

    /// Tell the cache to decode an image. Must be posted before GetImage/WaitForImage
    Decode(Url),

    /// Request an Image object for a URL. If the image is not is not immediately
    /// available then ImageNotReady is returned.
    GetImage(Url, Sender<ImageResponseMsg>),

    /// Wait for an image to become available (or fail to load).
    WaitForImage(Url, Sender<ImageResponseMsg>),

    /// Clients must wait for a response before shutting down the ResourceTask
    Exit(Sender<()>),

    /// Used by the prefetch tasks to post back image binaries
    StorePrefetchedImageData(Url, Result<Vec<u8>, ()>),

    /// Used by the decoder tasks to post decoded images back to the cache
    StoreImage(Url, Option<Arc<Box<Image>>>),

    /// For testing
    WaitForStore(Sender<()>),

    /// For testing
    WaitForStorePrefetched(Sender<()>),
}

#[derive(Clone)]
pub enum ImageResponseMsg {
    ImageReady(Arc<Box<Image>>),
    ImageNotReady,
    ImageFailed
}

impl PartialEq for ImageResponseMsg {
    fn eq(&self, other: &ImageResponseMsg) -> bool {
        match (self, other) {
            (&ImageResponseMsg::ImageReady(..), &ImageResponseMsg::ImageReady(..)) => panic!("unimplemented comparison"),
            (&ImageResponseMsg::ImageNotReady, &ImageResponseMsg::ImageNotReady) => true,
            (&ImageResponseMsg::ImageFailed, &ImageResponseMsg::ImageFailed) => true,

            (&ImageResponseMsg::ImageReady(..), _) | (&ImageResponseMsg::ImageNotReady, _) | (&ImageResponseMsg::ImageFailed, _) => false
        }
    }
}

#[derive(Clone)]
pub struct ImageCacheTask {
    pub chan: Sender<Msg>,
}

impl ImageCacheTask {
    pub fn send(&self, msg: Msg) {
        self.chan.send(msg).unwrap();
    }
}

pub fn load_image_data(url: Url, resource_task: ResourceTask, placeholder: &[u8]) -> Result<Vec<u8>, ()> {
    let (response_chan, response_port) = channel();
    resource_task.send(ControlMsg::Load(LoadData::new(url.clone(), response_chan))).unwrap();

    let mut image_data = vec!();

    let progress_port = response_port.recv().unwrap().progress_port;
    loop {
        match progress_port.recv().unwrap() {
            ProgressMsg::Payload(data) => {
                image_data.push_all(&data);
            }
            ProgressMsg::Done(Ok(..)) => {
                return Ok(image_data);
            }
            ProgressMsg::Done(Err(..)) => {
                // Failure to load the requested image will return the
                // placeholder instead. In case it failed to load at init(),
                // we still recover and return Err() but nothing will be drawn.
                if placeholder.len() != 0 {
                    debug!("image_cache_task: failed to load {:?}, use placeholder instead.", url);
                    // Clean in case there was an error after started loading the image.
                    image_data.clear();
                    image_data.push_all(&placeholder);
                    return Ok(image_data);
                } else {
                    debug!("image_cache_task: invalid placeholder.");
                    return Err(());
                }
            }
        }
    }
}

