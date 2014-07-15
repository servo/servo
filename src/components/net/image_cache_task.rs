/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use image::base::{Image, load_from_memory};
use resource_task;
use resource_task::{LoadData, ResourceTask};
use servo_util::url::{UrlMap, url_map};

use std::comm::{channel, Receiver, Sender};
use std::mem::replace;
use std::task::spawn;
use std::to_str::ToStr;
use std::result;
use sync::{Arc, Mutex};
use serialize::{Encoder, Encodable};
use url::Url;

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

#[deriving(Clone)]
pub enum ImageResponseMsg {
    ImageReady(Arc<Box<Image>>),
    ImageNotReady,
    ImageFailed
}

impl PartialEq for ImageResponseMsg {
    fn eq(&self, other: &ImageResponseMsg) -> bool {
        match (self, other) {
            (&ImageReady(..), &ImageReady(..)) => fail!("unimplemented comparison"),
            (&ImageNotReady, &ImageNotReady) => true,
            (&ImageFailed, &ImageFailed) => true,

            (&ImageReady(..), _) | (&ImageNotReady, _) | (&ImageFailed, _) => false
        }
    }
}

#[deriving(Clone)]
pub struct ImageCacheTask {
    chan: Sender<Msg>,
}

impl<E, S: Encoder<E>> Encodable<S, E> for ImageCacheTask {
    fn encode(&self, _: &mut S) -> Result<(), E> {
        Ok(())
    }
}

type DecoderFactory = fn() -> proc(&[u8]) -> Option<Image>;

impl ImageCacheTask {
    pub fn new(resource_task: ResourceTask) -> ImageCacheTask {
        let (chan, port) = channel();
        let chan_clone = chan.clone();

        spawn(proc() {
            let mut cache = ImageCache {
                resource_task: resource_task.clone(),
                port: port,
                chan: chan_clone,
                state_map: url_map(),
                wait_map: url_map(),
                need_exit: None
            };
            cache.run();
        });

        ImageCacheTask {
            chan: chan,
        }
    }

    pub fn new_sync(resource_task: ResourceTask) -> ImageCacheTask {
        let (chan, port) = channel();

        spawn(proc() {
            let inner_cache = ImageCacheTask::new(resource_task.clone());

            loop {
                let msg: Msg = port.recv();

                match msg {
                    GetImage(url, response) => {
                        inner_cache.send(WaitForImage(url, response));
                    }
                    Exit(response) => {
                        inner_cache.send(Exit(response));
                        break;
                    }
                    msg => inner_cache.send(msg)
                }
            }
        });

        ImageCacheTask {
            chan: chan,
        }
    }
}

struct ImageCache {
    /// A handle to the resource task for fetching the image binaries
    resource_task: ResourceTask,
    /// The port on which we'll receive client requests
    port: Receiver<Msg>,
    /// A copy of the shared chan to give to child tasks
    chan: Sender<Msg>,
    /// The state of processsing an image for a URL
    state_map: UrlMap<ImageState>,
    /// List of clients waiting on a WaitForImage response
    wait_map: UrlMap<Arc<Mutex<Vec<Sender<ImageResponseMsg>>>>>,
    need_exit: Option<Sender<()>>,
}

#[deriving(Clone)]
enum ImageState {
    Init,
    Prefetching(AfterPrefetch),
    Prefetched(Vec<u8>),
    Decoding,
    Decoded(Arc<Box<Image>>),
    Failed
}

#[deriving(Clone)]
enum AfterPrefetch {
    DoDecode,
    DoNotDecode
}

