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

#[cfg(not(target_os = "android"))]
include!("main2.rs");

#[cfg(target_os = "android")]
pub fn main() {
    println!(
        "Cannot start /ports/servo/ on Android. \
         Use /support/android/apk/ + /ports/libsimpleservo/ instead"
    );
}
