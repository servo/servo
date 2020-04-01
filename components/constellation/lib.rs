/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![deny(unsafe_code)]

#[macro_use]
extern crate crossbeam_channel;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde;

mod browsingcontext;
mod constellation;
mod event_loop;
mod network_listener;
mod pipeline;
#[cfg(all(
    not(target_os = "windows"),
    not(target_os = "ios"),
    not(target_os = "android"),
    not(target_arch = "arm"),
    not(target_arch = "aarch64")
))]
mod sandboxing;
mod serviceworker;
mod session_history;
mod timer_scheduler;

pub use crate::constellation::{
    Constellation, FromCompositorLogger, FromScriptLogger, InitialConstellationState,
};
pub use crate::pipeline::UnprivilegedPipelineContent;
#[cfg(all(
    not(target_os = "windows"),
    not(target_os = "ios"),
    not(target_os = "android"),
    not(target_arch = "arm"),
    not(target_arch = "aarch64")
))]
pub use crate::sandboxing::{content_process_sandbox_profile, UnprivilegedContent};
