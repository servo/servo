/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![feature(const_fn)]
#![feature(core_intrinsics)]
#![feature(drain_filter)]
#![feature(inner_deref)]
#![feature(on_unimplemented)]
#![feature(plugin)]
#![deny(unsafe_code)]
#![allow(non_snake_case)]
#![doc = "The script crate contains all matters DOM."]
#![plugin(script_plugins)]
#![cfg_attr(not(feature = "unrooted_must_root_lint"), allow(unknown_lints))]

#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate crossbeam_channel;
#[macro_use]
extern crate cssparser;
#[macro_use]
extern crate deny_public_fields;
#[macro_use]
extern crate domobject_derive;
#[macro_use]
extern crate html5ever;
#[macro_use]
extern crate js;
#[macro_use]
extern crate jstraceable_derive;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
#[macro_use]
extern crate malloc_size_of;
#[macro_use]
extern crate malloc_size_of_derive;
#[macro_use]
extern crate profile_traits;
#[macro_use]
extern crate serde;
#[macro_use]
extern crate servo_atoms;
#[macro_use]
extern crate style;

#[macro_use]
mod task;
mod body;
pub mod clipboard_provider;
mod devtools;
pub mod document_loader;
#[macro_use]
mod dom;
mod canvas_state;
mod compartments;
pub mod fetch;
mod image_listener;
mod init;
mod layout_image;
mod mem;
mod microtask;
mod network_listener;
pub mod script_runtime;
#[allow(unsafe_code)]
pub mod script_thread;
mod serviceworker_manager;
mod serviceworkerjob;
mod stylesheet_loader;
mod stylesheet_set;
mod task_manager;
mod task_queue;
mod task_source;
pub mod test;
pub mod textinput;
mod timers;
mod unpremultiplytable;
mod webdriver_handlers;

pub use init::{init, init_service_workers};

/// A module with everything layout can use from script.
///
/// Try to keep this small!
///
/// TODO(emilio): A few of the FooHelpers can go away, presumably...
pub mod layout_exports {
    pub use crate::dom::bindings::inheritance::{
        CharacterDataTypeId, DocumentFragmentTypeId, ElementTypeId,
    };
    pub use crate::dom::bindings::inheritance::{HTMLElementTypeId, NodeTypeId, TextTypeId};
    pub use crate::dom::bindings::root::LayoutDom;
    pub use crate::dom::characterdata::LayoutCharacterDataHelpers;
    pub use crate::dom::document::{Document, LayoutDocumentHelpers, PendingRestyle};
    pub use crate::dom::element::{Element, LayoutElementHelpers, RawLayoutElementHelpers};
    pub use crate::dom::node::NodeFlags;
    pub use crate::dom::node::{LayoutNodeHelpers, Node};
    pub use crate::dom::shadowroot::{LayoutShadowRootHelpers, ShadowRoot};
    pub use crate::dom::text::Text;
}
