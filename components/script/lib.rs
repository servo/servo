/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(alloc)]
#![feature(box_syntax)]
#![feature(collections)]
#![feature(core)]
#![feature(custom_attribute)]
#![feature(plugin)]
#![feature(rustc_private)]
#![feature(std_misc)]

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
extern crate num;
extern crate png;
extern crate rustc_serialize;
extern crate time;
extern crate canvas;
extern crate profile_traits;
extern crate script_traits;
extern crate selectors;
extern crate util;
extern crate websocket;
#[macro_use]
extern crate style;
extern crate unicase;
extern crate url;
extern crate uuid;
extern crate string_cache;
extern crate webdriver_traits;

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
