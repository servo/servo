/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![deny(unsafe_code)]
#![cfg_attr(feature = "unstable", feature(conservative_impl_trait))]
#![feature(mpsc_select)]

extern crate backtrace;
extern crate bluetooth_traits;
extern crate canvas;
extern crate canvas_traits;
extern crate clipboard;
extern crate compositing;
extern crate debugger;
extern crate devtools_traits;
extern crate euclid;
#[cfg(all(not(target_os = "windows"), not(target_os = "ios")))]
extern crate gaol;
extern crate gfx;
extern crate gfx_traits;
extern crate hyper;
extern crate ipc_channel;
extern crate itertools;
extern crate layout_traits;
#[macro_use]
extern crate log;
extern crate metrics;
extern crate msg;
extern crate net;
extern crate net_traits;
extern crate profile_traits;
extern crate script_traits;
#[macro_use] extern crate serde;
extern crate servo_config;
extern crate servo_rand;
extern crate servo_remutex;
extern crate servo_url;
extern crate style_traits;
extern crate webrender_api;
extern crate webvr_traits;

mod browsingcontext;
mod constellation;
mod event_loop;
mod network_listener;
mod pipeline;
#[cfg(all(not(target_os = "windows"), not(target_os = "ios")))]
mod sandboxing;
mod timer_scheduler;

pub use constellation::{Constellation, FromCompositorLogger, FromScriptLogger, InitialConstellationState};
pub use pipeline::UnprivilegedPipelineContent;
#[cfg(all(not(target_os = "windows"), not(target_os = "ios")))]
pub use sandboxing::content_process_sandbox_profile;