impl ImageCache {
    pub fn run(&mut self) {
        let mut store_chan: Option<Sender<()>> = None;
        let mut store_prefetched_chan: Option<Sender<()>> = None;

        loop {
            let msg = self.port.recv();

            debug!("image_cache_task: received: {:?}", msg);

            match msg {
                Prefetch(url) => self.prefetch(url),
                StorePrefetchedImageData(url, data) => {
                    store_prefetched_chan.map(|chan| {
                        chan.send(());
                    });
                    store_prefetched_chan = None;

                    self.store_prefetched_image_data(url, data);
                }
                Decode(url) => self.decode(url),
                StoreImage(url, image) => {
                    store_chan.map(|chan| {
                        chan.send(());
                    });
                    store_chan = None;

                    self.store_image(url, image)
                }
                GetImage(url, response) => self.get_image(url, response),
                WaitForImage(url, response) => {
                    self.wait_for_image(url, response)
                }
                WaitForStore(chan) => store_chan = Some(chan),
                WaitForStorePrefetched(chan) => store_prefetched_chan = Some(chan),
                Exit(response) => {
                    assert!(self.need_exit.is_none());
                    self.need_exit = Some(response);
                }
            }

            let need_exit = replace(&mut self.need_exit, None);

            match need_exit {
              Some(response) => {
                // Wait until we have no outstanding requests and subtasks
                // before exiting
                let mut can_exit = true;
                for (_, state) in self.state_map.iter() {
                    match *state {
                        Prefetching(..) => can_exit = false,
                        Decoding => can_exit = false,

                        Init | Prefetched(..) | Decoded(..) | Failed => ()
                    }
                }

                if can_exit {
                    response.send(());
                    break;
                } else {
                    self.need_exit = Some(response);
                }
              }
              None => ()
            }
        }
    }

    fn get_state(&self, url: Url) -> ImageState {
        match self.state_map.find(&url) {
            Some(state) => state.clone(),
            None => Init
        }
    }

    fn set_state(&mut self, url: Url, state: ImageState) {
        self.state_map.insert(url, state);
    }

    fn prefetch(&mut self, url: Url) {
        match self.get_state(url.clone()) {
            Init => {
                let to_cache = self.chan.clone();
                let resource_task = self.resource_task.clone();
                let url_clone = url.clone();

                spawn(proc() {
                    let url = url_clone;
                    debug!("image_cache_task: started fetch for {:s}", url.to_str());

                    let image = load_image_data(url.clone(), resource_task.clone());

                    let result = if image.is_ok() {
                        Ok(image.unwrap())
                    } else {
                        Err(())
                    };
                    to_cache.send(StorePrefetchedImageData(url.clone(), result));
                    debug!("image_cache_task: ended fetch for {:s}", (url.clone()).to_str());
                });

                self.set_state(url, Prefetching(DoNotDecode));
            }

            Prefetching(..) | Prefetched(..) | Decoding | Decoded(..) | Failed => {
                // We've already begun working on this image
            }
        }
    }

    fn store_prefetched_image_data(&mut self, url: Url, data: Result<Vec<u8>, ()>) {
        match self.get_state(url.clone()) {
          Prefetching(next_step) => {
            match data {
              Ok(data) => {
                self.set_state(url.clone(), Prefetched(data));
                match next_step {
                  DoDecode => self.decode(url),
                  _ => ()
                }
              }
              Err(..) => {
                self.set_state(url.clone(), Failed);
                self.purge_waiters(url, || ImageFailed);
              }
            }
          }

          Init
          | Prefetched(..)
          | Decoding
          | Decoded(..)
          | Failed => {
            fail!("wrong state for storing prefetched image")
          }
        }
    }

    fn decode(&mut self, url: Url) {
        match self.get_state(url.clone()) {
            Init => fail!("decoding image before prefetch"),

            Prefetching(DoNotDecode) => {
                // We don't have the data yet, queue up the decode
                self.set_state(url, Prefetching(DoDecode))
            }

            Prefetching(DoDecode) => {
                // We don't have the data yet, but the decode request is queued up
            }

            Prefetched(data) => {
                let to_cache = self.chan.clone();
                let url_clone = url.clone();

                spawn(proc() {
                    let url = url_clone;
                    debug!("image_cache_task: started image decode for {:s}", url.to_str());
                    let image = load_from_memory(data.as_slice());
                    let image = if image.is_some() {
                        Some(Arc::new(box image.unwrap()))
                    } else {
                        None
                    };
                    to_cache.send(StoreImage(url.clone(), image));
                    debug!("image_cache_task: ended image decode for {:s}", url.to_str());
                });

                self.set_state(url, Decoding);
            }

            Decoding | Decoded(..) | Failed => {
                // We've already begun decoding
            }
        }
    }

