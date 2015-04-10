/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use net::image_cache_task::*;
use net_traits::image_cache_task::ImageResponseMsg::*;
use net_traits::image_cache_task::Msg::*;

use net::resource_task::start_sending;
use net_traits::{ControlMsg, Metadata, ProgressMsg, ResourceTask};
use net_traits::image_cache_task::{ImageCacheTask, ImageCacheTaskClient, ImageResponseMsg, Msg};
use net_traits::ProgressMsg::{Payload, Done};
use profile::time;
use std::sync::mpsc::{Sender, channel, Receiver};
use url::Url;
use util::taskpool::TaskPool;

static TEST_IMAGE: &'static [u8] = include_bytes!("test.jpeg");

pub fn test_image_bin() -> Vec<u8> {
    TEST_IMAGE.iter().map(|&x| x).collect()
}

trait ImageCacheTaskHelper {
    fn wait_for_store(&self) -> Receiver<()>;
    fn wait_for_store_prefetched(&self) -> Receiver<()>;
}

impl ImageCacheTaskHelper for ImageCacheTask {
    fn wait_for_store(&self) -> Receiver<()> {
        let (chan, port) = channel();
        self.send(Msg::WaitForStore(chan));
        port
    }

    fn wait_for_store_prefetched(&self) -> Receiver<()> {
        let (chan, port) = channel();
        self.send(Msg::WaitForStorePrefetched(chan));
        port
    }
}

trait Closure {
    fn invoke(&self, _response: Sender<ProgressMsg>) { }
}
struct DoesNothing;
impl Closure for DoesNothing { }

struct JustSendOK {
    url_requested_chan: Sender<()>,
}
impl Closure for JustSendOK {
    fn invoke(&self, response: Sender<ProgressMsg>) {
        self.url_requested_chan.send(()).unwrap();
        response.send(Done(Ok(()))).unwrap();
    }
}

struct SendTestImage;
impl Closure for SendTestImage {
    fn invoke(&self, response: Sender<ProgressMsg>) {
        response.send(Payload(test_image_bin())).unwrap();
        response.send(Done(Ok(()))).unwrap();
    }
}

struct SendBogusImage;
impl Closure for SendBogusImage {
    fn invoke(&self, response: Sender<ProgressMsg>) {
        response.send(Payload(vec!())).unwrap();
        response.send(Done(Ok(()))).unwrap();
    }
}

struct SendTestImageErr;
impl Closure for SendTestImageErr {
    fn invoke(&self, response: Sender<ProgressMsg>) {
        response.send(Payload(test_image_bin())).unwrap();
        response.send(Done(Err("".to_string()))).unwrap();
    }
}

struct WaitSendTestImage {
    wait_port: Receiver<()>,
}
impl Closure for WaitSendTestImage {
    fn invoke(&self, response: Sender<ProgressMsg>) {
        // Don't send the data until after the client requests
        // the image
        self.wait_port.recv().unwrap();
        response.send(Payload(test_image_bin())).unwrap();
        response.send(Done(Ok(()))).unwrap();
    }
}

struct WaitSendTestImageErr {
    wait_port: Receiver<()>,
}
impl Closure for WaitSendTestImageErr {
    fn invoke(&self, response: Sender<ProgressMsg>) {
        // Don't send the data until after the client requests
        // the image
        self.wait_port.recv().unwrap();
        response.send(Payload(test_image_bin())).unwrap();
        response.send(Done(Err("".to_string()))).unwrap();
    }
}

fn mock_resource_task<T: Closure + Send + 'static>(on_load: Box<T>) -> ResourceTask {
    spawn_listener(move |port: Receiver<ControlMsg>| {
        loop {
            match port.recv().unwrap() {
                ControlMsg::Load(response) => {
                    let chan = start_sending(response.consumer, Metadata::default(
                        Url::parse("file:///fake").unwrap()));
                    on_load.invoke(chan);
                }
                ControlMsg::Exit => break,
                _ => {}
            }
        }
    })
}

fn profiler() -> time::ProfilerChan {
    time::Profiler::create(None)
}

