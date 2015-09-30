/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(box_syntax)]
#![feature(iter_cmp)]
#![feature(slice_bytes)]
#![feature(vec_push_all)]
#![feature(plugin)]
#![plugin(plugins)]

#![deny(unsafe_code)]

extern crate app_units;
#[macro_use]
extern crate log;
#[macro_use]
extern crate profile_traits;
#[macro_use]
extern crate util;
extern crate azure;
extern crate canvas;
extern crate canvas_traits;
extern crate clipboard;

#[cfg(target_os = "macos")]
extern crate core_graphics;
#[cfg(target_os = "macos")]
extern crate core_text;

extern crate devtools_traits;
extern crate euclid;
extern crate gfx;
extern crate gfx_traits;
extern crate gleam;
extern crate ipc_channel;
extern crate layers;
extern crate layout_traits;
extern crate msg;
extern crate net_traits;
extern crate num;
extern crate offscreen_gl_context;
extern crate png;
extern crate script_traits;
extern crate style_traits;
extern crate time;
extern crate url;

pub use compositor_task::{CompositorEventListener, CompositorProxy, CompositorTask};
pub use constellation::Constellation;

mod compositor;
mod compositor_layer;
mod headless;
mod scrolling;
mod surface_map;
pub mod compositor_task;
pub mod constellation;
pub mod pipeline;
pub mod windowing;
