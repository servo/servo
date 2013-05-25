/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use image::base::Image;
use image_cache_task::{ImageReady, ImageNotReady, ImageFailed};
use local_image_cache::LocalImageCache;

use core::util::replace;
use geom::size::Size2D;
use std::net::url::Url;
use std::arc::{ARC, clone, get};

// FIXME: Nasty coupling here This will be a problem if we want to factor out image handling from
// the network stack. This should probably be factored out into an interface and use dependency
// injection.

/// A struct to store image data. The image will be loaded once the first time it is requested,
/// and an ARC will be stored.  Clones of this ARC are given out on demand.
pub struct ImageHolder {
    url: Url,
    image: Option<ARC<~Image>>,
    cached_size: Size2D<int>,
    local_image_cache: @mut LocalImageCache,
}

pub impl ImageHolder {
    pub fn new(url: Url, local_image_cache: @mut LocalImageCache) -> ImageHolder {
        debug!("ImageHolder::new() %?", url.to_str());
        let holder = ImageHolder {
            url: url,
            image: None,
            cached_size: Size2D(0,0),
            local_image_cache: local_image_cache,
        };

        // Tell the image cache we're going to be interested in this url
        // FIXME: These two messages must be sent to prep an image for use
        // but they are intended to be spread out in time. Ideally prefetch
        // should be done as early as possible and decode only once we
        // are sure that the image will be used.
        local_image_cache.prefetch(&holder.url);
        local_image_cache.decode(&holder.url);

        holder
    }

    /// This version doesn't perform any computation, but may be stale w.r.t. newly-available image
    /// data that determines size.
    ///
    /// The intent is that the impure version is used during layout when dimensions are used for
    /// computing layout.
    fn size(&self) -> Size2D<int> {
        self.cached_size
    }
    
    /// Query and update the current image size.
    fn get_size(&mut self) -> Option<Size2D<int>> {
        debug!("get_size() %?", self.url);
        match self.get_image() {
            Some(img) => { 
                let img_ref = get(&img);
                self.cached_size = Size2D(img_ref.width as int,
                                          img_ref.height as int);
                Some(copy self.cached_size)
            },
            None => None
        }
    }

    fn get_image(&mut self) -> Option<ARC<~Image>> {
        debug!("get_image() %?", self.url);

        // If this is the first time we've called this function, load
        // the image and store it for the future
        if self.image.is_none() {
            match self.local_image_cache.get_image(&self.url).recv() {
                ImageReady(image) => {
                    self.image = Some(image);
                }
                ImageNotReady => {
                    debug!("image not ready for %s", self.url.to_str());
                }
                ImageFailed => {
                    debug!("image decoding failed for %s", self.url.to_str());
                }
            }
        }

        // Clone isn't pure so we have to swap out the mutable image option
        let image = replace(&mut self.image, None);

        let result = match image {
            Some(ref image) => Some(clone(image)),
            None => None
        };

        replace(&mut self.image, image);

        return result;
    }
}

