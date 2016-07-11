/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Timing functions.

use heartbeats;
use ipc_channel::ipc::{self, IpcReceiver};
use profile_traits::energy::{energy_interval_ms, read_energy_uj};
use profile_traits::time::{ProfilerCategory, ProfilerChan, ProfilerMsg, TimerMetadata};
use profile_traits::time::{TimerMetadataReflowType, TimerMetadataFrameType};
use std::borrow::ToOwned;
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::{self, Write};
use std::path;
use std::path::Path;
use std::time::Duration;
use std::{f64, thread, u32, u64};
use std_time::precise_time_ns;
use trace_dump::TraceDump;
use util::opts::OutputOptions;
use util::thread::spawn_named;

pub trait Formattable {
    fn format(&self, output: &Option<OutputOptions>) -> String;
}

impl Formattable for Option<TimerMetadata> {
    fn format(&self, output: &Option<OutputOptions>) -> String {
        match *self {
            // TODO(cgaebel): Center-align in the format strings as soon as rustc supports it.
            Some(ref meta) => {
                let url = &*meta.url;
                match *output {
                    Some(OutputOptions::FileName(_)) => {
                        /* The profiling output is a CSV file */
                        let incremental = match meta.incremental {
                            TimerMetadataReflowType::Incremental => "yes",
                            TimerMetadataReflowType::FirstReflow => "no",
                        };
                        let iframe = match meta.iframe {
                            TimerMetadataFrameType::RootWindow => "yes",
                            TimerMetadataFrameType::IFrame => "no",
                        };
                        format!(" {}\t{}\t{}", incremental, iframe, url)
                    },
                    _ => {
                        /* The profiling output is the terminal */
                        let url = if url.len() > 30 {
                            &url[..30]
                        } else {
                            url
                        };
                        let incremental = match meta.incremental {
                            TimerMetadataReflowType::Incremental => "    yes",
                            TimerMetadataReflowType::FirstReflow => "    no ",
                        };
                        let iframe = match meta.iframe {
                            TimerMetadataFrameType::RootWindow => "  yes",
                            TimerMetadataFrameType::IFrame => "  no ",
                        };
                        format!(" {:14} {:9} {:30}", incremental, iframe, url)
                    },
                }
            },
            None => {
                match *output {
                    Some(OutputOptions::FileName(_)) => {
                        format!(" {}\t{}\t{}", "    N/A", "  N/A", "             N/A")
                    },
                    _ => {
                        format!(" {:14} {:9} {:30}", "    N/A", "  N/A", "             N/A")
                    }
                }
            }
        }
    }
}

