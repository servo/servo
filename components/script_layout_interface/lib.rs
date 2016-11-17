/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! This module contains traits in script used generically in the rest of Servo.
//! The traits are here instead of in script so that these modules won't have
//! to depend on script.

#![deny(unsafe_code)]
#![feature(box_syntax)]
#![feature(nonzero)]
#![feature(plugin)]
#![feature(proc_macro)]
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
#[macro_use] extern crate heapsize_derive;
#[macro_use] extern crate html5ever_atoms;
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
#[macro_use] extern crate servo_atoms;
extern crate servo_url;
extern crate style;

pub mod message;
pub mod reporter;
pub mod rpc;
pub mod wrapper_traits;

use canvas_traits::CanvasMsg;
use core::nonzero::NonZero;
use ipc_channel::ipc::IpcSender;
use libc::c_void;
use std::sync::atomic::AtomicIsize;
use style::atomic_refcell::AtomicRefCell;
use style::data::ElementData;
use style::dom::TRestyleDamage;
use style::selector_impl::RestyleDamage;

pub struct PartialPersistentLayoutData {
    /// Data that the style system associates with a node. When the
    /// style system is being used standalone, this is all that hangs
    /// off the node. This must be first to permit the various
    /// transmutations between ElementData and PersistentLayoutData.
    pub style_data: ElementData,

    /// Description of how to account for recent style changes.
    pub restyle_damage: RestyleDamage,

    /// Information needed during parallel traversals.
    pub parallel: DomParallelInfo,
}

impl PartialPersistentLayoutData {
    pub fn new() -> Self {
        PartialPersistentLayoutData {
            style_data: ElementData::new(),
            // FIXME(bholley): This is needed for now to make sure we do frame
            // construction after initial styling. This will go away shortly when
            // we move restyle damage into the style system.
            restyle_damage: RestyleDamage::rebuild_and_reflow(),
            parallel: DomParallelInfo::new(),
        }
    }
}

#[derive(Copy, Clone, HeapSizeOf)]
pub struct OpaqueStyleAndLayoutData {
    #[ignore_heap_size_of = "TODO(#6910) Box value that should be counted but \
                             the type lives in layout"]
    pub ptr: NonZero<*mut AtomicRefCell<PartialPersistentLayoutData>>
}

#[allow(unsafe_code)]
unsafe impl Send for OpaqueStyleAndLayoutData {}

/// Information that we need stored in each DOM node.
#[derive(HeapSizeOf)]
pub struct DomParallelInfo {
    /// The number of children remaining to process during bottom-up traversal.
    pub children_to_process: AtomicIsize,
}

impl DomParallelInfo {
    pub fn new() -> DomParallelInfo {
        DomParallelInfo {
            children_to_process: AtomicIsize::new(0),
        }
    }
}


#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum LayoutNodeType {
    Element(LayoutElementType),
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
    SVGSVGElement,
}

pub struct HTMLCanvasData {
    pub ipc_renderer: Option<IpcSender<CanvasMsg>>,
    pub width: u32,
    pub height: u32,
}

pub struct SVGSVGData {
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
