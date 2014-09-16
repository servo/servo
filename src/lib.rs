/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![comment = "The Servo Parallel Browser Project"]
#![license = "MPL"]

#![feature(globs, macro_rules, phase, thread_local)]

#![deny(unused_imports, unused_variable)]

#[phase(plugin, link)]
extern crate log;

extern crate debug;

extern crate compositing;
extern crate devtools;
extern crate rustuv;
extern crate "net" as servo_net;
extern crate "msg" as servo_msg;
#[phase(plugin, link)]
extern crate "util" as servo_util;
extern crate script;
extern crate layout;
extern crate green;
extern crate gfx;
extern crate libc;
extern crate native;
extern crate rustrt;
extern crate url;

#[cfg(not(test))]
use compositing::{CompositorChan, CompositorTask, Constellation};
#[cfg(not(test))]
use servo_msg::constellation_msg::{ConstellationChan, InitLoadUrlMsg};
#[cfg(not(test))]
use script::dom::bindings::codegen::RegisterBindings;

#[cfg(not(test))]
use servo_net::image_cache_task::ImageCacheTask;
#[cfg(not(test))]
use servo_net::resource_task::new_resource_task;
#[cfg(not(test))]
use gfx::font_cache_task::FontCacheTask;
#[cfg(not(test))]
use servo_util::time::TimeProfiler;
#[cfg(not(test))]
use servo_util::memory::MemoryProfiler;
#[cfg(not(test))]
use servo_util::opts;

#[cfg(not(test))]
use green::GreenTaskBuilder;
#[cfg(not(test))]
use std::os;
#[cfg(not(test))]
use std::task::TaskBuilder;
#[cfg(not(test), target_os="android")]
use std::string;

#[cfg(not(test), target_os="android")]
#[no_mangle]
#[allow(dead_code)]
pub extern "C" fn android_start(argc: int, argv: *const *const u8) -> int {
    native::start(argc, argv, proc() {
        let mut args: Vec<String> = vec!();
        for i in range(0u, argc as uint) {
            unsafe {
                args.push(string::raw::from_buf(*argv.offset(i as int) as *const u8));
            }
        }

        let opts = opts::from_cmdline_args(args.as_slice());
        match opts {
            Some(mut o) => {
                // Always use CPU rendering on android.
                o.cpu_painting = true;
                run(o);
            },
            None => {}
        }
    })
}

#[cfg(not(test))]
pub fn run(opts: opts::Opts) {
    ::servo_util::opts::set_experimental_enabled(opts.enable_experimental);
    RegisterBindings::RegisterProxyHandlers();

    let mut pool_config = green::PoolConfig::new();
    pool_config.event_loop_factory = rustuv::event_loop;
    let mut pool = green::SchedPool::new(pool_config);

    let (compositor_port, compositor_chan) = CompositorChan::new();
    let time_profiler_chan = TimeProfiler::create(opts.time_profiler_period);
    let memory_profiler_chan = MemoryProfiler::create(opts.memory_profiler_period);
    let devtools_chan = if opts.devtools_server {
        Some(devtools::start_server())
    } else {
        None
    };

    let opts_clone = opts.clone();
    let time_profiler_chan_clone = time_profiler_chan.clone();

    let (result_chan, result_port) = channel();
    TaskBuilder::new()
        .green(&mut pool)
        .spawn(proc() {
        let opts = &opts_clone;
        // Create a Servo instance.
        let resource_task = new_resource_task();
        // If we are emitting an output file, then we need to block on
        // image load or we risk emitting an output file missing the
        // image.
        let image_cache_task = if opts.output_file.is_some() {
                ImageCacheTask::new_sync(resource_task.clone())
            } else {
                ImageCacheTask::new(resource_task.clone())
            };
        let font_cache_task = FontCacheTask::new(resource_task.clone());
        let constellation_chan = Constellation::<layout::layout_task::LayoutTask,
                                                 script::script_task::ScriptTask>::start(
                                                      compositor_chan,
                                                      opts,
                                                      resource_task,
                                                      image_cache_task,
                                                      font_cache_task,
                                                      time_profiler_chan_clone,
                                                      devtools_chan);

        // Send the URL command to the constellation.
        let cwd = os::getcwd();
        for url in opts.urls.iter() {
            let url = match url::Url::parse(url.as_slice()) {
                Ok(url) => url,
                Err(url::RelativeUrlWithoutBase)
                => url::Url::from_file_path(&cwd.join(url.as_slice())).unwrap(),
                Err(_) => fail!("URL parsing failed"),
            };

            let ConstellationChan(ref chan) = constellation_chan;
            chan.send(InitLoadUrlMsg(url));
        }

        // Send the constallation Chan as the result
        result_chan.send(constellation_chan);
    });

    let constellation_chan = result_port.recv();

    debug!("preparing to enter main loop");
    CompositorTask::create(opts,
                           compositor_port,
                           constellation_chan,
                           time_profiler_chan,
                           memory_profiler_chan);

    pool.shutdown();
}

