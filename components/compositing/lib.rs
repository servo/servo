/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(box_syntax)]
#![feature(iter_cmp)]
#![feature(slice_bytes)]
#![feature(vec_push_all)]

#![deny(unsafe_code)]

#[macro_use]
extern crate log;

extern crate azure;
extern crate canvas;
extern crate canvas_traits;
extern crate devtools_traits;
extern crate euclid;
extern crate gfx;
extern crate ipc_channel;
extern crate layers;
extern crate layout_traits;
extern crate offscreen_gl_context;
extern crate png;
extern crate script_traits;
extern crate msg;
extern crate num;
#[macro_use]
extern crate profile_traits;
extern crate net_traits;
extern crate gfx_traits;
extern crate style_traits;
#[macro_use]
extern crate util;
extern crate gleam;
extern crate clipboard;

extern crate time;
extern crate url;

#[cfg(target_os = "macos")]
extern crate core_graphics;
#[cfg(target_os = "macos")]
extern crate core_text;

pub use compositor_task::{CompositorEventListener, CompositorProxy, CompositorTask};
pub use constellation::Constellation;

pub mod compositor_task;

mod surface_map;
mod compositor_layer;
mod compositor;
mod headless;
mod scrolling;

pub mod pipeline;
pub mod constellation;
pub mod windowing;
