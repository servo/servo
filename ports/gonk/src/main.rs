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

extern crate servo;
extern crate time;
extern crate util;
extern crate errno;

extern crate compositing;
extern crate script_traits;

extern crate geom;
extern crate libc;
extern crate msg;
extern crate gleam;
extern crate layers;
extern crate egl;
extern crate url;
extern crate net;

use util::opts;
use net::resource_task;
use servo::Browser;
use compositing::windowing::WindowEvent;

use std::env;

mod window;
mod input;

struct BrowserWrapper {
    browser: Browser,
}

fn main() {
    // Parse the command line options and store them globally
    if opts::from_cmdline_args(env::args().collect::<Vec<_>>().as_slice()) {
        resource_task::global_init();

        let window = if opts::get().headless {
            None
        } else {
            Some(window::Window::new())
        };

        // Our wrapper around `Browser` that also implements some
        // callbacks required by the glutin window implementation.
        let mut browser = BrowserWrapper {
            browser: Browser::new(window.clone()),
        };

        match window {
            None => (),
            Some(ref window) => input::run_input_loop(&window.event_send)
        }

        browser.browser.handle_event(WindowEvent::InitializeCompositing);

        // Feed events from the window to the browser until the browser
        // says to stop.
        loop {
            let should_continue = match window {
                None => browser.browser.handle_event(WindowEvent::Idle),
                Some(ref window) => {
                    let event = window.wait_events();
                    browser.browser.handle_event(event)
                }
            };
            if !should_continue {
                break
            }
        }

        let BrowserWrapper {
            browser
        } = browser;
        browser.shutdown();
    }
}

