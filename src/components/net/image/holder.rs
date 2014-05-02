/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use image::base::Image;
use image_cache_task::{ImageReady, ImageNotReady, ImageFailed};
use local_image_cache::LocalImageCache;

use geom::size::Size2D;
use std::cast;
use std::mem;
use std::ptr;
use sync::{Arc, Mutex};
use url::Url;

// FIXME: Nasty coupling here This will be a problem if we want to factor out image handling from
// the network stack. This should probably be factored out into an interface and use dependency
// injection.

/// An unfortunate hack to make this `Arc<Mutex>` `Share`.
pub struct LocalImageCacheHandle {
    data: *uint,
}

impl Drop for LocalImageCacheHandle {
    fn drop(&mut self) {
        unsafe {
            let _: ~Arc<Mutex<~LocalImageCache>> =
                cast::transmute(mem::replace(&mut self.data, ptr::null()));
        }
    }
}

impl Clone for LocalImageCacheHandle {
    fn clone(&self) -> LocalImageCacheHandle {
        unsafe {
            let handle = cast::transmute::<&Arc<Mutex<~LocalImageCache>>,&Arc<*()>>(self.get());
            let new_handle = (*handle).clone();
            LocalImageCacheHandle::new(new_handle)
        }
    }
}

impl LocalImageCacheHandle {
    pub unsafe fn new(cache: Arc<*()>) -> LocalImageCacheHandle {
        LocalImageCacheHandle {
            data: cast::transmute(~cache),
        }
    }

    pub fn get<'a>(&'a self) -> &'a Arc<Mutex<~LocalImageCache>> {
        unsafe {
            cast::transmute::<*uint,&'a Arc<Mutex<~LocalImageCache>>>(self.data)
        }
    }
}

/// A struct to store image data. The image will be loaded once the first time it is requested,
/// and an Arc will be stored.  Clones of this Arc are given out on demand.
#[deriving(Clone)]
pub struct ImageHolder {
    url: Url,
    image: Option<Arc<~Image>>,
    cached_size: Size2D<int>,
    local_image_cache: LocalImageCacheHandle,
}

impl ImageHolder {
    pub fn new(url: Url, local_image_cache: LocalImageCacheHandle) -> ImageHolder {
        debug!("ImageHolder::new() {}", url.to_str());
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
            let val = holder.local_image_cache.get().lock();
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
    pub fn get_size(&mut self) -> Option<Size2D<int>> {
        debug!("get_size() {}", self.url.to_str());
        self.get_image().map(|img| {
            self.cached_size = Size2D(img.width as int,
                                      img.height as int);
            self.cached_size.clone()
        })
    }

    pub fn get_image(&mut self) -> Option<Arc<~Image>> {
        debug!("get_image() {}", self.url.to_str());

        // If this is the first time we've called this function, load
        // the image and store it for the future
        if self.image.is_none() {
            let port = {
                let val = self.local_image_cache.get().lock();
                let mut local_image_cache = val;
                local_image_cache.get_image(&self.url)
            };
            match port.recv() {
                ImageReady(image) => {
                    self.image = Some(image);
                }
                ImageNotReady => {
                    debug!("image not ready for {:s}", self.url.to_str());
                }
                ImageFailed => {
                    debug!("image decoding failed for {:s}", self.url.to_str());
                }
            }
        }

        // Clone isn't pure so we have to swap out the mutable image option
        let image = mem::replace(&mut self.image, None);
        let result = image.clone();
        mem::replace(&mut self.image, image);

        return result;
    }
}

