export Msg, Prefetch, Decode, GetImage, WaitForImage, Exit;
export ImageResponseMsg, ImageReady, ImageNotReady, ImageFailed;
export ImageCacheTask;
export ImageCacheTaskClient;
export SyncImageCacheTask;

import image::base::{Image, load_from_memory, test_image_bin};
import std::net::url::url;
import util::url::{make_url, UrlMap, url_map};
import comm::{Chan, Port, chan, port};
import task::{spawn, spawn_listener};
import resource::resource_task;
import resource_task::ResourceTask;
import std::arc::arc;
import clone_arc = std::arc::clone;
import std::cell::Cell;
import result::{result, ok, err};
import to_str::ToStr;

enum Msg {
    /// Tell the cache that we may need a particular image soon. Must be posted
    /// before Decode
    Prefetch(url),

    /// Used be the prefetch tasks to post back image binaries
    /*priv*/ StorePrefetchedImageData(url, result<Cell<~[u8]>, ()>),

    /// Tell the cache to decode an image. Must be posted before GetImage/WaitForImage
    Decode(url),

    /// Used by the decoder tasks to post decoded images back to the cache
    /*priv*/ StoreImage(url, option<arc<~Image>>),

    /// Request an Image object for a URL. If the image is not is not immediately
    /// available then ImageNotReady is returned.
    GetImage(url, Chan<ImageResponseMsg>),

    /// Wait for an image to become available (or fail to load).
    WaitForImage(url, Chan<ImageResponseMsg>),

    /// For testing
    /*priv*/ OnMsg(fn~(msg: &Msg)),

    /// Clients must wait for a response before shutting down the ResourceTask
    Exit(Chan<()>)
}

enum ImageResponseMsg {
    ImageReady(arc<~Image>),
    ImageNotReady,
    ImageFailed
}

type ImageCacheTask = Chan<Msg>;

type DecoderFactory = ~fn() -> ~fn(~[u8]) -> option<Image>;

fn ImageCacheTask(resource_task: ResourceTask) -> ImageCacheTask {
    ImageCacheTask_(resource_task, default_decoder_factory)
}

fn ImageCacheTask_(resource_task: ResourceTask, +decoder_factory: DecoderFactory) -> ImageCacheTask {
    // FIXME: Doing some dancing to avoid copying decoder_factory, our test
    // version of which contains an uncopyable type which rust will currently
    // copy unsoundly
    let decoder_factory_cell = Cell(move decoder_factory);
    do spawn_listener |from_client, move decoder_factory_cell| {
        ImageCache {
            resource_task: resource_task,
            decoder_factory: decoder_factory_cell.take(),
            from_client: from_client,
            state_map: url_map(),
            wait_map: url_map(),
            need_exit: none
        }.run();
    }
}

fn SyncImageCacheTask(resource_task: ResourceTask) -> ImageCacheTask {
    do spawn_listener |from_client: Port<Msg>| {
        let inner_cache = ImageCacheTask(resource_task);

        loop {
            let msg = from_client.recv();

            match msg {
              GetImage(url, response) => inner_cache.send(WaitForImage(url, response)),
              Exit(response) => {
                inner_cache.send(Exit(response));
                break;
              }
              _ => inner_cache.send(msg)
            }
        }
    }
}

struct ImageCache {
    /// A handle to the resource task for fetching the image binaries
    resource_task: ResourceTask;
    /// Creates image decoders
    decoder_factory: DecoderFactory;
    /// The port on which we'll receive client requests
    from_client: Port<Msg>;
    /// The state of processsing an image for a URL
    state_map: UrlMap<ImageState>;
    /// List of clients waiting on a WaitForImage response
    wait_map: UrlMap<@mut ~[Chan<ImageResponseMsg>]>;
    mut need_exit: option<Chan<()>>;
}

enum ImageState {
    Init,
    Prefetching(AfterPrefetch),
    Prefetched(@Cell<~[u8]>),
    Decoding,
    Decoded(@arc<~Image>),
    Failed
}

enum AfterPrefetch {
    DoDecode,
    DoNotDecode
}

#[allow(non_implicitly_copyable_typarams)]
impl ImageCache {

