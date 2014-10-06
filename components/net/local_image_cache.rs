/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/*!
An adapter for ImageCacheTask that does local caching to avoid
extra message traffic, it also avoids waiting on the same image
multiple times and thus triggering reflows multiple times.
*/

use image_cache_task::{Decode, GetImage, ImageCacheTask, ImageFailed, ImageNotReady, ImageReady};
use image_cache_task::{ImageResponseMsg, Prefetch, WaitForImage};

use std::comm::{Receiver, channel};
use std::collections::hashmap::HashMap;
use servo_util::task::spawn_named;
use url::Url;

pub trait ImageResponder<NodeAddress: Send> {
    fn respond(&self) -> proc(ImageResponseMsg, NodeAddress):Send;
}

pub struct LocalImageCache<NodeAddress> {
    image_cache_task: ImageCacheTask,
    round_number: uint,
    on_image_available: Option<Box<ImageResponder<NodeAddress>+Send>>,
    state_map: HashMap<Url, ImageState>
}

impl<NodeAddress: Send> LocalImageCache<NodeAddress> {
    pub fn new(image_cache_task: ImageCacheTask) -> LocalImageCache<NodeAddress> {
        LocalImageCache {
            image_cache_task: image_cache_task,
            round_number: 1,
            on_image_available: None,
            state_map: HashMap::new()
        }
    }
}

#[deriving(Clone)]
struct ImageState {
    prefetched: bool,
    decoded: bool,
    last_request_round: uint,
    last_response: ImageResponseMsg
}

impl<NodeAddress: Send> LocalImageCache<NodeAddress> {
    /// The local cache will only do a single remote request for a given
    /// URL in each 'round'. Layout should call this each time it begins
    pub fn next_round(&mut self, on_image_available: Box<ImageResponder<NodeAddress> + Send>) {
        self.round_number += 1;
        self.on_image_available = Some(on_image_available);
    }

    pub fn prefetch(&mut self, url: &Url) {
        {
            let state = self.get_state(url);
            if state.prefetched {
                return
            }

            state.prefetched = true;
        }

        self.image_cache_task.send(Prefetch((*url).clone()));
    }

    pub fn decode(&mut self, url: &Url) {
        {
            let state = self.get_state(url);
            if state.decoded {
                return
            }
            state.decoded = true;
        }

        self.image_cache_task.send(Decode((*url).clone()));
    }

    // FIXME: Should return a Future
    pub fn get_image(&mut self, node_address: NodeAddress, url: &Url) -> Receiver<ImageResponseMsg> {
        {
            let round_number = self.round_number;
            let state = self.get_state(url);

            // Save the previous round number for comparison
            let last_round = state.last_request_round;
            // Set the current round number for this image
            state.last_request_round = round_number;

            match state.last_response {
                ImageReady(ref image) => {
                    let (chan, port) = channel();
                    chan.send(ImageReady(image.clone()));
                    return port;
                }
                ImageNotReady => {
                    if last_round == round_number {
                        let (chan, port) = channel();
                        chan.send(ImageNotReady);
                        return port;
                    } else {
                        // We haven't requested the image from the
                        // remote cache this round
                    }
                }
                ImageFailed => {
                    let (chan, port) = channel();
                    chan.send(ImageFailed);
                    return port;
                }
            }
        }

        let (response_chan, response_port) = channel();
        self.image_cache_task.send(GetImage((*url).clone(), response_chan));

        let response = response_port.recv();
        match response {
            ImageNotReady => {
                // Need to reflow when the image is available
                // FIXME: Instead we should be just passing a Future
                // to the caller, then to the display list. Finally,
                // the compositor should be resonsible for waiting
                // on the image to load and triggering layout
                let image_cache_task = self.image_cache_task.clone();
                assert!(self.on_image_available.is_some());
                let on_image_available: proc(ImageResponseMsg, NodeAddress):Send =
                    self.on_image_available.as_ref().unwrap().respond();
                let url = (*url).clone();
                spawn_named("LocalImageCache", proc() {
                    let (response_chan, response_port) = channel();
                    image_cache_task.send(WaitForImage(url, response_chan));
                    on_image_available(response_port.recv(), node_address);
                });
            }
            _ => ()
        }

        // Put a copy of the response in the cache
        let response_copy = match response {
            ImageReady(ref image) => ImageReady(image.clone()),
            ImageNotReady => ImageNotReady,
            ImageFailed => ImageFailed
        };
        self.get_state(url).last_response = response_copy;

        let (chan, port) = channel();
        chan.send(response);
        return port;
    }

    fn get_state<'a>(&'a mut self, url: &Url) -> &'a mut ImageState {
        let state = self.state_map.find_or_insert_with(url.clone(), |_| {
            let new_state = ImageState {
                prefetched: false,
                decoded: false,
                last_request_round: 0,
                last_response: ImageNotReady
            };
            new_state
        });
        state
    }
}
