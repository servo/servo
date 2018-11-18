/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![cfg_attr(feature = "unstable", feature(core_intrinsics))]
#![cfg_attr(feature = "unstable", feature(on_unimplemented))]
#![feature(const_fn)]
#![feature(drain_filter)]
#![feature(plugin)]
#![feature(try_from)]
#![deny(unsafe_code)]
#![allow(non_snake_case)]
#![doc = "The script crate contains all matters DOM."]
#![plugin(script_plugins)]
#![cfg_attr(
    not(feature = "unrooted_must_root_lint"),
    allow(unknown_lints)
)]

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
extern crate enum_iterator;
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
pub mod fetch;
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
mod task_manager;
mod task_queue;
mod task_source;
pub mod test;
pub mod textinput;
mod timers;
mod unpremultiplytable;
mod webdriver_handlers;

/// A module with everything layout can use from script.
///
/// Try to keep this small!
///
/// TODO(emilio): A few of the FooHelpers can go away, presumably...
pub mod layout_exports {
    pub use crate::dom::bindings::inheritance::{CharacterDataTypeId, ElementTypeId};
    pub use crate::dom::bindings::inheritance::{HTMLElementTypeId, NodeTypeId};
    pub use crate::dom::bindings::root::LayoutDom;
    pub use crate::dom::characterdata::LayoutCharacterDataHelpers;
    pub use crate::dom::document::{Document, LayoutDocumentHelpers, PendingRestyle};
    pub use crate::dom::element::{Element, LayoutElementHelpers, RawLayoutElementHelpers};
    pub use crate::dom::node::NodeFlags;
    pub use crate::dom::node::{LayoutNodeHelpers, Node};
    pub use crate::dom::text::Text;
}

use crate::dom::bindings::codegen::RegisterBindings;
use crate::dom::bindings::proxyhandler;
use crate::serviceworker_manager::ServiceWorkerManager;
use script_traits::SWManagerSenders;

#[cfg(target_os = "linux")]
#[allow(unsafe_code)]
fn perform_platform_specific_initialization() {
    use std::mem;
    // 4096 is default max on many linux systems
    const MAX_FILE_LIMIT: libc::rlim_t = 4096;

    // Bump up our number of file descriptors to save us from impending doom caused by an onslaught
    // of iframes.
    unsafe {
        let mut rlim: libc::rlimit = mem::uninitialized();
        match libc::getrlimit(libc::RLIMIT_NOFILE, &mut rlim) {
            0 => {
                if rlim.rlim_cur >= MAX_FILE_LIMIT {
                    // we have more than enough
                    return;
                }

                rlim.rlim_cur = match rlim.rlim_max {
                    libc::RLIM_INFINITY => MAX_FILE_LIMIT,
                    _ => {
                        if rlim.rlim_max < MAX_FILE_LIMIT {
                            rlim.rlim_max
                        } else {
                            MAX_FILE_LIMIT
                        }
                    },
                };
                match libc::setrlimit(libc::RLIMIT_NOFILE, &rlim) {
                    0 => (),
                    _ => warn!("Failed to set file count limit"),
                };
            },
            _ => warn!("Failed to get file count limit"),
        };
    }
}

#[cfg(not(target_os = "linux"))]
fn perform_platform_specific_initialization() {}

pub fn init_service_workers(sw_senders: SWManagerSenders) {
    // Spawn the service worker manager passing the constellation sender
    ServiceWorkerManager::spawn_manager(sw_senders);
}

#[allow(unsafe_code)]
pub fn init() {
    unsafe {
        proxyhandler::init();

        // Create the global vtables used by the (generated) DOM
        // bindings to implement JS proxies.
        RegisterBindings::RegisterProxyHandlers();
    }

    perform_platform_specific_initialization();
}
