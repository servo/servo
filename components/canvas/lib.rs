/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![deny(unsafe_code)]

mod backend;
#[cfg(any(feature = "vello", feature = "vello_cpu"))]
mod peniko_conversions;
mod raqote_backend;
#[cfg(feature = "vello")]
mod vello_backend;
#[cfg(feature = "vello_cpu")]
mod vello_cpu_backend;

pub mod canvas_data;
pub mod canvas_paint_thread;
