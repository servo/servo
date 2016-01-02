/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![deny(unused_imports)]
#![deny(unused_variables)]

#![feature(box_syntax)]
#![feature(convert)]
// For FFI
#![allow(non_snake_case, dead_code)]

//! The `servo` test application.
//!
//! Creates a `Browser` instance with a simple implementation of
//! the compositor's `WindowMethods` to create a working web browser.
//!
//! This browser's implementation of `WindowMethods` is built on top
//! of [glutin], the cross-platform OpenGL utility and windowing
//! library.
//!
//! For the engine itself look next door in lib.rs.
//!
//! [glutin]: https://github.com/tomaka/glutin

extern crate compositing;
extern crate egl;
extern crate env_logger;
extern crate errno;
extern crate euclid;
extern crate gleam;
extern crate layers;
extern crate libc;
extern crate msg;
extern crate net_traits;
extern crate script_traits;
extern crate servo;
extern crate style_traits;
extern crate time;
extern crate url;
extern crate util;

#[link(name = "stlport")]
extern {}

use compositing::windowing::WindowEvent;
use servo::Browser;
use std::env;
use util::opts;

mod input;
mod window;

struct BrowserWrapper {
    browser: Browser,
}

fn main() {
    env_logger::init().unwrap();

    // Parse the command line options and store them globally
    opts::from_cmdline_args(env::args().collect::<Vec<_>>().as_slice());

    let window = window::Window::new();

    // Our wrapper around `Browser` that also implements some
    // callbacks required by the glutin window implementation.
    let mut browser = BrowserWrapper {
        browser: Browser::new(window.clone()),
    };

    input::run_input_loop(&window.event_send);

    browser.browser.handle_events(vec![WindowEvent::InitializeCompositing]);

    // Feed events from the window to the browser until the browser
    // says to stop.
    loop {
        let events = window.wait_events();
        let should_continue = browser.browser.handle_events(events);
        if !should_continue {
            break
        }
    }
}
