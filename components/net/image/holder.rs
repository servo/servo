/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use image::base::Image;
use image_cache_task::{ImageReady, ImageNotReady, ImageFailed};
use local_image_cache::LocalImageCache;

use geom::size::Size2D;
use std::mem;
use sync::{Arc, Mutex};
use url::Url;

// FIXME: Nasty coupling here This will be a problem if we want to factor out image handling from
// the network stack. This should probably be factored out into an interface and use dependency
// injection.

/// A struct to store image data. The image will be loaded once the first time it is requested,
/// and an Arc will be stored.  Clones of this Arc are given out on demand.
#[deriving(Clone)]
pub struct ImageHolder<NodeAddress> {
    url: Url,
    image: Option<Arc<Box<Image>>>,
    cached_size: Size2D<int>,
    local_image_cache: Arc<Mutex<LocalImageCache<NodeAddress>>>,
}

impl<NodeAddress: Send> ImageHolder<NodeAddress> {
    pub fn new(url: Url, local_image_cache: Arc<Mutex<LocalImageCache<NodeAddress>>>)
               -> ImageHolder<NodeAddress> {
        debug!("ImageHolder::new() {}", url.serialize());
        let holder = ImageHolder {
            url: url,
            image: None,
            cached_size: Size2D(0,0),
            local_image_cache: local_image_cache.clone(),
        };

        // Tell the image cache we're going to be interested in this url
        // FIXME: These two messages must be sent to prep an image for use
        // but they are intended to be spread out in time. Ideally prefetch
        // should be done as early as possible and decode only once we
        // are sure that the image will be used.
        {
            let val = holder.local_image_cache.lock();
            let mut local_image_cache = val;
            local_image_cache.prefetch(&holder.url);
            local_image_cache.decode(&holder.url);
        }

        holder
    }

    /// This version doesn't perform any computation, but may be stale w.r.t. newly-available image
    /// data that determines size.
    ///
    /// The intent is that the impure version is used during layout when dimensions are used for
    /// computing layout.
    pub fn size(&self) -> Size2D<int> {
        self.cached_size
    }

    /// Query and update the current image size.
    pub fn get_size(&mut self, node_address: NodeAddress) -> Option<Size2D<int>> {
        debug!("get_size() {}", self.url.serialize());
        self.get_image(node_address).map(|img| {
            self.cached_size = Size2D(img.width as int,
                                      img.height as int);
            self.cached_size.clone()
        })
    }

    pub fn get_image_if_present(&self) -> Option<Arc<Box<Image>>> {
        debug!("get_image_if_present() {}", self.url.serialize());
        self.image.clone()
    }

    pub fn get_image(&mut self, node_address: NodeAddress) -> Option<Arc<Box<Image>>> {
        debug!("get_image() {}", self.url.serialize());

        // If this is the first time we've called this function, load
        // the image and store it for the future
        if self.image.is_none() {
            let port = {
                let val = self.local_image_cache.lock();
                let mut local_image_cache = val;
                local_image_cache.get_image(node_address, &self.url)
            };
            match port.recv() {
                ImageReady(image) => {
                    self.image = Some(image);
                }
                ImageNotReady => {
                    debug!("image not ready for {:s}", self.url.serialize());
                }
                ImageFailed => {
                    debug!("image decoding failed for {:s}", self.url.serialize());
                }
            }
        }

        // Clone isn't pure so we have to swap out the mutable image option
        let image = mem::replace(&mut self.image, None);
        let result = image.clone();
        mem::replace(&mut self.image, image);

        return result;
    }

    pub fn url(&self) -> &Url {
        &self.url
    }
}
