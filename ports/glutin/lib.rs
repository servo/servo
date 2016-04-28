/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A simple application that uses glutin to open a window for Servo to display in.

#![feature(box_syntax)]

#[macro_use] extern crate bitflags;
extern crate compositing;
#[allow(unused_extern_crates)]
#[cfg(target_os = "android")] extern crate egl;
extern crate euclid;
extern crate gleam;
extern crate glutin;
extern crate layers;
#[macro_use] extern crate log;
extern crate msg;
extern crate net_traits;
extern crate script_traits;
extern crate style_traits;
extern crate url;
extern crate util;
#[cfg(target_os = "linux")] extern crate x11;

use compositing::windowing::WindowEvent;
use euclid::scale_factor::ScaleFactor;
use std::rc::Rc;
use util::opts;
use window::Window;

pub mod window;

pub type WindowID = glutin::WindowID;

pub trait NestedEventLoopListener {
    fn handle_event_from_nested_event_loop(&mut self, event: WindowEvent) -> bool;
}

pub fn create_window(parent: Option<WindowID>) -> Rc<Window> {
    // Read command-line options.
    let opts = opts::get();
    let foreground = opts.output_file.is_none() && !opts.headless;
    let scale_factor = ScaleFactor::new(opts.device_pixels_per_px.unwrap_or(1.0));
    let size_f32 = opts.initial_window_size.as_f32() * scale_factor;
    let size_u32 = size_f32.as_uint().cast().expect("Window size should fit in a u32.");

    // Open a window.
    Window::new(foreground, size_u32, parent)
}
