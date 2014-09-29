/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A simple application that uses GLUT to open a window for Servo to display in.

#![license = "MPL"]
#![feature(macro_rules, phase)]
#![deny(unused_imports, unused_variable)]

extern crate alert;
extern crate compositing;
extern crate geom;
extern crate glut;
extern crate layers;
extern crate libc;
#[phase(plugin,link)]
extern crate log;
extern crate msg;
extern crate util;

use geom::scale_factor::ScaleFactor;
use std::rc::Rc;
use window::Window;

use glut::glut::{init, init_display_mode, DOUBLE};

mod window;

pub fn create_window(opts: &util::opts::Opts) -> Rc<Window> {
    // Initialize GLUT.
    init();
    init_display_mode(DOUBLE);

    // Read command-line options.
    let scale_factor = opts.device_pixels_per_px.unwrap_or(ScaleFactor(1.0));
    let size = opts.initial_window_size.as_f32() * scale_factor;

    // Open a window.
    Window::new(size.as_uint())
}