impl Formattable for ProfilerCategory {
    // some categories are subcategories of LayoutPerformCategory
    // and should be printed to indicate this
    fn format(&self, _output: &Option<OutputOptions>) -> String {
        let padding = match *self {
            ProfilerCategory::LayoutStyleRecalc |
            ProfilerCategory::LayoutRestyleDamagePropagation |
            ProfilerCategory::LayoutNonIncrementalReset |
            ProfilerCategory::LayoutGeneratedContent |
            ProfilerCategory::LayoutDisplayListSorting |
            ProfilerCategory::LayoutFloatPlacementSpeculation |
            ProfilerCategory::LayoutMain |
            ProfilerCategory::LayoutStoreOverflow |
            ProfilerCategory::LayoutDispListBuild |
            ProfilerCategory::LayoutDamagePropagate |
            ProfilerCategory::PaintingPerTile |
            ProfilerCategory::PaintingPrepBuff => "+ ",
            ProfilerCategory::LayoutParallelWarmup |
            ProfilerCategory::LayoutSelectorMatch |
            ProfilerCategory::LayoutTreeBuilder |
            ProfilerCategory::LayoutTextShaping => "| + ",
            _ => ""
        };
        let name = match *self {
            ProfilerCategory::Compositing => "Compositing",
            ProfilerCategory::LayoutPerform => "Layout",
            ProfilerCategory::LayoutStyleRecalc => "Style Recalc",
            ProfilerCategory::LayoutTextShaping => "Text Shaping",
            ProfilerCategory::LayoutRestyleDamagePropagation => "Restyle Damage Propagation",
            ProfilerCategory::LayoutNonIncrementalReset => "Non-incremental reset (temporary)",
            ProfilerCategory::LayoutSelectorMatch => "Selector Matching",
            ProfilerCategory::LayoutTreeBuilder => "Tree Building",
            ProfilerCategory::LayoutDamagePropagate => "Damage Propagation",
            ProfilerCategory::LayoutDisplayListSorting => "Sorting Display List",
            ProfilerCategory::LayoutGeneratedContent => "Generated Content Resolution",
            ProfilerCategory::LayoutFloatPlacementSpeculation => "Float Placement Speculation",
            ProfilerCategory::LayoutMain => "Primary Layout Pass",
            ProfilerCategory::LayoutStoreOverflow => "Store Overflow",
            ProfilerCategory::LayoutParallelWarmup => "Parallel Warmup",
            ProfilerCategory::LayoutDispListBuild => "Display List Construction",
            ProfilerCategory::NetHTTPRequestResponse => "Network HTTP Request/Response",
            ProfilerCategory::PaintingPerTile => "Painting Per Tile",
            ProfilerCategory::PaintingPrepBuff => "Buffer Prep",
            ProfilerCategory::Painting => "Painting",
            ProfilerCategory::ImageDecoding => "Image Decoding",
            ProfilerCategory::ImageSaving => "Image Saving",
            ProfilerCategory::ScriptAttachLayout => "Script Attach Layout",
            ProfilerCategory::ScriptConstellationMsg => "Script Constellation Msg",
            ProfilerCategory::ScriptDevtoolsMsg => "Script Devtools Msg",
            ProfilerCategory::ScriptDocumentEvent => "Script Document Event",
            ProfilerCategory::ScriptDomEvent => "Script Dom Event",
            ProfilerCategory::ScriptEvaluate => "Script JS Evaluate",
            ProfilerCategory::ScriptFileRead => "Script File Read",
            ProfilerCategory::ScriptImageCacheMsg => "Script Image Cache Msg",
            ProfilerCategory::ScriptInputEvent => "Script Input Event",
            ProfilerCategory::ScriptNetworkEvent => "Script Network Event",
            ProfilerCategory::ScriptParseHTML => "Script Parse HTML",
            ProfilerCategory::ScriptPlannedNavigation => "Script Planned Navigation",
            ProfilerCategory::ScriptResize => "Script Resize",
            ProfilerCategory::ScriptEvent => "Script Event",
            ProfilerCategory::ScriptUpdateReplacedElement => "Script Update Replaced Element",
            ProfilerCategory::ScriptSetScrollState => "Script Set Scroll State",
            ProfilerCategory::ScriptSetViewport => "Script Set Viewport",
            ProfilerCategory::ScriptTimerEvent => "Script Timer Event",
            ProfilerCategory::ScriptStylesheetLoad => "Script Stylesheet Load",
            ProfilerCategory::ScriptWebSocketEvent => "Script Web Socket Event",
            ProfilerCategory::ScriptWorkerEvent => "Script Worker Event",
            ProfilerCategory::ScriptServiceWorkerEvent => "Script Service Worker Event",
            ProfilerCategory::ApplicationHeartbeat => "Application Heartbeat",
        };
        format!("{}{}", padding, name)
    }
}

type ProfilerBuckets = BTreeMap<(ProfilerCategory, Option<TimerMetadata>), Vec<f64>>;

// back end of the profiler that handles data aggregation and performance metrics
pub struct Profiler {
    pub port: IpcReceiver<ProfilerMsg>,
    buckets: ProfilerBuckets,
    output: Option<OutputOptions>,
    pub last_msg: Option<ProfilerMsg>,
    trace: Option<TraceDump>,
}