#[test]
fn should_exit_on_request() {
    let mock_resource_task = mock_resource_task(Box::new(DoesNothing));

    let image_cache_task: ImageCacheTask = ImageCacheTaskFactory::new(mock_resource_task.clone(),
                                                                      TaskPool::new(4), profiler(),
                                                                      LoadPlaceholder::Ignore);

    image_cache_task.exit();
    mock_resource_task.send(ControlMsg::Exit).unwrap();
}

#[test]
#[should_panic]
fn should_panic_if_unprefetched_image_is_requested() {
    let mock_resource_task = mock_resource_task(Box::new(DoesNothing));

    let image_cache_task: ImageCacheTask = ImageCacheTaskFactory::new(mock_resource_task.clone(),
                                                                      TaskPool::new(4), profiler(),
                                                                      LoadPlaceholder::Preload);
    let url = Url::parse("file:///").unwrap();

    let (chan, port) = channel();
    image_cache_task.send(Msg::GetImage(url, chan));
    port.recv().unwrap();
}

#[test]
fn should_request_url_from_resource_task_on_prefetch() {
    let (url_requested_chan, url_requested) = channel();

    let mock_resource_task = mock_resource_task(Box::new(JustSendOK { url_requested_chan: url_requested_chan}));

    let image_cache_task: ImageCacheTask = ImageCacheTaskFactory::new(mock_resource_task.clone(),
                                                                      TaskPool::new(4), profiler(),
                                                                      LoadPlaceholder::Preload);
    let url = Url::parse("file:///").unwrap();

    image_cache_task.send(Prefetch(url));
    url_requested.recv().unwrap();
    image_cache_task.exit();
    mock_resource_task.send(ControlMsg::Exit).unwrap();
}

#[test]
fn should_not_request_url_from_resource_task_on_multiple_prefetches() {
    let (url_requested_chan, url_requested) = channel();

    let mock_resource_task = mock_resource_task(Box::new(JustSendOK { url_requested_chan: url_requested_chan}));

    let image_cache_task: ImageCacheTask = ImageCacheTaskFactory::new(mock_resource_task.clone(),
                                                                      TaskPool::new(4), profiler(),
                                                                      LoadPlaceholder::Ignore);
    let url = Url::parse("file:///").unwrap();

    image_cache_task.send(Prefetch(url.clone()));
    image_cache_task.send(Prefetch(url));
    url_requested.recv().unwrap();
    image_cache_task.exit();
    mock_resource_task.send(ControlMsg::Exit).unwrap();
    match url_requested.try_recv() {
        Err(_) => (),
        Ok(_) => panic!(),
    };
}

#[test]
fn should_return_image_not_ready_if_data_has_not_arrived() {
    let (wait_chan, wait_port) = channel();

    let mock_resource_task = mock_resource_task(Box::new(WaitSendTestImage{wait_port: wait_port}));

    let image_cache_task: ImageCacheTask = ImageCacheTaskFactory::new(mock_resource_task.clone(),
                                                                      TaskPool::new(4), profiler(),
                                                                      LoadPlaceholder::Ignore);
    let url = Url::parse("file:///").unwrap();

    image_cache_task.send(Prefetch(url.clone()));
    image_cache_task.send(Decode(url.clone()));
    let (response_chan, response_port) = channel();
    image_cache_task.send(Msg::GetImage(url, response_chan));
    assert!(response_port.recv().unwrap() == ImageResponseMsg::ImageNotReady);
    wait_chan.send(()).unwrap();
    image_cache_task.exit();
    mock_resource_task.send(ControlMsg::Exit).unwrap();
}

#[test]
fn should_return_decoded_image_data_if_data_has_arrived() {
    let mock_resource_task = mock_resource_task(Box::new(SendTestImage));

    let image_cache_task: ImageCacheTask = ImageCacheTaskFactory::new(mock_resource_task.clone(),
                                                                      TaskPool::new(4), profiler(),
                                                                      LoadPlaceholder::Preload);
    let url = Url::parse("file:///").unwrap();

    let join_port = image_cache_task.wait_for_store();

    image_cache_task.send(Prefetch(url.clone()));
    image_cache_task.send(Decode(url.clone()));

    // Wait until our mock resource task has sent the image to the image cache
    join_port.recv().unwrap();

    let (response_chan, response_port) = channel();
    image_cache_task.send(Msg::GetImage(url, response_chan));
    match response_port.recv().unwrap() {
      ImageResponseMsg::ImageReady(_) => (),
      _ => panic!("bleh")
    }

    image_cache_task.exit();
    mock_resource_task.send(ControlMsg::Exit).unwrap();
}