    fn store_image(&mut self, url: Url, image: Option<Arc<Box<Image>>>) {

        match self.get_state(url.clone()) {
          Decoding => {
            match image {
              Some(image) => {
                self.set_state(url.clone(), Decoded(image.clone()));
                self.purge_waiters(url, || ImageReady(image.clone()) );
              }
              None => {
                self.set_state(url.clone(), Failed);
                self.purge_waiters(url, || ImageFailed );
              }
            }
          }

          Init
          | Prefetching(..)
          | Prefetched(..)
          | Decoded(..)
          | Failed => {
            fail!("incorrect state in store_image")
          }
        }

    }

    fn purge_waiters(&mut self, url: Url, f: || -> ImageResponseMsg) {
        match self.wait_map.pop(&url) {
            Some(waiters) => {
                let mut items = waiters.lock();
                for response in items.iter() {
                    response.send(f());
                }
            }
            None => ()
        }
    }

    fn get_image(&self, url: Url, response: Sender<ImageResponseMsg>) {
        match self.get_state(url.clone()) {
            Init => fail!("request for image before prefetch"),
            Prefetching(DoDecode) => response.send(ImageNotReady),
            Prefetching(DoNotDecode) | Prefetched(..) => fail!("request for image before decode"),
            Decoding => response.send(ImageNotReady),
            Decoded(image) => response.send(ImageReady(image.clone())),
            Failed => response.send(ImageFailed),
        }
    }

    fn wait_for_image(&mut self, url: Url, response: Sender<ImageResponseMsg>) {
        match self.get_state(url.clone()) {
            Init => fail!("request for image before prefetch"),

            Prefetching(DoNotDecode) | Prefetched(..) => fail!("request for image before decode"),

            Prefetching(DoDecode) | Decoding => {
                // We don't have this image yet
                if self.wait_map.contains_key(&url) {
                    let waiters = self.wait_map.find_mut(&url).unwrap();
                    let mut response = Some(response);
                    let mut items = waiters.lock();
                    items.push(response.take().unwrap());
                } else {
                    let response = vec!(response);
                    let wrapped = Arc::new(Mutex::new(response));
                    self.wait_map.insert(url, wrapped);
                }
            }

            Decoded(image) => {
                response.send(ImageReady(image.clone()));
            }

            Failed => {
                response.send(ImageFailed);
            }
        }
    }

}


pub trait ImageCacheTaskClient {
    fn exit(&self);
}

impl ImageCacheTaskClient for ImageCacheTask {
    fn exit(&self) {
        let (response_chan, response_port) = channel();
        self.send(Exit(response_chan));
        response_port.recv();
    }
}

impl ImageCacheTask {
    pub fn send(&self, msg: Msg) {
        self.chan.send(msg);
    }

    #[cfg(test)]
    fn wait_for_store(&self) -> Receiver<()> {
        let (chan, port) = channel();
        self.send(WaitForStore(chan));
        port
    }

    #[cfg(test)]
    fn wait_for_store_prefetched(&self) -> Receiver<()> {
        let (chan, port) = channel();
        self.send(WaitForStorePrefetched(chan));
        port
    }
}

fn load_image_data(url: Url, resource_task: ResourceTask) -> Result<Vec<u8>, ()> {
    let (response_chan, response_port) = channel();
    resource_task.send(resource_task::Load(LoadData::new(url), response_chan));

    let mut image_data = vec!();

    let progress_port = response_port.recv().progress_port;
    loop {
        match progress_port.recv() {
            resource_task::Payload(data) => {
                image_data.push_all(data.as_slice());
            }
            resource_task::Done(result::Ok(..)) => {
                return Ok(image_data.move_iter().collect());
            }
            resource_task::Done(result::Err(..)) => {
                return Err(());
            }
        }
    }
}


