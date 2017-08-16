/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![deny(unsafe_code)]
#![feature(box_syntax)]
#![feature(conservative_impl_trait)]
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
#[cfg(not(target_os = "windows"))]
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
extern crate offscreen_gl_context;
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
#[cfg(not(target_os = "windows"))]
mod sandboxing;
mod timer_scheduler;

pub use constellation::{Constellation, FromCompositorLogger, FromScriptLogger, InitialConstellationState};
pub use pipeline::UnprivilegedPipelineContent;
#[cfg(not(target_os = "windows"))]
pub use sandboxing::content_process_sandbox_profile;
