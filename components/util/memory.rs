/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Memory profiling functions.

use libc::{c_char,c_int,c_void,size_t};
use std::borrow::ToOwned;
use std::ffi::CString;
use std::old_io::timer::sleep;
#[cfg(target_os="linux")]
use std::old_io::File;
use std::mem::size_of;
use std::ptr::null_mut;
use std::sync::mpsc::{Sender, channel, Receiver};
use std::time::duration::Duration;
use task::spawn_named;
#[cfg(target_os="macos")]
use task_info::task_basic_info::{virtual_size,resident_size};

pub struct MemoryProfilerChan(pub Sender<MemoryProfilerMsg>);

impl MemoryProfilerChan {
    pub fn send(&self, msg: MemoryProfilerMsg) {
        let MemoryProfilerChan(ref c) = *self;
        c.send(msg).unwrap();
    }
}

pub enum MemoryProfilerMsg {
    /// Message used to force print the memory profiling metrics.
    Print,
    /// Tells the memory profiler to shut down.
    Exit,
}

pub struct MemoryProfiler {
    pub port: Receiver<MemoryProfilerMsg>,
}

impl MemoryProfiler {
    pub fn create(period: Option<f64>) -> MemoryProfilerChan {
        let (chan, port) = channel();
        match period {
            Some(period) => {
                let period = Duration::milliseconds((period * 1000f64) as i64);
                let chan = chan.clone();
                spawn_named("Memory profiler timer".to_owned(), move || {
                    loop {
                        sleep(period);
                        if chan.send(MemoryProfilerMsg::Print).is_err() {
                            break;
                        }
                    }
                });
                // Spawn the memory profiler.
                spawn_named("Memory profiler".to_owned(), move || {
                    let memory_profiler = MemoryProfiler::new(port);
                    memory_profiler.start();
                });
            }
            None => {
                // No-op to handle messages when the memory profiler is
                // inactive.
                spawn_named("Memory profiler".to_owned(), move || {
                    loop {
                        match port.recv() {
                            Err(_) | Ok(MemoryProfilerMsg::Exit) => break,
                            _ => {}
                        }
                    }
                });
            }
        }

        MemoryProfilerChan(chan)
    }

    pub fn new(port: Receiver<MemoryProfilerMsg>) -> MemoryProfiler {
        MemoryProfiler {
            port: port
        }
    }

    pub fn start(&self) {
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

    fn handle_msg(&self, msg: MemoryProfilerMsg) -> bool {
        match msg {
            MemoryProfilerMsg::Print => {
                self.handle_print_msg();
                true
            },
            MemoryProfilerMsg::Exit => false
        }
    }

    fn print_measurement(path: &str, nbytes: Option<u64>) {
        match nbytes {
            Some(nbytes) => {
                let mebi = 1024f64 * 1024f64;
                println!("{:12.2}: {}", (nbytes as f64) / mebi, path);
            }
            None => {
                println!("{:>12}: {}", "???", path);
            }
        }
    }

    fn handle_print_msg(&self) {
        println!("{:12}: {}", "_size (MiB)_", "_category_");

        // Virtual and physical memory usage, as reported by the OS.
        MemoryProfiler::print_measurement("vsize", get_vsize());
        MemoryProfiler::print_measurement("resident", get_resident());

        for seg in get_resident_segments().iter() {
            MemoryProfiler::print_measurement(seg.0.as_slice(), Some(seg.1));
        }

        // Total number of bytes allocated by the application on the system
        // heap.
        MemoryProfiler::print_measurement("system-heap-allocated",
                                          get_system_heap_allocated());

        // The descriptions of the following jemalloc measurements are taken
        // directly from the jemalloc documentation.

        // "Total number of bytes allocated by the application."
        MemoryProfiler::print_measurement("jemalloc-heap-allocated",
                                          get_jemalloc_stat("stats.allocated"));

        // "Total number of bytes in active pages allocated by the application.
        // This is a multiple of the page size, and greater than or equal to
        // |stats.allocated|."
        MemoryProfiler::print_measurement("jemalloc-heap-active",
                                          get_jemalloc_stat("stats.active"));

        // "Total number of bytes in chunks mapped on behalf of the application.
        // This is a multiple of the chunk size, and is at least as large as
        // |stats.active|. This does not include inactive chunks."
        MemoryProfiler::print_measurement("jemalloc-heap-mapped",
                                          get_jemalloc_stat("stats.mapped"));

        println!("");
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
    let epoch_c_name = CString::from_slice(epoch_name.as_bytes());
    let mut epoch: u64 = 0;
    let epoch_ptr = &mut epoch as *mut _ as *mut c_void;
    let mut epoch_len = size_of::<u64>() as size_t;

    let value_c_name = CString::from_slice(value_name.as_bytes());
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
        je_mallctl(value_c_name.as_ptr(), value_ptr, &mut value_len,
                   null_mut(), 0)
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
fn get_proc_self_statm_field(field: uint) -> Option<u64> {
    let mut f = File::open(&Path::new("/proc/self/statm"));
    match f.read_to_string() {
        Ok(contents) => {
            let s = option_try!(contents.as_slice().words().nth(field));
            let npages = option_try!(s.parse::<u64>().ok());
            Some(npages * (::std::env::page_size() as u64))
        }
        Err(_) => None
    }
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

    let path = Path::new("/proc/self/smaps");
    let mut f = ::std::old_io::BufferedReader::new(File::open(&path));

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
            curr_seg_name.push_str("- ");
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
                // Aggregate small segments into "- other".
                let seg_name = if rss < 512 * 1024 {
                    "- other".to_owned()
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

    // Get the total and add it to the vector. Note that this total differs
    // from the "resident" measurement obtained via /proc/<pid>/statm in
    // get_resident(). It's unclear why this difference occurs; for some
    // processes the measurements match, but for Servo they do not.
    let total = segs.iter().fold(0u64, |total, &(_, size)| total + size);
    segs.push(("resident-according-to-smaps".to_owned(), total));

    // Sort by size; the total will be first.
    segs.sort_by(|&(_, rss1), &(_, rss2)| rss2.cmp(&rss1));

    segs
}

#[cfg(not(target_os="linux"))]
fn get_resident_segments() -> Vec<(String, u64)> {
    vec![]
}

