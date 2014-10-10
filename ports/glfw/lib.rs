/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A simple application that uses GLFW to open a window for Servo to display in.

#![license = "MPL"]
#![feature(macro_rules)]
#![deny(unused_imports, unused_variable)]

extern crate alert;
extern crate compositing;
extern crate geom;
extern crate glfw;
extern crate layers;
extern crate libc;
extern crate msg;
extern crate time;
extern crate util;

use geom::scale_factor::ScaleFactor;
use std::rc::Rc;
use window::Window;

mod window;

pub fn create_window(opts: &util::opts::Opts) -> Rc<Window> {
    // Initialize GLFW.
    let glfw = glfw::init(glfw::LOG_ERRORS).unwrap_or_else(|_| {
        // handles things like inability to connect to X
        // cannot simply fail, since the runtime isn't up yet (causes a nasty abort)
        println!("GLFW initialization failed");
        unsafe { libc::exit(1); }
    });

    // Read command-line options.
    let foreground = opts.output_file.is_none();
    let scale_factor = opts.device_pixels_per_px.unwrap_or(ScaleFactor(1.0));
    let size = opts.initial_window_size.as_f32() * scale_factor;

    // Open a window.
    Window::new(glfw, foreground, size.as_uint())
}