impl Profiler {
    pub fn create(output: &Option<OutputOptions>, file_path: Option<String>) -> ProfilerChan {
        let (chan, port) = ipc::channel().unwrap();
        match *output {
            Some(ref option) => {
                // Spawn the time profiler thread
                let outputoption = option.clone();
                spawn_named("Time profiler".to_owned(), move || {
                    let trace = file_path.as_ref()
                        .map(path::Path::new)
                        .map(fs::File::create)
                        .map(|res| TraceDump::new(res.unwrap()));
                    let mut profiler = Profiler::new(port, trace, Some(outputoption));
                    profiler.start();
                });
                // decide if we need to spawn the timer thread
                match option {
                    &OutputOptions::FileName(_) => { /* no timer thread needed */ },
                    &OutputOptions::Stdout(period) => {
                        // Spawn a timer thread
                        let chan = chan.clone();
                        spawn_named("Time profiler timer".to_owned(), move || {
                            loop {
                                thread::sleep(duration_from_seconds(period));
                                if chan.send(ProfilerMsg::Print).is_err() {
                                    break;
                                }
                            }
                        });
                    },
                }
            },
            None => {
                // this is when the -p option hasn't been specified
                if file_path.is_some() {
                    // Spawn the time profiler
                    spawn_named("Time profiler".to_owned(), move || {
                        let trace = file_path.as_ref()
                            .map(path::Path::new)
                            .map(fs::File::create)
                            .map(|res| TraceDump::new(res.unwrap()));
                        let mut profiler = Profiler::new(port, trace, None);
                        profiler.start();
                    });
                } else {
                    // No-op to handle messages when the time profiler is not printing:
                    spawn_named("Time profiler".to_owned(), move || {
                        loop {
                            match port.recv() {
                                Err(_) => break,
                                Ok(ProfilerMsg::Exit(chan)) => {
                                    let _ = chan.send(());
                                    break;
                                },
                                _ => {}
                            }
                        }
                    });
                }
            }
        }

        heartbeats::init();
        let profiler_chan = ProfilerChan(chan);

        // only spawn the application-level profiler thread if its heartbeat is enabled
        let run_ap_thread = || {
            heartbeats::is_heartbeat_enabled(&ProfilerCategory::ApplicationHeartbeat)
        };
        if run_ap_thread() {
            let profiler_chan = profiler_chan.clone();
            // min of 1 heartbeat/sec, max of 20 should provide accurate enough power/energy readings
            // waking up more frequently allows the thread to end faster on exit
            const SLEEP_MS: u32 = 10;
            const MIN_ENERGY_INTERVAL_MS: u32 = 50;
            const MAX_ENERGY_INTERVAL_MS: u32 = 1000;
            let interval_ms = enforce_range(MIN_ENERGY_INTERVAL_MS, MAX_ENERGY_INTERVAL_MS, energy_interval_ms());
            let loop_count: u32 = (interval_ms as f32 / SLEEP_MS as f32).ceil() as u32;
            spawn_named("Application heartbeat profiler".to_owned(), move || {
                let mut start_time = precise_time_ns();
                let mut start_energy = read_energy_uj();
                loop {
                    for _ in 0..loop_count {
                        if run_ap_thread() {
                            thread::sleep(Duration::from_millis(SLEEP_MS as u64))
                        } else {
                            return
                        }
                    }
                    let end_time = precise_time_ns();
                    let end_energy = read_energy_uj();
                    // send using the inner channel
                    // (using ProfilerChan.send() forces an unwrap and sometimes panics for this background profiler)
                    let ProfilerChan(ref c) = profiler_chan;
                    match c.send(ProfilerMsg::Time((ProfilerCategory::ApplicationHeartbeat, None),
                                                   (start_time, end_time),
                                                   (start_energy, end_energy))) {
                        Ok(_) => {},
                        Err(_) => return,
                    };
                    start_time = end_time;
                    start_energy = end_energy;
                }
            });
        }

        profiler_chan
    }

    pub fn new(port: IpcReceiver<ProfilerMsg>, trace: Option<TraceDump>, output: Option<OutputOptions>) -> Profiler {
        Profiler {
            port: port,
            buckets: BTreeMap::new(),
            output: output,
            last_msg: None,
            trace: trace,
        }
    }

