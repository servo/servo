/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Memory profiling functions.

use std::io::timer::sleep;
#[cfg(target_os="linux")]
use std::io::File;
#[cfg(target_os="linux")]
use std::os::page_size;
use task::spawn_named;
#[cfg(target_os="macos")]
use task_info::task_basic_info::{virtual_size,resident_size};

pub struct MemoryProfilerChan(pub Sender<MemoryProfilerMsg>);

impl MemoryProfilerChan {
    pub fn send(&self, msg: MemoryProfilerMsg) {
        let MemoryProfilerChan(ref c) = *self;
        c.send(msg);
    }
}

pub enum MemoryProfilerMsg {
    /// Message used to force print the memory profiling metrics.
    PrintMsg,
    /// Tells the memory profiler to shut down.
    ExitMsg,
}

pub struct MemoryProfiler {
    pub port: Receiver<MemoryProfilerMsg>,
}

impl MemoryProfiler {
    pub fn create(period: Option<f64>) -> MemoryProfilerChan {
        let (chan, port) = channel();
        match period {
            Some(period) => {
                let period = (period * 1000f64) as u64;
                let chan = chan.clone();
                spawn_named("Memory profiler timer", proc() {
                    loop {
                        sleep(period);
                        if chan.send_opt(PrintMsg).is_err() {
                            break;
                        }
                    }
                });
                // Spawn the memory profiler.
                spawn_named("Memory profiler", proc() {
                    let memory_profiler = MemoryProfiler::new(port);
                    memory_profiler.start();
                });
            }
            None => {
                // No-op to handle messages when the memory profiler is
                // inactive.
                spawn_named("Memory profiler", proc() {
                    loop {
                        match port.recv_opt() {
                            Err(_) | Ok(ExitMsg) => break,
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
            match self.port.recv_opt() {
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
            PrintMsg => {
                self.handle_print_msg();
                true
            },
            ExitMsg => false
        }
    }

    fn print_measurement(path: &str, nbytes: Option<u64>) {
        match nbytes {
            Some(nbytes) => {
                let mebi = 1024f64 * 1024f64;
                println!("{:12s}: {:12.2f}", path, (nbytes as f64) / mebi);
            }
            None => {
                println!("{:12s}: {:>12s}", path, "???");
            }
        }
    }

    fn handle_print_msg(&self) {
        println!("{:12s}: {:12s}", "_category_", "_size (MiB)_");
        MemoryProfiler::print_measurement("vsize",    get_vsize());
        MemoryProfiler::print_measurement("resident", get_resident());
        println!("");
    }
}

// Like std::macros::try!, but for Option<>.
macro_rules! option_try(
    ($e:expr) => (match $e { Some(e) => e, None => return None })
)

#[cfg(target_os="linux")]
fn get_proc_self_statm_field(field: uint) -> Option<u64> {
    let mut f = File::open(&Path::new("/proc/self/statm"));
    match f.read_to_str() {
        Ok(contents) => {
            let s = option_try!(contents.as_slice().words().nth(field));
            let npages: u64 = option_try!(from_str(s));
            Some(npages * (page_size() as u64))
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

#[cfg(not(target_os="linux"), not(target_os = "macos"))]
fn get_vsize() -> Option<u64> {
    None
}

#[cfg(not(target_os="linux"), not(target_os = "macos"))]
fn get_resident() -> Option<u64> {
    None
}

