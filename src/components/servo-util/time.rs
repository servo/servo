/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// Timing functions.
use std::time::precise_time_ns;
use core::cell::Cell;
use core::comm::{Port, SharedChan};
use core::os::getenv;

pub enum ProfilerCategory {
    CompositingCategory,
    LayoutPerformCategory,
    LayoutQueryCategory,
    LayoutAuxInitCategory,
    LayoutSelectorMatchCategory,
    LayoutTreeBuilderCategory,
    LayoutMainCategory,
    LayoutDispListBuildCategory,
    GfxRegenAvailableFontsCategory,
    RenderingPrepBuffCategory,
    RenderingWaitSubtasksCategory,
    RenderingCategory,
}
// change this whenever buckets are added/rm'd
static NUM_BUCKETS: uint = 12;

pub type ProfilerChan = SharedChan<(ProfilerCategory, uint)>;
pub type ProfilerPort = Port<(ProfilerCategory, uint)>;
pub struct ProfilerTask {
    chan: ProfilerChan,
}

impl ProfilerTask {
    pub fn new(prof_port: ProfilerPort,
               prof_chan: ProfilerChan)
               -> ProfilerTask {
        let prof_port = Cell(prof_port);

        do spawn {
            let mut profiler_context = ProfilerContext::new(prof_port.take());
            profiler_context.start();
        }

        ProfilerTask {
            chan: prof_chan
        }
    }
}

pub struct ProfilerContext {
    port: ProfilerPort,
    buckets: [~[uint], ..NUM_BUCKETS],
    verbose: Option<~str>,
    mut last_print: u64,
}

impl ProfilerContext {
    pub fn new(port: ProfilerPort) -> ProfilerContext {
        ProfilerContext {
            port: port,
            buckets: [~[], ..NUM_BUCKETS],
            verbose: getenv("SERVO_PROFILER"),
            last_print: 0,
        }
    }

    pub fn start(&mut self) {
        loop {
            let msg = self.port.recv();
            self.handle_msg(msg);
        }
    }

    priv fn handle_msg(&mut self, msg: (ProfilerCategory, uint)) {
        let (prof_msg, t) = msg;
        self.buckets[prof_msg as uint].push(t);
        if self.verbose.is_some() {
            let cur_time = precise_time_ns() / 1000000000u64;
            if cur_time - self.last_print > 5 {
                self.last_print = cur_time;
                let mut i = 0;
                for self.buckets.each |bucket| {
                    let prof_msg = match i {
                        // must be in same order as ProfilerCategory
                        0 => CompositingCategory,
                        1 => LayoutPerformCategory,
                        2 => LayoutQueryCategory,
                        3 => LayoutAuxInitCategory,
                        4 => LayoutSelectorMatchCategory,
                        5 => LayoutTreeBuilderCategory,
                        6 => LayoutMainCategory,
                        7 => LayoutDispListBuildCategory,
                        8 => GfxRegenAvailableFontsCategory,
                        9 => RenderingPrepBuffCategory,
                        10 => RenderingWaitSubtasksCategory,
                        11 => RenderingCategory,
                        _ => fail!()
                    };
                    io::println(fmt!("%?: %f", prof_msg,
                                 (bucket.foldl(0 as uint, |a, b| a + *b) as float) / 
                                 (bucket.len() as float)));
                    i += 1;
                }
                io::println("");
            }
        }

    }
}

pub fn profile<T>(cat: ProfilerCategory, 
                  prof_chan: ProfilerChan,
                  callback: &fn() -> T)
                  -> T {
    let start_time = precise_time_ns();
    let val = callback();
    let end_time = precise_time_ns();
    let ms = ((end_time - start_time) / 1000000u64) as uint;
    prof_chan.send((cat, ms));
    return val;
}

pub fn time<T>(msg: &str, callback: &fn() -> T) -> T{
    let start_time = precise_time_ns();
    let val = callback();
    let end_time = precise_time_ns();
    let ms = ((end_time - start_time) / 1000000u64) as uint;
    if ms >= 5 {
        debug!("%s took %u ms", msg, ms);
    }
    return val;
}


