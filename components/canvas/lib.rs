/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![deny(unsafe_code)]
#![allow(clippy::too_many_arguments)]

mod backend;

#[cfg(any(
    not(any(feature = "vello", feature = "vello_cpu")),
    all(feature = "vello", feature = "vello_cpu")
))]
compile_error!("Either feature \"vello\" or \"vello_cpu\" must be enabled for this crate.");

#[cfg(any(feature = "vello", feature = "vello_cpu"))]
mod peniko_conversions;

#[cfg(feature = "vello")]
mod vello_backend;

#[cfg(feature = "vello_cpu")]
mod vello_cpu_backend;

pub mod canvas_data;
pub mod canvas_paint_thread;
