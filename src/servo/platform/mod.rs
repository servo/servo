/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Platform-specific functionality for Servo.

#[cfg(not(shared_gl_windowing))]
pub use platform::common::glut_windowing::{Application, Window};
#[cfg(shared_gl_windowing)]
pub use platform::common::shared_gl_windowing::{Application, Window};

pub mod common {
    #[cfg(not(shared_gl_windowing))]
    pub mod glut_windowing;
    #[cfg(shared_gl_windowing)]
    pub mod shared_gl_windowing;
}

