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

fn main() {
    cfg_if::cfg_if! {
        if #[cfg(not(any(target_os = "android", target_env = "ohos")))] {
            servoshell::main()
        } else {
            println!(
                "Cannot run the servoshell `bin` executable on platforms such as \
                 Android or OpenHarmony. On these platforms you need to compile \
                 the servoshell library as a `cdylib` and integrate it with the \
                 platform app code into an `apk` (android) or `hap` (OpenHarmony).\
                 For Android `mach build` will do these steps automatically for you."
            );
        }
    }
}