#[test]
fn should_return_decoded_image_data_for_multiple_requests() {
    let mock_resource_task = mock_resource_task(Box::new(SendTestImage));

    let image_cache_task: ImageCacheTask = ImageCacheTaskFactory::new(mock_resource_task.clone(),
                                                                      TaskPool::new(4), profiler(),
                                                                      LoadPlaceholder::Preload);
    let url = Url::parse("file:///").unwrap();

    let join_port = image_cache_task.wait_for_store();

    image_cache_task.send(Prefetch(url.clone()));
    image_cache_task.send(Decode(url.clone()));

    // Wait until our mock resource task has sent the image to the image cache
    join_port.recv().unwrap();

    for _ in 0..2 {
        let (response_chan, response_port) = channel();
        image_cache_task.send(Msg::GetImage(url.clone(), response_chan));
        match response_port.recv().unwrap() {
          ImageResponseMsg::ImageReady(_) => (),
          _ => panic!("bleh")
        }
    }

    image_cache_task.exit();
    mock_resource_task.send(ControlMsg::Exit).unwrap();
}

#[test]
fn should_not_request_image_from_resource_task_if_image_is_already_available() {
    let (image_bin_sent_chan, image_bin_sent) = channel();

    let (resource_task_exited_chan, resource_task_exited) = channel();

    let mock_resource_task = spawn_listener(move |port: Receiver<ControlMsg>| {
        loop {
            match port.recv().unwrap() {
                ControlMsg::Load(response) => {
                    let chan = start_sending(response.consumer, Metadata::default(
                        Url::parse("file:///fake").unwrap()));
                    chan.send(Payload(test_image_bin()));
                    chan.send(Done(Ok(())));
                    image_bin_sent_chan.send(());
                }
                ControlMsg::Exit => {
                    resource_task_exited_chan.send(());
                    break
                }
                _ => {}
            }
        }
    });

    let image_cache_task: ImageCacheTask = ImageCacheTaskFactory::new(mock_resource_task.clone(),
                                                                      TaskPool::new(4), profiler(),
                                                                      LoadPlaceholder::Ignore);
    let url = Url::parse("file:///").unwrap();

    image_cache_task.send(Prefetch(url.clone()));

    // Wait until our mock resource task has sent the image to the image cache
    image_bin_sent.recv().unwrap();

    image_cache_task.send(Prefetch(url.clone()));

    image_cache_task.exit();
    mock_resource_task.send(ControlMsg::Exit);

    resource_task_exited.recv().unwrap();

    // Our resource task should not have received another request for the image
    // because it's already cached
    match image_bin_sent.try_recv() {
        Err(_) => (),
        Ok(_) => panic!(),
    }
}

#[test]
fn should_not_request_image_from_resource_task_if_image_fetch_already_failed() {
    let (image_bin_sent_chan, image_bin_sent) = channel();

    let (resource_task_exited_chan, resource_task_exited) = channel();
    let mock_resource_task = spawn_listener(move |port: Receiver<ControlMsg>| {
        loop {
            match port.recv().unwrap() {
                ControlMsg::Load(response) => {
                    let chan = start_sending(response.consumer, Metadata::default(
                        Url::parse("file:///fake").unwrap()));
                    chan.send(Payload(test_image_bin()));
                    chan.send(Done(Err("".to_string())));
                    image_bin_sent_chan.send(());
                }
                ControlMsg::Exit => {
                    resource_task_exited_chan.send(());
                    break
                }
                _ => {}
            }
        }
    });

    let image_cache_task: ImageCacheTask = ImageCacheTaskFactory::new(mock_resource_task.clone(),
                                                                      TaskPool::new(4), profiler(),
                                                                      LoadPlaceholder::Ignore);
    let url = Url::parse("file:///").unwrap();

    image_cache_task.send(Prefetch(url.clone()));
    image_cache_task.send(Decode(url.clone()));

    // Wait until our mock resource task has sent the image to the image cache
    image_bin_sent.recv().unwrap();

    image_cache_task.send(Prefetch(url.clone()));
    image_cache_task.send(Decode(url.clone()));

    image_cache_task.exit();
    mock_resource_task.send(ControlMsg::Exit);

    resource_task_exited.recv().unwrap();

    // Our resource task should not have received another request for the image
    // because it's already cached
    match image_bin_sent.try_recv() {
        Err(_) => (),
        Ok(_) => panic!(),
    }
}

