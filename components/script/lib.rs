/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(alloc)]
#![feature(box_syntax)]
#![feature(collections)]
#![feature(core)]
#![feature(custom_attribute)]
#![feature(int_uint)]
#![feature(old_io)]
#![feature(path)]
#![feature(plugin)]
#![feature(rustc_private)]
#![feature(std_misc)]
#![feature(unicode)]
#![feature(unsafe_destructor)]

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
extern crate collections;
extern crate geom;
extern crate html5ever;
extern crate encoding;
extern crate hyper;
extern crate js;
extern crate libc;
extern crate msg;
extern crate net_traits;
extern crate "rustc-serialize" as rustc_serialize;
extern crate time;
extern crate canvas;
extern crate profile;
extern crate script_traits;
extern crate selectors;
extern crate util;
#[macro_use]
extern crate style;
extern crate url;
extern crate uuid;
extern crate string_cache;

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
