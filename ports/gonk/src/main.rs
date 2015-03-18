/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![deny(unused_imports)]
#![deny(unused_variables)]

#![feature(int_uint)]
#![feature(core, os, path, io, std_misc, env)]
// For FFI
#![allow(non_snake_case, dead_code)]

extern crate servo;
extern crate time;
extern crate util;

extern crate compositing;

extern crate geom;
extern crate libc;
extern crate msg;
extern crate gleam;
extern crate layers;
extern crate egl;
extern crate url;

use util::opts;
use servo::Browser;
use compositing::windowing::WindowEvent;

use std::env;

mod window;
mod input;

struct BrowserWrapper {
    browser: Browser,
}

fn main() {
    if opts::from_cmdline_args(env::args().collect::<Vec<_>>().as_slice()) {
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

        browser.browser.handle_event(WindowEvent::InitializeCompositing);

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

