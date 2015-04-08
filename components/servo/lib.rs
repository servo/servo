/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(libc, rustc_private, thread_local)]
#![cfg_attr(not(test), feature(path))]

#[macro_use]
extern crate log;

extern crate compositing;
extern crate devtools;
extern crate net;
extern crate net_traits;
extern crate msg;
extern crate profile;
#[macro_use]
extern crate util;
extern crate script;
extern crate layout;
extern crate gfx;
extern crate libc;
extern crate url;
extern crate webdriver_server;

use compositing::CompositorEventListener;
use compositing::windowing::WindowEvent;

#[cfg(not(test))]
use compositing::windowing::WindowMethods;
#[cfg(not(test))]
use compositing::{CompositorProxy, CompositorTask, Constellation};
#[cfg(not(test))]
use msg::constellation_msg::Msg as ConstellationMsg;
#[cfg(not(test))]
use msg::constellation_msg::ConstellationChan;
#[cfg(not(test))]
use script::dom::bindings::codegen::RegisterBindings;

#[cfg(not(test))]
use net::image_cache_task::{ImageCacheTaskFactory, LoadPlaceholder};
#[cfg(not(test))]
use net::storage_task::StorageTaskFactory;
#[cfg(not(test))]
use net::resource_task::new_resource_task;
#[cfg(not(test))]
use net_traits::image_cache_task::ImageCacheTask;
#[cfg(not(test))]
use net_traits::storage_task::StorageTask;
#[cfg(not(test))]
use gfx::font_cache_task::FontCacheTask;
#[cfg(not(test))]
use profile::mem;
#[cfg(not(test))]
use profile::time;
#[cfg(not(test))]
use util::opts;
#[cfg(not(test))]
use util::taskpool::TaskPool;

#[cfg(not(test))]
use std::rc::Rc;

pub struct Browser {
    compositor: Box<CompositorEventListener + 'static>,
}

impl Browser  {
    #[cfg(not(test))]
    pub fn new<Window>(window: Option<Rc<Window>>) -> Browser
    where Window: WindowMethods + 'static {
        use std::env;

        ::util::opts::set_experimental_enabled(opts::get().enable_experimental);
        let opts = opts::get();
        RegisterBindings::RegisterProxyHandlers();

        let shared_task_pool = TaskPool::new(8);

        let (compositor_proxy, compositor_receiver) =
            WindowMethods::create_compositor_channel(&window);
        let time_profiler_chan = time::Profiler::create(opts.time_profiler_period);
        let mem_profiler_chan = mem::Profiler::create(opts.mem_profiler_period);
        let devtools_chan = opts.devtools_port.map(|port| {
            devtools::start_server(port)
        });

        if let Some(port) = opts.webdriver_port {
            webdriver_server::start_server(port);
        }

        // Create a Servo instance.
        let resource_task = new_resource_task(opts.user_agent.clone());

        // If we are emitting an output file, then we need to block on
        // image load or we risk emitting an output file missing the
        // image.
        let image_cache_task: ImageCacheTask = if opts.output_file.is_some() {
            ImageCacheTaskFactory::new_sync(resource_task.clone(), shared_task_pool,
                                            time_profiler_chan.clone(), LoadPlaceholder::Preload)
        } else {
            ImageCacheTaskFactory::new(resource_task.clone(), shared_task_pool,
                                       time_profiler_chan.clone(), LoadPlaceholder::Preload)
        };

        let font_cache_task = FontCacheTask::new(resource_task.clone());
        let storage_task: StorageTask = StorageTaskFactory::new();

        let constellation_chan = Constellation::<layout::layout_task::LayoutTask,
                                                 script::script_task::ScriptTask>::start(
                                                      compositor_proxy.clone_compositor_proxy(),
                                                      resource_task,
                                                      image_cache_task,
                                                      font_cache_task,
                                                      time_profiler_chan.clone(),
                                                      mem_profiler_chan.clone(),
                                                      devtools_chan,
                                                      storage_task);

        // Send the URL command to the constellation.
        let cwd = env::current_dir().unwrap();
        let url = match url::Url::parse(&opts.url) {
            Ok(url) => url,
            Err(url::ParseError::RelativeUrlWithoutBase)
            => url::Url::from_file_path(&*cwd.join(&opts.url)).unwrap(),
            Err(_) => panic!("URL parsing failed"),
        };

        let ConstellationChan(ref chan) = constellation_chan;
        chan.send(ConstellationMsg::InitLoadUrl(url)).unwrap();

        let compositor = CompositorTask::create(window,
                                                compositor_proxy,
                                                compositor_receiver,
                                                constellation_chan.clone(),
                                                time_profiler_chan,
                                                mem_profiler_chan);

        Browser {
            compositor: compositor,
        }
    }

    pub fn handle_event(&mut self, event: WindowEvent) -> bool {
        self.compositor.handle_event(event)
    }

    pub fn repaint_synchronously(&mut self) {
        self.compositor.repaint_synchronously()
    }

    pub fn pinch_zoom_level(&self) -> f32 {
        self.compositor.pinch_zoom_level()
    }

    pub fn get_title_for_main_frame(&self) {
        self.compositor.get_title_for_main_frame()
    }

    pub fn shutdown(mut self) {
        self.compositor.shutdown();
    }
}
