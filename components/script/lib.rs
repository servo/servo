/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![feature(assoc_char_funcs)]
#![feature(const_fn)]
#![feature(core_intrinsics)]
#![feature(drain_filter)]
#![feature(plugin)]
#![feature(register_tool)]
#![deny(unsafe_code)]
#![doc = "The script crate contains all matters DOM."]
#![cfg_attr(not(feature = "unrooted_must_root_lint"), allow(unknown_lints))]
#![allow(deprecated)] // FIXME: Can we make `allow` only apply to the `plugin` crate attribute?
#![plugin(script_plugins)]
#![register_tool(unrooted_must_root_lint)]

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
mod euclidext;
#[warn(deprecated)]
pub mod fetch;
#[warn(deprecated)]
mod image_listener;
#[warn(deprecated)]
mod init;
#[warn(deprecated)]
mod layout_image;
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

pub use init::init;
pub use script_runtime::JSEngineSetup;

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
    pub use crate::dom::document::{Document, LayoutDocumentHelpers};
    pub use crate::dom::element::{Element, LayoutElementHelpers};
    pub use crate::dom::node::NodeFlags;
    pub use crate::dom::node::{LayoutNodeHelpers, Node};
    pub use crate::dom::shadowroot::{LayoutShadowRootHelpers, ShadowRoot};
    pub use crate::dom::text::Text;
}
