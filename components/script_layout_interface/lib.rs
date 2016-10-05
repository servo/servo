/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! This module contains traits in script used generically in the rest of Servo.
//! The traits are here instead of in script so that these modules won't have
//! to depend on script.

#![deny(unsafe_code)]
#![feature(box_syntax)]
#![feature(custom_attribute)]
#![feature(custom_derive)]
#![feature(nonzero)]
#![feature(plugin)]
#![plugin(heapsize_plugin)]
#![plugin(plugins)]

extern crate app_units;
#[allow(unused_extern_crates)]
#[macro_use]
extern crate bitflags;
extern crate canvas_traits;
extern crate core;
extern crate cssparser;
extern crate euclid;
extern crate gfx_traits;
extern crate heapsize;
extern crate ipc_channel;
extern crate libc;
#[macro_use]
extern crate log;
extern crate msg;
extern crate net_traits;
extern crate profile_traits;
extern crate range;
extern crate script_traits;
extern crate selectors;
#[macro_use(atom, ns)]
extern crate string_cache;
extern crate style;
extern crate url;
extern crate util;

pub mod message;
pub mod reporter;
pub mod restyle_damage;
pub mod rpc;
pub mod wrapper_traits;

use canvas_traits::CanvasMsg;
use core::nonzero::NonZero;
use ipc_channel::ipc::IpcSender;
use libc::c_void;
use restyle_damage::RestyleDamage;
use style::atomic_refcell::AtomicRefCell;
use style::data::PersistentStyleData;

pub struct PartialPersistentLayoutData {
    pub style_data: PersistentStyleData,
    pub restyle_damage: RestyleDamage,
}

#[derive(Copy, Clone, HeapSizeOf)]
pub struct OpaqueStyleAndLayoutData {
    #[ignore_heap_size_of = "TODO(#6910) Box value that should be counted but \
                             the type lives in layout"]
    pub ptr: NonZero<*mut AtomicRefCell<PartialPersistentLayoutData>>
}

#[allow(unsafe_code)]
unsafe impl Send for OpaqueStyleAndLayoutData {}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum LayoutNodeType {
    Comment,
    Document,
    DocumentFragment,
    DocumentType,
    Element(LayoutElementType),
    ProcessingInstruction,
    Text,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum LayoutElementType {
    Element,
    HTMLCanvasElement,
    HTMLIFrameElement,
    HTMLImageElement,
    HTMLInputElement,
    HTMLObjectElement,
    HTMLTableCellElement,
    HTMLTableColElement,
    HTMLTableElement,
    HTMLTableRowElement,
    HTMLTableSectionElement,
    HTMLTextAreaElement,
}

pub struct HTMLCanvasData {
    pub ipc_renderer: Option<IpcSender<CanvasMsg>>,
    pub width: u32,
    pub height: u32,
}

/// The address of a node known to be valid. These are sent from script to layout.
#[derive(Clone, PartialEq, Eq, Copy)]
pub struct TrustedNodeAddress(pub *const c_void);

#[allow(unsafe_code)]
unsafe impl Send for TrustedNodeAddress {}

pub fn is_image_data(uri: &str) -> bool {
    static TYPES: &'static [&'static str] = &["data:image/png", "data:image/gif", "data:image/jpeg"];
    TYPES.iter().any(|&type_| uri.starts_with(type_))
}
