/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Memory profiling functions.

use self::system_reporter::SystemMemoryReporter;
use std::borrow::ToOwned;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::old_io::timer::sleep;
use std::sync::mpsc::{Sender, channel, Receiver};
use std::time::duration::Duration;
use util::task::spawn_named;

#[derive(Clone)]
pub struct MemoryProfilerChan(pub Sender<MemoryProfilerMsg>);

impl MemoryProfilerChan {
    pub fn send(&self, msg: MemoryProfilerMsg) {
        let MemoryProfilerChan(ref c) = *self;
        c.send(msg).unwrap();
    }
}

/// An easy way to build a path for a report.
#[macro_export]
macro_rules! path {
    ($($x:expr),*) => {{
        use std::borrow::ToOwned;
        vec![$( $x.to_owned() ),*]
    }}
}

pub struct MemoryReport {
    /// The identifying path for this report.
    pub path: Vec<String>,

    /// The size, in bytes.
    pub size: u64,
}

/// A channel through which memory reports can be sent.
#[derive(Clone)]
pub struct MemoryReportsChan(pub Sender<Vec<MemoryReport>>);

impl MemoryReportsChan {
    pub fn send(&self, report: Vec<MemoryReport>) {
        let MemoryReportsChan(ref c) = *self;
        c.send(report).unwrap();
    }
}

/// A memory reporter is capable of measuring some data structure of interest. Because it needs
/// to be passed to and registered with the MemoryProfiler, it's typically a "small" (i.e. easily
/// cloneable) value that provides access to a "large" data structure, e.g. a channel that can
/// inject a request for measurements into the event queue associated with the "large" data
/// structure.
pub trait MemoryReporter {
    /// Collect one or more memory reports. Returns true on success, and false on failure.
    fn collect_reports(&self, reports_chan: MemoryReportsChan) -> bool;
}

/// Messages that can be sent to the memory profiler thread.
pub enum MemoryProfilerMsg {
    /// Register a MemoryReporter with the memory profiler. The String is only used to identify the
    /// reporter so it can be unregistered later. The String must be distinct from that used by any
    /// other registered reporter otherwise a panic will occur.
    RegisterMemoryReporter(String, Box<MemoryReporter + Send>),

    /// Unregister a MemoryReporter with the memory profiler. The String must match the name given
    /// when the reporter was registered. If the String does not match the name of a registered
    /// reporter a panic will occur.
    UnregisterMemoryReporter(String),

    /// Triggers printing of the memory profiling metrics.
    Print,

    /// Tells the memory profiler to shut down.
    Exit,
}

pub struct MemoryProfiler {
    /// The port through which messages are received.
    pub port: Receiver<MemoryProfilerMsg>,

    /// Registered memory reporters.
    reporters: HashMap<String, Box<MemoryReporter + Send>>,
}

impl MemoryProfiler {
    pub fn create(period: Option<f64>) -> MemoryProfilerChan {
        let (chan, port) = channel();

        // Create the timer thread if a period was provided.
        if let Some(period) = period {
            let period_ms = Duration::milliseconds((period * 1000f64) as i64);
            let chan = chan.clone();
            spawn_named("Memory profiler timer".to_owned(), move || {
                loop {
                    sleep(period_ms);
                    if chan.send(MemoryProfilerMsg::Print).is_err() {
                        break;
                    }
                }
            });
        }

        // Always spawn the memory profiler. If there is no timer thread it won't receive regular
        // `Print` events, but it will still receive the other events.
        spawn_named("Memory profiler".to_owned(), move || {
            let mut memory_profiler = MemoryProfiler::new(port);
            memory_profiler.start();
        });

        let memory_profiler_chan = MemoryProfilerChan(chan);

        // Register the system memory reporter, which will run on the memory profiler's own thread.
        // It never needs to be unregistered, because as long as the memory profiler is running the
        // system memory reporter can make measurements.
        let system_reporter = Box::new(SystemMemoryReporter);
        memory_profiler_chan.send(MemoryProfilerMsg::RegisterMemoryReporter("system".to_owned(),
                                                                            system_reporter));

        memory_profiler_chan
    }

    pub fn new(port: Receiver<MemoryProfilerMsg>) -> MemoryProfiler {
        MemoryProfiler {
            port: port,
            reporters: HashMap::new(),
        }
    }

    pub fn start(&mut self) {
        loop {
            match self.port.recv() {
               Ok(msg) => {
                   if !self.handle_msg(msg) {
                       break
                   }
               }
               _ => break
            }
        }
    }