    pub fn start(&mut self) {
        while let Ok(msg) = self.port.recv() {
           if !self.handle_msg(msg) {
               break
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
            ProfilerMsg::Time(k, t, e) => {
                heartbeats::maybe_heartbeat(&k.0, t.0, t.1, e.0, e.1);
                if let Some(ref mut trace) = self.trace {
                    trace.write_one(&k, t, e);
                }
                let ms = (t.1 - t.0) as f64 / 1000000f64;
                self.find_or_insert(k, ms);
            },
            ProfilerMsg::Print => if let Some(ProfilerMsg::Time(..)) = self.last_msg {
                // only print if more data has arrived since the last printout
                self.print_buckets();
            },
            ProfilerMsg::Exit(chan) => {
                heartbeats::cleanup();
                self.print_buckets();
                let _ = chan.send(());
                return false;
            },
        };
        self.last_msg = Some(msg);
        true
    }

    /// Get tuple (mean, median, min, max) for profiler statistics.
    pub fn get_statistics(data: &[f64]) -> (f64, f64, f64, f64) {
        data.iter().fold(-f64::INFINITY, |a, &b| {
            debug_assert!(a < b, "Data must be sorted");
            b
        });

        let data_len = data.len();
        debug_assert!(data_len > 0);
        let (mean, median, min, max) =
            (data.iter().sum::<f64>() / (data_len as f64),
            data[data_len / 2],
            data[0],
            data[data_len - 1]);
        (mean, median, min, max)
    }

    fn print_buckets(&mut self) {
        match self.output {
            Some(OutputOptions::FileName(ref filename)) => {
                let path = Path::new(&filename);
                let mut file = match File::create(&path) {
                    Err(e) => panic!("Couldn't create {}: {}",
                                     path.display(),
                                     Error::description(&e)),
                    Ok(file) => file,
                };
                write!(file, "_category_\t_incremental?_\t_iframe?_\t_url_\t_mean (ms)_\t\
                    _median (ms)_\t_min (ms)_\t_max (ms)_\t_events_\n").unwrap();
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
                        let (mean, median, min, max) = Self::get_statistics(data);
                        write!(file, "{}\t{}\t{:15.4}\t{:15.4}\t{:15.4}\t{:15.4}\t{:15}\n",
                            category.format(&self.output), meta.format(&self.output),
                            mean, median, min, max, data_len).unwrap();
                    }
                }
            },
            Some(OutputOptions::Stdout(_)) => {
                let stdout = io::stdout();
                let mut lock = stdout.lock();

                writeln!(&mut lock, "{:35} {:14} {:9} {:30} {:15} {:15} {:-15} {:-15} {:-15}",
                         "_category_", "_incremental?_", "_iframe?_",
                         "            _url_", "    _mean (ms)_", "  _median (ms)_",
                         "     _min (ms)_", "     _max (ms)_", "      _events_").unwrap();
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
                        let (mean, median, min, max) = Self::get_statistics(data);
                        writeln!(&mut lock, "{:-35}{} {:15.4} {:15.4} {:15.4} {:15.4} {:15}",
                                 category.format(&self.output), meta.format(&self.output), mean, median, min, max,
                                 data_len).unwrap();
                    }
                }
                writeln!(&mut lock, "").unwrap();
            },
            None => { /* Do nothing if not output option has been set */ },
        };
    }
}

fn enforce_range<T>(min: T, max: T, value: T) -> T where T: Ord {
    assert!(min <= max);
    match value.cmp(&max) {
        Ordering::Equal | Ordering::Greater => max,
        Ordering::Less => {
            match value.cmp(&min) {
                Ordering::Equal | Ordering::Less => min,
                Ordering::Greater => value,
            }
        },
    }
}

pub fn duration_from_seconds(secs: f64) -> Duration {
    pub const NANOS_PER_SEC: u32 = 1_000_000_000;

    // Get number of seconds and check that it fits in a u64.
    let whole_secs = secs.trunc();
    assert!(whole_secs >= 0.0 && whole_secs <= u64::MAX as f64);

    // Get number of nanoseconds. This should always fit in a u32, but check anyway.
    let nanos = (secs.fract() * (NANOS_PER_SEC as f64)).trunc();
    assert!(nanos >= 0.0 && nanos <= u32::MAX as f64);

    Duration::new(whole_secs as u64, nanos as u32)
}
