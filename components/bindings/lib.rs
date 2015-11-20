/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Generic code for DOM bindings. 

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

pub mod pointers;
pub mod inheritance;
pub mod reflector;
pub mod conversions;
pub mod utils;
pub mod trace;
pub mod no_trace;
