/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A simple application that uses GLUT to open a window for Servo to display in.

#![license = "MPL"]
#![feature(macro_rules, phase)]
#![deny(unused_imports)]
#![deny(unused_variables)]

extern crate compositing;
extern crate egl;
extern crate geom;
extern crate glut;
extern crate layers;
extern crate libc;
#[phase(plugin, link)] extern crate log;
extern crate msg;
extern crate native;
extern crate servo;
#[phase(plugin, link)] extern crate util;

use geom::scale_factor::ScaleFactor;
use std::rc::Rc;
use std::string;
use util::opts;
use window::Window;

use glut::glut::{init, init_display_mode, DOUBLE};

mod window;

pub fn create_window() -> Rc<Window> {
    // Initialize GLUT.
    init();
    init_display_mode(DOUBLE);

    // Read command-line options.
    let scale_factor = opts::get().device_pixels_per_px.unwrap_or(ScaleFactor(1.0));
    let size = opts::get().initial_window_size.as_f32() * scale_factor;

    // Open a window.
    Window::new(size.as_uint())
}

#[no_mangle]
#[allow(dead_code)]
pub extern "C" fn android_start(argc: int, argv: *const *const u8) -> int {
    native::start(argc, argv, proc() {
        let mut args: Vec<String> = vec!();
        for i in range(0u, argc as uint) {
            unsafe {
                args.push(string::raw::from_buf(*argv.offset(i as int) as *const u8));
            }
        }

        if opts::from_cmdline_args(args.as_slice()) {
            let window = create_window();
            let mut browser = servo::Browser::new(Some(window.clone()));
            while browser.handle_event(window.wait_events()) {}
            browser.shutdown();
        }
    })
}
