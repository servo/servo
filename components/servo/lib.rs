/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![comment = "The Servo Parallel Browser Project"]
#![license = "MPL"]

#![feature(globs, macro_rules, phase, thread_local)]

#![deny(unused_imports)]
#![deny(unused_variables)]

#[phase(plugin, link)]
extern crate log;

extern crate compositing;
extern crate devtools;
extern crate "net" as servo_net;
extern crate "msg" as servo_msg;
#[phase(plugin, link)]
extern crate "util" as servo_util;
extern crate script;
extern crate layout;
extern crate gfx;
extern crate libc;
extern crate native;
extern crate rustrt;
extern crate url;

use compositing::windowing::{WindowEvent, WindowMethods};

#[cfg(not(test))]
use compositing::{CompositorTask, Constellation};
#[cfg(not(test))]
use compositing::compositor_task::{mod, InitialCompositorState};
#[cfg(not(test))]
use compositing::constellation::InitialConstellationState;
use compositing::main_thread::MainThread;
#[cfg(not(test))]
use compositing::main_thread::MainThreadProxy;
#[cfg(not(test))]
use servo_msg::constellation_msg::{ConstellationChan, InitLoadUrlMsg};
#[cfg(not(test))]
use script::dom::bindings::codegen::RegisterBindings;

#[cfg(not(test))]
use servo_net::image_cache_task::ImageCacheTask;
#[cfg(not(test))]
use servo_net::resource_task::new_resource_task;
#[cfg(not(test))]
use servo_net::storage_task::StorageTaskFactory;
#[cfg(not(test))]
use gfx::font_cache_task::FontCacheTask;
#[cfg(not(test))]
use servo_util::time::TimeProfiler;
#[cfg(not(test))]
use servo_util::memory::MemoryProfiler;
#[cfg(not(test))]
use servo_util::opts;
#[cfg(not(test))]
use servo_util::taskpool::TaskPool;

#[cfg(not(test))]
use std::comm;
#[cfg(not(test))]
use std::os;
#[cfg(not(test))]
use std::rc::Rc;
#[cfg(not(test))]
use std::task::TaskBuilder;

pub struct Browser<Window> {
    main_thread: MainThread<Window>,
}

impl<Window> Browser<Window> where Window: WindowMethods + 'static {
    #[cfg(not(test))]
    pub fn new(window: Option<Rc<Window>>) -> Browser<Window> {
        // Get command-line options (global settings).
        ::servo_util::opts::set_experimental_enabled(opts::get().enable_experimental);
        let opts = opts::get();
        RegisterBindings::RegisterProxyHandlers();

        // Create a task pool for use by various tasks in the system.
        let shared_task_pool = TaskPool::new(8);

        // Set up communication channels.
        //
        // TODO(pcwalton): e10s-ify most of these. This will involve converting them into named
        // pipes/Unix sockets and `#[deriving(Encodable)]` on their messages.
        let (compositor_proxy, compositor_receiver) = compositor_task::create_channel();
        let time_profiler_chan = TimeProfiler::create(opts.time_profiler_period);
        let memory_profiler_chan = MemoryProfiler::create(opts.memory_profiler_period);
        let devtools_chan = opts.devtools_port.map(|port| devtools::start_server(port));
        let opts_clone = opts.clone();
        let (main_thread_sender, main_thread_receiver) = comm::channel();
        let main_thread_proxy =
            WindowMethods::create_main_thread_proxy(&window, main_thread_sender.clone());

        let (result_chan, result_port) = comm::channel();
        let compositor_proxy_for_constellation = if window.is_none() {
            // We're in headless mode. There is no compositor.
            None
        } else {
            Some(compositor_proxy.clone())
        };
        let main_thread_proxy_for_constellation = main_thread_proxy.clone_main_thread_proxy();
        let time_profiler_proxy_for_constellation = time_profiler_chan.clone();

        // Create the constellation.
        //
        // TODO(pcwalton): Support multiple constellations, for multiple tabs.
        // TODO(pcwalton): Put this in a separate process.
        TaskBuilder::new().spawn(proc() {
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
            let storage_task = StorageTaskFactory::new();
            let initial_state = InitialConstellationState {
                main_thread_proxy: main_thread_proxy_for_constellation,
                compositor_proxy: compositor_proxy_for_constellation,
                resource_task: resource_task,
                image_cache_task: image_cache_task,
                devtools_chan: devtools_chan,
                storage_task: storage_task,
                font_cache_task: font_cache_task,
                time_profiler_proxy: time_profiler_proxy_for_constellation,
            };
            let constellation_chan =
                Constellation::<layout::layout_task::LayoutTask,
                                script::script_task::ScriptTask>::start(initial_state);

            // Send the URL command to the constellation.
            let cwd = os::getcwd();
            for url in opts.urls.iter() {
                let url = match url::Url::parse(url.as_slice()) {
                    Ok(url) => url,
                    Err(url::RelativeUrlWithoutBase)
                    => url::Url::from_file_path(&cwd.join(url.as_slice())).unwrap(),
                    Err(_) => panic!("URL parsing failed"),
                };

                let ConstellationChan(ref chan) = constellation_chan;
                chan.send(InitLoadUrlMsg(url));
            }

            // Send the constallation Chan as the result
            result_chan.send(constellation_chan);
        });

        // Create the object that manages the main thread.
        let constellation_chan = result_port.recv();
        let compositor_proxy_for_main_thread = if window.is_none() {
            None
        } else {
            Some(compositor_proxy.clone())
        };
        let main_thread = MainThread::new(window.clone(),
                                          main_thread_sender,
                                          main_thread_receiver,
                                          compositor_proxy_for_main_thread,
                                          constellation_chan.clone());

        // Create the compositor.
        //
        // TODO(pcwalton): Support multiple compositors, for multiple windows.
        match window {
            None => {}
            Some(ref window) => {
                let main_thread_proxy_for_compositor = main_thread_proxy.clone_main_thread_proxy();
                let compositor_proxy_for_compositor = compositor_proxy.clone();
                let window_framebuffer_size = window.framebuffer_size();
                let hidpi_factor = window.hidpi_factor();
                let native_graphics_metadata = window.native_metadata();
                let compositor_support = window.create_compositor_support();
                TaskBuilder::new().spawn(proc() {
                    let mut compositor = CompositorTask::create(InitialCompositorState {
                        main_thread_proxy: main_thread_proxy_for_compositor,
                        sender: compositor_proxy_for_compositor,
                        receiver: compositor_receiver,
                        constellation_sender: constellation_chan,
                        time_profiler_sender: time_profiler_chan,
                        memory_profiler_sender: memory_profiler_chan,
                        window_framebuffer_size: window_framebuffer_size,
                        hidpi_factor: hidpi_factor,
                        native_graphics_metadata: Some(native_graphics_metadata),
                        compositor_support: compositor_support,
                    });
                    while compositor.handle_events() {}
                });
            }
        }

        Browser {
            main_thread: main_thread,
        }
    }

    /// Processes an event. Returns true if the browser is to continue processing events or false
    /// if the browser is to shut down.
    pub fn send_event(&mut self, event: WindowEvent) -> bool {
        self.main_thread.enqueue(event);
        self.main_thread.process_events()
    }
}

