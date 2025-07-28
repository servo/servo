/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![crate_name = "canvas_traits"]
#![crate_type = "rlib"]
#![deny(unsafe_code)]

use crossbeam_channel::Sender;
use euclid::default::Size2D;
use webrender_api::ImageKey;

use crate::canvas::CanvasId;

pub mod canvas;
#[macro_use]
pub mod webgl;

pub enum ConstellationCanvasMsg {
    Create {
        sender: Sender<Option<(CanvasId, ImageKey)>>,
        size: Size2D<u64>,
    },
    Exit(Sender<()>),
}
