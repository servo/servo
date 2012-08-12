export Msg, Prefetch, GetImage, Exit;
export ImageResponseMsg, ImageReady, ImageNotReady;
export ImageCacheTask;
export image_cache_task;

import image::base::{Image, load_from_memory, test_image_bin};
import std::net::url::url;
import util::url::{make_url, UrlMap, url_map};
import comm::{chan, port};
import task::{spawn, spawn_listener};
import resource::resource_task;
import resource_task::ResourceTask;
import std::arc::arc;
import clone_arc = std::arc::clone;

enum Msg {
    /// Tell the cache that we may need a particular image soon. Must be posted
    /// before GetImage
    Prefetch(url),
    /// Request an Image object for a URL
    GetImage(url, chan<ImageResponseMsg>),
    /// Used by the decoder tasks to post decoded images back to the cache
    StoreImage(url, arc<~Image>),
    Exit
}

enum ImageResponseMsg {
    ImageReady(arc<~Image>),
    ImageNotReady
}

type ImageCacheTask = chan<Msg>;

fn image_cache_task(resource_task: ResourceTask) -> ImageCacheTask {
    do spawn_listener |from_client| {
        ImageCache {
            resource_task: resource_task,
            from_client: from_client,
            state_map: url_map()
        }.run();
    }
}

struct ImageCache {
    /// A handle to the resource task for fetching the image binaries
    resource_task: ResourceTask;
    /// The port on which we'll receive client requests
    from_client: port<Msg>;
    /// The state of processsing an image for a URL
    state_map: UrlMap<ImageState>;
}

enum ImageState {
    Init,
    Prefetching(@PrefetchData),
    Decoding(@FutureData),
    Decoded(@arc<~Image>),
    Failed
}

struct PrefetchData {
    response_port: port<resource_task::ProgressMsg>;
    mut data: ~[u8];
}

struct FutureData {
    mut waiters: ~[chan<ImageResponseMsg>];
}

#[allow(non_implicitly_copyable_typarams)]
impl ImageCache {

    fn run() {

        loop {
            match self.from_client.recv() {
              Prefetch(url) => self.prefetch(url),
              GetImage(url, response) => self.get_image(url, response),
              StoreImage(url, image) => self.store_image(url, &image),
              Exit => break
            }
        }
    }

    /*priv*/ fn get_state(url: url) -> ImageState {
        match self.state_map.find(copy url) {
          some(state) => state,
          none => Init
        }
    }

    /*priv*/ fn set_state(url: url, state: ImageState) {
        self.state_map.insert(copy url, state);
    }

    /*priv*/ fn prefetch(url: url) {
        match self.get_state(url) {
          Init => {
            let response_port = port();
            self.resource_task.send(resource_task::Load(copy url, response_port.chan()));

            let prefetch_data = @PrefetchData {
                response_port: response_port,
                data: ~[]
            };

            self.set_state(url, Prefetching(prefetch_data));
          }

          Prefetching(*)
          | Decoding(*)
          | Decoded(*)
          | Failed => {
            // We've already begun working on this image
          }
        }
    }

    /*priv*/ fn get_image(url: url, response: chan<ImageResponseMsg>) {

        match self.get_state(url) {
          Init => fail ~"Request for image before prefetch",

          Prefetching(prefetch_data) => {

            let mut image_sent = false;

            while prefetch_data.response_port.peek() {
                match prefetch_data.response_port.recv() {
                  resource_task::Payload(data) => {
                    prefetch_data.data += data;
                  }
                  resource_task::Done(result::ok(*)) => {
                    // We've got the entire image binary
                    let mut data = ~[];
                    data <-> prefetch_data.data;
                    let data <- data; // freeze for capture

                    let to_cache = self.from_client.chan();

                    do spawn |copy url| {
                        let image = arc(~load_from_memory(data));
                        // Send the image to the original requester
                        response.send(ImageReady(clone_arc(&image)));
                        to_cache.send(StoreImage(copy url, clone_arc(&image)));
                    }

                    let future_data = @FutureData {
                        waiters: ~[]
                    };

                    self.set_state(copy url, Decoding(future_data));

                    image_sent = true;
                    break;
                  }
                  resource_task::Done(result::err(*)) => {
                    // There was an error loading the image binary. Put it
                    // in the error map so we remember the error for future
                    // requests.
                    self.set_state(copy url, Failed);
                    break;
                  }
                }
            }

            if !image_sent {
                response.send(ImageNotReady);
            }
          }

          Decoding(future_data) => {
            // We've started decoding this image but haven't recieved it back yet.
            // Put this client on the wait list
            vec::push(future_data.waiters, response);
          }

          Decoded(image) => {
            response.send(ImageReady(clone_arc(image)));
          }

          Failed => {
            response.send(ImageNotReady);
          }
        }
    }

