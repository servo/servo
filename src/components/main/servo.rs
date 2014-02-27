/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[crate_id = "github.com/mozilla/servo"];
#[comment = "The Servo Parallel Browser Project"];
#[license = "MPL"];

#[feature(globs, macro_rules, managed_boxes, thread_local)];

extern mod alert;
extern mod azure;
extern mod geom;
extern mod gfx;
#[cfg(not(target_os="android"))]
extern mod glfw;
#[cfg(target_os="android")]
extern mod glut;
extern mod js;
extern mod layers;
extern mod opengles;
extern mod png;
#[cfg(target_os="android")]
extern mod rustuv;
extern mod script;
extern mod servo_net = "net";
extern mod servo_msg = "msg";
extern mod servo_util = "util";
extern mod style;
extern mod sharegl;
extern mod stb_image;

extern mod extra;
extern mod green;
extern mod native;

#[cfg(target_os="macos")]
extern mod core_graphics;
#[cfg(target_os="macos")]
extern mod core_text;

#[cfg(not(test))]
use compositing::{CompositorChan, CompositorTask};
#[cfg(not(test))]
use constellation::Constellation;
#[cfg(not(test))]
use servo_msg::constellation_msg::InitLoadUrlMsg;

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

#[cfg(not(test))]
use std::os;
#[cfg(not(test))]
use extra::url::Url;
#[cfg(not(test), target_os="android")]
use std::str;
#[cfg(not(test))]
use std::task::TaskOpts;


#[path="compositing/compositor_task.rs"]
pub mod compositing;

pub mod macros;

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
    pub mod display_list_builder;
    pub mod floats;
    pub mod flow;
    pub mod flow_list;
    pub mod layout_task;
    pub mod inline;
    pub mod model;
    pub mod parallel;
    pub mod text;
    pub mod util;
    pub mod incremental;
    pub mod wrapper;
    pub mod extra;
}

pub mod windowing;

#[path="platform/mod.rs"]
pub mod platform;

#[path = "util/mod.rs"]
pub mod util;

#[cfg(not(test), target_os="linux")]
#[cfg(not(test), target_os="macos")]
#[start]
fn start(argc: int, argv: **u8) -> int {
    native::start(argc, argv, proc() {
        opts::from_cmdline_args(os::args()).map(run);
    })
}

#[cfg(not(test), target_os="android")]
#[no_mangle]
pub extern "C" fn android_start(argc: int, argv: **u8) -> int {
    native::start(argc, argv, proc() {
        let mut args:~[~str] = ~[];
        for i in range(0u, argc as uint) {
            unsafe {
                args.push(str::raw::from_c_str(*argv.offset(i as int) as *i8));
            }
        }
        opts::from_cmdline_args(os::args()).map(run);
    })
}

#[cfg(not(test))]
fn run(opts: opts::Opts) {
    let mut pool = green::SchedPool::new(green::PoolConfig::new());

    let (compositor_port, compositor_chan) = CompositorChan::new();
    let profiler_chan = Profiler::create(opts.profiler_period);

    let opts_clone = opts.clone();
    let profiler_chan_clone = profiler_chan.clone();

    let (result_port, result_chan) = Chan::new();
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
                Url::new(~"data", None, ~"", None,
                    filename.slice_from(5).to_owned(), ~[], None)
            } else {
                parse_url(*filename, None)
            };

            constellation_chan.send(InitLoadUrlMsg(url));
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

