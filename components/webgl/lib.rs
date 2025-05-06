/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![deny(unsafe_code)]

pub use webgl_mode::WebGLComm;

mod webgl_limits;
mod webgl_mode;
pub mod webgl_thread;
#[cfg(feature = "webxr")]
mod webxr;
