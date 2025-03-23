/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Memory profiling functions.

use std::borrow::ToOwned;
use std::collections::HashMap;
use std::thread;

use ipc_channel::ipc::{self, IpcReceiver};
use ipc_channel::router::ROUTER;
use profile_traits::mem::{
    MemoryReportResult, ProfilerChan, ProfilerMsg, Report, Reporter, ReporterRequest, ReportsChan,
};

pub struct Profiler {
    /// The port through which messages are received.
    pub port: IpcReceiver<ProfilerMsg>,

    /// Registered memory reporters.
    reporters: HashMap<String, Reporter>,
}

const JEMALLOC_HEAP_ALLOCATED_STR: &str = "jemalloc-heap-allocated";
const SYSTEM_HEAP_ALLOCATED_STR: &str = "system-heap-allocated";

impl Profiler {
    pub fn create() -> ProfilerChan {
        let (chan, port) = ipc::channel().unwrap();

        // Always spawn the memory profiler. If there is no timer thread it won't receive regular
        // `Print` events, but it will still receive the other events.
        thread::Builder::new()
            .name("MemoryProfiler".to_owned())
            .spawn(move || {
                let mut mem_profiler = Profiler::new(port);
                mem_profiler.start();
            })
            .expect("Thread spawning failed");

        let mem_profiler_chan = ProfilerChan(chan);

        // Register the system memory reporter, which will run on its own thread. It never needs to
        // be unregistered, because as long as the memory profiler is running the system memory
        // reporter can make measurements.
        let (system_reporter_sender, system_reporter_receiver) = ipc::channel().unwrap();
        ROUTER.add_typed_route(
            system_reporter_receiver,
            Box::new(|message| {
                let request: ReporterRequest = message.unwrap();
                system_reporter::collect_reports(request)
            }),
        );
        mem_profiler_chan.send(ProfilerMsg::RegisterReporter(
            "system".to_owned(),
            Reporter(system_reporter_sender),
        ));

        mem_profiler_chan
    }

    pub fn new(port: IpcReceiver<ProfilerMsg>) -> Profiler {
        Profiler {
            port,
            reporters: HashMap::new(),
        }
    }

    pub fn start(&mut self) {
        while let Ok(msg) = self.port.recv() {
            if !self.handle_msg(msg) {
                break;
            }
        }
    }

    fn handle_msg(&mut self, msg: ProfilerMsg) -> bool {
        match msg {
            ProfilerMsg::RegisterReporter(name, reporter) => {
                // Panic if it has already been registered.
                let name_clone = name.clone();
                match self.reporters.insert(name, reporter) {
                    None => true,
                    Some(_) => panic!("RegisterReporter: '{}' name is already in use", name_clone),
                }
            },

            ProfilerMsg::UnregisterReporter(name) => {
                // Panic if it hasn't previously been registered.
                match self.reporters.remove(&name) {
                    Some(_) => true,
                    None => panic!("UnregisterReporter: '{}' name is unknown", &name),
                }
            },

            ProfilerMsg::Report(sender) => {
                let reports = self.collect_reports();
                let content = serde_json::to_string(&reports)
                    .unwrap_or_else(|_| "{ error: \"failed to create memory report\"}".to_owned());
                let _ = sender.send(MemoryReportResult { content });
                true
            },

            ProfilerMsg::Exit => false,
        }
    }

    fn collect_reports(&self) -> Vec<Report> {
        let mut result = vec![];
        for reporter in self.reporters.values() {
            let (chan, port) = ipc::channel().unwrap();
            reporter.collect_reports(ReportsChan(chan));
            if let Ok(mut reports) = port.recv() {
                result.append(&mut reports);
            }
        }
        result
    }
}

//---------------------------------------------------------------------------

mod system_reporter {
    #[cfg(not(any(target_os = "windows", target_env = "ohos")))]
    use std::ffi::CString;
    #[cfg(not(any(target_os = "windows", target_env = "ohos")))]
    use std::mem::size_of;
    #[cfg(not(any(target_os = "windows", target_env = "ohos")))]
    use std::ptr::null_mut;

    #[cfg(all(target_os = "linux", target_env = "gnu"))]
    use libc::c_int;
    #[cfg(not(any(target_os = "windows", target_env = "ohos")))]
    use libc::{c_void, size_t};
    use profile_traits::mem::{Report, ReportKind, ReporterRequest};
    use profile_traits::path;
    #[cfg(target_os = "macos")]
    use task_info::task_basic_info::{resident_size, virtual_size};

    use super::{JEMALLOC_HEAP_ALLOCATED_STR, SYSTEM_HEAP_ALLOCATED_STR};

