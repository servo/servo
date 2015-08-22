/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Timing functions.

use heartbeats;
use ipc_channel::ipc::{self, IpcReceiver};
use profile_traits::time::{ProfilerCategory, ProfilerChan, ProfilerMsg, TimerMetadata};
use std::borrow::ToOwned;
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::f64;
use std::thread::sleep_ms;
use std_time::precise_time_ns;
use util::task::spawn_named;

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
    pub port: IpcReceiver<ProfilerMsg>,
    buckets: ProfilerBuckets,
    pub last_msg: Option<ProfilerMsg>,
}

impl Profiler {
    pub fn create(period: Option<f64>) -> ProfilerChan {
        let (chan, port) = ipc::channel().unwrap();
        match period {
            Some(period) => {
                let period = (period * 1000.) as u32;
                let chan = chan.clone();
                spawn_named("Time profiler timer".to_owned(), move || {
                    loop {
                        sleep_ms(period);
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
        heartbeats::init();

        ProfilerChan(chan)
    }

    pub fn new(port: IpcReceiver<ProfilerMsg>) -> Profiler {
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
            ProfilerMsg::Time(k, t) => {
                heartbeats::maybe_heartbeat(&k.0, t.0, t.1, 0, 0);
                let ms = (t.1 - t.0) as f64 / 1000000f64;
                self.find_or_insert(k, ms);
            },
            ProfilerMsg::Print => match self.last_msg {
                // only print if more data has arrived since the last printout
                Some(ProfilerMsg::Time(..)) => self.print_buckets(),
                _ => ()
            },
            ProfilerMsg::Exit => {
                heartbeats::cleanup();
                return false;
            },
        };
        self.last_msg = Some(msg);
        true
    }

    fn print_buckets(&mut self) {
        println!("{:35} {:14} {:9} {:30} {:15} {:15} {:-15} {:-15} {:-15}",
                 "_category_", "_incremental?_", "_iframe?_",
                 "            _url_", "    _mean (ms)_", "  _median (ms)_",
                 "     _min (ms)_", "     _max (ms)_", "      _events_");
        for (&(ref category, ref meta), ref mut data) in &mut self.buckets {
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
                    (data.iter().map(|&x|x).sum::<f64>() / (data_len as f64),
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
