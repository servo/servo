/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Timing functions.

use collections::BTreeMap;
use std::borrow::ToOwned;
use std::cmp::Ordering;
use std::f64;
use std::old_io::timer::sleep;
use std::iter::AdditiveIterator;
use std::num::Float;
use std::sync::mpsc::{Sender, channel, Receiver};
use std::time::duration::Duration;
use std_time::precise_time_ns;
use url::Url;
use util::task::spawn_named;

// front-end representation of the profiler used to communicate with the profiler
#[derive(Clone)]
pub struct ProfilerChan(pub Sender<ProfilerMsg>);

impl ProfilerChan {
    pub fn send(&self, msg: ProfilerMsg) {
        let ProfilerChan(ref c) = *self;
        c.send(msg).unwrap();
    }
}

#[derive(PartialEq, Clone, PartialOrd, Eq, Ord)]
pub struct TimerMetadata {
    url:         String,
    iframe:      bool,
    incremental: bool,
}

pub trait Formattable {
    fn format(&self) -> String;
}

impl Formattable for Option<TimerMetadata> {
    fn format(&self) -> String {
        match self {
            // TODO(cgaebel): Center-align in the format strings as soon as rustc supports it.
            &Some(ref meta) => {
                let url = &*meta.url;
                let url = if url.len() > 30 {
                    &url[..30]
                } else {
                    url
                };
                let incremental = if meta.incremental { "    yes" } else { "    no " };
                let iframe = if meta.iframe { "  yes" } else { "  no " };
                format!(" {:14} {:9} {:30}", incremental, iframe, url)
            },
            &None =>
                format!(" {:14} {:9} {:30}", "    N/A", "  N/A", "             N/A")
        }
    }
}

#[derive(Clone)]
pub enum ProfilerMsg {
    /// Normal message used for reporting time
    Time((ProfilerCategory, Option<TimerMetadata>), f64),
    /// Message used to force print the profiling metrics
    Print,
    /// Tells the profiler to shut down.
    Exit,
}

#[repr(u32)]
#[derive(PartialEq, Clone, PartialOrd, Eq, Ord)]
pub enum ProfilerCategory {
    Compositing,
    LayoutPerform,
    LayoutStyleRecalc,
    LayoutRestyleDamagePropagation,
    LayoutNonIncrementalReset,
    LayoutSelectorMatch,
    LayoutTreeBuilder,
    LayoutDamagePropagate,
    LayoutGeneratedContent,
    LayoutMain,
    LayoutParallelWarmup,
    LayoutShaping,
    LayoutDispListBuild,
    PaintingPerTile,
    PaintingPrepBuff,
    Painting,
    ImageDecoding,
}

impl Formattable for ProfilerCategory {
    // some categories are subcategories of LayoutPerformCategory
    // and should be printed to indicate this
    fn format(&self) -> String {
        let padding = match *self {
            ProfilerCategory::LayoutStyleRecalc |
            ProfilerCategory::LayoutRestyleDamagePropagation |
            ProfilerCategory::LayoutNonIncrementalReset |
            ProfilerCategory::LayoutGeneratedContent |
            ProfilerCategory::LayoutMain |
            ProfilerCategory::LayoutDispListBuild |
            ProfilerCategory::LayoutShaping |
            ProfilerCategory::LayoutDamagePropagate |
            ProfilerCategory::PaintingPerTile |
            ProfilerCategory::PaintingPrepBuff => "+ ",
            ProfilerCategory::LayoutParallelWarmup |
            ProfilerCategory::LayoutSelectorMatch |
            ProfilerCategory::LayoutTreeBuilder => "| + ",
            _ => ""
        };
        let name = match *self {
            ProfilerCategory::Compositing => "Compositing",
            ProfilerCategory::LayoutPerform => "Layout",
            ProfilerCategory::LayoutStyleRecalc => "Style Recalc",
            ProfilerCategory::LayoutRestyleDamagePropagation => "Restyle Damage Propagation",
            ProfilerCategory::LayoutNonIncrementalReset => "Non-incremental reset (temporary)",
            ProfilerCategory::LayoutSelectorMatch => "Selector Matching",
            ProfilerCategory::LayoutTreeBuilder => "Tree Building",
            ProfilerCategory::LayoutDamagePropagate => "Damage Propagation",
            ProfilerCategory::LayoutGeneratedContent => "Generated Content Resolution",
            ProfilerCategory::LayoutMain => "Primary Layout Pass",
            ProfilerCategory::LayoutParallelWarmup => "Parallel Warmup",
            ProfilerCategory::LayoutShaping => "Shaping",
            ProfilerCategory::LayoutDispListBuild => "Display List Construction",
            ProfilerCategory::PaintingPerTile => "Painting Per Tile",
            ProfilerCategory::PaintingPrepBuff => "Buffer Prep",
            ProfilerCategory::Painting => "Painting",
            ProfilerCategory::ImageDecoding => "Image Decoding",
        };
        format!("{}{}", padding, name)
    }
}