    /// Collects global measurements from the OS and heap allocators.
    pub fn collect_reports(request: ReporterRequest) {
        let mut reports = vec![];
        {
            let mut report = |path, size| {
                if let Some(size) = size {
                    reports.push(Report {
                        path,
                        kind: ReportKind::NonExplicitSize,
                        size,
                    });
                }
            };

            // Virtual and physical memory usage, as reported by the OS.
            report(path!["vsize"], vsize());
            report(path!["resident"], resident());

            // Memory segments, as reported by the OS.
            for seg in resident_segments() {
                report(path!["resident-according-to-smaps", seg.0], Some(seg.1));
            }

            // Total number of bytes allocated by the application on the system
            // heap.
            report(path![SYSTEM_HEAP_ALLOCATED_STR], system_heap_allocated());

            // The descriptions of the following jemalloc measurements are taken
            // directly from the jemalloc documentation.

            // "Total number of bytes allocated by the application."
            report(
                path![JEMALLOC_HEAP_ALLOCATED_STR],
                jemalloc_stat("stats.allocated"),
            );

            // "Total number of bytes in active pages allocated by the application.
            // This is a multiple of the page size, and greater than or equal to
            // |stats.allocated|."
            report(path!["jemalloc-heap-active"], jemalloc_stat("stats.active"));

            // "Total number of bytes in chunks mapped on behalf of the application.
            // This is a multiple of the chunk size, and is at least as large as
            // |stats.active|. This does not include inactive chunks."
            report(path!["jemalloc-heap-mapped"], jemalloc_stat("stats.mapped"));
        }

        request.reports_channel.send(reports);
    }

    #[cfg(all(target_os = "linux", target_env = "gnu"))]
    unsafe extern "C" {
        fn mallinfo() -> struct_mallinfo;
    }

    #[cfg(all(target_os = "linux", target_env = "gnu"))]
    #[repr(C)]
    pub struct struct_mallinfo {
        arena: c_int,
        ordblks: c_int,
        smblks: c_int,
        hblks: c_int,
        hblkhd: c_int,
        usmblks: c_int,
        fsmblks: c_int,
        uordblks: c_int,
        fordblks: c_int,
        keepcost: c_int,
    }

    #[cfg(all(target_os = "linux", target_env = "gnu"))]
    fn system_heap_allocated() -> Option<usize> {
        let info: struct_mallinfo = unsafe { mallinfo() };

        // The documentation in the glibc man page makes it sound like |uordblks| would suffice,
        // but that only gets the small allocations that are put in the brk heap. We need |hblkhd|
        // as well to get the larger allocations that are mmapped.
        //
        // These fields are unfortunately |int| and so can overflow (becoming negative) if memory
        // usage gets high enough. So don't report anything in that case. In the non-overflow case
        // we cast the two values to usize before adding them to make sure the sum also doesn't
        // overflow.
        if info.hblkhd < 0 || info.uordblks < 0 {
            None
        } else {
            Some(info.hblkhd as usize + info.uordblks as usize)
        }
    }

    #[cfg(not(all(target_os = "linux", target_env = "gnu")))]
    fn system_heap_allocated() -> Option<usize> {
        None
    }

    #[cfg(not(any(target_os = "windows", target_env = "ohos")))]
    use tikv_jemalloc_sys::mallctl;

    #[cfg(not(any(target_os = "windows", target_env = "ohos")))]
    fn jemalloc_stat(value_name: &str) -> Option<usize> {
        // Before we request the measurement of interest, we first send an "epoch"
        // request. Without that jemalloc gives cached statistics(!) which can be
        // highly inaccurate.
        let epoch_name = "epoch";
        let epoch_c_name = CString::new(epoch_name).unwrap();
        let mut epoch: u64 = 0;
        let epoch_ptr = &mut epoch as *mut _ as *mut c_void;
        let mut epoch_len = size_of::<u64>() as size_t;

        let value_c_name = CString::new(value_name).unwrap();
        let mut value: size_t = 0;
        let value_ptr = &mut value as *mut _ as *mut c_void;
        let mut value_len = size_of::<size_t>() as size_t;

        // Using the same values for the `old` and `new` parameters is enough
        // to get the statistics updated.
        let rv = unsafe {
            mallctl(
                epoch_c_name.as_ptr(),
                epoch_ptr,
                &mut epoch_len,
                epoch_ptr,
                epoch_len,
            )
        };
        if rv != 0 {
            return None;
        }

        let rv = unsafe {
            mallctl(
                value_c_name.as_ptr(),
                value_ptr,
                &mut value_len,
                null_mut(),
                0,
            )
        };
        if rv != 0 {
            return None;
        }

        Some(value as usize)
    }

    #[cfg(any(target_os = "windows", target_env = "ohos"))]
    fn jemalloc_stat(_value_name: &str) -> Option<usize> {
        None
    }

    #[cfg(target_os = "linux")]
    fn page_size() -> usize {
        unsafe { ::libc::sysconf(::libc::_SC_PAGESIZE) as usize }
    }

