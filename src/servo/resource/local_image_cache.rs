/*!
An adapter for ImageCacheTask that does local caching to avoid
extra message traffic, it also avoids waiting on the same image
multiple times and thus triggering reflows multiple times.
*/

use std::net::url::Url;
use pipes::{Port, Chan, stream};
use image_cache_task::{ImageCacheTask, ImageResponseMsg, Prefetch, Decode, GetImage, WaitForImage};

pub fn LocalImageCache(
    image_cache_task: ImageCacheTask,
    on_image_available: ~fn(ImageResponseMsg)
) -> LocalImageCache {
    LocalImageCache {
        round_number: 0,
        image_cache_task: move image_cache_task,
        on_image_available: on_image_available
    }
}

pub struct LocalImageCache {
    priv mut round_number: uint,
    priv image_cache_task: ImageCacheTask,
    priv on_image_available: ~fn(ImageResponseMsg)
}

pub impl LocalImageCache {
    /// The local cache will only do a single remote request for a given
    /// URL in each 'round'. Layout should call this each time it begins
    fn next_round() {
        self.round_number += 1;
    }

    fn prefetch(url: &Url) {
        self.image_cache_task.send(Prefetch(copy *url));
    }

    fn decode(url: &Url) {
        self.image_cache_task.send(Decode(copy *url));
    }

    // FIXME: Should return a Future
    fn get_image(url: &Url) -> Port<ImageResponseMsg> {
        let (response_chan, response_port) = pipes::stream();
        self.image_cache_task.send(image_cache_task::GetImage(copy *url, response_chan));

        let response = response_port.recv();
        match response {
            image_cache_task::ImageNotReady => {
                // Need to reflow when the image is available
                // FIXME: Instead we should be just passing a Future
                // to the caller, then to the display list. Finally,
                // the compositor should be resonsible for waiting
                // on the image to load and triggering layout
                let image_cache_task = self.image_cache_task.clone();
                let on_image_available = copy self.on_image_available;
                let url = copy *url;
                do task::spawn |move url, move on_image_available| {
                    let (response_chan, response_port) = pipes::stream();
                    image_cache_task.send(image_cache_task::WaitForImage(copy url, response_chan));
                    on_image_available(response_port.recv());
                }
            }
            _ => ()
        }

        let (chan, port) = pipes::stream();
        chan.send(response);
        return port;
    }
}

