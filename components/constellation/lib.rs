/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(box_syntax)]
#![feature(mpsc_select)]
#![feature(plugin)]
#![feature(proc_macro)]
#![plugin(plugins)]

#![deny(unsafe_code)]

extern crate backtrace;
extern crate bluetooth_traits;
extern crate canvas;
extern crate canvas_traits;
extern crate compositing;
extern crate debugger;
extern crate devtools_traits;
extern crate euclid;
#[cfg(not(target_os = "windows"))]
extern crate gaol;
extern crate gfx;
extern crate gfx_traits;
extern crate ipc_channel;
extern crate layout_traits;
#[macro_use]
extern crate log;
extern crate msg;
extern crate net_traits;
extern crate offscreen_gl_context;
#[macro_use]
extern crate profile_traits;
extern crate rand;
extern crate script_traits;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate style_traits;
extern crate url;
#[macro_use]
extern crate util;
extern crate webrender_traits;

mod constellation;
mod pipeline;
#[cfg(not(target_os = "windows"))]
mod sandboxing;
mod timer_scheduler;

pub use constellation::{Constellation, FromCompositorLogger, FromScriptLogger, InitialConstellationState};
pub use pipeline::UnprivilegedPipelineContent;
#[cfg(not(target_os = "windows"))]
pub use sandboxing::content_process_sandbox_profile;
