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
use std::time::duration::Duration;
use task::{spawn_named};
use url::Url;

// front-end representation of the profiler used to communicate with the profiler
#[deriving(Clone)]
pub struct TimeProfilerChan(pub Sender<TimeProfilerMsg>);

impl TimeProfilerChan {
    pub fn send(&self, msg: TimeProfilerMsg) {
        let TimeProfilerChan(ref c) = *self;
        c.send(msg);
    }
}

#[deriving(PartialEq, Clone, PartialOrd, Eq, Ord)]
pub struct TimerMetadata {
    url:          String,
    iframe:       bool,
    first_reflow: bool,
}

pub trait Formatable {
    fn format(&self) -> String;
}

impl Formatable for Option<TimerMetadata> {
    fn format(&self) -> String {
        match self {
            // TODO(cgaebel): Center-align in the format strings as soon as rustc supports it.
            &Some(ref meta) => {
                let url = meta.url.as_slice();
                let first_reflow = if meta.first_reflow { "    yes" } else { "    no " };
                let iframe = if meta.iframe { "  yes" } else { "  no " };
                format!(" {:14} {:9} {:30}", first_reflow, iframe, url)
            },
            &None =>
                format!(" {:14} {:9} {:30}", "    N/A", "  N/A", "             N/A")
        }
    }
}

#[deriving(Clone)]
pub enum TimeProfilerMsg {
    /// Normal message used for reporting time
    TimeMsg((TimeProfilerCategory, Option<TimerMetadata>), f64),
    /// Message used to force print the profiling metrics
    PrintMsg,
    /// Tells the profiler to shut down.
    ExitMsg,
}

#[repr(u32)]
#[deriving(PartialEq, Clone, PartialOrd, Eq, Ord)]
pub enum TimeProfilerCategory {
    CompositingCategory,
    LayoutPerformCategory,
    LayoutStyleRecalcCategory,
    LayoutSelectorMatchCategory,
    LayoutTreeBuilderCategory,
    LayoutDamagePropagateCategory,
    LayoutMainCategory,
    LayoutParallelWarmupCategory,
    LayoutShapingCategory,
    LayoutDispListBuildCategory,
    RenderingDrawingCategory,
    RenderingPrepBuffCategory,
    RenderingCategory,
}

impl Formatable for TimeProfilerCategory {
    // some categories are subcategories of LayoutPerformCategory
    // and should be printed to indicate this
    fn format(&self) -> String {
        let padding = match *self {
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
        let name = match *self {
            CompositingCategory => "Compositing",
            LayoutPerformCategory => "Layout",
            LayoutStyleRecalcCategory => "Style Recalc",
            LayoutSelectorMatchCategory => "Selector Matching",
            LayoutTreeBuilderCategory => "Tree Building",
            LayoutDamagePropagateCategory => "Damage Propagation",
            LayoutMainCategory => "Primary Layout Pass",
            LayoutParallelWarmupCategory => "Parallel Warmup",
            LayoutShapingCategory => "Shaping",
            LayoutDispListBuildCategory => "Display List Construction",
            RenderingDrawingCategory => "Draw",
            RenderingPrepBuffCategory => "Buffer Prep",
            RenderingCategory => "Rendering",
        };
        format!("{:s}{}", padding, name)
    }
}

type TimeProfilerBuckets = TreeMap<(TimeProfilerCategory, Option<TimerMetadata>), Vec<f64>>;

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
                let period = Duration::milliseconds((period * 1000f64) as i64);
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
            buckets: TreeMap::new(),
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

    fn find_or_insert(&mut self, k: (TimeProfilerCategory, Option<TimerMetadata>), t: f64) {
        match self.buckets.find_mut(&k) {
            None => {},
            Some(v) => { v.push(t); return; },
        }

        self.buckets.insert(k, vec!(t));
    }

    fn handle_msg(&mut self, msg: TimeProfilerMsg) -> bool {
        match msg.clone() {
            TimeMsg(k, t) => self.find_or_insert(k, t),
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
        println!("{:35s} {:14} {:9} {:30} {:15s} {:15s} {:-15s} {:-15s} {:-15s}",
                 "_category_", "_incremental?_", "_iframe?_",
                 "            _url_", "    _mean (ms)_", "  _median (ms)_",
                 "     _min (ms)_", "     _max (ms)_", "      _events_");
        for (&(ref category, ref meta), ref mut data) in self.buckets.iter_mut() {
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
                     data.as_slice()[data_len / 2],
                     data.iter().fold(f64::INFINITY, |a, &b| a.min(b)),
                     data.iter().fold(-f64::INFINITY, |a, &b| a.max(b)));
                println!("{:-35s}{} {:15.4f} {:15.4f} {:15.4f} {:15.4f} {:15u}",
                         category.format(), meta.format(), mean, median, min, max, data_len);
            }
        }
        println!("");
    }
}


pub fn profile<T>(category: TimeProfilerCategory,
                  // url, iframe?, first reflow?
                  meta: Option<(&Url, bool, bool)>,
                  time_profiler_chan: TimeProfilerChan,
                  callback: || -> T)
                  -> T {
    let start_time = precise_time_ns();
    let val = callback();
    let end_time = precise_time_ns();
    let ms = (end_time - start_time) as f64 / 1000000f64;
    let meta = meta.map(|(url, iframe, first_reflow)|
        TimerMetadata {
            url: url.serialize(),
            iframe: iframe,
            first_reflow: first_reflow,
        });
    time_profiler_chan.send(TimeMsg((category, meta), ms));
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
