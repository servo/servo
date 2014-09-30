/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![comment = "The Servo Parallel Browser Project"]
#![license = "MPL"]

#![deny(unused_imports, unused_variable)]

extern crate servo;
extern crate native;
extern crate "util" as servo_util;

#[cfg(not(test),not(target_os="android"))]
extern crate glfw_app;

#[cfg(not(test),not(target_os="android"))]
use servo_util::opts;

#[cfg(not(test),not(target_os="android"))]
use servo::run;

#[cfg(not(test),not(target_os="android"))]
use std::os;

#[cfg(not(test), not(target_os="android"))]
#[start]
#[allow(dead_code)]
fn start(argc: int, argv: *const *const u8) -> int {
    native::start(argc, argv, proc() {
        opts::from_cmdline_args(os::args().as_slice()).map(|opts| {
            let window = if opts.headless {
                None
            } else {
                Some(glfw_app::create_window(&opts))
            };
            run(opts, window);
        });
    })
}

#[cfg(not(test), target_os="android")]
fn main() {}