    fn handle_msg(&mut self, msg: MemoryProfilerMsg) -> bool {
        match msg {
            MemoryProfilerMsg::RegisterMemoryReporter(name, reporter) => {
                // Panic if it has already been registered.
                let name_clone = name.clone();
                match self.reporters.insert(name, reporter) {
                    None => true,
                    Some(_) =>
                        panic!(format!("RegisterMemoryReporter: '{}' name is already in use",
                                       name_clone)),
                }
            },

            MemoryProfilerMsg::UnregisterMemoryReporter(name) => {
                // Panic if it hasn't previously been registered.
                match self.reporters.remove(&name) {
                    Some(_) => true,
                    None =>
                        panic!(format!("UnregisterMemoryReporter: '{}' name is unknown", &name)),
                }
            },

            MemoryProfilerMsg::Print => {
                self.handle_print_msg();
                true
            },

            MemoryProfilerMsg::Exit => false
        }
    }

    fn handle_print_msg(&self) {
        println!("Begin memory reports");
        println!("|");

        // Collect reports from memory reporters.
        //
        // This serializes the report-gathering. It might be worth creating a new scoped thread for
        // each reporter once we have enough of them.
        //
        // If anything goes wrong with a reporter, we just skip it.
        let mut forest = ReportsForest::new();
        for reporter in self.reporters.values() {
            let (chan, port) = channel();
            if reporter.collect_reports(MemoryReportsChan(chan)) {
                if let Ok(reports) = port.recv() {
                    for report in reports.iter() {
                        forest.insert(&report.path, report.size);
                    }
                }
            }
        }
        forest.print();

        println!("|");
        println!("End memory reports");
        println!("");
    }
}

/// A collection of one or more reports with the same initial path segment. A ReportsTree
/// containing a single node is described as "degenerate".
struct ReportsTree {
    /// For leaf nodes, this is the sum of the sizes of all reports that mapped to this location.
    /// For interior nodes, this is the sum of the sizes of all its child nodes.
    size: u64,

    /// For leaf nodes, this is the count of all reports that mapped to this location.
    /// For interor nodes, this is always zero.
    count: u32,

    /// The segment from the report path that maps to this node.
    path_seg: String,

    /// Child nodes.
    children: Vec<ReportsTree>,
}

impl ReportsTree {
    fn new(path_seg: String) -> ReportsTree {
        ReportsTree {
            size: 0,
            count: 0,
            path_seg: path_seg,
            children: vec![]
        }
    }

    // Searches the tree's children for a path_seg match, and returns the index if there is a
    // match.
    fn find_child(&self, path_seg: &String) -> Option<usize> {
        for (i, child) in self.children.iter().enumerate() {
            if child.path_seg == *path_seg {
                return Some(i);
            }
        }
        None
    }

    // Insert the path and size into the tree, adding any nodes as necessary.
    fn insert(&mut self, path: &[String], size: u64) {
        let mut t: &mut ReportsTree = self;
        for path_seg in path.iter() {
            let i = match t.find_child(&path_seg) {
                Some(i) => i,
                None => {
                    let new_t = ReportsTree::new(path_seg.clone());
                    t.children.push(new_t);
                    t.children.len() - 1
                },
            };
            let tmp = t;    // this temporary is needed to satisfy the borrow checker
            t = &mut tmp.children[i];
        }

        t.size += size;
        t.count += 1;
    }

    // Fill in sizes for interior nodes. Should only be done once all the reports have been
    // inserted.
    fn compute_interior_node_sizes(&mut self) -> u64 {
        if !self.children.is_empty() {
            // Interior node. Derive its size from its children.
            if self.size != 0 {
                // This will occur if e.g. we have paths ["a", "b"] and ["a", "b", "c"].
                panic!("one report's path is a sub-path of another report's path");
            }
            for child in self.children.iter_mut() {
                self.size += child.compute_interior_node_sizes();
            }
        }
        self.size
    }

    fn print(&self, depth: i32) {
        if !self.children.is_empty() {
            assert_eq!(self.count, 0);
        }

        let mut indent_str = String::new();
        for _ in range(0, depth) {
            indent_str.push_str("   ");
        }

        let mebi = 1024f64 * 1024f64;
        let count_str = if self.count > 1 { format!(" {}", self.count) } else { "".to_owned() };
        println!("|{}{:8.2} MiB -- {}{}",
                 indent_str, (self.size as f64) / mebi, self.path_seg, count_str);

        for child in self.children.iter() {
            child.print(depth + 1);
        }
    }
}

/// A collection of ReportsTrees. It represents the data from multiple memory reports in a form
/// that's good to print.
struct ReportsForest {
    trees: HashMap<String, ReportsTree>,
}

impl ReportsForest {
    fn new() -> ReportsForest {
        ReportsForest {
            trees: HashMap::new(),
        }
    }

