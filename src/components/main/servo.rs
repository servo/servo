/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![crate_id = "github.com/mozilla/servo"]
#![comment = "The Servo Parallel Browser Project"]
#![license = "MPL"]

#![feature(globs, macro_rules, phase, thread_local)]

#[phase(plugin, link)]
extern crate log;

extern crate debug;

extern crate compositing;
extern crate rustuv;
extern crate servo_net = "net";
extern crate servo_msg = "msg";
#[phase(plugin, link)]
extern crate servo_util = "util";
extern crate script;
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
use servo_util::url::parse_url;


#[cfg(not(test), not(target_os="android"))]
use std::os;
#[cfg(not(test), target_os="android")]
use std::str;
#[cfg(not(test))]
use rustrt::task::TaskOpts;
#[cfg(not(test))]
use url::Url;


#[cfg(not(test), target_os="linux")]
#[cfg(not(test), target_os="macos")]
#[start]
#[allow(dead_code)]
fn start(argc: int, argv: **u8) -> int {
    native::start(argc, argv, proc() {
        opts::from_cmdline_args(os::args().as_slice()).map(run);
    })
}

#[cfg(not(test), target_os="android")]
#[no_mangle]
#[allow(dead_code)]
pub extern "C" fn android_start(argc: int, argv: **u8) -> int {
    native::start(argc, argv, proc() {
        let mut args: Vec<String> = vec!();
        for i in range(0u, argc as uint) {
            unsafe {
                args.push(str::raw::from_c_str(*argv.offset(i as int) as *i8));
            }
        }

        let mut opts = opts::from_cmdline_args(args.as_slice());
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
    RegisterBindings::RegisterProxyHandlers();

    let mut pool_config = green::PoolConfig::new();
    pool_config.event_loop_factory = rustuv::event_loop;
    let mut pool = green::SchedPool::new(pool_config);

    let (compositor_port, compositor_chan) = CompositorChan::new();
    let time_profiler_chan = TimeProfiler::create(opts.time_profiler_period);
    let memory_profiler_chan = MemoryProfiler::create(opts.memory_profiler_period);

    let opts_clone = opts.clone();
    let time_profiler_chan_clone = time_profiler_chan.clone();

    let (result_chan, result_port) = channel();
    pool.spawn(TaskOpts::new(), proc() {
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
        let font_cache_task = FontCacheTask::new();
        let constellation_chan = Constellation::start(compositor_chan,
                                                      opts,
                                                      resource_task,
                                                      image_cache_task,
                                                      font_cache_task,
                                                      time_profiler_chan_clone);

        // Send the URL command to the constellation.
        for filename in opts.urls.iter() {
            let url = if filename.as_slice().starts_with("data:") {
                // As a hack for easier command-line testing,
                // assume that data URLs are not URL-encoded.
                Url::new("data".to_string(), None, "".to_string(), None,
                    filename.as_slice().slice_from(5).to_string(), vec!(), None)
            } else {
                parse_url(filename.as_slice(), None)
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

