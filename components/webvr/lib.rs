/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![deny(unsafe_code)]

extern crate ipc_channel;
#[macro_use]
extern crate log;
extern crate msg;
extern crate script_traits;
extern crate servo_config;
extern crate webrender_api;
extern crate webvr_traits;

mod webvr_thread;
pub use webvr_thread::{WebVRThread, WebVRCompositorHandler};
