/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(custom_derive)]
#![feature(plugin)]
#![feature(proc_macro)]
#![deny(unsafe_code)]

extern crate ipc_channel;
extern crate msg;
extern crate serde;
#[macro_use]
extern crate serde_derive;
pub extern crate rust_webvr as webvr;

mod webvr_traits;
pub use webvr_traits::{WebVRMsg, WebVRResult};
