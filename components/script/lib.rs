/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(append)]
#![feature(arc_unique)]
#![feature(as_unsafe_cell)]
#![feature(borrow_state)]
#![feature(box_raw)]
#![feature(box_syntax)]
#![feature(core)]
#![feature(core_intrinsics)]
#![feature(custom_attribute)]
#![feature(custom_derive)]
#![feature(drain)]
#![feature(hashmap_hasher)]
#![feature(mpsc_select)]
#![feature(nonzero)]
#![feature(owned_ascii_ext)]
#![feature(plugin)]
#![feature(rc_unique)]
#![feature(slice_chars)]
#![feature(str_utf16)]
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
extern crate js;
extern crate libc;
extern crate msg;
extern crate net_traits;
extern crate num;
extern crate png;
extern crate rustc_serialize;
extern crate time;
extern crate canvas;
extern crate canvas_traits;
extern crate rand;
extern crate profile_traits;
extern crate script_traits;
extern crate selectors;
extern crate smallvec;
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
mod network_listener;
pub mod page;
pub mod script_task;
mod timers;
pub mod textinput;
pub mod clipboard_provider;
mod devtools;
mod horribly_inefficient_timers;
mod webdriver_handlers;

#[allow(unsafe_code)]
pub fn init() {
    unsafe {
        assert_eq!(js::jsapi::JS_Init(), 1);
    }
}
