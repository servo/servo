/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Timing functions.

use std_time::precise_time_ns;
use collections::treemap::TreeMap;
use std::comm::{Sender, channel, Receiver};
use std::f64;
use std::iter::AdditiveIterator;
use std::io::timer::sleep;
use task::{spawn_named};

// front-end representation of the profiler used to communicate with the profiler
#[deriving(Clone)]
pub struct TimeProfilerChan(pub Sender<TimeProfilerMsg>);

impl TimeProfilerChan {
    pub fn send(&self, msg: TimeProfilerMsg) {
        let TimeProfilerChan(ref c) = *self;
        c.send(msg);
    }
}

pub enum TimeProfilerMsg {
    /// Normal message used for reporting time
    TimeMsg(TimeProfilerCategory, f64),
    /// Message used to force print the profiling metrics
    PrintMsg,
    /// Tells the profiler to shut down.
    ExitMsg,
}

#[repr(u32)]
#[deriving(PartialEq, Clone, PartialOrd, Eq, Ord)]
pub enum TimeProfilerCategory {
    CompositingCategory,
    LayoutQueryCategory,
    LayoutPerformCategory,
    LayoutStyleRecalcCategory,
    LayoutSelectorMatchCategory,
    LayoutTreeBuilderCategory,
    LayoutDamagePropagateCategory,
    LayoutMainCategory,
    LayoutParallelWarmupCategory,
    LayoutShapingCategory,
    LayoutDispListBuildCategory,
    GfxRegenAvailableFontsCategory,
    RenderingDrawingCategory,
    RenderingPrepBuffCategory,
    RenderingCategory,
    // FIXME(rust#8803): workaround for lack of CTFE function on enum types to return length
    NumBuckets,
}

impl TimeProfilerCategory {
    // convenience function to not have to cast every time
    pub fn num_buckets() -> uint {
        NumBuckets as uint
    }

    // enumeration of all TimeProfilerCategory types
    fn empty_buckets() -> TimeProfilerBuckets {
        let mut buckets = TreeMap::new();
        buckets.insert(CompositingCategory, vec!());
        buckets.insert(LayoutQueryCategory, vec!());
        buckets.insert(LayoutPerformCategory, vec!());
        buckets.insert(LayoutStyleRecalcCategory, vec!());
        buckets.insert(LayoutSelectorMatchCategory, vec!());
        buckets.insert(LayoutTreeBuilderCategory, vec!());
        buckets.insert(LayoutMainCategory, vec!());
        buckets.insert(LayoutParallelWarmupCategory, vec!());
        buckets.insert(LayoutShapingCategory, vec!());
        buckets.insert(LayoutDamagePropagateCategory, vec!());
        buckets.insert(LayoutDispListBuildCategory, vec!());
        buckets.insert(GfxRegenAvailableFontsCategory, vec!());
        buckets.insert(RenderingDrawingCategory, vec!());
        buckets.insert(RenderingPrepBuffCategory, vec!());
        buckets.insert(RenderingCategory, vec!());

        buckets
    }

    // some categories are subcategories of LayoutPerformCategory
    // and should be printed to indicate this
    pub fn format(self) -> String {
        let padding = match self {
            LayoutStyleRecalcCategory |
            LayoutMainCategory |
            LayoutDispListBuildCategory |
            LayoutShapingCategory |
            LayoutDamagePropagateCategory => "+ ",
            LayoutParallelWarmupCategory |
            LayoutSelectorMatchCategory |
            LayoutTreeBuilderCategory => "| + ",
            _ => ""
        };
        format!("{:s}{:?}", padding, self)
    }
}

type TimeProfilerBuckets = TreeMap<TimeProfilerCategory, Vec<f64>>;

// back end of the profiler that handles data aggregation and performance metrics
pub struct TimeProfiler {
    pub port: Receiver<TimeProfilerMsg>,
    buckets: TimeProfilerBuckets,
    pub last_msg: Option<TimeProfilerMsg>,
}

