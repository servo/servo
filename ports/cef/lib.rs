/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(int_uint)]
#![feature(box_syntax)]
#![feature(plugin)]
#![feature(link_args)]
#![feature(thread_local)]
#![feature(libc)]
#![feature(unicode)]
#![feature(core)]
#![feature(std_misc)]
#![feature(rustc_private)]
#![feature(collections)]

#![allow(non_camel_case_types)]

#[macro_use]
extern crate log;
#[plugin] #[no_link]
extern crate "plugins" as servo_plugins;

extern crate servo;
extern crate compositing;

extern crate azure;
extern crate geom;
extern crate gfx;
extern crate gleam;
extern crate glutin_app;
extern crate js;
extern crate layers;
extern crate png;
extern crate script;
extern crate unicode;

extern crate net;
extern crate msg;
extern crate util;
extern crate style;
extern crate stb_image;

extern crate libc;
extern crate "url" as std_url;

#[cfg(target_os="macos")]
extern crate cgl;
#[cfg(target_os="macos")]
extern crate cocoa;
#[cfg(target_os="macos")]
extern crate core_graphics;
#[cfg(target_os="macos")]
extern crate core_text;

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
