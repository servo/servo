/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(thread_local)]
#![feature(box_syntax)]
#![feature(int_uint)]
#![feature(path, rustc_private)]
// For FFI
#![allow(non_snake_case, dead_code)]

#[macro_use]
extern crate log;

extern crate compositing;
extern crate devtools;
extern crate net;
extern crate net_traits;
extern crate msg;
#[macro_use]
extern crate util;
extern crate script;
extern crate layout;
extern crate gfx;
extern crate libc;
extern crate profile;
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
use net::image_cache_task::new_image_cache_task;
#[cfg(not(test))]
use net::storage_task::StorageTaskFactory;
#[cfg(not(test))]
use net::resource_task::new_resource_task;
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
use std::env;
#[cfg(not(test))]
use std::rc::Rc;

pub struct Browser {
    compositor: Box<CompositorEventListener + 'static>,
}

/// The in-process interface to Servo.
///
/// It does everything necessary to render the web, primarily
/// orchestrating the interaction between JavaScript, CSS layout,
/// rendering, and the client window.
///
/// Clients create a `Browser` for a given reference-counted type
/// implementing `WindowMethods`, which is the bridge to whatever
/// application Servo is embedded in. Clients then create an event
/// loop to pump messages between the embedding application and
/// various browser components.
impl Browser {
    #[cfg(not(test))]
    pub fn new<Window>(window: Option<Rc<Window>>) -> Browser
        where Window: WindowMethods + 'static {
        ::util::opts::set_experimental_enabled(opts::get().enable_experimental);
        // Global configuration options, parsed from the command line.
        let opts = opts::get();

        // Create the global vtables used by the (generated) DOM
        // bindings to implement JS proxies.
        RegisterBindings::RegisterProxyHandlers();

        // Use this thread pool to load-balance simple tasks, such as
        // image decoding.
        let shared_task_pool = TaskPool::new(8);

        // Get both endpoints of a special channel for communication between
        // the client window and the compositor. This channel is unique because
        // messages to client may need to pump a platform-specific event loop
        // to deliver the message.
        let (compositor_proxy, compositor_receiver) =
            WindowMethods::create_compositor_channel(&window);
        let time_profiler_chan = time::Profiler::create(opts.time_profiler_period);
        let mem_profiler_chan = mem::Profiler::create(opts.mem_profiler_period);
        let devtools_chan = opts.devtools_port.map(|port| {
            devtools::start_server(port)
        });

        // Create a Servo instance.
        let resource_task = new_resource_task(opts.user_agent.clone());

        let image_cache_task = new_image_cache_task(resource_task.clone());
        let font_cache_task = FontCacheTask::new(resource_task.clone());
        let storage_task = StorageTaskFactory::new();

        // Create the constellation, which maintains the engine
        // pipelines, including the script and layout threads, as well
        // as the navigation context.
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

        // The compositor coordinates with the client window to create the final
        // rendered page and display it somewhere.
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
