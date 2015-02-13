/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A simple application that uses glutin to open a window for Servo to display in.

#![feature(int_uint)]
#![feature(core)]
#![cfg_attr(feature = "window", feature(hash))]
#![feature(box_syntax)]
#![feature(libc)]

#[macro_use] extern crate bitflags;
#[cfg(target_os="macos")]
extern crate cgl;
extern crate compositing;
extern crate geom;
extern crate gleam;
extern crate glutin;
extern crate layers;
extern crate libc;
extern crate msg;
extern crate time;
extern crate util;
extern crate egl;

use compositing::windowing::WindowEvent;
use geom::scale_factor::ScaleFactor;
use std::rc::Rc;
use window::Window;
use util::opts;

pub mod window;

pub trait NestedEventLoopListener {
    fn handle_event_from_nested_event_loop(&mut self, event: WindowEvent) -> bool;
}

pub fn create_window() -> Rc<Window> {
    // Read command-line options.
    let opts = opts::get();
    let foreground = opts.output_file.is_none();
    let scale_factor = opts.device_pixels_per_px.unwrap_or(ScaleFactor(1.0));
    let size = opts.initial_window_size.as_f32() * scale_factor;

    // Open a window.
    Window::new(foreground, size.as_uint().cast().unwrap())
}
