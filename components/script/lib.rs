/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![cfg_attr(crown, feature(register_tool))]
#![deny(unsafe_code)]
#![doc = "The script crate contains all matters DOM."]
// Register the linter `crown`, which is the Servo-specific linter for the script crate.
#![cfg_attr(crown, register_tool(crown))]

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
extern crate stylo_atoms;

mod animations;
#[macro_use]
mod tasks;

pub(crate) mod conversions;
mod css;
mod devtools;
mod fetch;
#[macro_use]
mod dom;
pub(crate) use dom::canvas_context;
mod drag;
mod event_loop;
pub(crate) mod indexeddb;
mod init;
mod url;

pub mod layout_dom;
mod links;
pub(crate) mod messaging;
mod microtask;
pub(crate) mod mime;
pub(crate) mod modules;
mod navigation;
mod realms;
mod routed_promise;
pub(crate) mod script_runtime;
pub(crate) mod serviceworker_manager;
pub mod test;
mod timers;
mod unminify;
mod webdriver_handlers;
mod window_named_properties;
mod xpath;

pub use event_loop::script_thread::ScriptThread;
pub use init::init;
pub(crate) use script_bindings::DomTypes;
pub(crate) use script_bindings::reflector::{AssociatedMemory, DomObject, MutDomObject, Reflector};
pub use script_runtime::JSEngineSetup;
pub use serviceworker_manager::ServiceWorkerManager;

pub(crate) use crate::dom::bindings::codegen::DomTypeHolder::DomTypeHolder;
// These trait exports are public, because they are used in the DOM bindings.
// Since they are used in derive macros,
// it is useful that they are accessible at the root of the crate.
pub(crate) use crate::dom::bindings::inheritance::HasParent;
pub(crate) use crate::dom::bindings::trace::{CustomTraceable, JSTraceable};