    // Insert the path and size into the forest, adding any trees and nodes as necessary.
    fn insert(&mut self, path: &[String], size: u64) {
        // Get the right tree, creating it if necessary.
        if !self.trees.contains_key(&path[0]) {
            self.trees.insert(path[0].clone(), ReportsTree::new(path[0].clone()));
        }
        let t = self.trees.get_mut(&path[0]).unwrap();

        // Use tail() because the 0th path segment was used to find the right tree in the forest.
        t.insert(path.tail(), size);
    }

    fn print(&mut self) {
        // Fill in sizes of interior nodes.
        for (_, tree) in self.trees.iter_mut() {
            tree.compute_interior_node_sizes();
        }

        // Put the trees into a sorted vector. Primary sort: degenerate trees (those containing a
        // single node) come after non-degenerate trees. Secondary sort: alphabetical order of the
        // root node's path_seg.
        let mut v = vec![];
        for (_, tree) in self.trees.iter() {
            v.push(tree);
        }
        v.sort_by(|a, b| {
            if a.children.is_empty() && !b.children.is_empty() {
                Ordering::Greater
            } else if !a.children.is_empty() && b.children.is_empty() {
                Ordering::Less
            } else {
                a.path_seg.cmp(&b.path_seg)
            }
        });

        // Print the forest.
        for tree in v.iter() {
            tree.print(0);
            // Print a blank line after non-degenerate trees.
            if !tree.children.is_empty() {
                println!("|");
            }
        }
    }
}

//---------------------------------------------------------------------------

mod system_reporter {
    use libc::{c_char, c_int, c_void, size_t};
    use std::borrow::ToOwned;
    use std::ffi::CString;
    use std::mem::size_of;
    use std::ptr::null_mut;
    use super::{MemoryReport, MemoryReporter, MemoryReportsChan};
    #[cfg(target_os="macos")]
    use task_info::task_basic_info::{virtual_size, resident_size};

    /// Collects global measurements from the OS and heap allocators.
    pub struct SystemMemoryReporter;

    impl MemoryReporter for SystemMemoryReporter {
        fn collect_reports(&self, reports_chan: MemoryReportsChan) -> bool {
            let mut reports = vec![];
            {
                let mut report = |path, size| {
                    if let Some(size) = size {
                        reports.push(MemoryReport { path: path, size: size });
                    }
                };

                // Virtual and physical memory usage, as reported by the OS.
                report(path!["vsize"], get_vsize());
                report(path!["resident"], get_resident());

                // Memory segments, as reported by the OS.
                for seg in get_resident_segments().iter() {
                    report(path!["resident-according-to-smaps", seg.0], Some(seg.1));
                }

                // Total number of bytes allocated by the application on the system
                // heap.
                report(path!["system-heap-allocated"], get_system_heap_allocated());

                // The descriptions of the following jemalloc measurements are taken
                // directly from the jemalloc documentation.

                // "Total number of bytes allocated by the application."
                report(path!["jemalloc-heap-allocated"], get_jemalloc_stat("stats.allocated"));

                // "Total number of bytes in active pages allocated by the application.
                // This is a multiple of the page size, and greater than or equal to
                // |stats.allocated|."
                report(path!["jemalloc-heap-active"], get_jemalloc_stat("stats.active"));

                // "Total number of bytes in chunks mapped on behalf of the application.
                // This is a multiple of the chunk size, and is at least as large as
                // |stats.active|. This does not include inactive chunks."
                report(path!["jemalloc-heap-mapped"], get_jemalloc_stat("stats.mapped"));
            }
            reports_chan.send(reports);

            true
        }
    }


    #[cfg(target_os="linux")]
    extern {
        fn mallinfo() -> struct_mallinfo;
    }

    #[cfg(target_os="linux")]
    #[repr(C)]
    pub struct struct_mallinfo {
        arena:    c_int,
        ordblks:  c_int,
        smblks:   c_int,
        hblks:    c_int,
        hblkhd:   c_int,
        usmblks:  c_int,
        fsmblks:  c_int,
        uordblks: c_int,
        fordblks: c_int,
        keepcost: c_int,
    }

    #[cfg(target_os="linux")]
    fn get_system_heap_allocated() -> Option<u64> {
        let mut info: struct_mallinfo;
        unsafe {
            info = mallinfo();
        }
        // The documentation in the glibc man page makes it sound like |uordblks|
        // would suffice, but that only gets the small allocations that are put in
        // the brk heap. We need |hblkhd| as well to get the larger allocations
        // that are mmapped.
        Some((info.hblkhd + info.uordblks) as u64)
    }

    #[cfg(not(target_os="linux"))]
    fn get_system_heap_allocated() -> Option<u64> {
        None
    }

