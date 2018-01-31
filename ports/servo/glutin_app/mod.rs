/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A simple application that uses glutin to open a window for Servo to display in.

pub mod window;

use compositing::windowing::WindowEvent;
use servo_config::opts;
use std::rc::Rc;

pub trait NestedEventLoopListener {
    fn handle_event_from_nested_event_loop(&mut self, event: WindowEvent) -> bool;
}

pub fn create_window() -> Rc<window::Window> {
    // Read command-line options.
    let opts = opts::get();
    let foreground = opts.output_file.is_none() && !opts.headless;

    // Open a window.
    window::Window::new(foreground, opts.initial_window_size)
}