pub fn spawn_listener<A: Send>(f: proc(Receiver<A>):Send) -> Sender<A> {
    let (setup_chan, setup_port) = channel();

    spawn(proc() {
        let (chan, port) = channel();
        setup_chan.send(chan);
        f(port);
    });
    setup_port.recv()
}


#[cfg(test)]
mod tests {
    use super::*;

    use resource_task;
    use resource_task::{ResourceTask, Metadata, start_sending};
    use image::base::test_image_bin;
    use servo_util::url::parse_url;
    use std::comm;

    trait Closure {
        fn invoke(&self, _response: Sender<resource_task::ProgressMsg>) { }
    }
    struct DoesNothing;
    impl Closure for DoesNothing { }

    struct JustSendOK {
        url_requested_chan: Sender<()>,
    }
    impl Closure for JustSendOK {
        fn invoke(&self, response: Sender<resource_task::ProgressMsg>) {
            self.url_requested_chan.send(());
            response.send(resource_task::Done(Ok(())));
        }
    }

    struct SendTestImage;
    impl Closure for SendTestImage {
        fn invoke(&self, response: Sender<resource_task::ProgressMsg>) {
            response.send(resource_task::Payload(test_image_bin()));
            response.send(resource_task::Done(Ok(())));
        }
    }

    struct SendBogusImage;
    impl Closure for SendBogusImage {
        fn invoke(&self, response: Sender<resource_task::ProgressMsg>) {
            response.send(resource_task::Payload(vec!()));
            response.send(resource_task::Done(Ok(())));
        }
    }

    struct SendTestImageErr;
    impl Closure for SendTestImageErr {
        fn invoke(&self, response: Sender<resource_task::ProgressMsg>) {
            response.send(resource_task::Payload(test_image_bin()));
            response.send(resource_task::Done(Err("".to_string())));
        }
    }

    struct WaitSendTestImage {
        wait_port: Receiver<()>,
    }
    impl Closure for WaitSendTestImage {
        fn invoke(&self, response: Sender<resource_task::ProgressMsg>) {
            // Don't send the data until after the client requests
            // the image
            self.wait_port.recv();
            response.send(resource_task::Payload(test_image_bin()));
            response.send(resource_task::Done(Ok(())));
        }
    }

    struct WaitSendTestImageErr {
        wait_port: Receiver<()>,
    }
    impl Closure for WaitSendTestImageErr {
        fn invoke(&self, response: Sender<resource_task::ProgressMsg>) {
            // Don't send the data until after the client requests
            // the image
            self.wait_port.recv();
            response.send(resource_task::Payload(test_image_bin()));
            response.send(resource_task::Done(Err("".to_string())));
        }
    }

    fn mock_resource_task<T: Closure+Send>(on_load: Box<T>) -> ResourceTask {
        spawn_listener(proc(port: Receiver<resource_task::ControlMsg>) {
            loop {
                match port.recv() {
                    resource_task::Load(_, response) => {
                        let chan = start_sending(response, Metadata::default(parse_url("file:///fake", None)));
                        on_load.invoke(chan);
                    }
                    resource_task::Exit => break
                }
            }
        })
    }

    #[test]
    fn should_exit_on_request() {
        let mock_resource_task = mock_resource_task(box DoesNothing);

        let image_cache_task = ImageCacheTask::new(mock_resource_task.clone());
        let _url = parse_url("file", None);

        image_cache_task.exit();
        mock_resource_task.send(resource_task::Exit);
    }

    #[test]
    #[should_fail]
    fn should_fail_if_unprefetched_image_is_requested() {
        let mock_resource_task = mock_resource_task(box DoesNothing);

        let image_cache_task = ImageCacheTask::new(mock_resource_task.clone());
        let url = parse_url("file", None);

        let (chan, port) = channel();
        image_cache_task.send(GetImage(url, chan));
        port.recv();
    }

    #[test]
    fn should_request_url_from_resource_task_on_prefetch() {
        let (url_requested_chan, url_requested) = channel();

        let mock_resource_task = mock_resource_task(box JustSendOK { url_requested_chan: url_requested_chan});

        let image_cache_task = ImageCacheTask::new(mock_resource_task.clone());
        let url = parse_url("file", None);

        image_cache_task.send(Prefetch(url));
        url_requested.recv();
        image_cache_task.exit();
        mock_resource_task.send(resource_task::Exit);
    }

