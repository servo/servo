/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// Timing functions.
use std::time::precise_time_ns;
use core::cell::Cell;
use core::comm::{Port, SharedChan};
use std::sort::tim_sort;

// front-end representation of the profiler used to communicate with the profiler
#[deriving(Clone)]
pub struct ProfilerChan {
    chan: SharedChan<ProfilerMsg>,
}

impl ProfilerChan {
    pub fn new(chan: Chan<ProfilerMsg>) -> ProfilerChan {
        ProfilerChan {
            chan: SharedChan::new(chan),
        }
    }
    pub fn send(&self, msg: ProfilerMsg) {
        self.chan.send(msg);
    }
}

#[deriving(Eq)]
pub enum ProfilerCategory {
    CompositingCategory,
    LayoutQueryCategory,
    LayoutPerformCategory,
    LayoutAuxInitCategory,
    LayoutSelectorMatchCategory,
    LayoutTreeBuilderCategory,
    LayoutMainCategory,
    LayoutShapingCategory,
    LayoutDispListBuildCategory,
    GfxRegenAvailableFontsCategory,
    RenderingDrawingCategory,
    RenderingPrepBuffCategory,
    RenderingCategory,
    // hackish but helps prevent errors when adding new categories
    NUM_BUCKETS,
}
// FIXME(#5873) this should be initialized by a NUM_BUCKETS cast,
static BUCKETS: uint = 13;

pub enum ProfilerMsg {
    // Normal message used for reporting time
    TimeMsg(ProfilerCategory, f64),
    // Message used to force print the profiling metrics
    PrintMsg,
}

// back end of the profiler that handles data aggregation and performance metrics
pub struct Profiler {
    port: Port<ProfilerMsg>,
    buckets: ~[(ProfilerCategory, ~[f64])],
    last_msg: Option<ProfilerMsg>,
}

impl ProfilerCategory {
    // convenience function to not have to cast every time
    pub fn num_buckets() -> uint {
        NUM_BUCKETS as uint
    }

    // enumeration of all ProfilerCategory types
    // FIXME(tkuehn): this is ugly and error-prone,
    // but currently we lack better alternatives without an enum enumeration
    priv fn empty_buckets() -> ~[(ProfilerCategory, ~[f64])] {
        let mut vec = ~[];
        vec.push((CompositingCategory, ~[]));
        vec.push((LayoutQueryCategory, ~[]));
        vec.push((LayoutPerformCategory, ~[]));
        vec.push((LayoutAuxInitCategory, ~[]));
        vec.push((LayoutSelectorMatchCategory, ~[]));
        vec.push((LayoutTreeBuilderCategory, ~[]));
        vec.push((LayoutMainCategory, ~[]));
        vec.push((LayoutShapingCategory, ~[]));
        vec.push((LayoutDispListBuildCategory, ~[]));
        vec.push((GfxRegenAvailableFontsCategory, ~[]));
        vec.push((RenderingDrawingCategory, ~[]));
        vec.push((RenderingPrepBuffCategory, ~[]));
        vec.push((RenderingCategory, ~[]));

        ProfilerCategory::check_order(vec);
        vec
    }

    priv fn check_order(vec: &[(ProfilerCategory, ~[f64])]) {
        for vec.each |&(category, _)| {
            if category != vec[category as uint].first() {
                fail!("Enum category does not match bucket index. This is a bug.");
            }
        }
    }

    // some categories are subcategories of LayoutPerformCategory
    // and should be printed to indicate this
    pub fn format(self) -> ~str {
        let padding = match self {
            LayoutAuxInitCategory | LayoutSelectorMatchCategory | LayoutTreeBuilderCategory |
            LayoutMainCategory | LayoutDispListBuildCategory | LayoutShapingCategory=> " - ",
            _ => ""
        };
        fmt!("%s%?", padding, self)
    }
}

impl Profiler {
    pub fn create(port: Port<ProfilerMsg>) {
        let port = Cell(port);
        do spawn {
            let mut profiler = Profiler::new(port.take());
            profiler.start();
        }
    }

    pub fn new(port: Port<ProfilerMsg>) -> Profiler {
        Profiler {
            port: port,
            buckets: ProfilerCategory::empty_buckets(),
            last_msg: None,
        }
    }

    pub fn start(&mut self) {
        loop {
            let msg = self.port.recv();
            self.handle_msg(msg);
        }
    }

    priv fn handle_msg(&mut self, msg: ProfilerMsg) {
        match msg {
            TimeMsg(category, t) => match self.buckets[category as uint] {
                // FIXME(#3874): this should be a let (cat, ref mut bucket) = ...,
                // not a match
                (_, ref mut data) => {
                    data.push(t);
                }
            },
            PrintMsg => match self.last_msg {
                Some(TimeMsg(*)) => self.print_buckets(),
                _ => {}
            },
        };
        self.last_msg = Some(msg);
    }

    priv fn print_buckets(&mut self) {
        println(fmt!("%31s %15s %15s %15s %15s %15s",
                         "_category (ms)_", "_mean (ms)_", "_median (ms)_",
                         "_min (ms)_", "_max (ms)_", "_bucket size_"));
        for vec::each_mut(self.buckets) |bucket| {
            match *bucket {
                (category, ref mut data) => {
                    tim_sort(*data);
                    let data_len = data.len();
                    if data_len > 0 {
                        let (mean, median, min, max) =
                            (data.foldl(0f64, |a, b| a + *b) / (data_len as f64),
                             data[data_len / 2],
                             data.min(),
                             data.max());
                        println(fmt!("%-30s: %15.4? %15.4? %15.4? %15.4? %15u",
                                     category.format(), mean, median, min, max, data_len));
                    }
                }
            }
        }
        println("");
    }
}


pub fn profile<T>(category: ProfilerCategory, 
                  profiler_chan: ProfilerChan,
                  callback: &fn() -> T)
                  -> T {
    let start_time = precise_time_ns();
    let val = callback();
    let end_time = precise_time_ns();
    let ms = ((end_time - start_time) as f64 / 1000000f64);
    profiler_chan.send(TimeMsg(category, ms));
    return val;
}

pub fn time<T>(msg: &str, callback: &fn() -> T) -> T{
    let start_time = precise_time_ns();
    let val = callback();
    let end_time = precise_time_ns();
    let ms = ((end_time - start_time) as f64 / 1000000f64);
    if ms >= 5f64 {
        debug!("%s took %? ms", msg, ms);
    }
    return val;
}