    /*priv*/ fn store_image(url: url, image: &arc<~Image>) {

        match self.get_state(url) {
          Decoding(future_data) => {

            let mut waiters = ~[];
            waiters <-> future_data.waiters;

            // Send the image to all those who requested it while
            // it was being decoded
            for waiters.each |waiter| {
                waiter.send(ImageReady(clone_arc(image)))
            }

            self.set_state(url, Decoded(@clone_arc(image)));
          }

          Init
          | Prefetching(*)
          | Decoded(*)
          | Failed => {
            fail ~"incorrect state in store_image"
          }
        }

    }
}

#[test]
fn should_exit_on_request() {

    let mock_resource_task = do spawn_listener |from_client| {

        // infer me
        let from_client: port<resource_task::ControlMsg> = from_client;

        loop {
            match from_client.recv() {
              resource_task::Exit => break,
              _ => ()
            }
        }
    };

    let image_cache_task = image_cache_task(mock_resource_task);
    let _url = make_url(~"file", none);

    image_cache_task.send(Exit);
    mock_resource_task.send(resource_task::Exit);
}

#[test]
#[should_fail]
fn should_fail_if_unprefetched_image_is_requested() {

    let mock_resource_task = do spawn_listener |_from_client| {
    };

    let image_cache_task = image_cache_task(mock_resource_task);
    let url = make_url(~"file", none);

    let request = port();
    image_cache_task.send(GetImage(url, request.chan()));
    request.recv();
}

#[test]
fn should_request_url_from_resource_task_on_prefetch() {
    let url_requested = port();
    let url_requested_chan = url_requested.chan();

    let mock_resource_task = do spawn_listener |from_client| {

        // infer me
        let from_client: port<resource_task::ControlMsg> = from_client;

        loop {
            match from_client.recv() {
              resource_task::Load(url, _) => url_requested_chan.send(()),
              resource_task::Exit => break
            }
        }
    };

    let image_cache_task = image_cache_task(mock_resource_task);
    let url = make_url(~"file", none);

    image_cache_task.send(Prefetch(url));
    url_requested.recv();
    image_cache_task.send(Exit);
    mock_resource_task.send(resource_task::Exit);
}

#[test]
fn should_not_request_url_from_resource_task_on_multiple_prefetches() {
    let url_requested = port();
    let url_requested_chan = url_requested.chan();

    let mock_resource_task = do spawn_listener |from_client| {

        // infer me
        let from_client: port<resource_task::ControlMsg> = from_client;

        loop {
            match from_client.recv() {
              resource_task::Load(url, _) => url_requested_chan.send(()),
              resource_task::Exit => break
            }
        }
    };

    let image_cache_task = image_cache_task(mock_resource_task);
    let url = make_url(~"file", none);

    image_cache_task.send(Prefetch(copy url));
    image_cache_task.send(Prefetch(url));
    url_requested.recv();
    image_cache_task.send(Exit);
    mock_resource_task.send(resource_task::Exit);
    assert !url_requested.peek()
}

#[test]
fn should_return_image_not_ready_if_data_has_not_arrived() {
    let mock_resource_task = do spawn_listener |from_client| {

        // infer me
        let from_client: port<resource_task::ControlMsg> = from_client;

        loop {
            match from_client.recv() {
              resource_task::Exit => break,
              _ => ()
            }
        }
    };

    let image_cache_task = image_cache_task(mock_resource_task);
    let url = make_url(~"file", none);

    image_cache_task.send(Prefetch(copy url));
    let response_port = port();
    image_cache_task.send(GetImage(url, response_port.chan()));
    assert response_port.recv() == ImageNotReady;
    image_cache_task.send(Exit);
    mock_resource_task.send(resource_task::Exit);
}

#[test]
fn should_return_decoded_image_data_if_data_has_arrived() {

    let image_bin_sent = port();
    let image_bin_sent_chan = image_bin_sent.chan();

    let mock_resource_task = do spawn_listener |from_client| {

        // infer me
        let from_client: port<resource_task::ControlMsg> = from_client;

        loop {
            match from_client.recv() {
              resource_task::Load(_, response) => {
                response.send(resource_task::Payload(test_image_bin()));
                response.send(resource_task::Done(result::ok(())));
                image_bin_sent_chan.send(());
              }
              resource_task::Exit => break
            }
        }
    };

    let image_cache_task = image_cache_task(mock_resource_task);
    let url = make_url(~"file", none);

    image_cache_task.send(Prefetch(copy url));

    // Wait until our mock resource task has sent the image to the image cache
    image_bin_sent.recv();

    let response_port = port();
    image_cache_task.send(GetImage(url, response_port.chan()));
    match response_port.recv() {
      ImageReady(_) => (),
      _ => fail
    }

    image_cache_task.send(Exit);
    mock_resource_task.send(resource_task::Exit);
}