    #[test]
    fn should_not_request_url_from_resource_task_on_multiple_prefetches() {
        let (url_requested_chan, url_requested) = comm::channel();

        let mock_resource_task = mock_resource_task(box JustSendOK { url_requested_chan: url_requested_chan});

        let image_cache_task = ImageCacheTask::new(mock_resource_task.clone());
        let url = parse_url("file", None);

        image_cache_task.send(Prefetch(url.clone()));
        image_cache_task.send(Prefetch(url));
        url_requested.recv();
        image_cache_task.exit();
        mock_resource_task.send(resource_task::Exit);
        match url_requested.try_recv() {
            Err(_) => (),
            Ok(_) => fail!(),
        };
    }

    #[test]
    fn should_return_image_not_ready_if_data_has_not_arrived() {
        let (wait_chan, wait_port) = comm::channel();

        let mock_resource_task = mock_resource_task(box WaitSendTestImage{wait_port: wait_port});

        let image_cache_task = ImageCacheTask::new(mock_resource_task.clone());
        let url = parse_url("file", None);

        image_cache_task.send(Prefetch(url.clone()));
        image_cache_task.send(Decode(url.clone()));
        let (response_chan, response_port) = comm::channel();
        image_cache_task.send(GetImage(url, response_chan));
        assert!(response_port.recv() == ImageNotReady);
        wait_chan.send(());
        image_cache_task.exit();
        mock_resource_task.send(resource_task::Exit);
    }

    #[test]
    fn should_return_decoded_image_data_if_data_has_arrived() {
        let mock_resource_task = mock_resource_task(box SendTestImage);

        let image_cache_task = ImageCacheTask::new(mock_resource_task.clone());
        let url = parse_url("file", None);

        let join_port = image_cache_task.wait_for_store();

        image_cache_task.send(Prefetch(url.clone()));
        image_cache_task.send(Decode(url.clone()));

        // Wait until our mock resource task has sent the image to the image cache
        join_port.recv();

        let (response_chan, response_port) = comm::channel();
        image_cache_task.send(GetImage(url, response_chan));
        match response_port.recv() {
          ImageReady(_) => (),
          _ => fail!("bleh")
        }

        image_cache_task.exit();
        mock_resource_task.send(resource_task::Exit);
    }

    #[test]
    fn should_return_decoded_image_data_for_multiple_requests() {
        let mock_resource_task = mock_resource_task(box SendTestImage);

        let image_cache_task = ImageCacheTask::new(mock_resource_task.clone());
        let url = parse_url("file", None);

        let join_port = image_cache_task.wait_for_store();

        image_cache_task.send(Prefetch(url.clone()));
        image_cache_task.send(Decode(url.clone()));

        // Wait until our mock resource task has sent the image to the image cache
        join_port.recv();

        for _ in range(0,2) {
            let (response_chan, response_port) = comm::channel();
            image_cache_task.send(GetImage(url.clone(), response_chan));
            match response_port.recv() {
              ImageReady(_) => (),
              _ => fail!("bleh")
            }
        }

        image_cache_task.exit();
        mock_resource_task.send(resource_task::Exit);
    }

