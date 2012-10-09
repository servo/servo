use std::net::url::Url;
use std::arc::{ARC, clone, get};
use resource::image_cache_task::ImageCacheTask;
use mod resource::image_cache_task;
use geom::size::Size2D;

/** A struct to store image data. The image will be loaded once, the
    first time it is requested, and an arc will be stored.  Clones of
    this arc are given out on demand.
 */
pub struct ImageHolder {
    // Invariant: at least one of url and image is not none, except
    // occasionally while get_image is being called
    mut url : Option<Url>,
    mut image : Option<ARC<~Image>>,
    mut cached_size: Size2D<int>,
    image_cache_task: ImageCacheTask,
    reflow_cb: fn~(),

}

fn ImageHolder(url : &Url, image_cache_task: ImageCacheTask, +cb: fn~()) -> ImageHolder {
    debug!("ImageHolder() %?", url.to_str());
    let holder = ImageHolder {
        url : Some(copy *url),
        image : None,
        cached_size : Size2D(0,0),
        image_cache_task : image_cache_task,
        reflow_cb : copy cb,
    };

    // Tell the image cache we're going to be interested in this url
    // FIXME: These two messages must be sent to prep an image for use
    // but they are intended to be spread out in time. Ideally prefetch
    // should be done as early as possible and decode only once we
    // are sure that the image will be used.
    image_cache_task.send(image_cache_task::Prefetch(copy *url));
    image_cache_task.send(image_cache_task::Decode(copy *url));

    holder
}

impl ImageHolder {
    /**
    This version doesn't perform any computation, but may be stale w.r.t.
    newly-available image data that determines size.

    The intent is that the impure version is used during layout when
    dimensions are used for computing layout.
    */
    pure fn size() -> Size2D<int> {
        self.cached_size
    }
    
    /** Query and update current image size */
    fn get_size() -> Option<Size2D<int>> {
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

    // This function should not be called by two tasks at the same time
    fn get_image() -> Option<ARC<~Image>> {
        debug!("get_image() %?", self.url);

        // If this is the first time we've called this function, load
        // the image and store it for the future
        if self.image.is_none() {
            assert self.url.is_some();
            let url = copy self.url.get();

            let response_port = Port();
            self.image_cache_task.send(image_cache_task::GetImage(copy url, response_port.chan()));
            self.image = match response_port.recv() {
              image_cache_task::ImageReady(image) => Some(clone(&image)),
              image_cache_task::ImageNotReady => {
                // Need to reflow when the image is available
                let image_cache_task = self.image_cache_task;
                let reflow = copy self.reflow_cb;
                do task::spawn |copy url, move reflow| {
                    let response_port = Port();
                    image_cache_task.send(image_cache_task::WaitForImage(copy url, response_port.chan()));
                    match response_port.recv() {
                      image_cache_task::ImageReady(*) => reflow(),
                      image_cache_task::ImageNotReady => fail /*not possible*/,
                      image_cache_task::ImageFailed => ()
                    }
                }
                None
              }
              image_cache_task::ImageFailed => {
                debug!("image was not ready for %s", url.to_str());
                // FIXME: Need to schedule another layout when the image is ready
                None
              }
            };
        }

        if self.image.is_some() {
            // Temporarily swap out the arc of the image so we can clone
            // it without breaking purity, then put it back and return the
            // clone.  This is not threadsafe.
            let mut temp = None;
            temp <-> self.image;
            let im_arc = option::unwrap(temp);
            self.image = Some(clone(&im_arc));

            return Some(im_arc);
        } else {
            return None;
        }
    }
}