#[test]
fn should_return_decoded_image_data_for_multiple_requests() {

    let image_bin_sent = port();
    let image_bin_sent_chan = image_bin_sent.chan();

    let mock_resource_task = do spawn_listener |from_client| {

        // infer me
        let from_client: port<resource_task::ControlMsg> = from_client;

        loop {
            match from_client.recv() {
              resource_task::Load(_, response) => {
                response.send(resource_task::Payload(test_image_bin()));
                response.send(resource_task::Done(result::ok(())));
                image_bin_sent_chan.send(());
              }
              resource_task::Exit => break
            }
        }
    };

    let image_cache_task = image_cache_task(mock_resource_task);
    let url = make_url(~"file", none);

    image_cache_task.send(Prefetch(copy url));

    // Wait until our mock resource task has sent the image to the image cache
    image_bin_sent.recv();

    for iter::repeat(2) {
        let response_port = port();
        image_cache_task.send(GetImage(copy url, response_port.chan()));
        match response_port.recv() {
          ImageReady(_) => (),
          _ => fail
        }
    }

    image_cache_task.send(Exit);
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
        let from_client: port<resource_task::ControlMsg> = from_client;

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

    let image_cache_task = image_cache_task(mock_resource_task);
    let url = make_url(~"file", none);

    image_cache_task.send(Prefetch(copy url));

    // Wait until our mock resource task has sent the image to the image cache
    image_bin_sent.recv();

    let response_port = port();
    image_cache_task.send(GetImage(copy url, response_port.chan()));
    match response_port.recv() {
      ImageReady(_) => (),
      _ => fail
    }

    image_cache_task.send(Prefetch(copy url));

    let response_port = port();
    image_cache_task.send(GetImage(url, response_port.chan()));
    response_port.recv();

    image_cache_task.send(Exit);
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
        let from_client: port<resource_task::ControlMsg> = from_client;

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

    let image_cache_task = image_cache_task(mock_resource_task);
    let url = make_url(~"file", none);

    image_cache_task.send(Prefetch(copy url));

    // Wait until our mock resource task has sent the image to the image cache
    image_bin_sent.recv();

    let response_port = port();
    image_cache_task.send(GetImage(copy url, response_port.chan()));
    response_port.recv();

    image_cache_task.send(Prefetch(copy url));

    let response_port = port();
    image_cache_task.send(GetImage(url, response_port.chan()));
    response_port.recv();

    image_cache_task.send(Exit);
    mock_resource_task.send(resource_task::Exit);

    resource_task_exited.recv();

    // Our resource task should not have received another request for the image
    // because it's already cached
    assert !image_bin_sent.peek();
}

#[test]
fn should_return_not_ready_if_image_bin_cannot_be_fetched() {

    let image_bin_sent = port();
    let image_bin_sent_chan = image_bin_sent.chan();

    let mock_resource_task = do spawn_listener |from_client| {

        // infer me
        let from_client: port<resource_task::ControlMsg> = from_client;

        loop {
            match from_client.recv() {
              resource_task::Load(_, response) => {
                response.send(resource_task::Payload(test_image_bin()));
                // ERROR fetching image
                response.send(resource_task::Done(result::err(())));
                image_bin_sent_chan.send(());
              }
              resource_task::Exit => break
            }
        }
    };

    let image_cache_task = image_cache_task(mock_resource_task);
    let url = make_url(~"file", none);

    image_cache_task.send(Prefetch(copy url));

    // Wait until our mock resource task has sent the image to the image cache
    image_bin_sent.recv();

    let response_port = port();
    image_cache_task.send(GetImage(url, response_port.chan()));
    match response_port.recv() {
      ImageNotReady => (),
      _ => fail
    }

    image_cache_task.send(Exit);
    mock_resource_task.send(resource_task::Exit);
}

#[test]
fn should_return_not_ready_for_multiple_get_image_requests_if_image_bin_cannot_be_fetched() {

    let image_bin_sent = port();
    let image_bin_sent_chan = image_bin_sent.chan();

    let mock_resource_task = do spawn_listener |from_client| {

        // infer me
        let from_client: port<resource_task::ControlMsg> = from_client;

        loop {
            match from_client.recv() {
              resource_task::Load(_, response) => {
                response.send(resource_task::Payload(test_image_bin()));
                // ERROR fetching image
                response.send(resource_task::Done(result::err(())));
                image_bin_sent_chan.send(());
              }
              resource_task::Exit => break
            }
        }
    };

    let image_cache_task = image_cache_task(mock_resource_task);
    let url = make_url(~"file", none);

    image_cache_task.send(Prefetch(copy url));

    // Wait until our mock resource task has sent the image to the image cache
    image_bin_sent.recv();

    let response_port = port();
    image_cache_task.send(GetImage(copy url, response_port.chan()));
    match response_port.recv() {
      ImageNotReady => (),
      _ => fail
    }

    // And ask again, we should get the same response
    let response_port = port();
    image_cache_task.send(GetImage(url, response_port.chan()));
    match response_port.recv() {
      ImageNotReady => (),
      _ => fail
    }

    image_cache_task.send(Exit);
    mock_resource_task.send(resource_task::Exit);
}
