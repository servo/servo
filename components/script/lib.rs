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

#[macro_use]
mod tasks;

pub(crate) mod conversions;
mod css;
mod fetch;
#[macro_use]
mod dom;
pub(crate) use dom::canvas_context;
mod drag;
mod engine;
mod event_loop;
mod runtime;
mod url;

pub mod layout_dom;
pub(crate) mod messaging;
pub(crate) mod mime;
pub(crate) mod modules;
mod navigation;
mod realms;
mod routed_promise;
pub mod test;
mod unminify;
mod window_named_properties;
mod xpath;

pub use event_loop::script_thread::ScriptThread;
pub(crate) use script_bindings::DomTypes;
pub(crate) use script_bindings::reflector::{AssociatedMemory, DomObject, MutDomObject, Reflector};

pub(crate) use crate::dom::bindings::codegen::DomTypeHolder::DomTypeHolder;
// These trait exports are public, because they are used in the DOM bindings.
// Since they are used in derive macros,
// it is useful that they are accessible at the root of the crate.
pub(crate) use crate::dom::bindings::inheritance::HasParent;
pub(crate) use crate::dom::bindings::trace::{CustomTraceable, JSTraceable};
pub use crate::dom::serviceworker::serviceworker_manager::ServiceWorkerManager;
pub use crate::engine::handle::JSEngineSetup;
pub use crate::engine::init::init;
