/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#[cfg(target_os = "macos")]
use std::ptr;

#[cfg(all(target_os = "linux", target_env = "gnu"))]
use libc::c_int;
use profile_traits::mem::{ProcessReports, Report, ReportKind, ReporterRequest};
use profile_traits::path;

const SYSTEM_HEAP_ALLOCATED_STR: &str = "system-heap-allocated";
const SYSTEM_HEAP_RESERVED_STR: &str = "system-heap-reserved";

struct SystemHeapInfo {
    allocated: Option<usize>,
    reserved: Option<usize>,
}

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
        report(path!["pss"], proportional_set_size());

        // Memory segments, as reported by the OS.
        // Notice that the sum of this should be more accurate according to
        // the manpage of /proc/pid/statm
        for seg in resident_segments() {
            report(path!["resident-according-to-smaps", seg.0], Some(seg.1));
        }

        // Total number of bytes allocated by the application on the system
        // heap. Even if we use a custom global allocator, this doesn't affect
        // everything, e.g. C/C++ libraries might still use the default system allocator
        // unless we explicitly patch  / configure them.
        // Hence we always check system-heap info, since it allows us to know
        // how much memory bypasses the global allocator we defined.
        let system_heap = system_heap_info();
        report(path![SYSTEM_HEAP_ALLOCATED_STR], system_heap.allocated);
        report(path![SYSTEM_HEAP_RESERVED_STR], system_heap.reserved);

        for heap_report in servo_allocator::heap_reports() {
            report(path![heap_report.path], heap_report.size);
        }
    }

    request.reports_channel.send(ProcessReports::new(reports));
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
fn system_heap_info() -> SystemHeapInfo {
    let info: struct_mallinfo = unsafe { mallinfo() };

    // https://man7.org/linux/man-pages/man3/mallinfo.3.html
    // TODO: Switch to mallinfo2 or malloc_info.
    // The documentation in the glibc man page makes it sound like |uordblks| would suffice,
    // but that only gets the small allocations that are put in the brk heap. We need |hblkhd|
    // as well to get the larger allocations that are mmapped.
    //
    // These fields are unfortunately |int| and so can overflow (becoming negative) if memory
    // usage gets high enough. So don't report anything in that case. In the non-overflow case
    // we cast the two values to usize before adding them to make sure the sum also doesn't
    // overflow.
    let allocated = if info.hblkhd >= 0 && info.uordblks >= 0 {
        Some(info.hblkhd as usize + info.uordblks as usize)
    } else {
        None
    };

    let reserved = if info.arena >= 0 && info.hblkhd >= 0 {
        Some(info.arena as usize + info.hblkhd as usize)
    } else {
        None
    };

    SystemHeapInfo {
        allocated,
        reserved,
    }
}

#[cfg(target_os = "macos")]
fn macos_malloc_statistics() -> libc::malloc_statistics_t {
    let mut stats = libc::malloc_statistics_t {
        blocks_in_use: 0,
        size_in_use: 0,
        max_size_in_use: 0,
        size_allocated: 0,
    };
    unsafe {
        // A null zone aggregates statistics across all malloc zones.
        libc::malloc_zone_statistics(ptr::null_mut(), &mut stats);
    }
    stats
}

#[cfg(target_os = "macos")]
fn system_heap_info() -> SystemHeapInfo {
    let stats = macos_malloc_statistics();
    SystemHeapInfo {
        allocated: Some(stats.size_in_use),
        reserved: Some(stats.size_allocated),
    }
}

#[cfg(not(any(all(target_os = "linux", target_env = "gnu"), target_os = "macos")))]
fn system_heap_info() -> SystemHeapInfo {
    SystemHeapInfo {
        allocated: None,
        reserved: None,
    }
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
#[cfg(target_os = "linux")]
fn proportional_set_size() -> Option<usize> {
    use std::fs::File;
    use std::io::Read;
    let mut file = File::open("/proc/self/smaps_rollup").ok()?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).ok()?;
    let pss_line = contents
        .split("\n")
        .find(|string| string.contains("Pss:"))?;

    // String looks like: "Pss:                 227 kB"
    let pss_str = pss_line.split_whitespace().nth(1)?;
    pss_str.parse().ok()
}

#[cfg(not(target_os = "linux"))]
fn proportional_set_size() -> Option<usize> {
    None
}

#[cfg(target_os = "macos")]
fn task_basic_info() -> Option<mach2::task_info::task_basic_info> {
    use mach2::kern_return::KERN_SUCCESS;
    use mach2::task::task_info;
    use mach2::task_info::{TASK_BASIC_INFO, TASK_BASIC_INFO_COUNT, task_basic_info};
    use mach2::traps::mach_task_self;

    let mut info = task_basic_info::default();
    let mut count = TASK_BASIC_INFO_COUNT;
    if unsafe {
        task_info(
            mach_task_self(),
            TASK_BASIC_INFO,
            std::ptr::from_mut(&mut info).cast(),
            std::ptr::from_mut(&mut count),
        )
    } != KERN_SUCCESS
    {
        return None;
    }
    Some(info)
}

#[cfg(target_os = "macos")]
fn vsize() -> Option<usize> {
    task_basic_info().map(|task_basic_info| task_basic_info.virtual_size)
}

#[cfg(target_os = "macos")]
fn resident() -> Option<usize> {
    task_basic_info().map(|task_basic_info| task_basic_info.resident_size)
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
    // See https://www.kernel.org/doc/Documentation/filesystems/proc.txt

    let f = match File::open("/proc/self/smaps") {
        Ok(f) => BufReader::new(f),
        Err(_) => return vec![],
    };

    let seg_re = Regex::new(
        r"^[[:xdigit:]]+-[[:xdigit:]]+ (....) [[:xdigit:]]+ [[:xdigit:]]+:[[:xdigit:]]+ \d+ +(.*)",
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
