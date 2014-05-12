/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![crate_id = "github.com/mozilla/servo"]
#![comment = "The Servo Parallel Browser Project"]
#![license = "MPL"]

#![feature(globs, macro_rules, phase, thread_local)]

#[phase(syntax, link)]
extern crate log;

extern crate alert;
extern crate azure;
extern crate geom;
extern crate gfx;
#[cfg(not(target_os="android"))]
extern crate glfw;
#[cfg(target_os="android")]
extern crate glut;
extern crate js;
extern crate layers;
extern crate opengles;
extern crate png;
extern crate rustuv;
extern crate script;
#[phase(syntax)]
extern crate servo_macros = "macros";
extern crate servo_net = "net";
extern crate servo_msg = "msg";
#[phase(syntax, link)]
extern crate servo_util = "util";
extern crate style;
extern crate sharegl;
extern crate stb_image;

extern crate collections;
extern crate green;
extern crate libc;
extern crate native;
extern crate serialize;
extern crate sync;
extern crate time;
extern crate url;

#[cfg(target_os="macos")]
extern crate core_graphics;
#[cfg(target_os="macos")]
extern crate core_text;

#[cfg(not(test))]
use compositing::{CompositorChan, CompositorTask};
#[cfg(not(test))]
use constellation::Constellation;
#[cfg(not(test))]
use servo_msg::constellation_msg::{ConstellationChan, InitLoadUrlMsg};

#[cfg(not(test))]
use servo_net::image_cache_task::{ImageCacheTask, SyncImageCacheTask};
#[cfg(not(test))]
use servo_net::resource_task::ResourceTask;
#[cfg(not(test))]
use servo_util::time::Profiler;

#[cfg(not(test))]
use servo_util::opts;
#[cfg(not(test))]
use servo_util::url::parse_url;


#[cfg(not(test), not(target_os="android"))]
use std::os;
#[cfg(not(test), target_os="android")]
use std::str;
#[cfg(not(test))]
use std::task::TaskOpts;
#[cfg(not(test))]
use url::Url;


#[path="compositing/compositor_task.rs"]
pub mod compositing;

pub mod css {
    mod node_util;

    pub mod select;
    pub mod matching;
    pub mod node_style;
}

pub mod constellation;
pub mod pipeline;

pub mod layout {
    pub mod block;
    pub mod box_;
    pub mod construct;
    pub mod context;
    pub mod floats;
    pub mod flow;
    pub mod flow_list;
    pub mod layout_task;
    pub mod inline;
    pub mod model;
    pub mod parallel;
    pub mod table_wrapper;
    pub mod table;
    pub mod table_caption;
    pub mod table_colgroup;
    pub mod table_rowgroup;
    pub mod table_row;
    pub mod table_cell;
    pub mod text;
    pub mod util;
    pub mod incremental;
    pub mod wrapper;
    pub mod extra;
}

pub mod windowing;

#[path="platform/mod.rs"]
pub mod platform;

#[cfg(not(test), target_os="linux")]
#[cfg(not(test), target_os="macos")]
#[start]
#[allow(dead_code)]
fn start(argc: int, argv: **u8) -> int {
    native::start(argc, argv, proc() {
        opts::from_cmdline_args(os::args()).map(run);
    })
}

#[cfg(not(test), target_os="android")]
#[no_mangle]
#[allow(dead_code)]
pub extern "C" fn android_start(argc: int, argv: **u8) -> int {
    native::start(argc, argv, proc() {
        let mut args: Vec<~str> = vec!();
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
fn run(opts: opts::Opts) {
    let mut pool_config = green::PoolConfig::new();
    pool_config.event_loop_factory = rustuv::event_loop;
    let mut pool = green::SchedPool::new(pool_config);

    let (compositor_port, compositor_chan) = CompositorChan::new();
    let profiler_chan = Profiler::create(opts.profiler_period);

    let opts_clone = opts.clone();
    let profiler_chan_clone = profiler_chan.clone();

    let (result_chan, result_port) = channel();
    pool.spawn(TaskOpts::new(), proc() {
        let opts = &opts_clone;
        // Create a Servo instance.
        let resource_task = ResourceTask();
        // If we are emitting an output file, then we need to block on
        // image load or we risk emitting an output file missing the
        // image.
        let image_cache_task = if opts.output_file.is_some() {
                SyncImageCacheTask(resource_task.clone())
            } else {
                ImageCacheTask(resource_task.clone())
            };
        let constellation_chan = Constellation::start(compositor_chan,
                                                      opts,
                                                      resource_task,
                                                      image_cache_task,
                                                      profiler_chan_clone);

        // Send the URL command to the constellation.
        for filename in opts.urls.iter() {
            let url = if filename.starts_with("data:") {
                // As a hack for easier command-line testing,
                // assume that data URLs are not URL-encoded.
                Url::new("data".to_owned(), None, "".to_owned(), None,
                    filename.slice_from(5).to_owned(), vec!(), None)
            } else {
                parse_url(*filename, None)
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
                           profiler_chan);

    pool.shutdown();
}

