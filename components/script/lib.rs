/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(box_syntax)]
#![feature(conservative_impl_trait)]
#![feature(const_fn)]
#![feature(core_intrinsics)]
#![feature(mpsc_select)]
#![feature(nonzero)]
#![feature(on_unimplemented)]
#![feature(option_entry)]
#![feature(plugin)]
#![feature(proc_macro)]
#![feature(stmt_expr_attributes)]
#![feature(try_from)]
#![feature(unboxed_closures)]
#![feature(untagged_unions)]

#![deny(unsafe_code)]
#![allow(non_snake_case)]

#![doc = "The script crate contains all matters DOM."]

#![plugin(script_plugins)]

extern crate angle;
extern crate app_units;
extern crate audio_video_metadata;
extern crate base64;
#[macro_use]
extern crate bitflags;
extern crate bluetooth_traits;
extern crate byteorder;
extern crate canvas_traits;
extern crate caseless;
extern crate cookie as cookie_rs;
extern crate core;
#[macro_use] extern crate cssparser;
#[macro_use] extern crate deny_public_fields;
extern crate devtools_traits;
extern crate dom_struct;
#[macro_use]
extern crate domobject_derive;
extern crate encoding;
extern crate euclid;
extern crate fnv;
extern crate gleam;
extern crate half;
#[macro_use] extern crate heapsize;
#[macro_use] extern crate heapsize_derive;
#[macro_use] extern crate html5ever;
#[macro_use]
extern crate hyper;
extern crate hyper_serde;
extern crate image;
extern crate ipc_channel;
#[macro_use]
extern crate js;
#[macro_use]
extern crate jstraceable_derive;
#[macro_use]
extern crate lazy_static;
extern crate libc;
#[macro_use]
extern crate log;
#[macro_use]
extern crate mime;
extern crate mime_guess;
extern crate msg;
extern crate net_traits;
extern crate num_traits;
extern crate offscreen_gl_context;
extern crate open;
extern crate parking_lot;
extern crate phf;
#[macro_use]
extern crate profile_traits;
extern crate ref_filter_map;
extern crate ref_slice;
extern crate regex;
extern crate script_layout_interface;
extern crate script_traits;
extern crate selectors;
extern crate serde;
extern crate servo_arc;
#[macro_use] extern crate servo_atoms;
extern crate servo_config;
extern crate servo_geometry;
extern crate servo_rand;
extern crate servo_url;
extern crate smallvec;
#[macro_use]
extern crate style;
extern crate style_traits;
extern crate swapper;
extern crate time;
#[cfg(any(target_os = "macos", target_os = "linux", target_os = "windows"))]
extern crate tinyfiledialogs;
extern crate unicode_segmentation;
extern crate url;
extern crate utf8;
extern crate uuid;
extern crate webrender_api;
extern crate webvr_traits;
extern crate xml5ever;

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
    pub use dom::bindings::inheritance::{CharacterDataTypeId, ElementTypeId};
    pub use dom::bindings::inheritance::{HTMLElementTypeId, NodeTypeId};
    pub use dom::bindings::js::LayoutJS;
    pub use dom::characterdata::LayoutCharacterDataHelpers;
    pub use dom::document::{Document, LayoutDocumentHelpers, PendingRestyle};
    pub use dom::element::{Element, LayoutElementHelpers, RawLayoutElementHelpers};
    pub use dom::node::{CAN_BE_FRAGMENTED, DIRTY_ON_VIEWPORT_SIZE_CHANGE, HAS_DIRTY_DESCENDANTS, IS_IN_DOC};
    pub use dom::node::{HANDLED_SNAPSHOT, HAS_SNAPSHOT};
    pub use dom::node::{LayoutNodeHelpers, Node};
    pub use dom::text::Text;
}

use dom::bindings::codegen::RegisterBindings;
use dom::bindings::proxyhandler;
use script_traits::SWManagerSenders;
use serviceworker_manager::ServiceWorkerManager;

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
                    }
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
