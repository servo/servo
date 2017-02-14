/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use heartbeats_simple::HeartbeatPow as Heartbeat;
use profile_traits::time::ProfilerCategory;
use self::synchronized_heartbeat::{heartbeat_window_callback, lock_and_work};
use servo_config::opts;
use std::collections::HashMap;
use std::env::var_os;
use std::error::Error;
use std::fs::File;
use std::path::Path;

/// Initialize heartbeats
pub fn init() {
    lock_and_work(|hbs_opt|
        if hbs_opt.is_none() {
            let mut hbs: Box<HashMap<ProfilerCategory, Heartbeat>> = Box::new(HashMap::new());
            maybe_create_heartbeat(&mut hbs, ProfilerCategory::ApplicationHeartbeat);
            *hbs_opt = Some(Box::into_raw(hbs))
        }
    );
}

/// Log regmaining buffer data and cleanup heartbeats
pub fn cleanup() {
    let hbs_opt_box: Option<Box<HashMap<ProfilerCategory, Heartbeat>>> = lock_and_work(|hbs_opt|
        hbs_opt.take().map(|hbs_ptr|
            unsafe {
                Box::from_raw(hbs_ptr)
            }
        )
    );
    if let Some(mut hbs) = hbs_opt_box {
        for (_, mut v) in hbs.iter_mut() {
            // log any remaining heartbeat records before dropping
            log_heartbeat_records(v);
        }
        hbs.clear();
    }
}

/// Check if a heartbeat exists for the given category
pub fn is_heartbeat_enabled(category: &ProfilerCategory) -> bool {
    let is_enabled = lock_and_work(|hbs_opt|
        hbs_opt.map_or(false, |hbs_ptr|
            unsafe {
                (*hbs_ptr).contains_key(category)
            }
        )
    );
    is_enabled || is_create_heartbeat(category)
}

/// Issue a heartbeat (if one exists) for the given category
pub fn maybe_heartbeat(category: &ProfilerCategory,
                       start_time: u64,
                       end_time: u64,
                       start_energy: u64,
                       end_energy: u64) {
    lock_and_work(|hbs_opt|
        if let Some(hbs_ptr) = *hbs_opt {
            unsafe {
                if !(*hbs_ptr).contains_key(category) {
                    maybe_create_heartbeat(&mut (*hbs_ptr), category.clone());
                }
                if let Some(mut h) = (*hbs_ptr).get_mut(category) {
                    (*h).heartbeat(0, 1, start_time, end_time, start_energy, end_energy);
                }
            }
        }
    );
}

// TODO(cimes): Android doesn't really do environment variables. Need a better way to configure dynamically.

fn is_create_heartbeat(category: &ProfilerCategory) -> bool {
    opts::get().profile_heartbeats || var_os(format!("SERVO_HEARTBEAT_ENABLE_{:?}", category)).is_some()
}

fn open_heartbeat_log<P: AsRef<Path>>(name: P) -> Option<File> {
    match File::create(name) {
        Ok(f) => Some(f),
        Err(e) => {
            warn!("Failed to open heartbeat log: {}", Error::description(&e));
            None
        },
    }
}

#[cfg(target_os = "android")]
fn get_heartbeat_log(category: &ProfilerCategory) -> Option<File> {
    open_heartbeat_log(format!("/sdcard/servo/heartbeat-{:?}.log", category))
}

#[cfg(not(target_os = "android"))]
fn get_heartbeat_log(category: &ProfilerCategory) -> Option<File> {
    var_os(format!("SERVO_HEARTBEAT_LOG_{:?}", category)).and_then(|name| open_heartbeat_log(&name))
}

fn get_heartbeat_window_size(category: &ProfilerCategory) -> usize {
    const WINDOW_SIZE_DEFAULT: usize = 1;
    match var_os(format!("SERVO_HEARTBEAT_WINDOW_{:?}", category)) {
        Some(w) => match w.into_string() {
            Ok(s) => s.parse::<usize>().unwrap_or(WINDOW_SIZE_DEFAULT),
            _ => WINDOW_SIZE_DEFAULT,
        },
        None => WINDOW_SIZE_DEFAULT,
    }
}

/// Possibly create a heartbeat
fn maybe_create_heartbeat(hbs: &mut HashMap<ProfilerCategory, Heartbeat>,
                          category: ProfilerCategory) {
    if is_create_heartbeat(&category) {
        // get optional log file
        let logfile: Option<File> = get_heartbeat_log(&category);
        // window size
        let window_size: usize = get_heartbeat_window_size(&category);
        // create the heartbeat
        match Heartbeat::new(window_size, Some(heartbeat_window_callback), logfile) {
            Ok(hb) => {
                debug!("Created heartbeat for {:?}", category);
                hbs.insert(category, hb);
            },
            Err(e) => warn!("Failed to create heartbeat for {:?}: {}", category, e),
        }
    };
}

/// Log heartbeat records up to the buffer index
fn log_heartbeat_records(hb: &mut Heartbeat) {
    match hb.log_to_buffer_index() {
        Ok(_) => (),
        Err(e) => warn!("Failed to write heartbeat log: {}", Error::description(&e)),
    }
}

mod synchronized_heartbeat {
    use heartbeats_simple::HeartbeatPow as Heartbeat;
    use heartbeats_simple::HeartbeatPowContext as HeartbeatContext;
    use profile_traits::time::ProfilerCategory;
    use std::collections::HashMap;
    use std::sync::atomic::{ATOMIC_BOOL_INIT, AtomicBool, Ordering};
    use super::log_heartbeat_records;

    static mut HBS: Option<*mut HashMap<ProfilerCategory, Heartbeat>> = None;

    // unfortunately can't encompass the actual hashmap in a Mutex (Heartbeat isn't Send/Sync), so we'll use a spinlock
    static HBS_SPINLOCK: AtomicBool = ATOMIC_BOOL_INIT;

    pub fn lock_and_work<F, R>(work: F) -> R
        where F: FnOnce(&mut Option<*mut HashMap<ProfilerCategory, Heartbeat>>) -> R {
        while HBS_SPINLOCK.compare_and_swap(false, true, Ordering::SeqCst) {}
        let result = unsafe {
            work(&mut HBS)
        };
        HBS_SPINLOCK.store(false, Ordering::SeqCst);
        result
    }

    /// Callback function used to log the window buffer.
    /// When this is called from native C, the heartbeat is safely locked internally and the global lock is held.
    /// If calling from this file, you must already hold the global lock!
    pub extern fn heartbeat_window_callback(hb: *const HeartbeatContext) {
        unsafe {
            if let Some(hbs_ptr) = HBS {
                for (_, v) in (*hbs_ptr).iter_mut() {
                    if &v.hb as *const HeartbeatContext == hb {
                        log_heartbeat_records(v);
                    }
                }
            }
        }
    }
}
