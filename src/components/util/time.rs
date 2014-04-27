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
pub struct ProfilerChan(pub Sender<ProfilerMsg>);

impl ProfilerChan {
    pub fn send(&self, msg: ProfilerMsg) {
        let ProfilerChan(ref c) = *self;
        c.send(msg);
    }
}

pub enum ProfilerMsg {
    /// Normal message used for reporting time
    TimeMsg(ProfilerCategory, f64),
    /// Message used to force print the profiling metrics
    PrintMsg,
    /// Tells the profiler to shut down.
    ExitMsg,
}

#[repr(u32)]
#[deriving(Eq, Clone, Ord, TotalEq, TotalOrd)]
pub enum ProfilerCategory {
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
        buckets.insert(LayoutStyleRecalcCategory, ~[]);
        buckets.insert(LayoutSelectorMatchCategory, ~[]);
        buckets.insert(LayoutTreeBuilderCategory, ~[]);
        buckets.insert(LayoutMainCategory, ~[]);
        buckets.insert(LayoutParallelWarmupCategory, ~[]);
        buckets.insert(LayoutShapingCategory, ~[]);
        buckets.insert(LayoutDamagePropagateCategory, ~[]);
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

type ProfilerBuckets = TreeMap<ProfilerCategory, ~[f64]>;

// back end of the profiler that handles data aggregation and performance metrics
pub struct Profiler {
    pub port: Receiver<ProfilerMsg>,
    buckets: ProfilerBuckets,
    pub last_msg: Option<ProfilerMsg>,
}

impl Profiler {
    pub fn create(period: Option<f64>) -> ProfilerChan {
        let (chan, port) = channel();
        match period {
            Some(period) => {
                let period = (period * 1000f64) as u64;
                let chan = chan.clone();
                spawn_named("Profiler timer", proc() {
                    loop {
                        sleep(period);
                        if !chan.try_send(PrintMsg) {
                            break;
                        }
                    }
                });
                // Spawn the profiler
                spawn_named("Profiler", proc() {
                    let mut profiler = Profiler::new(port);
                    profiler.start();
                });
            }
            None => {
                // no-op to handle profiler messages when the profiler is inactive
                spawn_named("Profiler", proc() {
                    loop {
                        match port.recv_opt() {
                            None | Some(ExitMsg) => break,
                            _ => {}
                        }
                    }
                });
            }
        }

        ProfilerChan(chan)
    }

    pub fn new(port: Receiver<ProfilerMsg>) -> Profiler {
        Profiler {
            port: port,
            buckets: ProfilerCategory::empty_buckets(),
            last_msg: None,
        }
    }

    pub fn start(&mut self) {
        loop {
            let msg = self.port.recv_opt();
            match msg {
               Some(msg) => {
                   if !self.handle_msg(msg) {
                       break
                   }
               }
               None => break
            }
        }
    }

    fn handle_msg(&mut self, msg: ProfilerMsg) -> bool {
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
        for (category, data) in self.buckets.iter() {
            // FIXME(XXX): TreeMap currently lacks mut_iter()
            let mut data = data.clone();
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
                     data[data_len / 2],
                     data.iter().fold(f64::INFINITY, |a, &b| a.min(b)),
                     data.iter().fold(-f64::INFINITY, |a, &b| a.max(b)));
                println!("{:-35s}: {:15.4f} {:15.4f} {:15.4f} {:15.4f} {:15u}",
                         category.format(), mean, median, min, max, data_len);
            }
        }
        println!("");
    }
}


pub fn profile<T>(category: ProfilerCategory,
                  profiler_chan: ProfilerChan,
                  callback: || -> T)
                  -> T {
    let start_time = precise_time_ns();
    let val = callback();
    let end_time = precise_time_ns();
    let ms = (end_time - start_time) as f64 / 1000000f64;
    profiler_chan.send(TimeMsg(category, ms));
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
    let buckets = ProfilerCategory::empty_buckets();
    assert!(buckets.len() == NumBuckets as uint);
}
