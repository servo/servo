/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */


use hbs_pow::HeartbeatPow as Heartbeat;
use hbs_pow::HeartbeatPowContext as HeartbeatContext;
use profile_traits::time::ProfilerCategory;
use std::collections::HashMap;
use std::env::var_os;
use std::error::Error;
use std::fs::File;
use std::mem;


static mut HBS: Option<*mut HashMap<ProfilerCategory, Heartbeat>> = None;

/// Initialize heartbeats
pub fn init() {
    let mut hbs: HashMap<ProfilerCategory, Heartbeat> = HashMap::new();
    maybe_create_heartbeat(&mut hbs, ProfilerCategory::Compositing);
    maybe_create_heartbeat(&mut hbs, ProfilerCategory::LayoutPerform);
    maybe_create_heartbeat(&mut hbs, ProfilerCategory::LayoutStyleRecalc);
    maybe_create_heartbeat(&mut hbs, ProfilerCategory::LayoutRestyleDamagePropagation);
    maybe_create_heartbeat(&mut hbs, ProfilerCategory::LayoutNonIncrementalReset);
    maybe_create_heartbeat(&mut hbs, ProfilerCategory::LayoutSelectorMatch);
    maybe_create_heartbeat(&mut hbs, ProfilerCategory::LayoutTreeBuilder);
    maybe_create_heartbeat(&mut hbs, ProfilerCategory::LayoutDamagePropagate);
    maybe_create_heartbeat(&mut hbs, ProfilerCategory::LayoutGeneratedContent);
    maybe_create_heartbeat(&mut hbs, ProfilerCategory::LayoutMain);
    maybe_create_heartbeat(&mut hbs, ProfilerCategory::LayoutParallelWarmup);
    maybe_create_heartbeat(&mut hbs, ProfilerCategory::LayoutShaping);
    maybe_create_heartbeat(&mut hbs, ProfilerCategory::LayoutDispListBuild);
    maybe_create_heartbeat(&mut hbs, ProfilerCategory::PaintingPerTile);
    maybe_create_heartbeat(&mut hbs, ProfilerCategory::PaintingPrepBuff);
    maybe_create_heartbeat(&mut hbs, ProfilerCategory::Painting);
    maybe_create_heartbeat(&mut hbs, ProfilerCategory::ImageDecoding);
    maybe_create_heartbeat(&mut hbs, ProfilerCategory::ScriptAttachLayout);
    maybe_create_heartbeat(&mut hbs, ProfilerCategory::ScriptConstellationMsg);
    maybe_create_heartbeat(&mut hbs, ProfilerCategory::ScriptDevtoolsMsg);
    maybe_create_heartbeat(&mut hbs, ProfilerCategory::ScriptDocumentEvent);
    maybe_create_heartbeat(&mut hbs, ProfilerCategory::ScriptDomEvent);
    maybe_create_heartbeat(&mut hbs, ProfilerCategory::ScriptFileRead);
    maybe_create_heartbeat(&mut hbs, ProfilerCategory::ScriptImageCacheMsg);
    maybe_create_heartbeat(&mut hbs, ProfilerCategory::ScriptInputEvent);
    maybe_create_heartbeat(&mut hbs, ProfilerCategory::ScriptNetworkEvent);
    maybe_create_heartbeat(&mut hbs, ProfilerCategory::ScriptResize);
    maybe_create_heartbeat(&mut hbs, ProfilerCategory::ScriptEvent);
    maybe_create_heartbeat(&mut hbs, ProfilerCategory::ScriptUpdateReplacedElement);
    maybe_create_heartbeat(&mut hbs, ProfilerCategory::ScriptSetViewport);
    maybe_create_heartbeat(&mut hbs, ProfilerCategory::ScriptWebSocketEvent);
    maybe_create_heartbeat(&mut hbs, ProfilerCategory::ScriptWorkerEvent);
    maybe_create_heartbeat(&mut hbs, ProfilerCategory::ScriptXhrEvent);
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

/// Create a heartbeat if the correct environment variable is set
fn maybe_create_heartbeat(hbs: &mut HashMap<ProfilerCategory, Heartbeat>,
                          category: ProfilerCategory) {
    static WINDOW_SIZE_DEFAULT: usize = 20;
    if let Some(_) = var_os(format!("SERVO_HEARTBEAT_ENABLE_{:?}", category)) {
        // get optional log file
        let logfile: Option<File> = var_os(format!("SERVO_HEARTBEAT_LOG_{:?}", category))
                                    .and_then(|name| File::create(name).ok());
        // get window size
        let window_size: usize = match var_os(format!("SERVO_HEARTBEAT_WINDOW_{:?}", category)) {
            Some(w) => match w.into_string() {
                Ok(s) => s.parse::<usize>().unwrap_or(WINDOW_SIZE_DEFAULT),
                _ => WINDOW_SIZE_DEFAULT,
            },
            None => WINDOW_SIZE_DEFAULT,
        };
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
