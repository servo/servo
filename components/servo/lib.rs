/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(core, env, libc, path, rustc_private, std_misc, thread_local)]

#![allow(missing_copy_implementations)]

#[macro_use]
extern crate log;

extern crate compositing;
extern crate devtools;
extern crate net;
extern crate msg;
#[macro_use]
extern crate util;
extern crate script;
extern crate layout;
extern crate gfx;
extern crate libc;
extern crate url;

use compositing::CompositorEventListener;
use compositing::windowing::{WindowEvent, WindowMethods};

#[cfg(not(test))]
use compositing::{CompositorProxy, CompositorTask, Constellation};
#[cfg(not(test))]
use msg::constellation_msg::Msg as ConstellationMsg;
#[cfg(not(test))]
use msg::constellation_msg::ConstellationChan;
#[cfg(not(test))]
use script::dom::bindings::codegen::RegisterBindings;

#[cfg(not(test))]
use net::image_cache_task::ImageCacheTask;
#[cfg(not(test))]
use net::resource_task::new_resource_task;
#[cfg(not(test))]
use net::storage_task::{StorageTaskFactory, StorageTask};
#[cfg(not(test))]
use gfx::font_cache_task::FontCacheTask;
#[cfg(not(test))]
use util::time::TimeProfiler;
#[cfg(not(test))]
use util::memory::MemoryProfiler;
#[cfg(not(test))]
use util::opts;
#[cfg(not(test))]
use util::taskpool::TaskPool;

#[cfg(not(test))]
use std::rc::Rc;
#[cfg(not(test))]
use std::sync::mpsc::channel;
#[cfg(not(test))]
use std::thread::Builder;

pub struct Browser<Window> {
    compositor: Box<CompositorEventListener + 'static>,
}

impl<Window> Browser<Window> where Window: WindowMethods + 'static {
    #[cfg(not(test))]
    pub fn new(window: Option<Rc<Window>>) -> Browser<Window> {
        use std::env;

        ::util::opts::set_experimental_enabled(opts::get().enable_experimental);
        let opts = opts::get();
        RegisterBindings::RegisterProxyHandlers();

        let shared_task_pool = TaskPool::new(8);

        let (compositor_proxy, compositor_receiver) =
            WindowMethods::create_compositor_channel(&window);
        let time_profiler_chan = TimeProfiler::create(opts.time_profiler_period);
        let memory_profiler_chan = MemoryProfiler::create(opts.memory_profiler_period);
        let devtools_chan = opts.devtools_port.map(|port| {
            devtools::start_server(port)
        });

        let opts_clone = opts.clone();
        let time_profiler_chan_clone = time_profiler_chan.clone();

        let (result_chan, result_port) = channel();
        let compositor_proxy_for_constellation = compositor_proxy.clone_compositor_proxy();
        Builder::new()
            .spawn(move || {
            let opts = &opts_clone;
            // Create a Servo instance.
            let resource_task = new_resource_task(opts.user_agent.clone());
            // If we are emitting an output file, then we need to block on
            // image load or we risk emitting an output file missing the
            // image.
            let image_cache_task = if opts.output_file.is_some() {
                ImageCacheTask::new_sync(resource_task.clone(), shared_task_pool)
            } else {
                ImageCacheTask::new(resource_task.clone(), shared_task_pool)
            };
            let font_cache_task = FontCacheTask::new(resource_task.clone());
            let storage_task: StorageTask = StorageTaskFactory::new();
            let constellation_chan = Constellation::<layout::layout_task::LayoutTask,
                                                     script::script_task::ScriptTask>::start(
                                                          compositor_proxy_for_constellation,
                                                          resource_task,
                                                          image_cache_task,
                                                          font_cache_task,
                                                          time_profiler_chan_clone,
                                                          devtools_chan,
                                                          storage_task);

            // Send the URL command to the constellation.
            let cwd = env::current_dir().unwrap();
            for url in opts.urls.iter() {
                let url = match url::Url::parse(&*url) {
                    Ok(url) => url,
                    Err(url::ParseError::RelativeUrlWithoutBase)
                    => url::Url::from_file_path(&cwd.join(&*url)).unwrap(),
                    Err(_) => panic!("URL parsing failed"),
                };

                let ConstellationChan(ref chan) = constellation_chan;
                chan.send(ConstellationMsg::InitLoadUrl(url)).unwrap();
            }

            // Send the constallation Chan as the result
            result_chan.send(constellation_chan).unwrap();
        });

        let constellation_chan = result_port.recv().unwrap();

        debug!("preparing to enter main loop");
        let compositor = CompositorTask::create(window,
                                                compositor_proxy,
                                                compositor_receiver,
                                                constellation_chan,
                                                time_profiler_chan,
                                                memory_profiler_chan);

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