#[test]
fn should_return_failed_if_image_bin_cannot_be_fetched() {
    let mock_resource_task = mock_resource_task(Box::new(SendTestImageErr));

    let image_cache_task: ImageCacheTask = ImageCacheTaskFactory::new(mock_resource_task.clone(),
                                                                      TaskPool::new(4), profiler(),
                                                                      LoadPlaceholder::Preload);
    let url = Url::parse("file:///").unwrap();

    let join_port = image_cache_task.wait_for_store_prefetched();

    image_cache_task.send(Prefetch(url.clone()));
    image_cache_task.send(Decode(url.clone()));

    // Wait until our mock resource task has sent the image to the image cache
    join_port.recv().unwrap();

    let (response_chan, response_port) = channel();
    image_cache_task.send(Msg::GetImage(url, response_chan));
    match response_port.recv().unwrap() {
        ImageResponseMsg::ImageFailed => (),
        _ => panic!("bleh")
    }

    image_cache_task.exit();
    mock_resource_task.send(ControlMsg::Exit);
}

#[test]
fn should_return_failed_for_multiple_get_image_requests_if_image_bin_cannot_be_fetched() {
    let mock_resource_task = mock_resource_task(Box::new(SendTestImageErr));

    let image_cache_task: ImageCacheTask = ImageCacheTaskFactory::new(mock_resource_task.clone(),
                                                                      TaskPool::new(4), profiler(),
                                                                      LoadPlaceholder::Preload);
    let url = Url::parse("file:///").unwrap();

    let join_port = image_cache_task.wait_for_store_prefetched();

    image_cache_task.send(Prefetch(url.clone()));
    image_cache_task.send(Decode(url.clone()));

    // Wait until our mock resource task has sent the image to the image cache
    join_port.recv().unwrap();

    let (response_chan, response_port) = channel();
    image_cache_task.send(Msg::GetImage(url.clone(), response_chan));
    match response_port.recv().unwrap() {
        ImageResponseMsg::ImageFailed => (),
        _ => panic!("bleh")
    }

    // And ask again, we should get the same response
    let (response_chan, response_port) = channel();
    image_cache_task.send(Msg::GetImage(url, response_chan));
    match response_port.recv().unwrap() {
        ImageResponseMsg::ImageFailed => (),
        _ => panic!("bleh")
    }

    image_cache_task.exit();
    mock_resource_task.send(ControlMsg::Exit);
}

#[test]
fn should_return_failed_if_image_decode_fails() {
    let mock_resource_task = mock_resource_task(Box::new(SendBogusImage));

    let image_cache_task: ImageCacheTask = ImageCacheTaskFactory::new(mock_resource_task.clone(),
                                                                      TaskPool::new(4), profiler(),
                                                                      LoadPlaceholder::Preload);
    let url = Url::parse("file:///").unwrap();

    let join_port = image_cache_task.wait_for_store();

    image_cache_task.send(Prefetch(url.clone()));
    image_cache_task.send(Decode(url.clone()));

    // Wait until our mock resource task has sent the image to the image cache
    join_port.recv().unwrap();

    // Make the request
    let (response_chan, response_port) = channel();
    image_cache_task.send(Msg::GetImage(url, response_chan));

    match response_port.recv().unwrap() {
        ImageResponseMsg::ImageFailed => (),
        _ => panic!("bleh")
    }

    image_cache_task.exit();
    mock_resource_task.send(ControlMsg::Exit);
}

