/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![deny(unsafe_code)]

use canvas_traits::canvas::CanvasId;
use crossbeam_channel::Sender;
use euclid::default::Size2D;

#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate log;

#[cfg(feature = "canvas2d-azure")]
mod azure_backend;

#[cfg(feature = "canvas2d-raqote")]
mod raqote_backend;

pub use webgl_mode::WebGLComm;

pub mod canvas_data;
pub mod canvas_paint_thread;
mod webgl_limits;
mod webgl_mode;
pub mod webgl_thread;

pub enum ConstellationCanvasMsg {
    Create {
        id_sender: Sender<CanvasId>,
        size: Size2D<u64>,
        webrender_sender: webrender_api::RenderApiSender,
        antialias: bool,
    },
    Exit,
}
