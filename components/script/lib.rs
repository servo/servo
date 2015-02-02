/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(unsafe_destructor, plugin, box_syntax, int_uint)]

#![deny(unsafe_blocks)]
#![deny(unused_imports)]
#![deny(unused_variables)]
#![allow(non_snake_case)]
#![allow(missing_copy_implementations)]
#![allow(unstable)]

#![doc="The script crate contains all matters DOM."]

#[macro_use]
extern crate log;

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
extern crate serialize;
extern crate time;
extern crate canvas;
extern crate script_traits;
#[no_link] #[plugin] #[macro_use]
extern crate "plugins" as servo_plugins;
extern crate "net" as servo_net;
extern crate util;
#[macro_use]
extern crate style;
extern crate "msg" as servo_msg;
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