impl TimeProfiler {
    pub fn create(period: Option<f64>) -> TimeProfilerChan {
        let (chan, port) = channel();
        match period {
            Some(period) => {
                let period = (period * 1000f64) as u64;
                let chan = chan.clone();
                spawn_named("Time profiler timer", proc() {
                    loop {
                        sleep(period);
                        if chan.send_opt(PrintMsg).is_err() {
                            break;
                        }
                    }
                });
                // Spawn the time profiler.
                spawn_named("Time profiler", proc() {
                    let mut profiler = TimeProfiler::new(port);
                    profiler.start();
                });
            }
            None => {
                // No-op to handle messages when the time profiler is inactive.
                spawn_named("Time profiler", proc() {
                    loop {
                        match port.recv_opt() {
                            Err(_) | Ok(ExitMsg) => break,
                            _ => {}
                        }
                    }
                });
            }
        }

        TimeProfilerChan(chan)
    }

    pub fn new(port: Receiver<TimeProfilerMsg>) -> TimeProfiler {
        TimeProfiler {
            port: port,
            buckets: TimeProfilerCategory::empty_buckets(),
            last_msg: None,
        }
    }

    pub fn start(&mut self) {
        loop {
            let msg = self.port.recv_opt();
            match msg {
               Ok(msg) => {
                   if !self.handle_msg(msg) {
                       break
                   }
               }
               _ => break
            }
        }
    }

    fn handle_msg(&mut self, msg: TimeProfilerMsg) -> bool {
        match msg {
            TimeMsg(category, t) => self.buckets.find_mut(&category).unwrap().push(t),
            PrintMsg => match self.last_msg {
                // only print if more data has arrived since the last printout
                Some(TimeMsg(..)) => self.print_buckets(),
                _ => ()
            },
            ExitMsg => return false,
        };
        self.last_msg = Some(msg);
        true
    }

    fn print_buckets(&mut self) {
        println!("{:39s} {:15s} {:15s} {:15s} {:15s} {:15s}",
                 "_category_", "_mean (ms)_", "_median (ms)_",
                 "_min (ms)_", "_max (ms)_", "_bucket size_");
        for (category, data) in self.buckets.mut_iter() {
            data.sort_by(|a, b| {
                if a < b {
                    Less
                } else {
                    Greater
                }
            });
            let data_len = data.len();
            if data_len > 0 {
                let (mean, median, min, max) =
                    (data.iter().map(|&x|x).sum() / (data_len as f64),
                     (*data)[data_len / 2],
                     data.iter().fold(f64::INFINITY, |a, &b| a.min(b)),
                     data.iter().fold(-f64::INFINITY, |a, &b| a.max(b)));
                println!("{:-35s}: {:15.4f} {:15.4f} {:15.4f} {:15.4f} {:15u}",
                         category.format(), mean, median, min, max, data_len);
            }
        }
        println!("");
    }
}


pub fn profile<T>(category: TimeProfilerCategory,
                  time_profiler_chan: TimeProfilerChan,
                  callback: || -> T)
                  -> T {
    let start_time = precise_time_ns();
    let val = callback();
    let end_time = precise_time_ns();
    let ms = (end_time - start_time) as f64 / 1000000f64;
    time_profiler_chan.send(TimeMsg(category, ms));
    return val;
}

pub fn time<T>(msg: &str, callback: || -> T) -> T{
    let start_time = precise_time_ns();
    let val = callback();
    let end_time = precise_time_ns();
    let ms = (end_time - start_time) as f64 / 1000000f64;
    if ms >= 5f64 {
        debug!("{:s} took {} ms", msg, ms);
    }
    return val;
}

// ensure that the order of the buckets matches the order of the enum categories
#[test]
fn check_order() {
    let buckets = TimeProfilerCategory::empty_buckets();
    assert!(buckets.len() == NumBuckets as uint);
}
