/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! This module contains traits in script used generically in the rest of Servo.
//! The traits are here instead of in script so that these modules won't have
//! to depend on script.

#![deny(unsafe_code)]
#![feature(box_syntax)]
#![feature(nonzero)]

extern crate app_units;
extern crate atomic_refcell;
extern crate canvas_traits;
extern crate core;
extern crate cssparser;
extern crate euclid;
extern crate gfx_traits;
extern crate heapsize;
#[macro_use] extern crate heapsize_derive;
#[macro_use] extern crate html5ever;
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
extern crate servo_arc;
extern crate servo_atoms;
extern crate servo_url;
extern crate style;
extern crate webrender_api;

pub mod message;
pub mod reporter;
pub mod rpc;
pub mod wrapper_traits;

use atomic_refcell::AtomicRefCell;
use canvas_traits::CanvasMsg;
use core::nonzero::NonZero;
use ipc_channel::ipc::IpcSender;
use libc::c_void;
use net_traits::image_cache::PendingImageId;
use script_traits::UntrustedNodeAddress;
use servo_url::ServoUrl;
use std::sync::atomic::AtomicIsize;
use style::data::ElementData;

#[repr(C)]
pub struct StyleData {
    /// Data that the style system associates with a node. When the
    /// style system is being used standalone, this is all that hangs
    /// off the node. This must be first to permit the various
    /// transmutations between ElementData and PersistentLayoutData.
    pub element_data: AtomicRefCell<ElementData>,

    /// Information needed during parallel traversals.
    pub parallel: DomParallelInfo,
}

impl StyleData {
    pub fn new() -> Self {
        Self {
            element_data: AtomicRefCell::new(ElementData::default()),
            parallel: DomParallelInfo::new(),
        }
    }
}

#[derive(Copy, Clone, HeapSizeOf)]
pub struct OpaqueStyleAndLayoutData {
    // NB: We really store a `StyleAndLayoutData` here, so be careful!
    #[ignore_heap_size_of = "TODO(#6910) Box value that should be counted but \
                             the type lives in layout"]
    pub ptr: NonZero<*mut StyleData>
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
#[derive(Clone, Debug, PartialEq, Eq, Copy)]
pub struct TrustedNodeAddress(pub *const c_void);

#[allow(unsafe_code)]
unsafe impl Send for TrustedNodeAddress {}

pub fn is_image_data(uri: &str) -> bool {
    static TYPES: &'static [&'static str] = &["data:image/png", "data:image/gif", "data:image/jpeg"];
    TYPES.iter().any(|&type_| uri.starts_with(type_))
}

/// Whether the pending image needs to be fetched or is waiting on an existing fetch.
pub enum PendingImageState {
    Unrequested(ServoUrl),
    PendingResponse,
}

/// The data associated with an image that is not yet present in the image cache.
/// Used by the script thread to hold on to DOM elements that need to be repainted
/// when an image fetch is complete.
pub struct PendingImage {
    pub state: PendingImageState,
    pub node: UntrustedNodeAddress,
    pub id: PendingImageId,
}
