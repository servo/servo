/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// Timing functions.
use std::comm::{Port, SharedChan};
use std::iterator::AdditiveIterator;
use std::rt::io::timer::Timer;
use std::task::spawn_with;

use extra::sort::tim_sort;
use extra::time::precise_time_ns;
use extra::treemap::TreeMap;

// front-end representation of the profiler used to communicate with the profiler
#[deriving(Clone)]
pub struct ProfilerChan(SharedChan<ProfilerMsg>);
impl ProfilerChan {
    pub fn new(chan: Chan<ProfilerMsg>) -> ProfilerChan {
        ProfilerChan(SharedChan::new(chan))
    }
}

pub enum ProfilerMsg {
    // Normal message used for reporting time
    TimeMsg(ProfilerCategory, float),
    // Message used to force print the profiling metrics
    PrintMsg,
}

#[deriving(Eq, Clone, TotalEq, TotalOrd)]
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
    // FIXME(rust#8803): workaround for lack of CTFE function on enum types to return length
    NumBuckets,
}

impl ProfilerCategory {
    // convenience function to not have to cast every time
    pub fn num_buckets() -> uint {
        NumBuckets as uint
    }

    // enumeration of all ProfilerCategory types
    fn empty_buckets() -> ProfilerBuckets {
        let mut buckets = TreeMap::new();
        buckets.insert(CompositingCategory, ~[]);
        buckets.insert(LayoutQueryCategory, ~[]);
        buckets.insert(LayoutPerformCategory, ~[]);
        buckets.insert(LayoutAuxInitCategory, ~[]);
        buckets.insert(LayoutSelectorMatchCategory, ~[]);
        buckets.insert(LayoutTreeBuilderCategory, ~[]);
        buckets.insert(LayoutMainCategory, ~[]);
        buckets.insert(LayoutShapingCategory, ~[]);
        buckets.insert(LayoutDispListBuildCategory, ~[]);
        buckets.insert(GfxRegenAvailableFontsCategory, ~[]);
        buckets.insert(RenderingDrawingCategory, ~[]);
        buckets.insert(RenderingPrepBuffCategory, ~[]);
        buckets.insert(RenderingCategory, ~[]);

        buckets
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

type ProfilerBuckets = TreeMap<ProfilerCategory, ~[float]>;

// back end of the profiler that handles data aggregation and performance metrics
pub struct Profiler {
    port: Port<ProfilerMsg>,
    buckets: ProfilerBuckets,
    last_msg: Option<ProfilerMsg>,
}

impl Profiler {
    pub fn create(port: Port<ProfilerMsg>, chan: ProfilerChan, period: Option<float>) {
        match period {
            Some(period) => {
                let period = (period * 1000f) as u64;
                do spawn {
                    let mut timer = Timer::new().unwrap();
                    loop {
                        timer.sleep(period);
                        if !chan.try_send(PrintMsg) {
                            break;
                        }
                    }
                }
                // Spawn the profiler
                do spawn_with(port) |port| {
                    let mut profiler = Profiler::new(port);
                    profiler.start();
                }
            }
            None => {
                // no-op to handle profiler messages when the profiler is inactive
                do spawn_with(port) |port| {
                    while port.try_recv().is_some() {}
                }
            }
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
            let msg = self.port.try_recv();
            match msg {
               Some (msg) => self.handle_msg(msg),
               None => break
            }
        }
    }

    fn handle_msg(&mut self, msg: ProfilerMsg) {
        match msg {
            TimeMsg(category, t) => self.buckets.find_mut(&category).unwrap().push(t),
            PrintMsg => match self.last_msg {
                // only print if more data has arrived since the last printout
                Some(TimeMsg(*)) => self.print_buckets(),
                _ => ()
            },
        };
        self.last_msg = Some(msg);
    }

    fn print_buckets(&mut self) {
        println(fmt!("%31s %15s %15s %15s %15s %15s",
                         "_category_", "_mean (ms)_", "_median (ms)_",
                         "_min (ms)_", "_max (ms)_", "_bucket size_"));
        for (category, data) in self.buckets.iter() {
            // FIXME(XXX): TreeMap currently lacks mut_iter()
            let mut data = data.clone();
            tim_sort(data);
            let data_len = data.len();
            if data_len > 0 {
                let (mean, median, &min, &max) =
                    (data.iter().map(|&x|x).sum() / (data_len as float),
                     data[data_len / 2],
                     data.iter().min().unwrap(),
                     data.iter().max().unwrap());
                println(fmt!("%-30s: %15.4f %15.4f %15.4f %15.4f %15u",
                             category.format(), mean, median, min, max, data_len));
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
    let ms = ((end_time - start_time) as float / 1000000f);
    profiler_chan.send(TimeMsg(category, ms));
    return val;
}

pub fn time<T>(msg: &str, callback: &fn() -> T) -> T{
    let start_time = precise_time_ns();
    let val = callback();
    let end_time = precise_time_ns();
    let ms = ((end_time - start_time) as float / 1000000f);
    if ms >= 5f {
        debug!("%s took %? ms", msg, ms);
    }
    return val;
}

#[cfg(test)]
mod test {
    // ensure that the order of the buckets matches the order of the enum categories
    #[test]
    fn check_order() {
        let buckets = ProfilerCategory::empty_buckets();
        assert!(buckets.len() == NumBuckets as uint);
    }
}
