use std::net::url::Url;
use std::arc::{ARC, clone};
use resource::image_cache_task::ImageCacheTask;
use resource::image_cache_task;

/** A struct to store image data. The image will be loaded once, the
    first time it is requested, and an arc will be stored.  Clones of
    this arc are given out on demand.
 */
pub struct ImageHolder {
    // Invariant: at least one of url and image is not none, except
    // occasionally while get_image is being called
    mut url : Option<Url>,
    mut image : Option<ARC<~Image>>,
    image_cache_task: ImageCacheTask,
    reflow_cb: fn~(),

}

fn ImageHolder(-url : Url, image_cache_task: ImageCacheTask, cb: fn~()) -> ImageHolder {
    let holder = ImageHolder {
        url : Some(copy url),
        image : None,
        image_cache_task : image_cache_task,
        reflow_cb : copy cb,
    };

    // Tell the image cache we're going to be interested in this url
    // FIXME: These two messages must be sent to prep an image for use
    // but they are intended to be spread out in time. Ideally prefetch
    // should be done as early as possible and decode only once we
    // are sure that the image will be used.
    image_cache_task.send(image_cache_task::Prefetch(copy url));
    image_cache_task.send(image_cache_task::Decode(move url));

    holder
}

impl ImageHolder {
    // This function should not be called by two tasks at the same time
    fn get_image() -> Option<ARC<~Image>> {
        // If this is the first time we've called this function, load
        // the image and store it for the future
        if self.image.is_none() {
            assert self.url.is_some();

            let mut temp = None;
            temp <-> self.url;
            let url = option::unwrap(temp);

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
                #info("image was not ready for %s", url.to_str());
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