    fn run() {

        let mut msg_handlers: ~[fn~(msg: &Msg)] = ~[];

        loop {
            let msg = self.from_client.recv();

            for msg_handlers.each |handler| { handler(&msg) }

            #debug("image_cache_task: received: %?", msg);

            // FIXME: Need to move out the urls
            match msg {
              Prefetch(url) => self.prefetch(copy url),
              StorePrefetchedImageData(url, data) => self.store_prefetched_image_data(copy url, &data),
              Decode(url) => self.decode(copy url),
              StoreImage(url, image) => self.store_image(copy url, &image),
              GetImage(url, response) => self.get_image(copy url, response),
              WaitForImage(url, response) => self.wait_for_image(copy url, response),
              OnMsg(handler) => msg_handlers += [copy handler],
              Exit(response) => {
                assert self.need_exit.is_none();
                self.need_exit = some(response);
              }
            }

            match copy self.need_exit {
              some(response) => {
                // Wait until we have no outstanding requests and subtasks
                // before exiting
                let mut can_exit = true;
                for self.state_map.each_value |state| {
                    match state {
                      Prefetching(*) => can_exit = false,
                      Decoding => can_exit = false,

                      Init
                      | Prefetched(*)
                      | Decoded(*)
                      | Failed => ()
                    }
                }

                if can_exit {
                    response.send(());
                    break;
                }
              }
              none => ()
            }
        }
    }

    /*priv*/ fn get_state(+url: url) -> ImageState {
        match self.state_map.find(url) {
          some(state) => state,
          none => Init
        }
    }

    /*priv*/ fn set_state(+url: url, state: ImageState) {
        self.state_map.insert(url, state);
    }

    /*priv*/ fn prefetch(+url: url) {
        match self.get_state(copy url) {
          Init => {
            let to_cache = self.from_client.chan();
            let resource_task = self.resource_task;
            let url_cell = Cell(copy url);

            do spawn |move url_cell| {
                let url = url_cell.take();
                #debug("image_cache_task: started fetch for %s", url.to_str());

                let image = load_image_data(copy url, resource_task);

                let result = if image.is_ok() {
                    ok(Cell(result::unwrap(image)))
                } else {
                    err(())
                };
                to_cache.send(StorePrefetchedImageData(copy url, result));
                #debug("image_cache_task: ended fetch for %s", (copy url).to_str());
            }

            self.set_state(url, Prefetching(DoNotDecode));
          }

          Prefetching(*)
          | Prefetched(*)
          | Decoding
          | Decoded(*)
          | Failed => {
            // We've already begun working on this image
          }
        }
    }

    /*priv*/ fn store_prefetched_image_data(+url: url, data: &result<Cell<~[u8]>, ()>) {
        match self.get_state(copy url) {
          Prefetching(next_step) => {
            match *data {
              ok(data_cell) => {
                let data = data_cell.take();
                self.set_state(copy url, Prefetched(@Cell(data)));
                if next_step == DoDecode {
                    self.decode(url);
                }
              }
              err(*) => {
                self.set_state(copy url, Failed);
                self.purge_waiters(url, || ImageFailed);
              }
            }
          }

          Init
          | Prefetched(*)
          | Decoding
          | Decoded(*)
          | Failed => {
            fail ~"wrong state for storing prefetched image"
          }
        }
    }

    /*priv*/ fn decode(+url: url) {

        match self.get_state(copy url) {
          Init => fail ~"decoding image before prefetch",

          Prefetching(DoNotDecode) => {
            // We don't have the data yet, queue up the decode
            self.set_state(url, Prefetching(DoDecode))
          }

          Prefetching(DoDecode) => {
            // We don't have the data yet, but the decode request is queued up
          }

          Prefetched(data_cell) => {
            assert !data_cell.is_empty();

            let data = data_cell.take();
            let to_cache = self.from_client.chan();
            let url_cell = Cell(copy url);
            let decode = self.decoder_factory();

            do spawn |move url_cell, move decode| {
                let url = url_cell.take();
                #debug("image_cache_task: started image decode for %s", url.to_str());
                let image = decode(data);
                let image = if image.is_some() {
                    some(arc(~option::unwrap(image)))
                } else {
                    none
                };
                to_cache.send(StoreImage(copy url, move image));
                #debug("image_cache_task: ended image decode for %s", url.to_str());
            }

            self.set_state(url, Decoding);
          }

          Decoding
          | Decoded(*)
          | Failed => {
            // We've already begun decoding
          }
        }
    }

