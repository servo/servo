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
extern crate core;
extern crate heapsize;
extern crate style;

pub mod restyle_damage;

use core::nonzero::NonZero;
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
pub enum NodeType {
    Comment,
    Document,
    DocumentFragment,
    DocumentType,
    Element(ElementType),
    ProcessingInstruction,
    Text,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum ElementType {
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
