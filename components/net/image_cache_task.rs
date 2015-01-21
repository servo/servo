/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use image::base::{Image, load_from_memory};
use resource_task;
use resource_task::{LoadData, ResourceTask};
use resource_task::ProgressMsg::{Payload, Done};

use servo_util::task::spawn_named;
use servo_util::taskpool::TaskPool;
use std::borrow::ToOwned;
use std::comm::{channel, Receiver, Sender};
use std::collections::HashMap;
use std::collections::hash_map::{Occupied, Vacant};
use std::mem::replace;
use std::sync::{Arc, Mutex};
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
            (&ImageResponseMsg::ImageReady(..), &ImageResponseMsg::ImageReady(..)) => panic!("unimplemented comparison"),
            (&ImageResponseMsg::ImageNotReady, &ImageResponseMsg::ImageNotReady) => true,
            (&ImageResponseMsg::ImageFailed, &ImageResponseMsg::ImageFailed) => true,

            (&ImageResponseMsg::ImageReady(..), _) | (&ImageResponseMsg::ImageNotReady, _) | (&ImageResponseMsg::ImageFailed, _) => false
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

type DecoderFactory = fn() -> (proc(&[u8]) : 'static -> Option<Image>);

impl ImageCacheTask {
    pub fn new(resource_task: ResourceTask, task_pool: TaskPool) -> ImageCacheTask {
        let (chan, port) = channel();
        let chan_clone = chan.clone();

        spawn_named("ImageCacheTask".to_owned(), proc() {
            let mut cache = ImageCache {
                resource_task: resource_task,
                port: port,
                chan: chan_clone,
                state_map: HashMap::new(),
                wait_map: HashMap::new(),
                need_exit: None,
                task_pool: task_pool,
            };
            cache.run();
        });

        ImageCacheTask {
            chan: chan,
        }
    }

    pub fn new_sync(resource_task: ResourceTask, task_pool: TaskPool) -> ImageCacheTask {
        let (chan, port) = channel();

        spawn_named("ImageCacheTask (sync)".to_owned(), proc() {
            let inner_cache = ImageCacheTask::new(resource_task, task_pool);

            loop {
                let msg: Msg = port.recv();

                match msg {
                    Msg::GetImage(url, response) => {
                        inner_cache.send(Msg::WaitForImage(url, response));
                    }
                    Msg::Exit(response) => {
                        inner_cache.send(Msg::Exit(response));
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
    /// The state of processing an image for a URL
    state_map: HashMap<Url, ImageState>,
    /// List of clients waiting on a WaitForImage response
    wait_map: HashMap<Url, Arc<Mutex<Vec<Sender<ImageResponseMsg>>>>>,
    need_exit: Option<Sender<()>>,
    task_pool: TaskPool,
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

            match msg {
                Msg::Prefetch(url) => self.prefetch(url),
                Msg::StorePrefetchedImageData(url, data) => {
                    store_prefetched_chan.map(|chan| {
                        chan.send(());
                    });
                    store_prefetched_chan = None;

                    self.store_prefetched_image_data(url, data);
                }
                Msg::Decode(url) => self.decode(url),
                Msg::StoreImage(url, image) => {
                    store_chan.map(|chan| {
                        chan.send(());
                    });
                    store_chan = None;

                    self.store_image(url, image)
                }
                Msg::GetImage(url, response) => self.get_image(url, response),
                Msg::WaitForImage(url, response) => {
                    self.wait_for_image(url, response)
                }
                Msg::WaitForStore(chan) => store_chan = Some(chan),
                Msg::WaitForStorePrefetched(chan) => store_prefetched_chan = Some(chan),
                Msg::Exit(response) => {
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
                        ImageState::Prefetching(..) => can_exit = false,
                        ImageState::Decoding => can_exit = false,

                        ImageState::Init | ImageState::Prefetched(..) |
                        ImageState::Decoded(..) | ImageState::Failed => ()
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

    fn get_state(&self, url: &Url) -> ImageState {
        match self.state_map.get(url) {
            Some(state) => state.clone(),
            None => ImageState::Init
        }
    }

    fn set_state(&mut self, url: Url, state: ImageState) {
        self.state_map.insert(url, state);
    }

    fn prefetch(&mut self, url: Url) {
        match self.get_state(&url) {
            ImageState::Init => {
                let to_cache = self.chan.clone();
                let resource_task = self.resource_task.clone();
                let url_clone = url.clone();

                spawn_named("ImageCacheTask (prefetch)".to_owned(), proc() {
                    let url = url_clone;
                    debug!("image_cache_task: started fetch for {}", url.serialize());

                    let image = load_image_data(url.clone(), resource_task.clone());
                    to_cache.send(Msg::StorePrefetchedImageData(url.clone(), image));
                    debug!("image_cache_task: ended fetch for {}", url.serialize());
                });

                self.set_state(url, ImageState::Prefetching(AfterPrefetch::DoNotDecode));
            }

            ImageState::Prefetching(..) | ImageState::Prefetched(..) |
            ImageState::Decoding | ImageState::Decoded(..) | ImageState::Failed => {
                // We've already begun working on this image
            }
        }
    }

    fn store_prefetched_image_data(&mut self, url: Url, data: Result<Vec<u8>, ()>) {
        match self.get_state(&url) {
          ImageState::Prefetching(next_step) => {
            match data {
              Ok(data) => {
                self.set_state(url.clone(), ImageState::Prefetched(data));
                match next_step {
                  AfterPrefetch::DoDecode => self.decode(url),
                  _ => ()
                }
              }
              Err(..) => {
                self.set_state(url.clone(), ImageState::Failed);
                self.purge_waiters(url, || ImageResponseMsg::ImageFailed);
              }
            }
          }

          ImageState::Init
          | ImageState::Prefetched(..)
          | ImageState::Decoding
          | ImageState::Decoded(..)
          | ImageState::Failed => {
            panic!("wrong state for storing prefetched image")
          }
        }
    }

    fn decode(&mut self, url: Url) {
        match self.get_state(&url) {
            ImageState::Init => panic!("decoding image before prefetch"),

            ImageState::Prefetching(AfterPrefetch::DoNotDecode) => {
                // We don't have the data yet, queue up the decode
                self.set_state(url, ImageState::Prefetching(AfterPrefetch::DoDecode))
            }

            ImageState::Prefetching(AfterPrefetch::DoDecode) => {
                // We don't have the data yet, but the decode request is queued up
            }

            ImageState::Prefetched(data) => {
                let to_cache = self.chan.clone();
                let url_clone = url.clone();

                self.task_pool.execute(proc() {
                    let url = url_clone;
                    debug!("image_cache_task: started image decode for {}", url.serialize());
                    let image = load_from_memory(data.as_slice());
                    let image = image.map(|image| Arc::new(box image));
                    to_cache.send(Msg::StoreImage(url.clone(), image));
                    debug!("image_cache_task: ended image decode for {}", url.serialize());
                });

                self.set_state(url, ImageState::Decoding);
            }

            ImageState::Decoding | ImageState::Decoded(..) | ImageState::Failed => {
                // We've already begun decoding
            }
        }
    }

    fn store_image(&mut self, url: Url, image: Option<Arc<Box<Image>>>) {

        match self.get_state(&url) {
          ImageState::Decoding => {
            match image {
              Some(image) => {
                self.set_state(url.clone(), ImageState::Decoded(image.clone()));
                self.purge_waiters(url, || ImageResponseMsg::ImageReady(image.clone()) );
              }
              None => {
                self.set_state(url.clone(), ImageState::Failed);
                self.purge_waiters(url, || ImageResponseMsg::ImageFailed );
              }
            }
          }

          ImageState::Init
          | ImageState::Prefetching(..)
          | ImageState::Prefetched(..)
          | ImageState::Decoded(..)
          | ImageState::Failed => {
            panic!("incorrect state in store_image")
          }
        }

    }

    fn purge_waiters(&mut self, url: Url, f: || -> ImageResponseMsg) {
        match self.wait_map.remove(&url) {
            Some(waiters) => {
                let items = waiters.lock();
                for response in items.iter() {
                    response.send(f());
                }
            }
            None => ()
        }
    }

    fn get_image(&self, url: Url, response: Sender<ImageResponseMsg>) {
        match self.get_state(&url) {
            ImageState::Init => panic!("request for image before prefetch"),
            ImageState::Prefetching(AfterPrefetch::DoDecode) => response.send(ImageResponseMsg::ImageNotReady),
            ImageState::Prefetching(AfterPrefetch::DoNotDecode) | ImageState::Prefetched(..) => panic!("request for image before decode"),
            ImageState::Decoding => response.send(ImageResponseMsg::ImageNotReady),
            ImageState::Decoded(image) => response.send(ImageResponseMsg::ImageReady(image)),
            ImageState::Failed => response.send(ImageResponseMsg::ImageFailed),
        }
    }

    fn wait_for_image(&mut self, url: Url, response: Sender<ImageResponseMsg>) {
        match self.get_state(&url) {
            ImageState::Init => panic!("request for image before prefetch"),

            ImageState::Prefetching(AfterPrefetch::DoNotDecode) | ImageState::Prefetched(..) => panic!("request for image before decode"),

            ImageState::Prefetching(AfterPrefetch::DoDecode) | ImageState::Decoding => {
                // We don't have this image yet
                match self.wait_map.entry(url) {
                    Occupied(mut entry) => {
                        entry.get_mut().lock().push(response);
                    }
                    Vacant(entry) => {
                        entry.set(Arc::new(Mutex::new(vec!(response))));
                    }
                }
            }

            ImageState::Decoded(image) => {
                response.send(ImageResponseMsg::ImageReady(image));
            }

            ImageState::Failed => {
                response.send(ImageResponseMsg::ImageFailed);
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
        self.send(Msg::Exit(response_chan));
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
        self.send(Msg::WaitForStore(chan));
        port
    }

    #[cfg(test)]
    fn wait_for_store_prefetched(&self) -> Receiver<()> {
        let (chan, port) = channel();
        self.send(Msg::WaitForStorePrefetched(chan));
        port
    }
}

fn load_image_data(url: Url, resource_task: ResourceTask) -> Result<Vec<u8>, ()> {
    let (response_chan, response_port) = channel();
    resource_task.send(resource_task::ControlMsg::Load(LoadData::new(url, response_chan)));

    let mut image_data = vec!();

    let progress_port = response_port.recv().progress_port;
    loop {
        match progress_port.recv() {
            Payload(data) => {
                image_data.push_all(data.as_slice());
            }
            Done(Ok(..)) => {
                return Ok(image_data);
            }
            Done(Err(..)) => {
                return Err(());
            }
        }
    }
}


pub fn spawn_listener<A: Send>(f: proc(Receiver<A>):Send) -> Sender<A> {
    let (setup_chan, setup_port) = channel();

    spawn_named("ImageCacheTask (listener)".to_owned(), proc() {
        let (chan, port) = channel();
        setup_chan.send(chan);
        f(port);
    });
    setup_port.recv()
}


#[cfg(test)]
mod tests {
    use super::*;
    use super::ImageResponseMsg::*;
    use super::Msg::*;

    use resource_task;
    use resource_task::{ResourceTask, Metadata, start_sending, ResponseSenders};
    use resource_task::ProgressMsg::{Payload, Done};
    use sniffer_task;
    use image::base::test_image_bin;
    use servo_util::taskpool::TaskPool;
    use std::comm;
    use url::Url;

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
            response.send(Done(Ok(())));
        }
    }

    struct SendTestImage;
    impl Closure for SendTestImage {
        fn invoke(&self, response: Sender<resource_task::ProgressMsg>) {
            response.send(Payload(test_image_bin()));
            response.send(Done(Ok(())));
        }
    }

    struct SendBogusImage;
    impl Closure for SendBogusImage {
        fn invoke(&self, response: Sender<resource_task::ProgressMsg>) {
            response.send(Payload(vec!()));
            response.send(Done(Ok(())));
        }
    }

    struct SendTestImageErr;
    impl Closure for SendTestImageErr {
        fn invoke(&self, response: Sender<resource_task::ProgressMsg>) {
            response.send(Payload(test_image_bin()));
            response.send(Done(Err("".to_string())));
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
            response.send(Payload(test_image_bin()));
            response.send(Done(Ok(())));
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
            response.send(Payload(test_image_bin()));
            response.send(Done(Err("".to_string())));
        }
    }

    fn mock_resource_task<T: Closure+Send>(on_load: Box<T>) -> ResourceTask {
        spawn_listener(proc(port: Receiver<resource_task::ControlMsg>) {
            loop {
                match port.recv() {
                    resource_task::ControlMsg::Load(response) => {
                        let sniffer_task = sniffer_task::new_sniffer_task();
                        let senders = ResponseSenders {
                            immediate_consumer: sniffer_task,
                            eventual_consumer: response.consumer.clone(),
                        };
                        let chan = start_sending(senders, Metadata::default(
                            Url::parse("file:///fake").unwrap()));
                        on_load.invoke(chan);
                    }
                    resource_task::ControlMsg::Exit => break
                }
            }
        })
    }

    #[test]
    fn should_exit_on_request() {
        let mock_resource_task = mock_resource_task(box DoesNothing);

        let image_cache_task = ImageCacheTask::new(mock_resource_task.clone(), TaskPool::new(4));

        image_cache_task.exit();
        mock_resource_task.send(resource_task::ControlMsg::Exit);
    }

    #[test]
    #[should_fail]
    fn should_fail_if_unprefetched_image_is_requested() {
        let mock_resource_task = mock_resource_task(box DoesNothing);

        let image_cache_task = ImageCacheTask::new(mock_resource_task.clone(), TaskPool::new(4));
        let url = Url::parse("file:///").unwrap();

        let (chan, port) = channel();
        image_cache_task.send(Msg::GetImage(url, chan));
        port.recv();
    }

    #[test]
    fn should_request_url_from_resource_task_on_prefetch() {
        let (url_requested_chan, url_requested) = channel();

        let mock_resource_task = mock_resource_task(box JustSendOK { url_requested_chan: url_requested_chan});

        let image_cache_task = ImageCacheTask::new(mock_resource_task.clone(), TaskPool::new(4));
        let url = Url::parse("file:///").unwrap();

        image_cache_task.send(Prefetch(url));
        url_requested.recv();
        image_cache_task.exit();
        mock_resource_task.send(resource_task::ControlMsg::Exit);
    }

    #[test]
    fn should_not_request_url_from_resource_task_on_multiple_prefetches() {
        let (url_requested_chan, url_requested) = comm::channel();

        let mock_resource_task = mock_resource_task(box JustSendOK { url_requested_chan: url_requested_chan});

        let image_cache_task = ImageCacheTask::new(mock_resource_task.clone(), TaskPool::new(4));
        let url = Url::parse("file:///").unwrap();

        image_cache_task.send(Prefetch(url.clone()));
        image_cache_task.send(Prefetch(url));
        url_requested.recv();
        image_cache_task.exit();
        mock_resource_task.send(resource_task::ControlMsg::Exit);
        match url_requested.try_recv() {
            Err(_) => (),
            Ok(_) => panic!(),
        };
    }

    #[test]
    fn should_return_image_not_ready_if_data_has_not_arrived() {
        let (wait_chan, wait_port) = comm::channel();

        let mock_resource_task = mock_resource_task(box WaitSendTestImage{wait_port: wait_port});

        let image_cache_task = ImageCacheTask::new(mock_resource_task.clone(), TaskPool::new(4));
        let url = Url::parse("file:///").unwrap();

        image_cache_task.send(Prefetch(url.clone()));
        image_cache_task.send(Decode(url.clone()));
        let (response_chan, response_port) = comm::channel();
        image_cache_task.send(Msg::GetImage(url, response_chan));
        assert!(response_port.recv() == ImageResponseMsg::ImageNotReady);
        wait_chan.send(());
        image_cache_task.exit();
        mock_resource_task.send(resource_task::ControlMsg::Exit);
    }

    #[test]
    fn should_return_decoded_image_data_if_data_has_arrived() {
        let mock_resource_task = mock_resource_task(box SendTestImage);

        let image_cache_task = ImageCacheTask::new(mock_resource_task.clone(), TaskPool::new(4));
        let url = Url::parse("file:///").unwrap();

        let join_port = image_cache_task.wait_for_store();

        image_cache_task.send(Prefetch(url.clone()));
        image_cache_task.send(Decode(url.clone()));

        // Wait until our mock resource task has sent the image to the image cache
        join_port.recv();

        let (response_chan, response_port) = comm::channel();
        image_cache_task.send(Msg::GetImage(url, response_chan));
        match response_port.recv() {
          ImageResponseMsg::ImageReady(_) => (),
          _ => panic!("bleh")
        }

        image_cache_task.exit();
        mock_resource_task.send(resource_task::ControlMsg::Exit);
    }

    #[test]
    fn should_return_decoded_image_data_for_multiple_requests() {
        let mock_resource_task = mock_resource_task(box SendTestImage);

        let image_cache_task = ImageCacheTask::new(mock_resource_task.clone(), TaskPool::new(4));
        let url = Url::parse("file:///").unwrap();

        let join_port = image_cache_task.wait_for_store();

        image_cache_task.send(Prefetch(url.clone()));
        image_cache_task.send(Decode(url.clone()));

        // Wait until our mock resource task has sent the image to the image cache
        join_port.recv();

        for _ in range(0u32, 2u32) {
            let (response_chan, response_port) = comm::channel();
            image_cache_task.send(Msg::GetImage(url.clone(), response_chan));
            match response_port.recv() {
              ImageResponseMsg::ImageReady(_) => (),
              _ => panic!("bleh")
            }
        }

        image_cache_task.exit();
        mock_resource_task.send(resource_task::ControlMsg::Exit);
    }

    #[test]
    fn should_not_request_image_from_resource_task_if_image_is_already_available() {
        let (image_bin_sent_chan, image_bin_sent) = comm::channel();

        let (resource_task_exited_chan, resource_task_exited) = comm::channel();

        let mock_resource_task = spawn_listener(proc(port: Receiver<resource_task::ControlMsg>) {
            loop {
                match port.recv() {
                    resource_task::ControlMsg::Load(response) => {
                        let sniffer_task = sniffer_task::new_sniffer_task();
                        let senders = ResponseSenders {
                            immediate_consumer: sniffer_task,
                            eventual_consumer: response.consumer.clone(),
                        };
                        let chan = start_sending(senders, Metadata::default(
                            Url::parse("file:///fake").unwrap()));
                        chan.send(Payload(test_image_bin()));
                        chan.send(Done(Ok(())));
                        image_bin_sent_chan.send(());
                    }
                    resource_task::ControlMsg::Exit => {
                        resource_task_exited_chan.send(());
                        break
                    }
                }
            }
        });

        let image_cache_task = ImageCacheTask::new(mock_resource_task.clone(), TaskPool::new(4));
        let url = Url::parse("file:///").unwrap();

        image_cache_task.send(Prefetch(url.clone()));

        // Wait until our mock resource task has sent the image to the image cache
        image_bin_sent.recv();

        image_cache_task.send(Prefetch(url.clone()));

        image_cache_task.exit();
        mock_resource_task.send(resource_task::ControlMsg::Exit);

        resource_task_exited.recv();

        // Our resource task should not have received another request for the image
        // because it's already cached
        match image_bin_sent.try_recv() {
            Err(_) => (),
            Ok(_) => panic!(),
        }
    }

    #[test]
    fn should_not_request_image_from_resource_task_if_image_fetch_already_failed() {
        let (image_bin_sent_chan, image_bin_sent) = comm::channel();

        let (resource_task_exited_chan, resource_task_exited) = comm::channel();

        let mock_resource_task = spawn_listener(proc(port: Receiver<resource_task::ControlMsg>) {
            loop {
                match port.recv() {
                    resource_task::ControlMsg::Load(response) => {
                        let sniffer_task = sniffer_task::new_sniffer_task();
                        let senders = ResponseSenders {
                            immediate_consumer: sniffer_task,
                            eventual_consumer: response.consumer.clone(),
                        };
                        let chan = start_sending(senders, Metadata::default(
                            Url::parse("file:///fake").unwrap()));
                        chan.send(Payload(test_image_bin()));
                        chan.send(Done(Err("".to_string())));
                        image_bin_sent_chan.send(());
                    }
                    resource_task::ControlMsg::Exit => {
                        resource_task_exited_chan.send(());
                        break
                    }
                }
            }
        });

        let image_cache_task = ImageCacheTask::new(mock_resource_task.clone(), TaskPool::new(4));
        let url = Url::parse("file:///").unwrap();

        image_cache_task.send(Prefetch(url.clone()));
        image_cache_task.send(Decode(url.clone()));

        // Wait until our mock resource task has sent the image to the image cache
        image_bin_sent.recv();

        image_cache_task.send(Prefetch(url.clone()));
        image_cache_task.send(Decode(url.clone()));

        image_cache_task.exit();
        mock_resource_task.send(resource_task::ControlMsg::Exit);

        resource_task_exited.recv();

        // Our resource task should not have received another request for the image
        // because it's already cached
        match image_bin_sent.try_recv() {
            Err(_) => (),
            Ok(_) => panic!(),
        }
    }

    #[test]
    fn should_return_failed_if_image_bin_cannot_be_fetched() {
        let mock_resource_task = mock_resource_task(box SendTestImageErr);

        let image_cache_task = ImageCacheTask::new(mock_resource_task.clone(), TaskPool::new(4));
        let url = Url::parse("file:///").unwrap();

        let join_port = image_cache_task.wait_for_store_prefetched();

        image_cache_task.send(Prefetch(url.clone()));
        image_cache_task.send(Decode(url.clone()));

        // Wait until our mock resource task has sent the image to the image cache
        join_port.recv();

        let (response_chan, response_port) = comm::channel();
        image_cache_task.send(Msg::GetImage(url, response_chan));
        match response_port.recv() {
          ImageResponseMsg::ImageFailed => (),
          _ => panic!("bleh")
        }

        image_cache_task.exit();
        mock_resource_task.send(resource_task::ControlMsg::Exit);
    }

    #[test]
    fn should_return_failed_for_multiple_get_image_requests_if_image_bin_cannot_be_fetched() {
        let mock_resource_task = mock_resource_task(box SendTestImageErr);

        let image_cache_task = ImageCacheTask::new(mock_resource_task.clone(), TaskPool::new(4));
        let url = Url::parse("file:///").unwrap();

        let join_port = image_cache_task.wait_for_store_prefetched();

        image_cache_task.send(Prefetch(url.clone()));
        image_cache_task.send(Decode(url.clone()));

        // Wait until our mock resource task has sent the image to the image cache
        join_port.recv();

        let (response_chan, response_port) = comm::channel();
        image_cache_task.send(Msg::GetImage(url.clone(), response_chan));
        match response_port.recv() {
          ImageResponseMsg::ImageFailed => (),
          _ => panic!("bleh")
        }

        // And ask again, we should get the same response
        let (response_chan, response_port) = comm::channel();
        image_cache_task.send(Msg::GetImage(url, response_chan));
        match response_port.recv() {
          ImageResponseMsg::ImageFailed => (),
          _ => panic!("bleh")
        }

        image_cache_task.exit();
        mock_resource_task.send(resource_task::ControlMsg::Exit);
    }

    #[test]
    fn should_return_failed_if_image_decode_fails() {
        let mock_resource_task = mock_resource_task(box SendBogusImage);

        let image_cache_task = ImageCacheTask::new(mock_resource_task.clone(), TaskPool::new(4));
        let url = Url::parse("file:///").unwrap();

        let join_port = image_cache_task.wait_for_store();

        image_cache_task.send(Prefetch(url.clone()));
        image_cache_task.send(Decode(url.clone()));

        // Wait until our mock resource task has sent the image to the image cache
        join_port.recv();

        // Make the request
        let (response_chan, response_port) = comm::channel();
        image_cache_task.send(Msg::GetImage(url, response_chan));

        match response_port.recv() {
          ImageResponseMsg::ImageFailed => (),
          _ => panic!("bleh")
        }

        image_cache_task.exit();
        mock_resource_task.send(resource_task::ControlMsg::Exit);
    }

    #[test]
    fn should_return_image_on_wait_if_image_is_already_loaded() {
        let mock_resource_task = mock_resource_task(box SendTestImage);

        let image_cache_task = ImageCacheTask::new(mock_resource_task.clone(), TaskPool::new(4));
        let url = Url::parse("file:///").unwrap();

        let join_port = image_cache_task.wait_for_store();

        image_cache_task.send(Prefetch(url.clone()));
        image_cache_task.send(Decode(url.clone()));

        // Wait until our mock resource task has sent the image to the image cache
        join_port.recv();

        let (response_chan, response_port) = comm::channel();
        image_cache_task.send(Msg::WaitForImage(url, response_chan));
        match response_port.recv() {
          ImageResponseMsg::ImageReady(..) => (),
          _ => panic!("bleh")
        }

        image_cache_task.exit();
        mock_resource_task.send(resource_task::ControlMsg::Exit);
    }

    #[test]
    fn should_return_image_on_wait_if_image_is_not_yet_loaded() {
        let (wait_chan, wait_port) = comm::channel();

        let mock_resource_task = mock_resource_task(box WaitSendTestImage {wait_port: wait_port});

        let image_cache_task = ImageCacheTask::new(mock_resource_task.clone(), TaskPool::new(4));
        let url = Url::parse("file:///").unwrap();

        image_cache_task.send(Prefetch(url.clone()));
        image_cache_task.send(Decode(url.clone()));

        let (response_chan, response_port) = comm::channel();
        image_cache_task.send(Msg::WaitForImage(url, response_chan));

        wait_chan.send(());

        match response_port.recv() {
          ImageResponseMsg::ImageReady(..) => (),
          _ => panic!("bleh")
        }

        image_cache_task.exit();
        mock_resource_task.send(resource_task::ControlMsg::Exit);
    }

    #[test]
    fn should_return_image_failed_on_wait_if_image_fails_to_load() {
        let (wait_chan, wait_port) = comm::channel();

        let mock_resource_task = mock_resource_task(box WaitSendTestImageErr{wait_port: wait_port});

        let image_cache_task = ImageCacheTask::new(mock_resource_task.clone(), TaskPool::new(4));
        let url = Url::parse("file:///").unwrap();

        image_cache_task.send(Prefetch(url.clone()));
        image_cache_task.send(Decode(url.clone()));

        let (response_chan, response_port) = comm::channel();
        image_cache_task.send(Msg::WaitForImage(url, response_chan));

        wait_chan.send(());

        match response_port.recv() {
          ImageResponseMsg::ImageFailed => (),
          _ => panic!("bleh")
        }

        image_cache_task.exit();
        mock_resource_task.send(resource_task::ControlMsg::Exit);
    }

    #[test]
    fn sync_cache_should_wait_for_images() {
        let mock_resource_task = mock_resource_task(box SendTestImage);

        let image_cache_task = ImageCacheTask::new_sync(mock_resource_task.clone(), TaskPool::new(4));
        let url = Url::parse("file:///").unwrap();

        image_cache_task.send(Prefetch(url.clone()));
        image_cache_task.send(Decode(url.clone()));

        let (response_chan, response_port) = comm::channel();
        image_cache_task.send(Msg::GetImage(url, response_chan));
        match response_port.recv() {
          ImageResponseMsg::ImageReady(_) => (),
          _ => panic!("bleh")
        }

        image_cache_task.exit();
        mock_resource_task.send(resource_task::ControlMsg::Exit);
    }
}