type ProfilerBuckets = BTreeMap<(ProfilerCategory, Option<TimerMetadata>), Vec<f64>>;

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
                let period = Duration::milliseconds((period * 1000f64) as i64);
                let chan = chan.clone();
                spawn_named("Time profiler timer".to_owned(), move || {
                    loop {
                        sleep(period);
                        if chan.send(ProfilerMsg::Print).is_err() {
                            break;
                        }
                    }
                });
                // Spawn the time profiler.
                spawn_named("Time profiler".to_owned(), move || {
                    let mut profiler = Profiler::new(port);
                    profiler.start();
                });
            }
            None => {
                // No-op to handle messages when the time profiler is inactive.
                spawn_named("Time profiler".to_owned(), move || {
                    loop {
                        match port.recv() {
                            Err(_) | Ok(ProfilerMsg::Exit) => break,
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
            buckets: BTreeMap::new(),
            last_msg: None,
        }
    }

    pub fn start(&mut self) {
        loop {
            let msg = self.port.recv();
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

    fn find_or_insert(&mut self, k: (ProfilerCategory, Option<TimerMetadata>), t: f64) {
        match self.buckets.get_mut(&k) {
            None => {},
            Some(v) => { v.push(t); return; },
        }

        self.buckets.insert(k, vec!(t));
    }

    fn handle_msg(&mut self, msg: ProfilerMsg) -> bool {
        match msg.clone() {
            ProfilerMsg::Time(k, t) => self.find_or_insert(k, t),
            ProfilerMsg::Print => match self.last_msg {
                // only print if more data has arrived since the last printout
                Some(ProfilerMsg::Time(..)) => self.print_buckets(),
                _ => ()
            },
            ProfilerMsg::Exit => return false,
        };
        self.last_msg = Some(msg);
        true
    }

    fn print_buckets(&mut self) {
        println!("{:35} {:14} {:9} {:30} {:15} {:15} {:-15} {:-15} {:-15}",
                 "_category_", "_incremental?_", "_iframe?_",
                 "            _url_", "    _mean (ms)_", "  _median (ms)_",
                 "     _min (ms)_", "     _max (ms)_", "      _events_");
        for (&(ref category, ref meta), ref mut data) in self.buckets.iter_mut() {
            data.sort_by(|a, b| {
                if a < b {
                    Ordering::Less
                } else {
                    Ordering::Greater
                }
            });
            let data_len = data.len();
            if data_len > 0 {
                let (mean, median, min, max) =
                    (data.iter().map(|&x|x).sum() / (data_len as f64),
                     data[data_len / 2],
                     data.iter().fold(f64::INFINITY, |a, &b| a.min(b)),
                     data.iter().fold(-f64::INFINITY, |a, &b| a.max(b)));
                println!("{:-35}{} {:15.4} {:15.4} {:15.4} {:15.4} {:15}",
                         category.format(), meta.format(), mean, median, min, max, data_len);
            }
        }
        println!("");
    }
}

#[derive(Eq, PartialEq)]
pub enum TimerMetadataFrameType {
    RootWindow,
    IFrame,
}

#[derive(Eq, PartialEq)]
pub enum TimerMetadataReflowType {
    Incremental,
    FirstReflow,
}

pub type ProfilerMetadata<'a> = Option<(&'a Url, TimerMetadataFrameType, TimerMetadataReflowType)>;

pub fn profile<T, F>(category: ProfilerCategory,
                     meta: ProfilerMetadata,
                     profiler_chan: ProfilerChan,
                     callback: F)
                  -> T
    where F: FnOnce() -> T
{
    let start_time = precise_time_ns();
    let val = callback();
    let end_time = precise_time_ns();
    let ms = (end_time - start_time) as f64 / 1000000f64;
    let meta = meta.map(|(url, iframe, reflow_type)|
        TimerMetadata {
            url: url.serialize(),
            iframe: iframe == TimerMetadataFrameType::IFrame,
            incremental: reflow_type == TimerMetadataReflowType::Incremental,
        });
    profiler_chan.send(ProfilerMsg::Time((category, meta), ms));
    return val;
}

pub fn time<T, F>(msg: &str, callback: F) -> T
    where F: Fn() -> T
{
    let start_time = precise_time_ns();
    let val = callback();
    let end_time = precise_time_ns();
    let ms = (end_time - start_time) as f64 / 1000000f64;
    if ms >= 5f64 {
        debug!("{} took {} ms", msg, ms);
    }
    return val;
}
