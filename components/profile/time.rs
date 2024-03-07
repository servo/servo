/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Timing functions.

use std::borrow::ToOwned;
use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;
use std::time::Duration;
use std::{f64, thread, u32, u64};

use ipc_channel::ipc::{self, IpcReceiver};
use profile_traits::time::{
    ProfilerCategory, ProfilerChan, ProfilerData, ProfilerMsg, TimerMetadata,
    TimerMetadataFrameType, TimerMetadataReflowType,
};
use servo_config::opts::OutputOptions;

use crate::trace_dump::TraceDump;

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
                        let url = if url.len() > 30 { &url[..30] } else { url };
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
            None => match *output {
                Some(OutputOptions::FileName(_)) => {
                    format!(" {}\t{}\t{}", "    N/A", "  N/A", "             N/A")
                },
                _ => format!(" {:14} {:9} {:30}", "    N/A", "  N/A", "             N/A"),
            },
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
            _ => "",
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
            ProfilerCategory::ScriptHistoryEvent => "Script History Event",
            ProfilerCategory::ScriptImageCacheMsg => "Script Image Cache Msg",
            ProfilerCategory::ScriptInputEvent => "Script Input Event",
            ProfilerCategory::ScriptNetworkEvent => "Script Network Event",
            ProfilerCategory::ScriptParseHTML => "Script Parse HTML",
            ProfilerCategory::ScriptParseXML => "Script Parse XML",
            ProfilerCategory::ScriptPlannedNavigation => "Script Planned Navigation",
            ProfilerCategory::ScriptPortMessage => "Script Port Message",
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
            ProfilerCategory::ScriptEnterFullscreen => "Script Enter Fullscreen",
            ProfilerCategory::ScriptExitFullscreen => "Script Exit Fullscreen",
            ProfilerCategory::ScriptWebVREvent => "Script WebVR Event",
            ProfilerCategory::ScriptWorkletEvent => "Script Worklet Event",
            ProfilerCategory::ScriptPerformanceEvent => "Script Performance Event",
            ProfilerCategory::ScriptWebGPUMsg => "Script WebGPU Message",
            ProfilerCategory::TimeToFirstPaint => "Time To First Paint",
            ProfilerCategory::TimeToFirstContentfulPaint => "Time To First Contentful Paint",
            ProfilerCategory::TimeToInteractive => "Time to Interactive",
            ProfilerCategory::IpcReceiver => "Blocked at IPC Receive",
            ProfilerCategory::IpcBytesReceiver => "Blocked at IPC Bytes Receive",
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
    blocked_layout_queries: HashMap<String, u32>,
}

impl Profiler {
    pub fn create(output: &Option<OutputOptions>, file_path: Option<String>) -> ProfilerChan {
        let (chan, port) = ipc::channel().unwrap();
        match *output {
            Some(ref option) => {
                // Spawn the time profiler thread
                let outputoption = option.clone();
                thread::Builder::new()
                    .name("TimeProfiler".to_owned())
                    .spawn(move || {
                        let trace = file_path.as_ref().and_then(|p| TraceDump::new(p).ok());
                        let mut profiler = Profiler::new(port, trace, Some(outputoption));
                        profiler.start();
                    })
                    .expect("Thread spawning failed");
                // decide if we need to spawn the timer thread
                match *option {
                    OutputOptions::FileName(_) => { /* no timer thread needed */ },
                    OutputOptions::Stdout(period) => {
                        // Spawn a timer thread
                        let chan = chan.clone();
                        thread::Builder::new()
                            .name("TimeProfTimer".to_owned())
                            .spawn(move || loop {
                                thread::sleep(duration_from_seconds(period));
                                if chan.send(ProfilerMsg::Print).is_err() {
                                    break;
                                }
                            })
                            .expect("Thread spawning failed");
                    },
                }
            },
            None => {
                // this is when the -p option hasn't been specified
                if file_path.is_some() {
                    // Spawn the time profiler
                    thread::Builder::new()
                        .name("TimeProfiler".to_owned())
                        .spawn(move || {
                            let trace = file_path.as_ref().and_then(|p| TraceDump::new(p).ok());
                            let mut profiler = Profiler::new(port, trace, None);
                            profiler.start();
                        })
                        .expect("Thread spawning failed");
                } else {
                    // No-op to handle messages when the time profiler is not printing:
                    thread::Builder::new()
                        .name("TimeProfiler".to_owned())
                        .spawn(move || loop {
                            match port.recv() {
                                Err(_) => break,
                                Ok(ProfilerMsg::Exit(chan)) => {
                                    let _ = chan.send(());
                                    break;
                                },
                                _ => {},
                            }
                        })
                        .expect("Thread spawning failed");
                }
            },
        }

        ProfilerChan(chan)
    }

    pub fn new(
        port: IpcReceiver<ProfilerMsg>,
        trace: Option<TraceDump>,
        output: Option<OutputOptions>,
    ) -> Profiler {
        Profiler {
            port,
            buckets: BTreeMap::new(),
            output,
            last_msg: None,
            trace,
            blocked_layout_queries: HashMap::new(),
        }
    }

