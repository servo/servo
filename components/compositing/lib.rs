/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(box_syntax)]
#![feature(iter_cmp)]
#![feature(slice_bytes)]
#![feature(vec_push_all)]

#[macro_use]
extern crate log;

extern crate azure;
extern crate devtools_traits;
extern crate euclid;
extern crate gfx;
extern crate layers;
extern crate layout_traits;
extern crate png;
extern crate script_traits;
extern crate msg;
extern crate net;
extern crate num;
extern crate profile_traits;
extern crate net_traits;
extern crate gfx_traits;
extern crate style;
#[macro_use]
extern crate util;
extern crate gleam;
extern crate clipboard;

extern crate libc;
extern crate time;
extern crate url;

#[cfg(target_os="macos")]
extern crate core_graphics;
#[cfg(target_os="macos")]
extern crate core_text;

pub use compositor_task::{CompositorEventListener, CompositorProxy, CompositorTask};
pub use constellation::Constellation;

pub mod compositor_task;

mod buffer_map;
mod compositor_layer;
mod compositor;
mod headless;
mod scrolling;

pub mod pipeline;
pub mod constellation;
pub mod windowing;

