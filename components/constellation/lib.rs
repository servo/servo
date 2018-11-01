/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![deny(unsafe_code)]
#![cfg_attr(feature = "unstable", feature(conservative_impl_trait))]

#[macro_use]
extern crate log;
#[macro_use]
extern crate servo_channel;

mod browsingcontext;
mod constellation;
mod event_loop;
mod network_listener;
mod pipeline;
#[cfg(all(not(target_os = "windows"), not(target_os = "ios")))]
mod sandboxing;
mod session_history;
mod timer_scheduler;

pub use crate::constellation::{
    Constellation, FromCompositorLogger, FromScriptLogger, InitialConstellationState,
};
pub use crate::pipeline::UnprivilegedPipelineContent;
#[cfg(all(not(target_os = "windows"), not(target_os = "ios")))]
pub use crate::sandboxing::content_process_sandbox_profile;
