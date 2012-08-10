export Msg, Prefetch, GetImage, Exit;
export ImageResponseMsg, ImageReady, ImageNotReady;
export ImageCacheTask;
export image_cache_task;

import image::base::{ImageBuffer, SharedImageBuffer};
import std::net::url::url;
import util::url::{make_url, UrlMap, url_map};
import comm::{chan, port};
import task::spawn_listener;
import resource::resource_task;
import resource_task::ResourceTask;

enum Msg {
    Prefetch(url),
    GetImage(url, chan<ImageResponseMsg>),
    Exit
}

enum ImageResponseMsg {
    ImageReady(ImageBuffer),
    ImageNotReady
}

type ImageCacheTask = chan<Msg>;

fn image_cache_task(resource_task: ResourceTask) -> ImageCacheTask {
    do spawn_listener |from_client| {
        ImageCache {
            resource_task: resource_task,
            from_client: from_client,
            prefetch_map: url_map()
        }.run();
    }
}

struct ImageCache {
    resource_task: ResourceTask;
    from_client: port<Msg>;
    prefetch_map: UrlMap<PrefetchData>;
}

struct PrefetchData {
    response_port: @port<resource_task::ProgressMsg>;
    data: @mut ~[u8];
}

impl ImageCache {

    fn run() {

        loop {
            match self.from_client.recv() {
              Prefetch(url) => {
                if self.prefetch_map.contains_key(url) {
                    // We're already waiting for this image
                    again
                }
                let response_port = port();
                self.resource_task.send(resource_task::Load(url, response_port.chan()));

                let prefetch_data = PrefetchData {
                    response_port: @response_port,
                    data: @mut ~[]
                };

                self.prefetch_map.insert(url, prefetch_data);
              }
              GetImage(url, response) => {
                if self.prefetch_map.contains_key(url) {
                    response.send(ImageNotReady);
                } else {
                    fail ~"got a request for image data without prefetch";
                }
              }
              Exit => break
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
    let url = make_url(~"file", none);

    image_cache_task.send(Exit);
    mock_resource_task.send(resource_task::Exit);
}

#[test]
#[should_fail]
fn should_fail_if_unprefetched_image_is_requested() {

    let mock_resource_task = do spawn_listener |from_client| {
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

    image_cache_task.send(Prefetch(url));
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

    image_cache_task.send(Prefetch(url));
    let response_port = port();
    image_cache_task.send(GetImage(url, response_port.chan()));
    assert response_port.recv() == ImageNotReady;
    image_cache_task.send(Exit);
    mock_resource_task.send(resource_task::Exit);
}