    /*priv*/ fn store_image(+url: url, image: &option<arc<~Image>>) {

        match self.get_state(copy url) {
          Decoding => {
            match *image {
              some(image) => {
                self.set_state(copy url, Decoded(@clone_arc(&image)));
                self.purge_waiters(url, || ImageReady(clone_arc(&image)) );
              }
              none => {
                self.set_state(copy url, Failed);
                self.purge_waiters(url, || ImageFailed );
              }
            }
          }

          Init
          | Prefetching(*)
          | Prefetched(*)
          | Decoded(*)
          | Failed => {
            fail ~"incorrect state in store_image"
          }
        }

    }

    /*priv*/ fn purge_waiters(+url: url, f: fn() -> ImageResponseMsg) {
        match self.wait_map.find(copy url) {
          some(@waiters) => {
            for waiters.each |response| {
                response.send(f());
            }
            self.wait_map.remove(url);
          }
          none => ()
        }
    }


    /*priv*/ fn get_image(+url: url, response: Chan<ImageResponseMsg>) {

        match self.get_state(copy url) {
          Init => fail ~"request for image before prefetch",

          Prefetching(DoDecode) => {
            response.send(ImageNotReady);
          }

          Prefetching(DoNotDecode)
          | Prefetched(*) => fail ~"request for image before decode",

          Decoding => {
            response.send(ImageNotReady)
          }

          Decoded(image) => {
            response.send(ImageReady(clone_arc(image)));
          }

          Failed => {
            response.send(ImageFailed);
          }
        }
    }

    /*priv*/ fn wait_for_image(+url: url, response: Chan<ImageResponseMsg>) {

        match self.get_state(copy url) {
          Init => fail ~"request for image before prefetch",

          Prefetching(DoNotDecode)
          | Prefetched(*) => fail ~"request for image before decode",

          Prefetching(DoDecode)
          | Decoding => {
            // We don't have this image yet
            match self.wait_map.find(copy url) {
              some(waiters) => {
                vec::push(*waiters, response);
              }
              none => {
                self.wait_map.insert(url, @mut ~[response]);
              }
            }
          }

          Decoded(image) => {
            response.send(ImageReady(clone_arc(image)));
          }

          Failed => {
            response.send(ImageFailed);
          }
        }
    }

}


trait ImageCacheTaskClient {
    fn exit();
}

impl ImageCacheTask: ImageCacheTaskClient {

    fn exit() {
        let response = port();
        self.send(Exit(response.chan()));
        response.recv();
    }

}

fn load_image_data(+url: url, resource_task: ResourceTask) -> result<~[u8], ()> {
    let response_port = port();
    resource_task.send(resource_task::Load(url, response_port.chan()));

    let mut image_data = ~[];

    loop {
        match response_port.recv() {
          resource_task::Payload(data) => {
            image_data += data;
          }
          resource_task::Done(result::ok(*)) => {
            return ok(image_data);
          }
          resource_task::Done(result::err(*)) => {
            return err(());
          }
        }
    }
}

fn default_decoder_factory() -> ~fn(~[u8]) -> option<Image> {
    fn~(data: ~[u8]) -> option<Image> { load_from_memory(data) }
}

