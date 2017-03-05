/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(box_syntax)]
#![feature(conservative_impl_trait)]
#![feature(const_fn)]
#![feature(core_intrinsics)]
#![feature(mpsc_select)]
#![feature(nonzero)]
#![feature(on_unimplemented)]
#![feature(optin_builtin_traits)]
#![feature(plugin)]
#![feature(proc_macro)]
#![feature(slice_patterns)]
#![feature(stmt_expr_attributes)]
#![feature(try_from)]
#![feature(untagged_unions)]

#![deny(unsafe_code)]
#![allow(non_snake_case)]

#![doc = "The script crate contains all matters DOM."]

#![plugin(script_plugins)]

extern crate angle;
extern crate app_units;
extern crate atomic_refcell;
extern crate audio_video_metadata;
#[macro_use]
extern crate bitflags;
extern crate bluetooth_traits;
extern crate byteorder;
extern crate canvas_traits;
extern crate caseless;
extern crate cookie as cookie_rs;
extern crate core;
#[macro_use] extern crate cssparser;
#[macro_use] extern crate deny_public_fields;
extern crate devtools_traits;
extern crate dom_struct;
#[macro_use]
extern crate domobject_derive;
extern crate encoding;
extern crate euclid;
extern crate fnv;
extern crate gfx_traits;
extern crate heapsize;
#[macro_use] extern crate heapsize_derive;
extern crate html5ever;
#[macro_use] extern crate html5ever_atoms;
#[macro_use]
extern crate hyper;
extern crate hyper_serde;
extern crate image;
extern crate ipc_channel;
#[macro_use]
extern crate js;
#[macro_use]
extern crate jstraceable_derive;
extern crate libc;
#[macro_use]
extern crate log;
#[macro_use]
extern crate mime;
extern crate mime_guess;
extern crate msg;
extern crate net_traits;
extern crate num_traits;
extern crate offscreen_gl_context;
extern crate open;
extern crate parking_lot;
extern crate phf;
#[macro_use]
extern crate profile_traits;
extern crate range;
extern crate ref_filter_map;
extern crate ref_slice;
extern crate regex;
extern crate rustc_serialize;
extern crate script_layout_interface;
extern crate script_traits;
extern crate selectors;
extern crate serde;
#[macro_use] extern crate servo_atoms;
extern crate servo_config;
extern crate servo_geometry;
extern crate servo_rand;
extern crate servo_url;
extern crate smallvec;
#[macro_use]
extern crate style;
extern crate style_traits;
extern crate time;
#[cfg(any(target_os = "macos", target_os = "linux", target_os = "windows"))]
extern crate tinyfiledialogs;
extern crate url;
extern crate uuid;
extern crate webrender_traits;
extern crate websocket;
extern crate webvr_traits;
extern crate xml5ever;

mod body;
pub mod clipboard_provider;
mod devtools;
pub mod document_loader;
#[macro_use]
mod dom;
pub mod fetch;
mod layout_image;
pub mod layout_wrapper;
mod mem;
mod microtask;
mod network_listener;
pub mod script_runtime;
#[allow(unsafe_code)]
pub mod script_thread;
mod serviceworker_manager;
mod serviceworkerjob;
mod stylesheet_loader;
mod task_source;
pub mod test;
pub mod textinput;
mod timers;
mod unpremultiplytable;
mod webdriver_handlers;

use dom::bindings::codegen::RegisterBindings;
use dom::bindings::proxyhandler;
use script_traits::SWManagerSenders;
use serviceworker_manager::ServiceWorkerManager;

#[cfg(target_os = "linux")]
#[allow(unsafe_code)]
fn perform_platform_specific_initialization() {
    use std::mem;
    // 4096 is default max on many linux systems
    const MAX_FILE_LIMIT: libc::rlim_t = 4096;

    // Bump up our number of file descriptors to save us from impending doom caused by an onslaught
    // of iframes.
    unsafe {
        let mut rlim: libc::rlimit = mem::uninitialized();
        match libc::getrlimit(libc::RLIMIT_NOFILE, &mut rlim) {
            0 => {
                if rlim.rlim_cur >= MAX_FILE_LIMIT {
                    // we have more than enough
                    return;
                }

                rlim.rlim_cur = match rlim.rlim_max {
                    libc::RLIM_INFINITY => MAX_FILE_LIMIT,
                    _ => {
                        if rlim.rlim_max < MAX_FILE_LIMIT {
                            rlim.rlim_max
                        } else {
                            MAX_FILE_LIMIT
                        }
                    }
                };
                match libc::setrlimit(libc::RLIMIT_NOFILE, &rlim) {
                    0 => (),
                    _ => warn!("Failed to set file count limit"),
                };
            },
            _ => warn!("Failed to get file count limit"),
        };
    }
}

#[cfg(not(target_os = "linux"))]
fn perform_platform_specific_initialization() {}

pub fn init_service_workers(sw_senders: SWManagerSenders) {
    // Spawn the service worker manager passing the constellation sender
    ServiceWorkerManager::spawn_manager(sw_senders);
}

#[allow(unsafe_code)]
pub fn init() {
    unsafe {
        proxyhandler::init();

        // Create the global vtables used by the (generated) DOM
        // bindings to implement JS proxies.
        RegisterBindings::RegisterProxyHandlers();
    }

    perform_platform_specific_initialization();
}
