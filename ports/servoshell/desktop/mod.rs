/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Contains files specific to the servoshell app for Desktop systems.

mod accelerated_gl_media;
pub(crate) mod app;
mod app_state;
pub(crate) mod cli;
mod dialog;
mod egui_glue;
mod embedder;
pub(crate) mod events_loop;
mod gamepad;
pub mod geometry;
mod headed_window;
mod headless_window;
mod keyutils;
mod minibrowser;
mod protocols;
mod tracing;
mod window_trait;
