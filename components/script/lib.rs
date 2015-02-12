/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(alloc)]
#![feature(box_syntax)]
#![feature(collections)]
#![feature(core)]
#![feature(hash)]
#![feature(int_uint)]
#![feature(io)]
#![feature(libc)]
#![feature(plugin)]
#![feature(rustc_private)]
#![feature(std_misc)]
#![feature(unicode)]
#![feature(unsafe_destructor)]

#![deny(unsafe_blocks)]
#![allow(non_snake_case)]
#![allow(missing_copy_implementations)]

#![doc="The script crate contains all matters DOM."]

#[macro_use]
extern crate log;

#[macro_use] extern crate bitflags;
extern crate core;
extern crate devtools_traits;
extern crate cssparser;
extern crate collections;
extern crate geom;
extern crate html5ever;
extern crate encoding;
extern crate hyper;
extern crate js;
extern crate libc;
extern crate msg;
extern crate net;
extern crate "rustc-serialize" as serialize;
extern crate time;
extern crate canvas;
extern crate script_traits;
#[no_link] #[plugin] #[macro_use]
extern crate "plugins" as servo_plugins;
extern crate util;
#[macro_use]
extern crate style;
extern crate url;
extern crate uuid;
extern crate string_cache;
#[no_link] #[macro_use] #[plugin]
extern crate string_cache_macros;

pub mod cors;

#[macro_use]
pub mod dom;
pub mod parse;

pub mod layout_interface;
pub mod page;
pub mod script_task;
mod timers;
pub mod textinput;
mod devtools;

#[cfg(all(test, target_pointer_width = "64"))]
mod tests;