    #[test]
    fn should_not_request_image_from_resource_task_if_image_is_already_available() {
        let (image_bin_sent_chan, image_bin_sent) = comm::channel();

        let (resource_task_exited_chan, resource_task_exited) = comm::channel();

        let mock_resource_task = spawn_listener(proc(port: Receiver<resource_task::ControlMsg>) {
            loop {
                match port.recv() {
                    resource_task::Load(_, response) => {
                        let chan = start_sending(response, Metadata::default(parse_url("file:///fake", None)));
                        chan.send(resource_task::Payload(test_image_bin()));
                        chan.send(resource_task::Done(Ok(())));
                        image_bin_sent_chan.send(());
                    }
                    resource_task::Exit => {
                        resource_task_exited_chan.send(());
                        break
                    }
                }
            }
        });

        let image_cache_task = ImageCacheTask::new(mock_resource_task.clone());
        let url = parse_url("file", None);

        image_cache_task.send(Prefetch(url.clone()));

        // Wait until our mock resource task has sent the image to the image cache
        image_bin_sent.recv();

        image_cache_task.send(Prefetch(url.clone()));

        image_cache_task.exit();
        mock_resource_task.send(resource_task::Exit);

        resource_task_exited.recv();

        // Our resource task should not have received another request for the image
        // because it's already cached
        match image_bin_sent.try_recv() {
            Err(_) => (),
            Ok(_) => fail!(),
        }
    }

    #[test]
    fn should_not_request_image_from_resource_task_if_image_fetch_already_failed() {
        let (image_bin_sent_chan, image_bin_sent) = comm::channel();

        let (resource_task_exited_chan, resource_task_exited) = comm::channel();

        let mock_resource_task = spawn_listener(proc(port: Receiver<resource_task::ControlMsg>) {
            loop {
                match port.recv() {
                    resource_task::Load(_, response) => {
                        let chan = start_sending(response, Metadata::default(parse_url("file:///fake", None)));
                        chan.send(resource_task::Payload(test_image_bin()));
                        chan.send(resource_task::Done(Err("".to_string())));
                        image_bin_sent_chan.send(());
                    }
                    resource_task::Exit => {
                        resource_task_exited_chan.send(());
                        break
                    }
                }
            }
        });

        let image_cache_task = ImageCacheTask::new(mock_resource_task.clone());
        let url = parse_url("file", None);

        image_cache_task.send(Prefetch(url.clone()));
        image_cache_task.send(Decode(url.clone()));

        // Wait until our mock resource task has sent the image to the image cache
        image_bin_sent.recv();

        image_cache_task.send(Prefetch(url.clone()));
        image_cache_task.send(Decode(url.clone()));

        image_cache_task.exit();
        mock_resource_task.send(resource_task::Exit);

        resource_task_exited.recv();

        // Our resource task should not have received another request for the image
        // because it's already cached
        match image_bin_sent.try_recv() {
            Err(_) => (),
            Ok(_) => fail!(),
        }
    }

    #[test]
    fn should_return_failed_if_image_bin_cannot_be_fetched() {
        let mock_resource_task = mock_resource_task(box SendTestImageErr);

        let image_cache_task = ImageCacheTask::new(mock_resource_task.clone());
        let url = parse_url("file", None);

        let join_port = image_cache_task.wait_for_store_prefetched();

        image_cache_task.send(Prefetch(url.clone()));
        image_cache_task.send(Decode(url.clone()));

        // Wait until our mock resource task has sent the image to the image cache
        join_port.recv();

        let (response_chan, response_port) = comm::channel();
        image_cache_task.send(GetImage(url, response_chan));
        match response_port.recv() {
          ImageFailed => (),
          _ => fail!("bleh")
        }

        image_cache_task.exit();
        mock_resource_task.send(resource_task::Exit);
    }

    #[test]
    fn should_return_failed_for_multiple_get_image_requests_if_image_bin_cannot_be_fetched() {
        let mock_resource_task = mock_resource_task(box SendTestImageErr);

        let image_cache_task = ImageCacheTask::new(mock_resource_task.clone());
        let url = parse_url("file", None);

        let join_port = image_cache_task.wait_for_store_prefetched();

        image_cache_task.send(Prefetch(url.clone()));
        image_cache_task.send(Decode(url.clone()));

        // Wait until our mock resource task has sent the image to the image cache
        join_port.recv();

        let (response_chan, response_port) = comm::channel();
        image_cache_task.send(GetImage(url.clone(), response_chan));
        match response_port.recv() {
          ImageFailed => (),
          _ => fail!("bleh")
        }

        // And ask again, we should get the same response
        let (response_chan, response_port) = comm::channel();
        image_cache_task.send(GetImage(url, response_chan));
        match response_port.recv() {
          ImageFailed => (),
          _ => fail!("bleh")
        }

        image_cache_task.exit();
        mock_resource_task.send(resource_task::Exit);
    }

