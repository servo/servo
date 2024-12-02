/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

/// Convenient type alias of Result type for Servo.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors returned by Servo.
#[non_exhaustive]
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// The error type for when the OS cannot perform the requested operation.
    #[error(transparent)]
    OsError(#[from] winit::error::OsError),
    /// A general error that may occur while running the Winit event loop.
    #[error(transparent)]
    EventLoopError(#[from] winit::error::EventLoopError),
    /// Glutin errors.
    #[error(transparent)]
    GlutinError(#[from] glutin::error::Error),
}
