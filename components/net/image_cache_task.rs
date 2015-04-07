/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use net_traits::ResourceTask;
use net_traits::image::base::{Image, load_from_memory};
use net_traits::image_cache_task::{ImageResponseMsg, ImageCacheTask, Msg};
use net_traits::image_cache_task::{load_image_data};
use profile::time::{self, profile};
use url::Url;

use std::borrow::ToOwned;
use std::collections::HashMap;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::mem::replace;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Receiver, Sender};
use util::resource_files::resources_dir_path;
use util::task::spawn_named;
use util::taskpool::TaskPool;

pub trait ImageCacheTaskFactory {
    fn new(resource_task: ResourceTask, task_pool: TaskPool,
               time_profiler_chan: time::ProfilerChan,
               load_placeholder: LoadPlaceholder) -> Self;
    fn new_sync(resource_task: ResourceTask, task_pool: TaskPool,
                    time_profiler_chan: time::ProfilerChan,
                    load_placeholder: LoadPlaceholder) -> Self;
}

impl ImageCacheTaskFactory for ImageCacheTask {
    fn new(resource_task: ResourceTask, task_pool: TaskPool,
           time_profiler_chan: time::ProfilerChan,
           load_placeholder: LoadPlaceholder) -> ImageCacheTask {
        let (chan, port) = channel();
        let chan_clone = chan.clone();

        spawn_named("ImageCacheTask".to_owned(), move || {
            let mut cache = ImageCache {
                resource_task: resource_task,
                port: port,
                chan: chan_clone,
                state_map: HashMap::new(),
                wait_map: HashMap::new(),
                need_exit: None,
                task_pool: task_pool,
                time_profiler_chan: time_profiler_chan,
                placeholder_data: Arc::new(vec!()),
            };
            cache.run(load_placeholder);
        });

        ImageCacheTask {
            chan: chan,
        }
    }

    fn new_sync(resource_task: ResourceTask, task_pool: TaskPool,
                time_profiler_chan: time::ProfilerChan,
                load_placeholder: LoadPlaceholder) -> ImageCacheTask {
        let (chan, port) = channel();

        spawn_named("ImageCacheTask (sync)".to_owned(), move || {
            let inner_cache: ImageCacheTask = ImageCacheTaskFactory::new(resource_task, task_pool,
                                                                         time_profiler_chan, load_placeholder);

            loop {
                let msg: Msg = port.recv().unwrap();

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
    time_profiler_chan: time::ProfilerChan,
    // Default image used when loading fails.
    placeholder_data: Arc<Vec<u8>>,
}

#[derive(Clone)]
enum ImageState {
    Init,
    Prefetching(AfterPrefetch),
    Prefetched(Vec<u8>),
    Decoding,
    Decoded(Arc<Box<Image>>),
    Failed
}

#[derive(Clone)]
enum AfterPrefetch {
    DoDecode,
    DoNotDecode
}

pub enum LoadPlaceholder {
    Preload,
    Ignore
}

impl ImageCache {
    // Used to preload the default placeholder.
    fn init(&mut self) {
        let mut placeholder_url = resources_dir_path();
        // TODO (Savago): replace for a prettier one.
        placeholder_url.push("rippy.jpg");
        let image = load_image_data(Url::from_file_path(&*placeholder_url).unwrap(), self.resource_task.clone(), &self.placeholder_data);

        match image {
            Err(..) => debug!("image_cache_task: failed loading the placeholder."),
            Ok(image_data) => self.placeholder_data = Arc::new(image_data),
        }
    }

    pub fn run(&mut self, load_placeholder: LoadPlaceholder) {
        // We have to load the placeholder before running.
        match load_placeholder {
            LoadPlaceholder::Preload => self.init(),
            LoadPlaceholder::Ignore => debug!("image_cache_task: use old behavior."),
        }

        let mut store_chan: Option<Sender<()>> = None;
        let mut store_prefetched_chan: Option<Sender<()>> = None;

        loop {
            let msg = self.port.recv().unwrap();

            match msg {
                Msg::Prefetch(url) => self.prefetch(url),
                Msg::StorePrefetchedImageData(url, data) => {
                    store_prefetched_chan.map(|chan| {
                        chan.send(()).unwrap();
                    });
                    store_prefetched_chan = None;

                    self.store_prefetched_image_data(url, data);
                }
                Msg::Decode(url) => self.decode(url),
                Msg::StoreImage(url, image) => {
                    store_chan.map(|chan| {
                        chan.send(()).unwrap();
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
                    response.send(()).unwrap();
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
                let placeholder = self.placeholder_data.clone();
                spawn_named("ImageCacheTask (prefetch)".to_owned(), move || {
                    let url = url_clone;
                    debug!("image_cache_task: started fetch for {}", url.serialize());

                    let image = load_image_data(url.clone(), resource_task.clone(), &placeholder);
                    to_cache.send(Msg::StorePrefetchedImageData(url.clone(), image)).unwrap();
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
                let time_profiler_chan = self.time_profiler_chan.clone();

                self.task_pool.execute(move || {
                    let url = url_clone;
                    debug!("image_cache_task: started image decode for {}", url.serialize());
                    let image = profile(time::ProfilerCategory::ImageDecoding,
                                        None, time_profiler_chan, || {
                        load_from_memory(&data)
                    });

                    let image = image.map(|image| Arc::new(box image));
                    to_cache.send(Msg::StoreImage(url.clone(), image)).unwrap();
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

    fn purge_waiters<F>(&mut self, url: Url, f: F) where F: Fn() -> ImageResponseMsg {
        match self.wait_map.remove(&url) {
            Some(waiters) => {
                let items = waiters.lock().unwrap();
                for response in items.iter() {
                    response.send(f()).unwrap();
                }
            }
            None => ()
        }
    }

    fn get_image(&self, url: Url, response: Sender<ImageResponseMsg>) {
        match self.get_state(&url) {
            ImageState::Init => panic!("request for image before prefetch"),
            ImageState::Prefetching(AfterPrefetch::DoDecode) => response.send(ImageResponseMsg::ImageNotReady).unwrap(),
            ImageState::Prefetching(AfterPrefetch::DoNotDecode) | ImageState::Prefetched(..) => panic!("request for image before decode"),
            ImageState::Decoding => response.send(ImageResponseMsg::ImageNotReady).unwrap(),
            ImageState::Decoded(image) => response.send(ImageResponseMsg::ImageReady(image)).unwrap(),
            ImageState::Failed => response.send(ImageResponseMsg::ImageFailed).unwrap(),
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
                        entry.get_mut().lock().unwrap().push(response);
                    }
                    Vacant(entry) => {
                        entry.insert(Arc::new(Mutex::new(vec!(response))));
                    }
                }
            }

            ImageState::Decoded(image) => {
                response.send(ImageResponseMsg::ImageReady(image)).unwrap();
            }

            ImageState::Failed => {
                response.send(ImageResponseMsg::ImageFailed).unwrap();
            }
        }
    }
}

pub fn spawn_listener<F, A>(f: F) -> Sender<A>
    where F: FnOnce(Receiver<A>) + Send + 'static,
          A: Send + 'static
{
    let (setup_chan, setup_port) = channel();

    spawn_named("ImageCacheTask (listener)".to_owned(), move || {
        let (chan, port) = channel();
        setup_chan.send(chan).unwrap();
        f(port);
    });
    setup_port.recv().unwrap()
}
