/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![deny(unsafe_code)]

#[macro_use]
extern crate log;

pub use crate::compositor::IOCompositor;
pub use crate::compositor::ShutdownState;

mod compositor;
#[cfg(feature = "gl")]
mod gl;
mod touch;
pub mod windowing;
