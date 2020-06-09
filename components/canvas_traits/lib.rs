/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![crate_name = "canvas_traits"]
#![crate_type = "rlib"]
#![deny(unsafe_code)]

use crate::canvas::CanvasId;
use crossbeam_channel::Sender;
use euclid::default::Size2D;

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate malloc_size_of_derive;
#[macro_use]
extern crate serde;

pub mod canvas;
#[macro_use]
pub mod webgl;
mod webgl_channel;

pub enum ConstellationCanvasMsg {
    Create {
        id_sender: Sender<CanvasId>,
        size: Size2D<u64>,
        antialias: bool,
    },
    Exit,
}
