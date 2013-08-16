/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use image::base::{Image, load_from_memory};
use resource_task;
use resource_task::ResourceTask;
use servo_util::url::{UrlMap, url_map};

use std::cell::Cell;
use std::comm::{Chan, Port, SharedChan, stream};
use std::task::spawn;
use std::to_str::ToStr;
use std::util::replace;
use std::result;
use extra::arc::Arc;
use extra::url::Url;

pub enum Msg {
    /// Tell the cache that we may need a particular image soon. Must be posted
    /// before Decode
    Prefetch(Url),

    // FIXME: We can probably get rid of this Cell now
    /// Used be the prefetch tasks to post back image binaries
    priv StorePrefetchedImageData(Url, Result<Cell<~[u8]>, ()>),

    /// Tell the cache to decode an image. Must be posted before GetImage/WaitForImage
    Decode(Url),

    /// Used by the decoder tasks to post decoded images back to the cache
    priv StoreImage(Url, Option<Arc<~Image>>),

    /// Request an Image object for a URL. If the image is not is not immediately
    /// available then ImageNotReady is returned.
    GetImage(Url, Chan<ImageResponseMsg>),

    /// Wait for an image to become available (or fail to load).
    WaitForImage(Url, Chan<ImageResponseMsg>),

    /// For testing
    priv OnMsg(~fn(msg: &Msg)),

    /// Clients must wait for a response before shutting down the ResourceTask
    Exit(Chan<()>),
}

pub enum ImageResponseMsg {
    ImageReady(Arc<~Image>),
    ImageNotReady,
    ImageFailed
}

impl ImageResponseMsg {
    fn clone(&self) -> ImageResponseMsg {
        match *self {
            ImageReady(ref img) => ImageReady(img.clone()),
            ImageNotReady => ImageNotReady,
            ImageFailed => ImageFailed,
        }
    }
}

impl Eq for ImageResponseMsg {
    fn eq(&self, other: &ImageResponseMsg) -> bool {
        // FIXME: Bad copies
        match (self.clone(), other.clone()) {
            (ImageReady(*), ImageReady(*)) => fail!(~"unimplemented comparison"),
            (ImageNotReady, ImageNotReady) => true,
            (ImageFailed, ImageFailed) => true,

            (ImageReady(*), _) | (ImageNotReady, _) | (ImageFailed, _) => false
        }
    }

    fn ne(&self, other: &ImageResponseMsg) -> bool {
        !(*self).eq(other)
    }
}

pub type ImageCacheTask = SharedChan<Msg>;

type DecoderFactory = ~fn() -> ~fn(&[u8]) -> Option<Image>;

pub fn ImageCacheTask(resource_task: ResourceTask) -> ImageCacheTask {
    ImageCacheTask_(resource_task, default_decoder_factory)
}

pub fn ImageCacheTask_(resource_task: ResourceTask, decoder_factory: DecoderFactory)
                       -> ImageCacheTask {
    // FIXME: Doing some dancing to avoid copying decoder_factory, our test
    // version of which contains an uncopyable type which rust will currently
    // copy unsoundly
    let decoder_factory_cell = Cell::new(decoder_factory);

    let (port, chan) = stream();
    let chan = SharedChan::new(chan);
    let port_cell = Cell::new(port);
    let chan_cell = Cell::new(chan.clone());

    do spawn {
        let mut cache = ImageCache {
            resource_task: resource_task.clone(),
            decoder_factory: decoder_factory_cell.take(),
            port: port_cell.take(),
            chan: chan_cell.take(),
            state_map: url_map(),
            wait_map: url_map(),
            need_exit: None
        };
        cache.run();
    }

    chan
}

fn SyncImageCacheTask(resource_task: ResourceTask) -> ImageCacheTask {
    let (port, chan) = stream();
    let port_cell = Cell::new(port);

    do spawn {
        let port = port_cell.take();
        let inner_cache = ImageCacheTask(resource_task.clone());

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
    }

    SharedChan::new(chan)
}

