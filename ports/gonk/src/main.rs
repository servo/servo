/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![comment = "The Servo Parallel Browser Project"]
#![license = "MPL"]

#![deny(unused_imports)]
#![deny(unused_variables)]

#![allow(non_snake_case)]

extern crate servo;
extern crate native;
extern crate time;
extern crate "util" as servo_util;

extern crate compositing;

extern crate geom;
extern crate libc;
extern crate msg;
extern crate gleam;
extern crate layers;
extern crate egl;

use servo_util::opts;
use servo_util::rtinstrument;
use servo::Browser;
use compositing::windowing::IdleWindowEvent;
use compositing::windowing::InitializeCompositingWindowEvent;

use std::os;

mod window;
mod input;

struct BrowserWrapper {
    browser: Browser<window::Window>,
}

#[start]
#[allow(dead_code)]
fn start(argc: int, argv: *const *const u8) -> int {
    native::start(argc, argv, proc() {
        if opts::from_cmdline_args(os::args().as_slice()) {
            let window = if opts::get().headless {
                None
            } else {
                Some(window::Window::new())
            };

            let mut browser = BrowserWrapper {
                browser: Browser::new(window.clone()),
            };

            match window {
                None => (),
                Some(ref window) => input::run_input_loop(&window.event_send)
            }

            browser.browser.handle_event(InitializeCompositingWindowEvent);

            loop {
                let should_continue = match window {
                    None => browser.browser.handle_event(IdleWindowEvent),
                    Some(ref window) => {
                        match window.wait_events() {
                            Some(evt) => browser.browser.handle_event(evt),
                            None => browser.browser.handle_event(IdleWindowEvent),
                        }
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

            rtinstrument::teardown();
        }
    })
}

