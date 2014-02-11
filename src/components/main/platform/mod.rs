/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Platform-specific functionality for Servo.

#[cfg(target_os="android")]
pub use platform::common::glut_windowing::{Application, Window};
#[cfg(not(target_os="android"))]
pub use platform::common::glfw_windowing::{Application, Window};

pub mod common {
    #[cfg(target_os="android")]
    pub mod glut_windowing;
    #[cfg(not(target_os="android"))]
    pub mod glfw_windowing;
}

