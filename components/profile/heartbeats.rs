/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */


use heartbeats_simple::HeartbeatPow as Heartbeat;
use heartbeats_simple::HeartbeatPowContext as HeartbeatContext;
use profile_traits::time::ProfilerCategory;
use std::collections::HashMap;
use std::env::var_os;
use std::error::Error;
use std::fs::File;
use std::mem;
use std::path::Path;
use util::opts;


static mut HBS: Option<*mut HashMap<ProfilerCategory, Heartbeat>> = None;

/// Initialize heartbeats
pub fn init() {
    let mut hbs: HashMap<ProfilerCategory, Heartbeat> = HashMap::new();
    maybe_create_heartbeat(&mut hbs, ProfilerCategory::Compositing);
    maybe_create_heartbeat(&mut hbs, ProfilerCategory::LayoutPerform);
    maybe_create_heartbeat(&mut hbs, ProfilerCategory::LayoutStyleRecalc);
    maybe_create_heartbeat(&mut hbs, ProfilerCategory::LayoutTextShaping);
    maybe_create_heartbeat(&mut hbs, ProfilerCategory::LayoutRestyleDamagePropagation);
    maybe_create_heartbeat(&mut hbs, ProfilerCategory::LayoutNonIncrementalReset);
    maybe_create_heartbeat(&mut hbs, ProfilerCategory::LayoutSelectorMatch);
    maybe_create_heartbeat(&mut hbs, ProfilerCategory::LayoutTreeBuilder);
    maybe_create_heartbeat(&mut hbs, ProfilerCategory::LayoutDamagePropagate);
    maybe_create_heartbeat(&mut hbs, ProfilerCategory::LayoutGeneratedContent);
    maybe_create_heartbeat(&mut hbs, ProfilerCategory::LayoutDisplayListSorting);
    maybe_create_heartbeat(&mut hbs, ProfilerCategory::LayoutFloatPlacementSpeculation);
    maybe_create_heartbeat(&mut hbs, ProfilerCategory::LayoutMain);
    maybe_create_heartbeat(&mut hbs, ProfilerCategory::LayoutStoreOverflow);
    maybe_create_heartbeat(&mut hbs, ProfilerCategory::LayoutParallelWarmup);
    maybe_create_heartbeat(&mut hbs, ProfilerCategory::LayoutDispListBuild);
    maybe_create_heartbeat(&mut hbs, ProfilerCategory::NetHTTPRequestResponse);
    maybe_create_heartbeat(&mut hbs, ProfilerCategory::PaintingPerTile);
    maybe_create_heartbeat(&mut hbs, ProfilerCategory::PaintingPrepBuff);
    maybe_create_heartbeat(&mut hbs, ProfilerCategory::Painting);
    maybe_create_heartbeat(&mut hbs, ProfilerCategory::ImageDecoding);
    maybe_create_heartbeat(&mut hbs, ProfilerCategory::ImageSaving);
    maybe_create_heartbeat(&mut hbs, ProfilerCategory::ScriptAttachLayout);
    maybe_create_heartbeat(&mut hbs, ProfilerCategory::ScriptConstellationMsg);
    maybe_create_heartbeat(&mut hbs, ProfilerCategory::ScriptDevtoolsMsg);
    maybe_create_heartbeat(&mut hbs, ProfilerCategory::ScriptDocumentEvent);
    maybe_create_heartbeat(&mut hbs, ProfilerCategory::ScriptDomEvent);
    maybe_create_heartbeat(&mut hbs, ProfilerCategory::ScriptEvaluate);
    maybe_create_heartbeat(&mut hbs, ProfilerCategory::ScriptEvent);
    maybe_create_heartbeat(&mut hbs, ProfilerCategory::ScriptFileRead);
    maybe_create_heartbeat(&mut hbs, ProfilerCategory::ScriptImageCacheMsg);
    maybe_create_heartbeat(&mut hbs, ProfilerCategory::ScriptInputEvent);
    maybe_create_heartbeat(&mut hbs, ProfilerCategory::ScriptNetworkEvent);
    maybe_create_heartbeat(&mut hbs, ProfilerCategory::ScriptParseHTML);
    maybe_create_heartbeat(&mut hbs, ProfilerCategory::ScriptPlannedNavigation);
    maybe_create_heartbeat(&mut hbs, ProfilerCategory::ScriptResize);
    maybe_create_heartbeat(&mut hbs, ProfilerCategory::ScriptSetScrollState);
    maybe_create_heartbeat(&mut hbs, ProfilerCategory::ScriptSetViewport);
    maybe_create_heartbeat(&mut hbs, ProfilerCategory::ScriptTimerEvent);
    maybe_create_heartbeat(&mut hbs, ProfilerCategory::ScriptStylesheetLoad);
    maybe_create_heartbeat(&mut hbs, ProfilerCategory::ScriptUpdateReplacedElement);
    maybe_create_heartbeat(&mut hbs, ProfilerCategory::ScriptWebSocketEvent);
    maybe_create_heartbeat(&mut hbs, ProfilerCategory::ScriptWorkerEvent);
    maybe_create_heartbeat(&mut hbs, ProfilerCategory::ScriptServiceWorkerEvent);
    maybe_create_heartbeat(&mut hbs, ProfilerCategory::ScriptParseXML);
    maybe_create_heartbeat(&mut hbs, ProfilerCategory::ApplicationHeartbeat);
    unsafe {
        HBS = Some(mem::transmute(Box::new(hbs)));
    }
}

/// Log regmaining buffer data and cleanup heartbeats
pub fn cleanup() {
    unsafe {
        if let Some(hbs) = HBS {
            let mut h: Box<HashMap<ProfilerCategory, Heartbeat>> = mem::transmute(hbs);
            for (_, mut v) in h.iter_mut() {
                // log any remaining heartbeat records before dropping
                log_heartbeat_records(v);
            }
            h.clear();
        }
        HBS = None;
    }
}

/// Check if a heartbeat exists for the given category
pub fn is_heartbeat_enabled(category: &ProfilerCategory) -> bool {
    unsafe {
        HBS.map_or(false, |m| (*m).contains_key(category))
    }
}

/// Issue a heartbeat (if one exists) for the given category
pub fn maybe_heartbeat(category: &ProfilerCategory,
                       start_time: u64,
                       end_time: u64,
                       start_energy: u64,
                       end_energy: u64) {
    unsafe {
        if let Some(map) = HBS {
            if let Some(mut h) = (*map).get_mut(category) {
                (*h).heartbeat(0, 1, start_time, end_time, start_energy, end_energy);
            }
        }
    }
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

/// Callback function used to log the window buffer.
/// When this is called from native C, the heartbeat is safely locked
extern fn heartbeat_window_callback(hb: *const HeartbeatContext) {
    unsafe {
        if let Some(map) = HBS {
            for (_, v) in (*map).iter_mut() {
                if &v.hb as *const HeartbeatContext == hb {
                    log_heartbeat_records(v);
                }
            }
        }
    }
}
