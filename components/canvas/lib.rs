/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![deny(unsafe_code)]

mod backend;
mod raqote_backend;
#[cfg(feature = "vello")]
mod vello_backend;

pub mod canvas_data;
pub mod canvas_paint_thread;
