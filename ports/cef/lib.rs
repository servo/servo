/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![allow(non_camel_case_types)]
#![feature(box_syntax)]
#![feature(core_intrinsics)]
#![feature(link_args)]

#[macro_use]
extern crate log;

extern crate servo;
extern crate compositing;

extern crate euclid;
extern crate gleam;
extern crate glutin_app;
extern crate script_traits;
extern crate servo_config;
extern crate servo_geometry;
extern crate servo_url;
extern crate style_traits;

extern crate net_traits;
extern crate msg;
extern crate webrender_api;

extern crate libc;

#[cfg(target_os="macos")]
#[link_args="-Xlinker -undefined -Xlinker dynamic_lookup"]
extern { }

#[cfg(target_os="macos")]
extern crate cocoa;
#[cfg(target_os="macos")]
#[macro_use]
extern crate objc;

// Must come first.
pub mod macros;

pub mod browser;
pub mod browser_host;
pub mod command_line;
pub mod cookie;
pub mod core;
pub mod drag_data;
pub mod eutil;
pub mod frame;
pub mod interfaces;
pub mod print_settings;
pub mod process_message;
pub mod render_handler;
pub mod request;
pub mod request_context;
pub mod response;
pub mod stream;
pub mod string;
pub mod string_list;
pub mod string_map;
pub mod string_multimap;
pub mod stubs;
pub mod switches;
pub mod task;
pub mod types;
pub mod urlrequest;
pub mod v8;
pub mod values;
pub mod window;
pub mod wrappers;
pub mod xml_reader;
pub mod zip_reader;
