/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(custom_derive)]
#![feature(plugin)]
#![feature(proc_macro)]
#![deny(unsafe_code)]

extern crate ipc_channel;
#[macro_use]
extern crate log;
extern crate msg;
extern crate script_traits;
extern crate vr_traits;
extern crate webrender_traits;

mod webvr_thread;
pub use webvr_thread::{WebVRThread, WebVRCompositorHandler};
