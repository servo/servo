/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![deny(unsafe_code)]

#[macro_use]
mod tracing;

mod broadcastchannel;
mod browsingcontext;
mod constellation;
mod constellation_webview;
mod embedder;
mod event_loop;
mod logging;
mod pipeline;
#[cfg(feature = "multiprocess")]
mod process_manager;
#[cfg(feature = "multiprocess")]
mod sandboxing;
mod screenshot_readiness_request;
mod serviceworker;
mod session_history;

pub use crate::constellation::{Constellation, InitialConstellationState};
pub use crate::embedder::ConstellationToEmbedderMsg;
pub use crate::event_loop::EventLoop;
#[cfg(feature = "multiprocess")]
pub use crate::event_loop::NewScriptEventLoopProcessInfo;
pub use crate::logging::{FromEmbedderLogger, FromScriptLogger};
#[cfg(feature = "multiprocess")]
pub use crate::sandboxing::{UnprivilegedContent, content_process_sandbox_profile};