struct ImageCache {
    /// A handle to the resource task for fetching the image binaries
    resource_task: ResourceTask,
    /// Creates image decoders
    decoder_factory: DecoderFactory,
    /// The port on which we'll receive client requests
    port: Port<Msg>,
    /// A copy of the shared chan to give to child tasks
    chan: SharedChan<Msg>,
    /// The state of processsing an image for a URL
    state_map: UrlMap<ImageState>,
    /// List of clients waiting on a WaitForImage response
    wait_map: UrlMap<@mut ~[Chan<ImageResponseMsg>]>,
    need_exit: Option<Chan<()>>,
}

#[deriving(Clone)]
enum ImageState {
    Init,
    Prefetching(AfterPrefetch),
    Prefetched(@Cell<~[u8]>),
    Decoding,
    Decoded(@Arc<~Image>),
    Failed
}

#[deriving(Clone)]
enum AfterPrefetch {
    DoDecode,
    DoNotDecode
}

impl ImageCache {
    pub fn run(&mut self) {
        let mut msg_handlers: ~[~fn(msg: &Msg)] = ~[];

        loop {
            let msg = self.port.recv();

            for handler in msg_handlers.iter() {
                (*handler)(&msg)
            }

            debug!("image_cache_task: received: %?", msg);

            match msg {
                Prefetch(url) => self.prefetch(url),
                StorePrefetchedImageData(url, data) => {
                    self.store_prefetched_image_data(url, data);
                }
                Decode(url) => self.decode(url),
                StoreImage(url, image) => self.store_image(url, image),
                GetImage(url, response) => self.get_image(url, response),
                WaitForImage(url, response) => {
                    self.wait_for_image(url, response)
                }
                OnMsg(handler) => msg_handlers.push(handler),
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
                        Prefetching(*) => can_exit = false,
                        Decoding => can_exit = false,

                        Init | Prefetched(*) | Decoded(*) | Failed => ()
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
            Some(state) => *state,
            None => Init
        }
    }

    fn set_state(&self, url: Url, state: ImageState) {
        self.state_map.insert(url, state);
    }

    fn prefetch(&self, url: Url) {
        match self.get_state(url.clone()) {
            Init => {
                let to_cache = self.chan.clone();
                let resource_task = self.resource_task.clone();
                let url_cell = Cell::new(url.clone());

                do spawn {
                    let url = url_cell.take();
                    debug!("image_cache_task: started fetch for %s", url.to_str());

                    let image = load_image_data(url.clone(), resource_task.clone());

                    let result = if image.is_ok() {
                        Ok(Cell::new(image.unwrap()))
                    } else {
                        Err(())
                    };
                    to_cache.send(StorePrefetchedImageData(url.clone(), result));
                    debug!("image_cache_task: ended fetch for %s", (url.clone()).to_str());
                }

                self.set_state(url, Prefetching(DoNotDecode));
            }

            Prefetching(*) | Prefetched(*) | Decoding | Decoded(*) | Failed => {
                // We've already begun working on this image
            }
        }
    }

    fn store_prefetched_image_data(&self, url: Url, data: Result<Cell<~[u8]>, ()>) {
        match self.get_state(url.clone()) {
          Prefetching(next_step) => {
            match data {
              Ok(data_cell) => {
                let data = data_cell.take();
                self.set_state(url.clone(), Prefetched(@Cell::new(data)));
                match next_step {
                  DoDecode => self.decode(url),
                  _ => ()
                }
              }
              Err(*) => {
                self.set_state(url.clone(), Failed);
                self.purge_waiters(url, || ImageFailed);
              }
            }
          }

          Init
          | Prefetched(*)
          | Decoding
          | Decoded(*)
          | Failed => {
            fail!(~"wrong state for storing prefetched image")
          }
        }
    }

    fn decode(&self, url: Url) {
        match self.get_state(url.clone()) {
            Init => fail!(~"decoding image before prefetch"),

            Prefetching(DoNotDecode) => {
                // We don't have the data yet, queue up the decode
                self.set_state(url, Prefetching(DoDecode))
            }

            Prefetching(DoDecode) => {
                // We don't have the data yet, but the decode request is queued up
            }

            Prefetched(data_cell) => {
                assert!(!data_cell.is_empty());

                let data = data_cell.take();
                let to_cache = self.chan.clone();
                let url_cell = Cell::new(url.clone());
                let decode = (self.decoder_factory)();

                do spawn {
                    let url = url_cell.take();
                    debug!("image_cache_task: started image decode for %s", url.to_str());
                    let image = decode(data);
                    let image = if image.is_some() {
                        Some(Arc::new(~image.unwrap()))
                    } else {
                        None
                    };
                    to_cache.send(StoreImage(url.clone(), image));
                    debug!("image_cache_task: ended image decode for %s", url.to_str());
                }

                self.set_state(url, Decoding);
            }

            Decoding | Decoded(*) | Failed => {
                // We've already begun decoding
            }
        }
    }

    fn store_image(&self, url: Url, image: Option<Arc<~Image>>) {

        match self.get_state(url.clone()) {
          Decoding => {
            match image {
              Some(image) => {
                self.set_state(url.clone(), Decoded(@image.clone()));
                self.purge_waiters(url, || ImageReady(image.clone()) );
              }
              None => {
                self.set_state(url.clone(), Failed);
                self.purge_waiters(url, || ImageFailed );
              }
            }
          }

          Init
          | Prefetching(*)
          | Prefetched(*)
          | Decoded(*)
          | Failed => {
            fail!(~"incorrect state in store_image")
          }
        }

    }

    fn purge_waiters(&self, url: Url, f: &fn() -> ImageResponseMsg) {
        match self.wait_map.pop(&url) {
            Some(waiters) => {
                for response in waiters.iter() {
                    response.send(f());
                }
            }
            None => ()
        }
    }

    fn get_image(&self, url: Url, response: Chan<ImageResponseMsg>) {
        match self.get_state(url.clone()) {
            Init => fail!(~"request for image before prefetch"),
            Prefetching(DoDecode) => response.send(ImageNotReady),
            Prefetching(DoNotDecode) | Prefetched(*) => fail!(~"request for image before decode"),
            Decoding => response.send(ImageNotReady),
            Decoded(image) => response.send(ImageReady((*image).clone())),
            Failed => response.send(ImageFailed),
        }
    }

    fn wait_for_image(&self, url: Url, response: Chan<ImageResponseMsg>) {
        match self.get_state(url.clone()) {
            Init => fail!(~"request for image before prefetch"),

            Prefetching(DoNotDecode) | Prefetched(*) => fail!(~"request for image before decode"),

            Prefetching(DoDecode) | Decoding => {
                // We don't have this image yet
                if self.wait_map.contains_key(&url) {
                    let waiters = self.wait_map.find_mut(&url).unwrap();
                    waiters.push(response);
                } else {
                    self.wait_map.insert(url, @mut ~[response]);
                }
            }

            Decoded(image) => {
                response.send(ImageReady((*image).clone()));
            }

            Failed => {
                response.send(ImageFailed);
            }
        }
    }

}


trait ImageCacheTaskClient {
    fn exit(&self);
}

impl ImageCacheTaskClient for ImageCacheTask {
    fn exit(&self) {
        let (response_port, response_chan) = stream();
        self.send(Exit(response_chan));
        response_port.recv();
    }
}

fn load_image_data(url: Url, resource_task: ResourceTask) -> Result<~[u8], ()> {
    let (response_port, response_chan) = stream();
    resource_task.send(resource_task::Load(url, response_chan));

    let mut image_data = ~[];

    loop {
        match response_port.recv() {
            resource_task::Payload(data) => {
                image_data.push_all(data);
            }
            resource_task::Done(result::Ok(*)) => {
                return Ok(image_data);
            }
            resource_task::Done(result::Err(*)) => {
                return Err(());
            }
        }
    }
}

fn default_decoder_factory() -> ~fn(&[u8]) -> Option<Image> {
    let foo: ~fn(&[u8]) -> Option<Image> = |data: &[u8]| { load_from_memory(data) };
    foo
}

#[cfg(test)]
fn mock_resource_task(on_load: ~fn(resource: Chan<resource_task::ProgressMsg>)) -> ResourceTask {
    do spawn_listener |port: Port<resource_task::ControlMsg>| {
        loop {
            match port.recv() {
              resource_task::Load(_, response) => {
                on_load(response);
              }
              resource_task::Exit => break
            }
        }
    }
}

#[test]
fn should_exit_on_request() {
    let mock_resource_task = mock_resource_task(|_response| () );

    let image_cache_task = ImageCacheTask(mock_resource_task);
    let _url = make_url(~"file", None);

    image_cache_task.exit();
    mock_resource_task.send(resource_task::Exit);
}

#[test]
#[should_fail]
fn should_fail_if_unprefetched_image_is_requested() {
    let mock_resource_task = mock_resource_task(|_response| () );

    let image_cache_task = ImageCacheTask(mock_resource_task);
    let url = make_url(~"file", None);

    let (chan, port) = stream();
    image_cache_task.send(GetImage(url, chan));
    port.recv();
}

#[test]
fn should_request_url_from_resource_task_on_prefetch() {
    let url_requested = Port();
    let url_requested_chan = url_requested.chan();

    let mock_resource_task = do mock_resource_task |response| {
        url_requested_chan.send(());
        response.send(resource_task::Done(result::Ok(())));
    };

    let image_cache_task = ImageCacheTask(mock_resource_task);
    let url = make_url(~"file", None);

    image_cache_task.send(Prefetch(url));
    url_requested.recv();
    image_cache_task.exit();
    mock_resource_task.send(resource_task::Exit);
}


#[test]
#[should_fail]
fn should_fail_if_requesting_decode_of_an_unprefetched_image() {
    let mock_resource_task = mock_resource_task(|_response| () );

    let image_cache_task = ImageCacheTask(mock_resource_task);
    let url = make_url(~"file", None);

    image_cache_task.send(Decode(url));
    image_cache_task.exit();
}

#[test]
#[should_fail]
fn should_fail_if_requesting_image_before_requesting_decode() {
    let mock_resource_task = do mock_resource_task |response| {
        response.send(resource_task::Done(result::Ok(())));
    };

    let image_cache_task = ImageCacheTask(mock_resource_task);
    let url = make_url(~"file", None);

    image_cache_task.send(Prefetch(url.clone()));
    // no decode message

    let (chan, _port) = stream();
    image_cache_task.send(GetImage(url, chan));

    image_cache_task.exit();
    mock_resource_task.send(resource_task::Exit);
}

#[test]
fn should_not_request_url_from_resource_task_on_multiple_prefetches() {
    let url_requested = comm::Port();
    let url_requested_chan = url_requested.chan();

    let mock_resource_task = do mock_resource_task |response| {
        url_requested_chan.send(());
        response.send(resource_task::Done(result::Ok(())));
    };

    let image_cache_task = ImageCacheTask(mock_resource_task);
    let url = make_url(~"file", None);

    image_cache_task.send(Prefetch(url.clone()));
    image_cache_task.send(Prefetch(url));
    url_requested.recv();
    image_cache_task.exit();
    mock_resource_task.send(resource_task::Exit);
    assert!(!url_requested.peek())
}

#[test]
fn should_return_image_not_ready_if_data_has_not_arrived() {
    let (wait_chan, wait_port) = pipes::stream();

    let mock_resource_task = do mock_resource_task |response| {
        // Don't send the data until after the client requests
        // the image
        wait_port.recv();
        response.send(resource_task::Payload(test_image_bin()));
        response.send(resource_task::Done(result::Ok(())));
    };

    let image_cache_task = ImageCacheTask(mock_resource_task);
    let url = make_url(~"file", None);

    image_cache_task.send(Prefetch(url.clone()));
    image_cache_task.send(Decode(url.clone()));
    let (response_chan, response_port) = stream();
    image_cache_task.send(GetImage(url, response_chan));
    assert!(response_port.recv() == ImageNotReady);
    wait_chan.send(());
    image_cache_task.exit();
    mock_resource_task.send(resource_task::Exit);
}

#[test]
fn should_return_decoded_image_data_if_data_has_arrived() {
    let mock_resource_task = do mock_resource_task |response| {
        response.send(resource_task::Payload(test_image_bin()));
        response.send(resource_task::Done(result::Ok(())));
    };

    let image_cache_task = ImageCacheTask(mock_resource_task);
    let url = make_url(~"file", None);

    let wait_for_image = comm::Port();
    let wait_for_image_chan = wait_for_image.chan();

    image_cache_task.send(OnMsg(|msg| {
        match *msg {
          StoreImage(*) => wait_for_image_chan.send(()),
          _ => ()
        }
    }));

    image_cache_task.send(Prefetch(url.clone()));
    image_cache_task.send(Decode(url.clone()));

    // Wait until our mock resource task has sent the image to the image cache
    wait_for_image_chan.recv();

    let (response_chan, response_port) = stream();
    image_cache_task.send(GetImage(url, response_chan));
    match response_port.recv() {
      ImageReady(_) => (),
      _ => fail
    }

    image_cache_task.exit();
    mock_resource_task.send(resource_task::Exit);
}

#[test]
fn should_return_decoded_image_data_for_multiple_requests() {
    let mock_resource_task = do mock_resource_task |response| {
        response.send(resource_task::Payload(test_image_bin()));
        response.send(resource_task::Done(result::Ok(())));
    };

    let image_cache_task = ImageCacheTask(mock_resource_task);
    let url = make_url(~"file", None);

    let wait_for_image = comm::Port();
    let wait_for_image_chan = wait_for_image.chan();

    image_cache_task.send(OnMsg(|msg| {
        match *msg {
          StoreImage(*) => wait_for_image_chan.send(()),
          _ => ()
        }
    }));

    image_cache_task.send(Prefetch(url.clone()));
    image_cache_task.send(Decode(url.clone()));

    // Wait until our mock resource task has sent the image to the image cache
    wait_for_image.recv();

    for _ in iter::repeat(2) {
        let (response_chan, response_port) = stream();
        image_cache_task.send(GetImage(url.clone(), response_chan));
        match response_port.recv() {
          ImageReady(_) => (),
          _ => fail
        }
    }

    image_cache_task.exit();
    mock_resource_task.send(resource_task::Exit);
}

#[test]
fn should_not_request_image_from_resource_task_if_image_is_already_available() {
    let image_bin_sent = comm::Port();
    let image_bin_sent_chan = image_bin_sent.chan();

    let resource_task_exited = comm::Port();
    let resource_task_exited_chan = resource_task_exited.chan();

    let mock_resource_task = do spawn_listener |port: comm::Port<resource_task::ControlMsg>| {
        loop {
            match port.recv() {
                resource_task::Load(_, response) => {
                    response.send(resource_task::Payload(test_image_bin()));
                    response.send(resource_task::Done(result::Ok(())));
                    image_bin_sent_chan.send(());
                }
                resource_task::Exit => {
                    resource_task_exited_chan.send(());
                    break
                }
            }
        }
    };

    let image_cache_task = ImageCacheTask(mock_resource_task);
    let url = make_url(~"file", None);

    image_cache_task.send(Prefetch(url.clone()));

    // Wait until our mock resource task has sent the image to the image cache
    image_bin_sent.recv();

    image_cache_task.send(Prefetch(url.clone()));

    image_cache_task.exit();
    mock_resource_task.send(resource_task::Exit);

    resource_task_exited.recv();

    // Our resource task should not have received another request for the image
    // because it's already cached
    assert!(!image_bin_sent.peek());
}

#[test]
fn should_not_request_image_from_resource_task_if_image_fetch_already_failed() {
    let image_bin_sent = comm::Port();
    let image_bin_sent_chan = image_bin_sent.chan();

    let resource_task_exited = comm::Port();
    let resource_task_exited_chan = resource_task_exited.chan();

    let mock_resource_task = do spawn_listener |port: comm::Port<resource_task::ControlMsg>| {
        loop {
            match port.recv() {
                resource_task::Load(_, response) => {
                    response.send(resource_task::Payload(test_image_bin()));
                    response.send(resource_task::Done(result::Err(())));
                    image_bin_sent_chan.send(());
                }
                resource_task::Exit => {
                    resource_task_exited_chan.send(());
                    break
                }
            }
        }
    };

    let image_cache_task = ImageCacheTask(mock_resource_task);
    let url = make_url(~"file", None);

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
    assert!(!image_bin_sent.peek());
}

#[test]
fn should_return_failed_if_image_bin_cannot_be_fetched() {
    let mock_resource_task = do mock_resource_task |response| {
        response.send(resource_task::Payload(test_image_bin()));
        // ERROR fetching image
        response.send(resource_task::Done(result::Err(())));
    };

    let image_cache_task = ImageCacheTask(mock_resource_task);
    let url = make_url(~"file", None);

    let wait_for_prefetech = comm::Port();
    let wait_for_prefetech_chan = wait_for_prefetech.chan();

    image_cache_task.send(OnMsg(|msg| {
        match *msg {
          StorePrefetchedImageData(*) => wait_for_prefetech_chan.send(()),
          _ => ()
        }
    }));

    image_cache_task.send(Prefetch(url.clone()));
    image_cache_task.send(Decode(url.clone()));

    // Wait until our mock resource task has sent the image to the image cache
    wait_for_prefetech.recv();

    let (response_chan, response_port) = stream();
    image_cache_task.send(GetImage(url, response_chan));
    match response_port.recv() {
      ImageFailed => (),
      _ => fail
    }

    image_cache_task.exit();
    mock_resource_task.send(resource_task::Exit);
}

#[test]
fn should_return_failed_for_multiple_get_image_requests_if_image_bin_cannot_be_fetched() {
    let mock_resource_task = do mock_resource_task |response | {
        response.send(resource_task::Payload(test_image_bin()));
        // ERROR fetching image
        response.send(resource_task::Done(result::Err(())));
    };

    let image_cache_task = ImageCacheTask(mock_resource_task);
    let url = make_url(~"file", None);

    let wait_for_prefetech = comm::Port();
    let wait_for_prefetech_chan = wait_for_prefetech.chan();

    image_cache_task.send(OnMsg(|msg| {
        match *msg {
          StorePrefetchedImageData(*) => wait_for_prefetech_chan.send(()),
          _ => ()
        }
    }));

    image_cache_task.send(Prefetch(url.clone()));
    image_cache_task.send(Decode(url.clone()));

    // Wait until our mock resource task has sent the image to the image cache
    wait_for_prefetech.recv();

    let (response_chan, response_port) = stream();
    image_cache_task.send(GetImage(url.clone(), response_chan));
    match response_port.recv() {
      ImageFailed => (),
      _ => fail
    }

    // And ask again, we should get the same response
    let (response_chan, response_port) = stream();
    image_cache_task.send(GetImage(url, response_chan));
    match response_port.recv() {
      ImageFailed => (),
      _ => fail
    }

    image_cache_task.exit();
    mock_resource_task.send(resource_task::Exit);
}

#[test]
fn should_return_not_ready_if_image_is_still_decoding() {
    let (wait_to_decode_chan, wait_to_decode_port) = pipes::stream();

    let mock_resource_task = do mock_resource_task |response| {
        response.send(resource_task::Payload(test_image_bin()));
        response.send(resource_task::Done(result::Ok(())));
    };

    let wait_to_decode_port_cell = Cell(wait_to_decode_port);
    let decoder_factory = || {
        let wait_to_decode_port = wait_to_decode_port_cell.take();
        |data: &[u8]| {
            // Don't decode until after the client requests the image
            wait_to_decode_port.recv();
            load_from_memory(data)
        }
    };

    let image_cache_task = ImageCacheTask_(mock_resource_task, decoder_factory);
    let url = make_url(~"file", None);

    let wait_for_prefetech = comm::Port();
    let wait_for_prefetech_chan = wait_for_prefetech.chan();

    image_cache_task.send(OnMsg(|msg| {
        match *msg {
          StorePrefetchedImageData(*) => wait_for_prefetech_chan.send(()),
          _ => ()
        }
    }));

    image_cache_task.send(Prefetch(url.clone()));
    image_cache_task.send(Decode(url.clone()));

    // Wait until our mock resource task has sent the image to the image cache
    wait_for_prefetech.recv();

    // Make the request
    let (response_chan, response_port) = stream();
    image_cache_task.send(GetImage(url, response_chan));

    match response_port.recv() {
      ImageNotReady => (),
      _ => fail
    }

    // Now decode
    wait_to_decode_chan.send(());

    image_cache_task.exit();
    mock_resource_task.send(resource_task::Exit);
}

#[test]
fn should_return_failed_if_image_decode_fails() {
    let mock_resource_task = do mock_resource_task |response| {
        // Bogus data
        response.send(resource_task::Payload(~[]));
        response.send(resource_task::Done(result::Ok(())));
    };

    let image_cache_task = ImageCacheTask(mock_resource_task);
    let url = make_url(~"file", None);

    let wait_for_decode = comm::Port();
    let wait_for_decode_chan = wait_for_decode.chan();

    image_cache_task.send(OnMsg(|msg| {
        match *msg {
          StoreImage(*) => wait_for_decode_chan.send(()),
          _ => ()
        }
    }));

    image_cache_task.send(Prefetch(url.clone()));
    image_cache_task.send(Decode(url.clone()));

    // Wait until our mock resource task has sent the image to the image cache
    wait_for_decode.recv();

    // Make the request
    let (response_chan, response_port) = stream();
    image_cache_task.send(GetImage(url, response_chan));

    match response_port.recv() {
      ImageFailed => (),
      _ => fail
    }

    image_cache_task.exit();
    mock_resource_task.send(resource_task::Exit);
}

#[test]
fn should_return_image_on_wait_if_image_is_already_loaded() {
    let mock_resource_task = do mock_resource_task |response| {
        response.send(resource_task::Payload(test_image_bin()));
        response.send(resource_task::Done(result::Ok(())));
    };

    let image_cache_task = ImageCacheTask(mock_resource_task);
    let url = make_url(~"file", None);

    let wait_for_decode = comm::Port();
    let wait_for_decode_chan = wait_for_decode.chan();

    image_cache_task.send(OnMsg(|msg| {
        match *msg {
          StoreImage(*) => wait_for_decode_chan.send(()),
          _ => ()
        }
    }));

    image_cache_task.send(Prefetch(url.clone()));
    image_cache_task.send(Decode(url.clone()));

    // Wait until our mock resource task has sent the image to the image cache
    wait_for_decode.recv();

    let (response_chan, response_port) = stream();
    image_cache_task.send(WaitForImage(url, response_chan));
    match response_port.recv() {
      ImageReady(*) => (),
      _ => fail
    }

    image_cache_task.exit();
    mock_resource_task.send(resource_task::Exit);
}

#[test]
fn should_return_image_on_wait_if_image_is_not_yet_loaded() {
    let (wait_chan, wait_port) = pipes::stream();

    let mock_resource_task = do mock_resource_task |response| {
        wait_port.recv();
        response.send(resource_task::Payload(test_image_bin()));
        response.send(resource_task::Done(result::Ok(())));
    };

    let image_cache_task = ImageCacheTask(mock_resource_task);
    let url = make_url(~"file", None);

    image_cache_task.send(Prefetch(url.clone()));
    image_cache_task.send(Decode(url.clone()));

    let (response_chan, response_port) = stream();
    image_cache_task.send(WaitForImage(url, response_chan));

    wait_chan.send(());

    match response_port.recv() {
      ImageReady(*) => (),
      _ => fail
    }

    image_cache_task.exit();
    mock_resource_task.send(resource_task::Exit);
}

#[test]
fn should_return_image_failed_on_wait_if_image_fails_to_load() {
    let (wait_chan, wait_port) = pipes::stream();

    let mock_resource_task = do mock_resource_task |response| {
        wait_port.recv();
        response.send(resource_task::Payload(test_image_bin()));
        response.send(resource_task::Done(result::Err(())));
    };

    let image_cache_task = ImageCacheTask(mock_resource_task);
    let url = make_url(~"file", None);

    image_cache_task.send(Prefetch(url.clone()));
    image_cache_task.send(Decode(url.clone()));

    let (response_chan, response_port) = stream();
    image_cache_task.send(WaitForImage(url, response_chan));

    wait_chan.send(());

    match response_port.recv() {
      ImageFailed => (),
      _ => fail
    }

    image_cache_task.exit();
    mock_resource_task.send(resource_task::Exit);
}

#[test]
fn sync_cache_should_wait_for_images() {
    let mock_resource_task = do mock_resource_task |response| {
        response.send(resource_task::Payload(test_image_bin()));
        response.send(resource_task::Done(result::Ok(())));
    };

    let image_cache_task = SyncImageCacheTask(mock_resource_task);
    let url = make_url(~"file", None);

    image_cache_task.send(Prefetch(url.clone()));
    image_cache_task.send(Decode(url.clone()));

    let (response_chan, response_port) = stream();
    image_cache_task.send(GetImage(url, response_chan));
    match response_port.recv() {
      ImageReady(_) => (),
      _ => fail
    }

    image_cache_task.exit();
    mock_resource_task.send(resource_task::Exit);
}

