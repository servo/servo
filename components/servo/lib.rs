/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// Servo, the mighty web browser engine from the future.
//
// This is a very simple library that wires all of Servo's components
// together as type `Browser`, along with a generic client
// implementing the `WindowMethods` trait, to create a working web
// browser.
//
// The `Browser` type is responsible for configuring a
// `Constellation`, which does the heavy lifting of coordinating all
// of Servo's internal subsystems, including the `ScriptTask` and the
// `LayoutTask`, as well maintains the navigation context.
//
// The `Browser` is fed events from a generic type that implements the
// `WindowMethods` trait.

#![feature(core, env, libc, path, rustc_private, thread_local)]

#![allow(missing_copy_implementations)]

extern crate compositing;
extern crate devtools;
extern crate devtools_traits;
extern crate net;
extern crate msg;
#[macro_use]
extern crate util;
extern crate script;
extern crate layout;
extern crate gfx;
extern crate libc;
extern crate url;
#[macro_use]
extern crate log;

use compositing::CompositorEventListener;
use compositing::windowing::{WindowEvent, WindowMethods};
use compositing::{CompositorProxy, CompositorTask, Constellation};

use msg::constellation_msg::Msg as ConstellationMsg;
use msg::constellation_msg::ConstellationChan;

use script::dom::bindings::codegen::RegisterBindings;

use net::image_cache_task::ImageCacheTask;
use net::resource_task::new_resource_task;
use net::storage_task::StorageTaskFactory;

use gfx::font_cache_task::FontCacheTask;

use util::time::{TimeProfiler, TimeProfilerChan};
use util::memory::MemoryProfiler;
use util::opts;
use util::taskpool::TaskPool;

use std::env;
use std::rc::Rc;
use std::sync::mpsc::Sender;

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
pub struct Browser<Window> {
    compositor: Box<CompositorEventListener + 'static>,
}

impl<Window> Browser<Window> where Window: WindowMethods + 'static {
    pub fn new(window: Option<Rc<Window>>) -> Browser<Window> {
        util::opts::set_experimental_enabled(opts::get().enable_experimental);

        // Global configuration options, parsed from the command line.
        let opts = opts::get();

        // Create the global vtables used by the (generated) DOM
        // bindings to implement JS proxies.
        RegisterBindings::RegisterProxyHandlers();

        // Use this thread pool to load-balance simple tasks, such as
        // image decoding.
        let shared_task_pool = TaskPool::new(8);

        let time_profiler_chan = TimeProfiler::create(opts.time_profiler_period);
        let memory_profiler_chan = MemoryProfiler::create(opts.memory_profiler_period);
        let devtools_chan = opts.devtools_port.map(|port| {
            devtools::start_server(port)
        });

        // Get both endpoints of a special channel for communication between
        // the client window and the compositor. This channel is unique because
        // messages to client may need to pump a platform-specific event loop
        // to deliver the message.
        let (compositor_proxy, compositor_receiver) =
            WindowMethods::create_compositor_channel(&window);

        // Create the constellation, which maintains the engine
        // pipelines, including the script and layout threads,as well
        // as the navigation context.
        let constellation_chan = create_constellation(opts.clone(),
                                                      compositor_proxy.clone_compositor_proxy(),
                                                      time_profiler_chan.clone(),
                                                      devtools_chan,
                                                      shared_task_pool);

        // The compositor coordinates with the client window to create the final
        // rendered page and display it somewhere.
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

fn create_constellation(opts: opts::Opts,
                        compositor_proxy: Box<CompositorProxy+Send>,
                        time_profiler_chan: TimeProfilerChan,
                        devtools_chan: Option<Sender<devtools_traits::DevtoolsControlMsg>>,
                        shared_task_pool: TaskPool) -> ConstellationChan {

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
    let storage_task = StorageTaskFactory::new();

    let constellation_chan = Constellation::<layout::layout_task::LayoutTask,
                                             script::script_task::ScriptTask>::start(
                                                 compositor_proxy,
                                                 resource_task,
                                                 image_cache_task,
                                                 font_cache_task,
                                                 time_profiler_chan,
                                                 devtools_chan,
                                                 storage_task);

    // If the global configuration asked to load a URL then send
    // it to the constellation.
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

    constellation_chan
}
