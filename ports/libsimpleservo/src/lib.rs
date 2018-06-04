/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern crate android_logger;
extern crate jni;
extern crate libc;
#[macro_use]
extern crate log;
extern crate servo;

mod api;
mod gl_glue;
mod jniapi;

#[cfg(not(target_os = "android"))]
pub use api::*;

#[cfg(target_os = "android")]
pub use jniapi::*;
