/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![feature(register_tool)]
#![deny(unsafe_code)]
#![doc = "The script crate contains all matters DOM."]
// Register the linter `crown`, which is the Servo-specific linter for the script
// crate. Issue a warning if `crown` is not being used to compile, but not when
// building rustdoc or running clippy.
#![register_tool(crown)]
#![cfg_attr(any(doc, clippy), allow(unknown_lints))]
#![deny(crown_is_not_used)]

// These are used a lot so let's keep them for now
#[macro_use]
extern crate js;
#[macro_use]
extern crate jstraceable_derive;
#[macro_use]
extern crate log;
#[macro_use]
extern crate malloc_size_of_derive;
#[macro_use]
extern crate servo_atoms;

mod animation_timeline;
mod animations;
#[warn(deprecated)]
#[macro_use]
mod task;
#[warn(deprecated)]
mod body;
#[warn(deprecated)]
pub mod clipboard_provider;
#[warn(deprecated)]
mod devtools;
#[warn(deprecated)]
pub mod document_loader;
#[warn(deprecated)]
#[macro_use]
mod dom;
#[warn(deprecated)]
mod canvas_state;
#[warn(deprecated)]
pub mod fetch;
#[warn(deprecated)]
mod image_listener;
#[warn(deprecated)]
mod init;
#[warn(deprecated)]
mod layout_image;

pub mod layout_dom;
#[warn(deprecated)]
mod mem;
#[warn(deprecated)]
mod microtask;
#[warn(deprecated)]
mod network_listener;
#[warn(deprecated)]
mod realms;
#[warn(deprecated)]
mod script_module;
#[warn(deprecated)]
pub mod script_runtime;
#[warn(deprecated)]
#[allow(unsafe_code)]
pub mod script_thread;
#[warn(deprecated)]
pub mod serviceworker_manager;
#[warn(deprecated)]
mod stylesheet_loader;
#[warn(deprecated)]
mod stylesheet_set;
#[warn(deprecated)]
mod task_manager;
#[warn(deprecated)]
mod task_queue;
#[warn(deprecated)]
mod task_source;
#[warn(deprecated)]
pub mod test;
#[warn(deprecated)]
pub mod textinput;
#[warn(deprecated)]
mod timers;
#[warn(deprecated)]
mod unpremultiplytable;
#[warn(deprecated)]
mod webdriver_handlers;
#[warn(deprecated)]
mod window_named_properties;

pub use init::init;
pub use script_runtime::JSEngineSetup;
