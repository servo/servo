/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The `servo` test application.
//!
//! Creates a `Servo` instance with a simple implementation of
//! the compositor's `WindowMethods` to create a working web browser.
//!
//! This browser's implementation of `WindowMethods` is built on top
//! of [winit], the cross-platform windowing library.
//!
//! For the engine itself look next door in `components/servo/lib.rs`.
//!
//! [winit]: https://github.com/rust-windowing/winit

// Normally, rust uses the "Console" Windows subsystem, which pops up a console
// when running an application. Switching to the "Windows" subsystem prevents
// this, but also hides debugging output. Use the "Windows" console unless debug
// mode is turned on.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

cfg_if::cfg_if! {
    if #[cfg(not(target_os = "android"))] {
        #[cfg(any(target_os = "macos", target_os = "linux"))]
        #[macro_use]
        extern crate sig;

        #[cfg(test)]
        mod test;

        mod app;
        mod backtrace;
        mod browser;
        mod crash_handler;
        mod egui_glue;
        mod embedder;
        mod events_loop;
        mod headed_window;
        mod headless_window;
        mod keyutils;
        mod main2;
        mod minibrowser;
        mod parser;
        mod prefs;
        mod resources;
        mod window_trait;

        pub mod platform {
            #[cfg(target_os = "macos")]
            pub use crate::platform::macos::deinit;

            #[cfg(target_os = "macos")]
            pub mod macos;

            #[cfg(not(target_os = "macos"))]
            pub fn deinit(_clean_shutdown: bool) {}
        }

        pub fn main() {
            main2::main()
        }
    } else {
        pub fn main() {
            println!(
                "Cannot start /ports/servo/ on Android. \
                 Use /support/android/apk/ + /ports/libsimpleservo/ instead"
            );
        }
    }
}
