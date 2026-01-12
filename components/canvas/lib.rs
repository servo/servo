/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![deny(unsafe_code)]
#![allow(clippy::too_many_arguments)]

mod backend;
pub mod canvas_data;
pub mod canvas_paint_thread;
mod peniko_conversions;
#[cfg(feature = "vello")]
mod vello_backend;
mod vello_cpu_backend;