    pub fn start(&mut self) {
        while let Ok(msg) = self.port.recv() {
            if !self.handle_msg(msg) {
                break;
            }
        }
    }

    fn find_or_insert(&mut self, k: (ProfilerCategory, Option<TimerMetadata>), t: f64) {
        self.buckets.entry(k).or_default().push(t);
    }

    fn handle_msg(&mut self, msg: ProfilerMsg) -> bool {
        match msg.clone() {
            ProfilerMsg::Time(k, t) => {
                if let Some(ref mut trace) = self.trace {
                    trace.write_one(&k, t);
                }
                let ms = (t.1 - t.0) as f64 / 1000000f64;
                self.find_or_insert(k, ms);
            },
            ProfilerMsg::Print => {
                if let Some(ProfilerMsg::Time(..)) = self.last_msg {
                    // only print if more data has arrived since the last printout
                    self.print_buckets();
                }
            },
            ProfilerMsg::Get(k, sender) => {
                let vec_option = self.buckets.get(&k);
                match vec_option {
                    Some(vec_entry) => sender
                        .send(ProfilerData::Record(vec_entry.to_vec()))
                        .unwrap(),
                    None => sender.send(ProfilerData::NoRecords).unwrap(),
                };
            },
            ProfilerMsg::BlockedLayoutQuery(url) => {
                *self.blocked_layout_queries.entry(url).or_insert(0) += 1;
            },
            ProfilerMsg::Exit(chan) => {
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
            debug_assert!(a <= b, "Data must be sorted");
            b
        });

        let data_len = data.len();
        debug_assert!(data_len > 0);
        let (mean, median, min, max) = (
            data.iter().sum::<f64>() / (data_len as f64),
            data[data_len / 2],
            data[0],
            data[data_len - 1],
        );
        (mean, median, min, max)
    }

    fn print_buckets(&mut self) {
        match self.output {
            Some(OutputOptions::FileName(ref filename)) => {
                let path = Path::new(&filename);
                let mut file = match File::create(path) {
                    Err(e) => panic!("Couldn't create {}: {}", path.display(), e),
                    Ok(file) => file,
                };
                writeln!(
                    file,
                    "_category_\t_incremental?_\t_iframe?_\t_url_\t_mean (ms)_\t\
                     _median (ms)_\t_min (ms)_\t_max (ms)_\t_events_"
                )
                .unwrap();
                for ((category, meta), ref mut data) in &mut self.buckets {
                    data.sort_by(|a, b| a.partial_cmp(b).expect("No NaN values in profiles"));
                    let data_len = data.len();
                    if data_len > 0 {
                        let (mean, median, min, max) = Self::get_statistics(data);
                        writeln!(
                            file,
                            "{}\t{}\t{:15.4}\t{:15.4}\t{:15.4}\t{:15.4}\t{:15}",
                            category.format(&self.output),
                            meta.format(&self.output),
                            mean,
                            median,
                            min,
                            max,
                            data_len
                        )
                        .unwrap();
                    }
                }

                writeln!(file, "_url\t_blocked layout queries_").unwrap();
                for (url, count) in &self.blocked_layout_queries {
                    writeln!(file, "{}\t{}", url, count).unwrap();
                }
            },
            Some(OutputOptions::Stdout(_)) => {
                let stdout = io::stdout();
                let mut lock = stdout.lock();

                writeln!(
                    &mut lock,
                    "{:35} {:14} {:9} {:30} {:15} {:15} {:-15} {:-15} {:-15}",
                    "_category_",
                    "_incremental?_",
                    "_iframe?_",
                    "            _url_",
                    "    _mean (ms)_",
                    "  _median (ms)_",
                    "     _min (ms)_",
                    "     _max (ms)_",
                    "      _events_"
                )
                .unwrap();
                for ((category, meta), ref mut data) in &mut self.buckets {
                    data.sort_by(|a, b| a.partial_cmp(b).expect("No NaN values in profiles"));
                    let data_len = data.len();
                    if data_len > 0 {
                        let (mean, median, min, max) = Self::get_statistics(data);
                        writeln!(
                            &mut lock,
                            "{:-35}{} {:15.4} {:15.4} {:15.4} {:15.4} {:15}",
                            category.format(&self.output),
                            meta.format(&self.output),
                            mean,
                            median,
                            min,
                            max,
                            data_len
                        )
                        .unwrap();
                    }
                }
                writeln!(&mut lock).unwrap();

                writeln!(&mut lock, "_url_\t_blocked layout queries_").unwrap();
                for (url, count) in &self.blocked_layout_queries {
                    writeln!(&mut lock, "{}\t{}", url, count).unwrap();
                }
                writeln!(&mut lock).unwrap();
            },
            None => { /* Do nothing if no output option has been set */ },
        };
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
