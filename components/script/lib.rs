/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(box_syntax)]
#![feature(conservative_impl_trait)]
#![feature(const_fn)]
#![feature(core_intrinsics)]
#![feature(custom_attribute)]
#![feature(custom_derive)]
#![feature(fnbox)]
#![feature(mpsc_select)]
#![feature(nonzero)]
#![feature(on_unimplemented)]
#![feature(optin_builtin_traits)]
#![feature(plugin)]
#![feature(question_mark)]
#![feature(slice_patterns)]
#![feature(stmt_expr_attributes)]
#![feature(try_from)]
#![feature(untagged_unions)]

#![deny(unsafe_code)]
#![allow(non_snake_case)]

#![doc = "The script crate contains all matters DOM."]

#![plugin(heapsize_plugin)]
#![plugin(phf_macros)]
#![plugin(plugins)]

extern crate angle;
extern crate app_units;
extern crate audio_video_metadata;
#[allow(unused_extern_crates)]
#[macro_use]
extern crate bitflags;
extern crate canvas_traits;
extern crate caseless;
extern crate cookie as cookie_rs;
extern crate core;
#[macro_use]
extern crate cssparser;
extern crate devtools_traits;
extern crate encoding;
extern crate euclid;
extern crate fnv;
extern crate gfx_traits;
extern crate heapsize;
extern crate html5ever;
extern crate hyper;
extern crate hyper_serde;
extern crate image;
extern crate ipc_channel;
#[macro_use]
extern crate js;
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
extern crate rand;
extern crate range;
extern crate ref_slice;
extern crate regex;
extern crate rustc_serialize;
extern crate script_layout_interface;
extern crate script_traits;
extern crate selectors;
extern crate serde;
extern crate smallvec;
#[macro_use(atom, ns)] extern crate string_cache;
#[macro_use]
extern crate style;
extern crate time;
#[cfg(any(target_os = "macos", target_os = "linux", target_os = "windows"))]
extern crate tinyfiledialogs;
extern crate url;
#[macro_use]
extern crate util;
extern crate uuid;
extern crate webrender_traits;
extern crate websocket;
extern crate xml5ever;

mod body;
pub mod clipboard_provider;
mod devtools;
pub mod document_loader;
#[macro_use]
pub mod dom;
pub mod fetch;
pub mod layout_wrapper;
mod mem;
mod network_listener;
pub mod origin;
pub mod parse;
pub mod script_runtime;
#[allow(unsafe_code)]
pub mod script_thread;
mod serviceworker_manager;
mod task_source;
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

#[allow(unsafe_code)]
pub fn init(sw_senders: SWManagerSenders) {
    unsafe {
        proxyhandler::init();
    }

    // Spawn the service worker manager passing the constellation sender
    ServiceWorkerManager::spawn_manager(sw_senders);

    // Create the global vtables used by the (generated) DOM
    // bindings to implement JS proxies.
    RegisterBindings::RegisterProxyHandlers();

    perform_platform_specific_initialization();
}
