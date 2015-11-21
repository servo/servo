/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(ascii)]
#![feature(as_unsafe_cell)]
#![feature(borrow_state)]
#![feature(box_syntax)]
#![feature(cell_extras)]
#![feature(const_fn)]
#![feature(core)]
#![feature(core_intrinsics)]
#![feature(custom_attribute)]
#![feature(custom_derive)]
#![feature(drain)]
#![feature(fnbox)]
#![feature(hashmap_hasher)]
#![feature(iter_arith)]
#![feature(mpsc_select)]
#![feature(nonzero)]
#![feature(on_unimplemented)]
#![feature(plugin)]
#![feature(ref_slice)]
#![feature(slice_patterns)]
#![feature(str_utf16)]
#![feature(unicode)]
#![feature(vec_push_all)]

#![deny(unsafe_code)]
#![allow(non_snake_case)]

#![doc = "The script crate contains all matters DOM."]

#![plugin(string_cache_plugin)]
#![plugin(plugins)]

extern crate app_units;
#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate log;
#[macro_use]
extern crate profile_traits;
#[macro_use]
extern crate style;
#[macro_use]
extern crate util;
extern crate angle;
extern crate canvas;
extern crate canvas_traits;
extern crate caseless;
extern crate core;
extern crate cssparser;
extern crate devtools_traits;
extern crate encoding;
extern crate euclid;
extern crate fnv;
extern crate html5ever;
extern crate hyper;
extern crate image;
extern crate ipc_channel;
extern crate js;
extern crate libc;
extern crate msg;
extern crate net_traits;
extern crate num;
extern crate offscreen_gl_context;
extern crate rand;
extern crate rustc_serialize;
extern crate rustc_unicode;
extern crate script_traits;
#[macro_use(state_pseudo_classes)] extern crate selectors;
extern crate serde;
extern crate smallvec;
extern crate string_cache;
extern crate tendril;
extern crate time;
extern crate unicase;
extern crate url;
extern crate uuid;
extern crate websocket;

pub mod clipboard_provider;
pub mod cors;
mod devtools;
pub mod document_loader;
#[macro_use]
pub mod dom;
pub mod layout_interface;
mod mem;
mod network_listener;
pub mod page;
pub mod parse;
#[allow(unsafe_code)]
pub mod script_task;
pub mod textinput;
mod timers;
mod unpremultiplytable;
mod webdriver_handlers;

use dom::bindings::codegen::RegisterBindings;
use js::jsapi::SetDOMProxyInformation;
use std::ptr;

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
                match libc::setrlimit(libc::RLIMIT_NOFILE, &mut rlim) {
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
pub fn init() {
    unsafe {
        assert_eq!(js::jsapi::JS_Init(), true);
        SetDOMProxyInformation(ptr::null(), 0, Some(script_task::shadow_check_callback));
    }

    // Create the global vtables used by the (generated) DOM
    // bindings to implement JS proxies.
    RegisterBindings::RegisterProxyHandlers();

    perform_platform_specific_initialization();
}