#[cfg(test)]
fn mock_resource_task(+on_load: ~fn(resource: Chan<resource_task::ProgressMsg>)) -> ResourceTask {
    do spawn_listener |from_client, move on_load| {

        // infer me
        let from_client: Port<resource_task::ControlMsg> = from_client;

        loop {
            match from_client.recv() {
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
    let _url = make_url(~"file", none);

    image_cache_task.exit();
    mock_resource_task.send(resource_task::Exit);
}

#[test]
#[should_fail]
fn should_fail_if_unprefetched_image_is_requested() {

    let mock_resource_task = mock_resource_task(|_response| () );

    let image_cache_task = ImageCacheTask(mock_resource_task);
    let url = make_url(~"file", none);

    let request = port();
    image_cache_task.send(GetImage(url, request.chan()));
    request.recv();
}

#[test]
fn should_request_url_from_resource_task_on_prefetch() {
    let url_requested = port();
    let url_requested_chan = url_requested.chan();

    let mock_resource_task = do mock_resource_task |response| {
        url_requested_chan.send(());
        response.send(resource_task::Done(result::ok(())));
    };

    let image_cache_task = ImageCacheTask(mock_resource_task);
    let url = make_url(~"file", none);

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
    let url = make_url(~"file", none);

    image_cache_task.send(Decode(url));
    image_cache_task.exit();
}

#[test]
#[should_fail]
fn should_fail_if_requesting_image_before_requesting_decode() {

    let mock_resource_task = do mock_resource_task |response| {
        response.send(resource_task::Done(result::ok(())));
    };

    let image_cache_task = ImageCacheTask(mock_resource_task);
    let url = make_url(~"file", none);

    image_cache_task.send(Prefetch(copy url));
    // no decode message

    let response_port = port();
    image_cache_task.send(GetImage(url, response_port.chan()));

    image_cache_task.exit();
    mock_resource_task.send(resource_task::Exit);
}

#[test]
fn should_not_request_url_from_resource_task_on_multiple_prefetches() {
    let url_requested = port();
    let url_requested_chan = url_requested.chan();

    let mock_resource_task = do mock_resource_task |response| {
        url_requested_chan.send(());
        response.send(resource_task::Done(result::ok(())));
    };

    let image_cache_task = ImageCacheTask(mock_resource_task);
    let url = make_url(~"file", none);

    image_cache_task.send(Prefetch(copy url));
    image_cache_task.send(Prefetch(url));
    url_requested.recv();
    image_cache_task.exit();
    mock_resource_task.send(resource_task::Exit);
    assert !url_requested.peek()
}

#[test]
fn should_return_image_not_ready_if_data_has_not_arrived() {

    let (wait_chan, wait_port) = pipes::stream();

    let mock_resource_task = do mock_resource_task |response| {
        // Don't send the data until after the client requests
        // the image
        wait_port.recv();
        response.send(resource_task::Payload(test_image_bin()));
        response.send(resource_task::Done(result::ok(())));
    };

    let image_cache_task = ImageCacheTask(mock_resource_task);
    let url = make_url(~"file", none);

    image_cache_task.send(Prefetch(copy url));
    image_cache_task.send(Decode(copy url));
    let response_port = port();
    image_cache_task.send(GetImage(url, response_port.chan()));
    assert response_port.recv() == ImageNotReady;
    wait_chan.send(());
    image_cache_task.exit();
    mock_resource_task.send(resource_task::Exit);
}

#[test]
fn should_return_decoded_image_data_if_data_has_arrived() {

    let mock_resource_task = do mock_resource_task |response| {
        response.send(resource_task::Payload(test_image_bin()));
        response.send(resource_task::Done(result::ok(())));
    };

    let image_cache_task = ImageCacheTask(mock_resource_task);
    let url = make_url(~"file", none);

    let wait_for_image = port();
    let wait_for_image_chan = wait_for_image.chan();

    image_cache_task.send(OnMsg(|msg| {
        match *msg {
          StoreImage(*) => wait_for_image_chan.send(()),
          _ => ()
        }
    }));

    image_cache_task.send(Prefetch(copy url));
    image_cache_task.send(Decode(copy url));

    // Wait until our mock resource task has sent the image to the image cache
    wait_for_image_chan.recv();

    let response_port = port();
    image_cache_task.send(GetImage(url, response_port.chan()));
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
        response.send(resource_task::Done(result::ok(())));
    };

    let image_cache_task = ImageCacheTask(mock_resource_task);
    let url = make_url(~"file", none);

    let wait_for_image = port();
    let wait_for_image_chan = wait_for_image.chan();

    image_cache_task.send(OnMsg(|msg| {
        match *msg {
          StoreImage(*) => wait_for_image_chan.send(()),
          _ => ()
        }
    }));

    image_cache_task.send(Prefetch(copy url));
    image_cache_task.send(Decode(copy url));

    // Wait until our mock resource task has sent the image to the image cache
    wait_for_image.recv();

    for iter::repeat(2) {
        let response_port = port();
        image_cache_task.send(GetImage(copy url, response_port.chan()));
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

    let image_bin_sent = port();
    let image_bin_sent_chan = image_bin_sent.chan();

    let resource_task_exited = port();
    let resource_task_exited_chan = resource_task_exited.chan();

    let mock_resource_task = do spawn_listener |from_client| {

        // infer me
        let from_client: Port<resource_task::ControlMsg> = from_client;

        loop {
            match from_client.recv() {
              resource_task::Load(_, response) => {
                response.send(resource_task::Payload(test_image_bin()));
                response.send(resource_task::Done(result::ok(())));
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
    let url = make_url(~"file", none);

    image_cache_task.send(Prefetch(copy url));

    // Wait until our mock resource task has sent the image to the image cache
    image_bin_sent.recv();

    image_cache_task.send(Prefetch(copy url));

    image_cache_task.exit();
    mock_resource_task.send(resource_task::Exit);

    resource_task_exited.recv();

    // Our resource task should not have received another request for the image
    // because it's already cached
    assert !image_bin_sent.peek();
}

#[test]
fn should_not_request_image_from_resource_task_if_image_fetch_already_failed() {

    let image_bin_sent = port();
    let image_bin_sent_chan = image_bin_sent.chan();

    let resource_task_exited = port();
    let resource_task_exited_chan = resource_task_exited.chan();

    let mock_resource_task = do spawn_listener |from_client| {

        // infer me
        let from_client: Port<resource_task::ControlMsg> = from_client;

        loop {
            match from_client.recv() {
              resource_task::Load(_, response) => {
                response.send(resource_task::Payload(test_image_bin()));
                response.send(resource_task::Done(result::err(())));
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
    let url = make_url(~"file", none);

    image_cache_task.send(Prefetch(copy url));
    image_cache_task.send(Decode(copy url));

    // Wait until our mock resource task has sent the image to the image cache
    image_bin_sent.recv();

    image_cache_task.send(Prefetch(copy url));
    image_cache_task.send(Decode(copy url));

    image_cache_task.exit();
    mock_resource_task.send(resource_task::Exit);

    resource_task_exited.recv();

    // Our resource task should not have received another request for the image
    // because it's already cached
    assert !image_bin_sent.peek();
}

#[test]
fn should_return_failed_if_image_bin_cannot_be_fetched() {

    let mock_resource_task = do mock_resource_task |response| {
        response.send(resource_task::Payload(test_image_bin()));
        // ERROR fetching image
        response.send(resource_task::Done(result::err(())));
    };

    let image_cache_task = ImageCacheTask(mock_resource_task);
    let url = make_url(~"file", none);

    let wait_for_prefetech = port();
    let wait_for_prefetech_chan = wait_for_prefetech.chan();

    image_cache_task.send(OnMsg(|msg| {
        match *msg {
          StorePrefetchedImageData(*) => wait_for_prefetech_chan.send(()),
          _ => ()
        }
    }));

    image_cache_task.send(Prefetch(copy url));
    image_cache_task.send(Decode(copy url));

    // Wait until our mock resource task has sent the image to the image cache
    wait_for_prefetech.recv();

    let response_port = port();
    image_cache_task.send(GetImage(url, response_port.chan()));
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
        response.send(resource_task::Done(result::err(())));
    };

    let image_cache_task = ImageCacheTask(mock_resource_task);
    let url = make_url(~"file", none);

    let wait_for_prefetech = port();
    let wait_for_prefetech_chan = wait_for_prefetech.chan();

    image_cache_task.send(OnMsg(|msg| {
        match *msg {
          StorePrefetchedImageData(*) => wait_for_prefetech_chan.send(()),
          _ => ()
        }
    }));

    image_cache_task.send(Prefetch(copy url));
    image_cache_task.send(Decode(copy url));

    // Wait until our mock resource task has sent the image to the image cache
    wait_for_prefetech.recv();

    let response_port = port();
    image_cache_task.send(GetImage(copy url, response_port.chan()));
    match response_port.recv() {
      ImageFailed => (),
      _ => fail
    }

    // And ask again, we should get the same response
    let response_port = port();
    image_cache_task.send(GetImage(url, response_port.chan()));
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
        response.send(resource_task::Done(result::ok(())));
    };

    let wait_to_decode_port_cell = Cell(wait_to_decode_port);
    let decoder_factory = fn~(move wait_to_decode_port_cell) -> ~fn(~[u8]) -> option<Image> {
        let wait_to_decode_port = wait_to_decode_port_cell.take();
        fn~(data: ~[u8], move wait_to_decode_port) -> option<Image> {
            // Don't decode until after the client requests the image
            wait_to_decode_port.recv();
            load_from_memory(data)
        }
    };

    let image_cache_task = ImageCacheTask_(mock_resource_task, decoder_factory);
    let url = make_url(~"file", none);

    let wait_for_prefetech = port();
    let wait_for_prefetech_chan = wait_for_prefetech.chan();

    image_cache_task.send(OnMsg(|msg| {
        match *msg {
          StorePrefetchedImageData(*) => wait_for_prefetech_chan.send(()),
          _ => ()
        }
    }));

    image_cache_task.send(Prefetch(copy url));
    image_cache_task.send(Decode(copy url));

    // Wait until our mock resource task has sent the image to the image cache
    wait_for_prefetech.recv();

    // Make the request
    let response_port = port();
    image_cache_task.send(GetImage(url, response_port.chan()));

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
        response.send(resource_task::Done(result::ok(())));
    };

    let image_cache_task = ImageCacheTask(mock_resource_task);
    let url = make_url(~"file", none);

    let wait_for_decode = port();
    let wait_for_decode_chan = wait_for_decode.chan();

    image_cache_task.send(OnMsg(|msg| {
        match *msg {
          StoreImage(*) => wait_for_decode_chan.send(()),
          _ => ()
        }
    }));

    image_cache_task.send(Prefetch(copy url));
    image_cache_task.send(Decode(copy url));

    // Wait until our mock resource task has sent the image to the image cache
    wait_for_decode.recv();

    // Make the request
    let response_port = port();
    image_cache_task.send(GetImage(url, response_port.chan()));

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
        response.send(resource_task::Done(result::ok(())));
    };

    let image_cache_task = ImageCacheTask(mock_resource_task);
    let url = make_url(~"file", none);

    let wait_for_decode = port();
    let wait_for_decode_chan = wait_for_decode.chan();

    image_cache_task.send(OnMsg(|msg| {
        match *msg {
          StoreImage(*) => wait_for_decode_chan.send(()),
          _ => ()
        }
    }));

    image_cache_task.send(Prefetch(copy url));
    image_cache_task.send(Decode(copy url));

    // Wait until our mock resource task has sent the image to the image cache
    wait_for_decode.recv();

    let response_port = port();
    image_cache_task.send(WaitForImage(url, response_port.chan()));
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
        response.send(resource_task::Done(result::ok(())));
    };

    let image_cache_task = ImageCacheTask(mock_resource_task);
    let url = make_url(~"file", none);

    image_cache_task.send(Prefetch(copy url));
    image_cache_task.send(Decode(copy url));

    let response_port = port();
    image_cache_task.send(WaitForImage(url, response_port.chan()));

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
        response.send(resource_task::Done(result::err(())));
    };

    let image_cache_task = ImageCacheTask(mock_resource_task);
    let url = make_url(~"file", none);

    image_cache_task.send(Prefetch(copy url));
    image_cache_task.send(Decode(copy url));

    let response_port = port();
    image_cache_task.send(WaitForImage(url, response_port.chan()));

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
        response.send(resource_task::Done(result::ok(())));
    };

    let image_cache_task = SyncImageCacheTask(mock_resource_task);
    let url = make_url(~"file", none);

    image_cache_task.send(Prefetch(copy url));
    image_cache_task.send(Decode(copy url));

    let response_port = port();
    image_cache_task.send(GetImage(url, response_port.chan()));
    match response_port.recv() {
      ImageReady(_) => (),
      _ => fail
    }

    image_cache_task.exit();
    mock_resource_task.send(resource_task::Exit);
}