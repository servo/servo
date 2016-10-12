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
extern crate gfx_traits;
extern crate gleam;
extern crate glutin;
#[macro_use] extern crate log;
extern crate msg;
extern crate net_traits;
#[cfg(any(target_os = "linux", target_os = "macos"))] extern crate osmesa_sys;
extern crate script_traits;
extern crate style_traits;
extern crate url;
extern crate util;
#[cfg(target_os = "windows")] extern crate winapi;
#[cfg(target_os = "windows")] extern crate user32;
#[cfg(target_os = "windows")] extern crate gdi32;

use compositing::windowing::WindowEvent;
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

    // Open a window.
    Window::new(foreground, opts.initial_window_size, parent)
}
