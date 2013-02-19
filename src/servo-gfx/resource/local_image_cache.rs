/*!
An adapter for ImageCacheTask that does local caching to avoid
extra message traffic, it also avoids waiting on the same image
multiple times and thus triggering reflows multiple times.
*/

use clone_arc = std::arc::clone;
use std::net::url::Url;
use pipes::{Port, Chan, stream};
use resource::image_cache_task::{ImageCacheTask, ImageResponseMsg, Prefetch, Decode, GetImage};
use resource::image_cache_task::{ WaitForImage, ImageReady, ImageNotReady, ImageFailed};
use util::url::{UrlMap, url_map};

pub fn LocalImageCache(image_cache_task: ImageCacheTask) -> LocalImageCache {
    LocalImageCache {
        image_cache_task: move image_cache_task,
        round_number: 1,
        mut on_image_available: None,
        state_map: url_map()
    }
}

pub struct LocalImageCache {
    priv image_cache_task: ImageCacheTask,
    priv mut round_number: uint,
    priv mut on_image_available: Option<@fn() -> ~fn(ImageResponseMsg)>,
    priv state_map: UrlMap<@ImageState>
}

priv struct ImageState {
    mut prefetched: bool,
    mut decoded: bool,
    mut last_request_round: uint,
    mut last_response: ImageResponseMsg
}

#[allow(non_implicitly_copyable_typarams)] // Using maps of Urls
pub impl LocalImageCache {
    /// The local cache will only do a single remote request for a given
    /// URL in each 'round'. Layout should call this each time it begins
    // FIXME: 'pub' is an unexpected token?
    /* pub */ fn next_round(&self, on_image_available: @fn() -> ~fn(ImageResponseMsg)) {
        self.round_number += 1;
        self.on_image_available = Some(move on_image_available);
    }

    pub fn prefetch(&self, url: &Url) {
        let state = self.get_state(url);
        if !state.prefetched {
            self.image_cache_task.send(Prefetch(copy *url));
            state.prefetched = true;
        }
    }

    pub fn decode(&self, url: &Url) {
        let state = self.get_state(url);
        if !state.decoded {
            self.image_cache_task.send(Decode(copy *url));
            state.decoded = true;
        }
    }

    // FIXME: Should return a Future
    pub fn get_image(&self, url: &Url) -> Port<ImageResponseMsg> {
        let state = self.get_state(url);

        // Save the previous round number for comparison
        let last_round = state.last_request_round;
        // Set the current round number for this image
        state.last_request_round = self.round_number;

        match state.last_response {
            ImageReady(ref image) => {
                // FIXME: appease borrowck
                unsafe {
                    let (port, chan) = pipes::stream();
                    chan.send(ImageReady(clone_arc(image)));
                    return move port;
                }
            }
            ImageNotReady => {
                if last_round == self.round_number {
                    let (port, chan) = pipes::stream();
                    chan.send(ImageNotReady);
                    return move port;
                } else {
                    // We haven't requested the image from the
                    // remote cache this round
                }
            }
            ImageFailed => {
                let (port, chan) = pipes::stream();
                chan.send(ImageFailed);
                return move port;
            }
        }

        let (response_port, response_chan) = pipes::stream();
        self.image_cache_task.send(GetImage(copy *url, move response_chan));

        let response = response_port.recv();
        match response {
            ImageNotReady => {
                // Need to reflow when the image is available
                // FIXME: Instead we should be just passing a Future
                // to the caller, then to the display list. Finally,
                // the compositor should be resonsible for waiting
                // on the image to load and triggering layout
                let image_cache_task = self.image_cache_task.clone();
                assert self.on_image_available.is_some();
                let on_image_available = self.on_image_available.get()();
                let url = copy *url;
                do task::spawn |move url, move on_image_available, move image_cache_task| {
                    let (response_port, response_chan) = pipes::stream();
                    image_cache_task.send(WaitForImage(copy url, move response_chan));
                    on_image_available(response_port.recv());
                }
            }
            _ => ()
        }

        // Put a copy of the response in the cache
        let response_copy = match response {
            ImageReady(ref image) => ImageReady(clone_arc(image)),
            ImageNotReady => ImageNotReady,
            ImageFailed => ImageFailed
        };
        state.last_response = move response_copy;

        let (port, chan) = pipes::stream();
        chan.send(move response);
        return move port;
    }

    priv fn get_state(&self, url: &Url) -> @ImageState {
        match self.state_map.find(url) {
            Some(state) => state,
            None => {
                let new_state = @ImageState {
                    prefetched: false,
                    decoded: false,
                    last_request_round: 0,
                    last_response: ImageNotReady
                };
                self.state_map.insert(copy *url, new_state);
                self.get_state(url)
            }
        }
    }
}