#[test]
fn should_return_image_on_wait_if_image_is_already_loaded() {
    let mock_resource_task = mock_resource_task(Box::new(SendTestImage));

    let image_cache_task: ImageCacheTask = ImageCacheTaskFactory::new(mock_resource_task.clone(),
                                                                      TaskPool::new(4), profiler(),
                                                                      LoadPlaceholder::Preload);
    let url = Url::parse("file:///").unwrap();

    let join_port = image_cache_task.wait_for_store();

    image_cache_task.send(Prefetch(url.clone()));
    image_cache_task.send(Decode(url.clone()));

    // Wait until our mock resource task has sent the image to the image cache
    join_port.recv().unwrap();

    let (response_chan, response_port) = channel();
    image_cache_task.send(Msg::WaitForImage(url, response_chan));
    match response_port.recv().unwrap() {
        ImageResponseMsg::ImageReady(..) => (),
        _ => panic!("bleh")
    }

    image_cache_task.exit();
    mock_resource_task.send(ControlMsg::Exit);
}

#[test]
fn should_return_image_on_wait_if_image_is_not_yet_loaded() {
    let (wait_chan, wait_port) = channel();

    let mock_resource_task = mock_resource_task(Box::new(WaitSendTestImage {wait_port: wait_port}));

    let image_cache_task: ImageCacheTask = ImageCacheTaskFactory::new(mock_resource_task.clone(),
                                                                      TaskPool::new(4), profiler(),
                                                                      LoadPlaceholder::Ignore);
    let url = Url::parse("file:///").unwrap();

    image_cache_task.send(Prefetch(url.clone()));
    image_cache_task.send(Decode(url.clone()));

    let (response_chan, response_port) = channel();
    image_cache_task.send(Msg::WaitForImage(url, response_chan));

    wait_chan.send(());

    match response_port.recv().unwrap() {
        ImageResponseMsg::ImageReady(..) => (),
        _ => panic!("bleh")
    }

    image_cache_task.exit();
    mock_resource_task.send(ControlMsg::Exit);
}

#[test]
fn should_return_image_failed_on_wait_if_image_fails_to_load() {
    let (wait_chan, wait_port) = channel();

    let mock_resource_task = mock_resource_task(Box::new(WaitSendTestImageErr{wait_port: wait_port}));

    let image_cache_task: ImageCacheTask = ImageCacheTaskFactory::new(mock_resource_task.clone(),
                                                                      TaskPool::new(4), profiler(),
                                                                      LoadPlaceholder::Ignore);
    let url = Url::parse("file:///").unwrap();

    image_cache_task.send(Prefetch(url.clone()));
    image_cache_task.send(Decode(url.clone()));

    let (response_chan, response_port) = channel();
    image_cache_task.send(Msg::WaitForImage(url, response_chan));

    wait_chan.send(());

    match response_port.recv().unwrap() {
        ImageResponseMsg::ImageFailed => (),
        _ => panic!("bleh")
    }

    image_cache_task.exit();
    mock_resource_task.send(ControlMsg::Exit);
}

#[test]
fn sync_cache_should_wait_for_images() {
    let mock_resource_task = mock_resource_task(Box::new(SendTestImage));

    let image_cache_task: ImageCacheTask = ImageCacheTaskFactory::new_sync(mock_resource_task.clone(),
                                                                           TaskPool::new(4), profiler(),
                                                                           LoadPlaceholder::Preload);
    let url = Url::parse("file:///").unwrap();

    image_cache_task.send(Prefetch(url.clone()));
    image_cache_task.send(Decode(url.clone()));

    let (response_chan, response_port) = channel();
    image_cache_task.send(Msg::GetImage(url, response_chan));
    match response_port.recv().unwrap() {
        ImageResponseMsg::ImageReady(_) => (),
        _ => panic!("bleh")
    }

    image_cache_task.exit();
    mock_resource_task.send(ControlMsg::Exit);
}