    #[cfg(target_os = "linux")]
    fn proc_self_statm_field(field: usize) -> Option<usize> {
        use std::fs::File;
        use std::io::Read;

        let mut f = File::open("/proc/self/statm").ok()?;
        let mut contents = String::new();
        f.read_to_string(&mut contents).ok()?;
        let s = contents.split_whitespace().nth(field)?;
        let npages = s.parse::<usize>().ok()?;
        Some(npages * page_size())
    }

    #[cfg(target_os = "linux")]
    fn vsize() -> Option<usize> {
        proc_self_statm_field(0)
    }

    #[cfg(target_os = "linux")]
    fn resident() -> Option<usize> {
        proc_self_statm_field(1)
    }

    #[cfg(target_os = "macos")]
    fn vsize() -> Option<usize> {
        virtual_size()
    }

    #[cfg(target_os = "macos")]
    fn resident() -> Option<usize> {
        resident_size()
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    fn vsize() -> Option<usize> {
        None
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    fn resident() -> Option<usize> {
        None
    }

    #[cfg(target_os = "linux")]
    fn resident_segments() -> Vec<(String, usize)> {
        use std::collections::HashMap;
        use std::collections::hash_map::Entry;
        use std::fs::File;
        use std::io::{BufRead, BufReader};

        use regex::Regex;

        // The first line of an entry in /proc/<pid>/smaps looks just like an entry
        // in /proc/<pid>/maps:
        //
        //   address           perms offset  dev   inode  pathname
        //   02366000-025d8000 rw-p 00000000 00:00 0      [heap]
        //
        // Each of the following lines contains a key and a value, separated
        // by ": ", where the key does not contain either of those characters.
        // For example:
        //
        //   Rss:           132 kB

        let f = match File::open("/proc/self/smaps") {
            Ok(f) => BufReader::new(f),
            Err(_) => return vec![],
        };

        let seg_re = Regex::new(
            r"^[:xdigit:]+-[:xdigit:]+ (....) [:xdigit:]+ [:xdigit:]+:[:xdigit:]+ \d+ +(.*)",
        )
        .unwrap();
        let rss_re = Regex::new(r"^Rss: +(\d+) kB").unwrap();

        // We record each segment's resident size.
        let mut seg_map: HashMap<String, usize> = HashMap::new();

        #[derive(PartialEq)]
        enum LookingFor {
            Segment,
            Rss,
        }
        let mut looking_for = LookingFor::Segment;

        let mut curr_seg_name = String::new();

        // Parse the file.
        for line in f.lines() {
            let line = match line {
                Ok(line) => line,
                Err(_) => continue,
            };
            if looking_for == LookingFor::Segment {
                // Look for a segment info line.
                let cap = match seg_re.captures(&line) {
                    Some(cap) => cap,
                    None => continue,
                };
                let perms = cap.get(1).unwrap().as_str();
                let pathname = cap.get(2).unwrap().as_str();

                // Construct the segment name from its pathname and permissions.
                curr_seg_name.clear();
                if pathname.is_empty() || pathname.starts_with("[stack:") {
                    // Anonymous memory. Entries marked with "[stack:nnn]"
                    // look like thread stacks but they may include other
                    // anonymous mappings, so we can't trust them and just
                    // treat them as entirely anonymous.
                    curr_seg_name.push_str("anonymous");
                } else {
                    curr_seg_name.push_str(pathname);
                }
                curr_seg_name.push_str(" (");
                curr_seg_name.push_str(perms);
                curr_seg_name.push(')');

                looking_for = LookingFor::Rss;
            } else {
                // Look for an "Rss:" line.
                let cap = match rss_re.captures(&line) {
                    Some(cap) => cap,
                    None => continue,
                };
                let rss = cap.get(1).unwrap().as_str().parse::<usize>().unwrap() * 1024;

                if rss > 0 {
                    // Aggregate small segments into "other".
                    let seg_name = if rss < 512 * 1024 {
                        "other".to_owned()
                    } else {
                        curr_seg_name.clone()
                    };
                    match seg_map.entry(seg_name) {
                        Entry::Vacant(entry) => {
                            entry.insert(rss);
                        },
                        Entry::Occupied(mut entry) => *entry.get_mut() += rss,
                    }
                }

                looking_for = LookingFor::Segment;
            }
        }

        // Note that the sum of all these segments' RSS values differs from the "resident"
        // measurement obtained via /proc/<pid>/statm in resident(). It's unclear why this
        // difference occurs; for some processes the measurements match, but for Servo they do not.
        seg_map.into_iter().collect()
    }

    #[cfg(not(target_os = "linux"))]
    fn resident_segments() -> Vec<(String, usize)> {
        vec![]
    }
}