    #[test]
    fn should_return_failed_if_image_decode_fails() {
        let mock_resource_task = mock_resource_task(box SendBogusImage);

        let image_cache_task = ImageCacheTask::new(mock_resource_task.clone());
        let url = parse_url("file", None);

        let join_port = image_cache_task.wait_for_store();

        image_cache_task.send(Prefetch(url.clone()));
        image_cache_task.send(Decode(url.clone()));

        // Wait until our mock resource task has sent the image to the image cache
        join_port.recv();

        // Make the request
        let (response_chan, response_port) = comm::channel();
        image_cache_task.send(GetImage(url, response_chan));

        match response_port.recv() {
          ImageFailed => (),
          _ => fail!("bleh")
        }

        image_cache_task.exit();
        mock_resource_task.send(resource_task::Exit);
    }

    #[test]
    fn should_return_image_on_wait_if_image_is_already_loaded() {
        let mock_resource_task = mock_resource_task(box SendTestImage);

        let image_cache_task = ImageCacheTask::new(mock_resource_task.clone());
        let url = parse_url("file", None);

        let join_port = image_cache_task.wait_for_store();

        image_cache_task.send(Prefetch(url.clone()));
        image_cache_task.send(Decode(url.clone()));

        // Wait until our mock resource task has sent the image to the image cache
        join_port.recv();

        let (response_chan, response_port) = comm::channel();
        image_cache_task.send(WaitForImage(url, response_chan));
        match response_port.recv() {
          ImageReady(..) => (),
          _ => fail!("bleh")
        }

        image_cache_task.exit();
        mock_resource_task.send(resource_task::Exit);
    }

    #[test]
    fn should_return_image_on_wait_if_image_is_not_yet_loaded() {
        let (wait_chan, wait_port) = comm::channel();

        let mock_resource_task = mock_resource_task(box WaitSendTestImage {wait_port: wait_port});

        let image_cache_task = ImageCacheTask::new(mock_resource_task.clone());
        let url = parse_url("file", None);

        image_cache_task.send(Prefetch(url.clone()));
        image_cache_task.send(Decode(url.clone()));

        let (response_chan, response_port) = comm::channel();
        image_cache_task.send(WaitForImage(url, response_chan));

        wait_chan.send(());

        match response_port.recv() {
          ImageReady(..) => (),
          _ => fail!("bleh")
        }

        image_cache_task.exit();
        mock_resource_task.send(resource_task::Exit);
    }

    #[test]
    fn should_return_image_failed_on_wait_if_image_fails_to_load() {
        let (wait_chan, wait_port) = comm::channel();

        let mock_resource_task = mock_resource_task(box WaitSendTestImageErr{wait_port: wait_port});

        let image_cache_task = ImageCacheTask::new(mock_resource_task.clone());
        let url = parse_url("file", None);

        image_cache_task.send(Prefetch(url.clone()));
        image_cache_task.send(Decode(url.clone()));

        let (response_chan, response_port) = comm::channel();
        image_cache_task.send(WaitForImage(url, response_chan));

        wait_chan.send(());

        match response_port.recv() {
          ImageFailed => (),
          _ => fail!("bleh")
        }

        image_cache_task.exit();
        mock_resource_task.send(resource_task::Exit);
    }

    #[test]
    fn sync_cache_should_wait_for_images() {
        let mock_resource_task = mock_resource_task(box SendTestImage);

        let image_cache_task = ImageCacheTask::new_sync(mock_resource_task.clone());
        let url = parse_url("file", None);

        image_cache_task.send(Prefetch(url.clone()));
        image_cache_task.send(Decode(url.clone()));

        let (response_chan, response_port) = comm::channel();
        image_cache_task.send(GetImage(url, response_chan));
        match response_port.recv() {
          ImageReady(_) => (),
          _ => fail!("bleh")
        }

        image_cache_task.exit();
        mock_resource_task.send(resource_task::Exit);
    }
}
