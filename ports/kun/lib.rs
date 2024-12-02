/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Kun is another Servo port which is focussed on building ergonomic embedding APIs. It has a few
//! design choices in hope for better embedding. While it doesn't provide many features yet,
//! it already contains some building blocks that could solve some common use case scenarios:
//!
//! - Multiple webview types like panel UI, context menu, and more.
//! - Basic multi-window / multi-webview support.
//! - Improve drawing order to overcome difficult rendering challenges like smoother resizing.
//!
//! We also hope it can be easy to start with for users. Right now it focuses on working well with
//! Winit primarily. See `main.rs` as an example of using it with the Winit event loop. You can
//! simple run it by `cargo run`. It's also to build if from mach by `./mach build --servo_kun`,
//! and then run it by `./mach run --bin PATH_TO_BBINARY`

#![deny(missing_docs)]

/// Compositor component to handle webrender.
pub mod compositor;
/// Utilities to read options and preferences.
pub mod config;
/// Error and result types.
pub mod errors;
/// Utilities to handle keyboard inputs and states.
pub mod keyboard;
/// Glutin rendering context.
pub mod rendering;
/// Main entry types and functions.
pub mod servo;
/// Utilities to handle touch inputs and states.
pub mod touch;
/// Web view types to handle web browsing contexts.
pub mod webview;
/// Window types to handle Winit's window.
pub mod window;
pub use errors::{Error, Result};
pub use servo::Servo;
/// Re-exporting Winit for the sake of convenience.
pub use winit;
/// Context menu types.
pub mod context_menu;
