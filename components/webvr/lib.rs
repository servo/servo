/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![deny(unsafe_code)]
#![feature(stmt_expr_attributes)]

#[macro_use]
extern crate log;

mod webvr_test;
mod webvr_thread;
pub use crate::webvr_thread::{WebVRCompositorHandler, WebVRThread};
