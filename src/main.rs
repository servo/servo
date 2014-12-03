/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![comment = "The Servo Parallel Browser Project"]
#![license = "MPL"]

#![feature(phase)]

#![deny(unused_imports)]
#![deny(unused_variables)]

extern crate servo;
extern crate native;
extern crate time;
extern crate "util" as servo_util;

#[cfg(all(feature = "glutin",not(test)))]
extern crate "glutin_app" as app;
#[cfg(all(feature = "glfw_app",not(test)))]
extern crate "glfw_app" as app;

#[cfg(not(test))]
extern crate compositing;

#[cfg(target_os="android")]
#[phase(plugin, link)]
extern crate android_glue;

#[cfg(not(test))]
use servo_util::opts;

#[cfg(not(test))]
use servo_util::rtinstrument;

#[cfg(not(test))]
use servo::Browser;
#[cfg(not(test))]
use compositing::windowing::{IdleWindowEvent, ResizeWindowEvent, WindowEvent};

#[cfg(not(any(test,target_os="android")))]
use std::os;

#[cfg(not(test))]
struct BrowserWrapper {
    browser: Browser<app::window::Window>,
}

#[cfg(target_os="android")]
android_start!(main)

#[cfg(target_os="android")]
fn get_args() -> Vec<String> {
    vec![
        "servo".to_string(),
        "/sdcard/html/about-mozilla.html".to_string()
    ]
}

#[cfg(not(target_os="android"))]
fn get_args() -> Vec<String> {
    os::args()
}

fn main() {
    if opts::from_cmdline_args(get_args().as_slice()) {
        let window = if opts::get().headless {
            None
        } else {
            Some(app::create_window())
        };

        let mut browser = BrowserWrapper {
            browser: Browser::new(window.clone()),
        };

        match window {
            None => {}
            Some(ref window) => {
                unsafe {
                    window.set_nested_event_loop_listener(&mut browser);
                }
            }
        }

        loop {
            let should_continue = match window {
                None => browser.browser.handle_event(IdleWindowEvent),
                Some(ref window) => {
                    let event = window.wait_events();
                    browser.browser.handle_event(event)
                }
            };
            if !should_continue {
                break
            }
        }

        match window {
            None => {}
            Some(ref window) => {
                unsafe {
                    window.remove_nested_event_loop_listener();
                }
            }
        }

        let BrowserWrapper {
            browser
        } = browser;
        browser.shutdown();

        rtinstrument::teardown();
    }
}

impl app::NestedEventLoopListener for BrowserWrapper {
    fn handle_event_from_nested_event_loop(&mut self, event: WindowEvent) -> bool {
        let is_resize = match event {
            ResizeWindowEvent(..) => true,
            _ => false,
        };
        if !self.browser.handle_event(event) {
            return false
        }
        if is_resize {
            self.browser.repaint_synchronously()
        }
        true
    }
}