    extern {
        fn je_mallctl(name: *const c_char, oldp: *mut c_void, oldlenp: *mut size_t,
                      newp: *mut c_void, newlen: size_t) -> c_int;
    }

    fn get_jemalloc_stat(value_name: &str) -> Option<u64> {
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
            je_mallctl(epoch_c_name.as_ptr(), epoch_ptr, &mut epoch_len, epoch_ptr,
                       epoch_len)
        };
        if rv != 0 {
            return None;
        }

        let rv = unsafe {
            je_mallctl(value_c_name.as_ptr(), value_ptr, &mut value_len, null_mut(), 0)
        };
        if rv != 0 {
            return None;
        }

        Some(value as u64)
    }

    // Like std::macros::try!, but for Option<>.
    macro_rules! option_try(
        ($e:expr) => (match $e { Some(e) => e, None => return None })
    );

    #[cfg(target_os="linux")]
    fn get_proc_self_statm_field(field: usize) -> Option<u64> {
        use std::fs::File;
        use std::io::Read;

        let mut f = option_try!(File::open("/proc/self/statm").ok());
        let mut contents = String::new();
        option_try!(f.read_to_string(&mut contents).ok());
        let s = option_try!(contents.words().nth(field));
        let npages = option_try!(s.parse::<u64>().ok());
        Some(npages * (::std::env::page_size() as u64))
    }

    #[cfg(target_os="linux")]
    fn get_vsize() -> Option<u64> {
        get_proc_self_statm_field(0)
    }

    #[cfg(target_os="linux")]
    fn get_resident() -> Option<u64> {
        get_proc_self_statm_field(1)
    }

    #[cfg(target_os="macos")]
    fn get_vsize() -> Option<u64> {
        virtual_size()
    }

    #[cfg(target_os="macos")]
    fn get_resident() -> Option<u64> {
        resident_size()
    }

    #[cfg(not(any(target_os="linux", target_os = "macos")))]
    fn get_vsize() -> Option<u64> {
        None
    }

    #[cfg(not(any(target_os="linux", target_os = "macos")))]
    fn get_resident() -> Option<u64> {
        None
    }

    #[cfg(target_os="linux")]
    fn get_resident_segments() -> Vec<(String, u64)> {
        use regex::Regex;
        use std::collections::HashMap;
        use std::collections::hash_map::Entry;
        use std::fs::File;
        use std::io::{BufReader, BufReadExt};

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
            r"^[:xdigit:]+-[:xdigit:]+ (....) [:xdigit:]+ [:xdigit:]+:[:xdigit:]+ \d+ +(.*)").unwrap();
        let rss_re = Regex::new(r"^Rss: +(\d+) kB").unwrap();

        // We record each segment's resident size.
        let mut seg_map: HashMap<String, u64> = HashMap::new();

        #[derive(PartialEq)]
        enum LookingFor { Segment, Rss }
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
                let cap = match seg_re.captures(line.as_slice()) {
                    Some(cap) => cap,
                    None => continue,
                };
                let perms = cap.at(1).unwrap();
                let pathname = cap.at(2).unwrap();

                // Construct the segment name from its pathname and permissions.
                curr_seg_name.clear();
                if pathname == "" || pathname.starts_with("[stack:") {
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
                curr_seg_name.push_str(")");

                looking_for = LookingFor::Rss;
            } else {
                // Look for an "Rss:" line.
                let cap = match rss_re.captures(line.as_slice()) {
                    Some(cap) => cap,
                    None => continue,
                };
                let rss = cap.at(1).unwrap().parse::<u64>().unwrap() * 1024;

                if rss > 0 {
                    // Aggregate small segments into "other".
                    let seg_name = if rss < 512 * 1024 {
                        "other".to_owned()
                    } else {
                        curr_seg_name.clone()
                    };
                    match seg_map.entry(seg_name) {
                        Entry::Vacant(entry) => { entry.insert(rss); },
                        Entry::Occupied(mut entry) => *entry.get_mut() += rss,
                    }
                }

                looking_for = LookingFor::Segment;
            }
        }

        let mut segs: Vec<(String, u64)> = seg_map.into_iter().collect();

        // Note that the sum of all these segments' RSS values differs from the "resident" measurement
        // obtained via /proc/<pid>/statm in get_resident(). It's unclear why this difference occurs;
        // for some processes the measurements match, but for Servo they do not.
        segs.sort_by(|&(_, rss1), &(_, rss2)| rss2.cmp(&rss1));

        segs
    }

    #[cfg(not(target_os="linux"))]
    fn get_resident_segments() -> Vec<(String, u64)> {
        vec![]
    }
}
