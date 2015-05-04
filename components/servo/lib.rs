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
#![feature(libc, thread_local)]

extern crate compositing;
extern crate devtools;
extern crate devtools_traits;
extern crate net;
extern crate net_traits;
extern crate msg;
extern crate profile;
extern crate profile_traits;
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

use compositing::windowing::WindowMethods;
use compositing::{CompositorProxy, CompositorTask, Constellation};

use msg::constellation_msg::Msg as ConstellationMsg;
use msg::constellation_msg::ConstellationChan;

use script::dom::bindings::codegen::RegisterBindings;

use net::image_cache_task::new_image_cache_task;
use net::storage_task::StorageTaskFactory;
use net::resource_task::new_resource_task;
use net_traits::storage_task::StorageTask;

use gfx::font_cache_task::FontCacheTask;
use profile::mem as profile_mem;
use profile::time as profile_time;
use profile_traits::mem;
use profile_traits::time;
use util::opts;

use std::rc::Rc;
use std::sync::mpsc::Sender;

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
impl Browser  {
    pub fn new<Window>(window: Option<Rc<Window>>) -> Browser
    where Window: WindowMethods + 'static {
        ::util::opts::set_experimental_enabled(opts::get().enable_experimental);

        // Global configuration options, parsed from the command line.
        let opts = opts::get();

        // Create the global vtables used by the (generated) DOM
        // bindings to implement JS proxies.
        RegisterBindings::RegisterProxyHandlers();

        // Get both endpoints of a special channel for communication between
        // the client window and the compositor. This channel is unique because
        // messages to client may need to pump a platform-specific event loop
        // to deliver the message.
        let (compositor_proxy, compositor_receiver) =
            WindowMethods::create_compositor_channel(&window);
        let time_profiler_chan = profile_time::Profiler::create(opts.time_profiler_period);
        let mem_profiler_chan = profile_mem::Profiler::create(opts.mem_profiler_period);
        let devtools_chan = opts.devtools_port.map(|port| {
            devtools::start_server(port)
        });

        // Create the constellation, which maintains the engine
        // pipelines, including the script and layout threads, as well
        // as the navigation context.
        let constellation_chan = create_constellation(opts.clone(),
                                                      compositor_proxy.clone_compositor_proxy(),
                                                      time_profiler_chan.clone(),
                                                      devtools_chan,
                                                      mem_profiler_chan.clone());

        if let Some(port) = opts.webdriver_port {
            webdriver_server::start_server(port, constellation_chan.clone());
        };

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
fn create_constellation(opts: opts::Opts,
                        compositor_proxy: Box<CompositorProxy+Send>,
                        time_profiler_chan: time::ProfilerChan,
                        devtools_chan: Option<Sender<devtools_traits::DevtoolsControlMsg>>,
                        mem_profiler_chan: mem::ProfilerChan) -> ConstellationChan {
    use std::env;

    // Create a Servo instance.
    let resource_task = new_resource_task(opts.user_agent.clone());

    let image_cache_task = new_image_cache_task(resource_task.clone());
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

    {
        let ConstellationChan(ref chan) = constellation_chan;
        chan.send(ConstellationMsg::InitLoadUrl(url)).unwrap();
    }

    constellation_chan
}
