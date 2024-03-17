/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![deny(unsafe_code)]

#[macro_use]
mod tracing;

mod browsingcontext;
mod constellation;
mod event_loop;
mod logging;
mod network_listener;
mod pipeline;
mod sandboxing;
mod serviceworker;
mod session_history;
mod timer_scheduler;
mod webview;

pub use crate::constellation::{Constellation, InitialConstellationState};
pub use crate::logging::{FromCompositorLogger, FromScriptLogger};
pub use crate::pipeline::UnprivilegedPipelineContent;
pub use crate::sandboxing::{content_process_sandbox_profile, UnprivilegedContent};
