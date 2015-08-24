/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(append)]
#![feature(arc_unique)]
#![feature(ascii)]
#![feature(as_slice)]
#![feature(as_unsafe_cell)]
#![feature(borrow_state)]
#![feature(box_raw)]
#![feature(box_syntax)]
#![feature(cell_extras)]
#![feature(core)]
#![feature(core_intrinsics)]
#![feature(custom_attribute)]
#![feature(custom_derive)]
#![feature(drain)]
#![feature(fnbox)]
#![feature(hashmap_hasher)]
#![feature(mpsc_select)]
#![feature(nonzero)]
#![feature(plugin)]
#![feature(ref_slice)]
#![feature(rc_unique)]
#![feature(slice_patterns)]
#![feature(str_utf16)]
#![feature(unicode)]
#![feature(vec_push_all)]

#![deny(unsafe_code)]
#![allow(non_snake_case)]

#![doc="The script crate contains all matters DOM."]

#![plugin(string_cache_plugin)]
#![plugin(plugins)]

#[macro_use]
extern crate log;

#[macro_use] extern crate bitflags;
extern crate core;
extern crate devtools_traits;
extern crate cssparser;
extern crate euclid;
extern crate html5ever;
extern crate encoding;
extern crate fnv;
extern crate hyper;
extern crate ipc_channel;
extern crate js;
extern crate libc;
extern crate msg;
extern crate net_traits;
extern crate num;
extern crate rustc_serialize;
extern crate rustc_unicode;
extern crate serde;
extern crate time;
extern crate canvas;
extern crate canvas_traits;
extern crate rand;
#[macro_use]
extern crate profile_traits;
extern crate script_traits;
extern crate selectors;
extern crate smallvec;
#[macro_use]
extern crate util;
extern crate websocket;
#[macro_use]
extern crate style;
extern crate unicase;
extern crate url;
extern crate uuid;
extern crate string_cache;
extern crate offscreen_gl_context;
extern crate tendril;

pub mod cors;
pub mod document_loader;

#[macro_use]
pub mod dom;

pub mod parse;

pub mod layout_interface;
mod mem;
mod network_listener;
pub mod page;
pub mod script_task;
mod timers;
pub mod textinput;
pub mod clipboard_provider;
mod devtools;
mod horribly_inefficient_timers;
mod webdriver_handlers;

use dom::bindings::codegen::RegisterBindings;

#[cfg(target_os="linux")]
#[allow(unsafe_code)]
fn perform_platform_specific_initialization() {
    use std::mem;
    const RLIMIT_NOFILE: libc::c_int = 7;

    // Bump up our number of file descriptors to save us from impending doom caused by an onslaught
    // of iframes.
    unsafe {
        let mut rlim = mem::uninitialized();
        assert!(libc::getrlimit(RLIMIT_NOFILE, &mut rlim) == 0);
        rlim.rlim_cur = rlim.rlim_max;
        assert!(libc::setrlimit(RLIMIT_NOFILE, &mut rlim) == 0);
    }
}

#[cfg(not(target_os="linux"))]
fn perform_platform_specific_initialization() {}

#[allow(unsafe_code)]
pub fn init() {
    unsafe {
        assert_eq!(js::jsapi::JS_Init(), 1);
    }

    // Create the global vtables used by the (generated) DOM
    // bindings to implement JS proxies.
    RegisterBindings::RegisterProxyHandlers();

    perform_platform_specific_initialization();
}
