/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! This module contains traits in script used generically in the rest of Servo.
//! The traits are here instead of in script so that these modules won't have
//! to depend on script.

#![deny(unsafe_code)]
#![feature(custom_attribute)]
#![feature(custom_derive)]
#![feature(nonzero)]
#![feature(plugin)]
#![plugin(heapsize_plugin)]
#![plugin(plugins)]

#[allow(unused_extern_crates)]
#[macro_use]
extern crate bitflags;
extern crate canvas_traits;
extern crate core;
extern crate gfx_traits;
extern crate heapsize;
extern crate ipc_channel;
extern crate msg;
extern crate range;
extern crate selectors;
#[macro_use(atom, ns)]
extern crate string_cache;
extern crate style;
extern crate url;

pub mod restyle_damage;
pub mod wrapper_traits;

use canvas_traits::CanvasMsg;
use core::nonzero::NonZero;
use ipc_channel::ipc::IpcSender;
use restyle_damage::RestyleDamage;
use std::cell::RefCell;
use style::servo::PrivateStyleData;

pub struct PartialStyleAndLayoutData {
    pub style_data: PrivateStyleData,
    pub restyle_damage: RestyleDamage,
}

#[derive(Copy, Clone, HeapSizeOf)]
pub struct OpaqueStyleAndLayoutData {
    #[ignore_heap_size_of = "TODO(#6910) Box value that should be counted but \
                             the type lives in layout"]
    pub ptr: NonZero<*mut RefCell<PartialStyleAndLayoutData>>
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
