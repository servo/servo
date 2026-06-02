/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Timing functions.

use std::borrow::ToOwned;
use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;
use std::thread;

use base::generic_channel::{self, GenericReceiver};
use profile_traits::time::{
    ProfilerCategory, ProfilerChan, ProfilerData, ProfilerMsg, TimerMetadata,
    TimerMetadataFrameType, TimerMetadataReflowType,
};
use servo_config::opts::OutputOptions;
use time::Duration;

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

type ProfilerBuckets = BTreeMap<(ProfilerCategory, Option<TimerMetadata>), Vec<Duration>>;

// back end of the profiler that handles data aggregation and performance metrics
pub struct Profiler {
    pub port: GenericReceiver<ProfilerMsg>,
    buckets: ProfilerBuckets,
    output: Option<OutputOptions>,
    pub last_msg: Option<ProfilerMsg>,
    trace: Option<TraceDump>,
    blocked_layout_queries: HashMap<String, u32>,
}

impl Profiler {
    pub fn create(output: &Option<OutputOptions>, file_path: Option<String>) -> ProfilerChan {
        match *output {
            Some(ref option) => {
                let (chan, port) = generic_channel::channel().unwrap();
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
                            .spawn(move || {
                                loop {
                                    thread::sleep(std::time::Duration::from_secs_f64(period));
                                    if chan.send(ProfilerMsg::Print).is_err() {
                                        break;
                                    }
                                }
                            })
                            .expect("Thread spawning failed");
                    },
                }

                ProfilerChan(Some(chan))
            },
            None => {
                match file_path {
                    Some(path) => {
                        let (chan, port) = generic_channel::channel().unwrap();
                        // Spawn the time profiler
                        thread::Builder::new()
                            .name("TimeProfiler".to_owned())
                            .spawn(move || {
                                let trace = TraceDump::new(path).ok();
                                let mut profiler = Profiler::new(port, trace, None);
                                profiler.start();
                            })
                            .expect("Thread spawning failed");

                        ProfilerChan(Some(chan))
                    },
                    None => ProfilerChan(None),
                }
            },
        }
    }

    pub fn new(
        port: GenericReceiver<ProfilerMsg>,
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

    fn find_or_insert(&mut self, k: (ProfilerCategory, Option<TimerMetadata>), duration: Duration) {
        self.buckets.entry(k).or_default().push(duration);
    }

    fn handle_msg(&mut self, msg: ProfilerMsg) -> bool {
        match msg.clone() {
            ProfilerMsg::Time(category_and_metadata, (start_time, end_time)) => {
                if let Some(ref mut trace) = self.trace {
                    trace.write_one(&category_and_metadata, start_time, end_time);
                }
                self.find_or_insert(category_and_metadata, end_time - start_time);
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
    pub fn get_statistics(data: &[Duration]) -> (Duration, Duration, Duration, Duration) {
        debug_assert!(
            data.windows(2).all(|window| window[0] <= window[1]),
            "Data must be sorted"
        );

        let data_len = data.len();
        debug_assert!(data_len > 0);
        let (mean, median, min, max) = (
            data.iter().sum::<Duration>() / data_len as u32,
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
                            category.variant_name(),
                            meta.format(&self.output),
                            mean.as_seconds_f64() * 1000.,
                            median.as_seconds_f64() * 1000.,
                            min.as_seconds_f64() * 1000.,
                            max.as_seconds_f64() * 1000.,
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
                            category.variant_name(),
                            meta.format(&self.output),
                            mean.as_seconds_f64() * 1000.,
                            median.as_seconds_f64() * 1000.,
                            min.as_seconds_f64() * 1000.,
                            max.as_seconds_f64() * 1000.,
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
