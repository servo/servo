/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#[macro_use]
extern crate log;

mod api;
mod gl_glue;

// If not Android, expose the C-API
#[cfg(not(target_os = "android"))]
mod capi;
#[cfg(not(target_os = "android"))]
pub use crate::capi::*;

// If Android, expose the JNI-API
#[cfg(target_os = "android")]
mod jniapi;
#[cfg(target_os = "android")]
pub use crate::jniapi::*;
