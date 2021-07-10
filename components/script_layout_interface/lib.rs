/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! This module contains traits in script used generically in the rest of Servo.
//! The traits are here instead of in script so that these modules won't have
//! to depend on script.

#![deny(unsafe_code)]

#[macro_use]
extern crate html5ever;
#[macro_use]
extern crate malloc_size_of_derive;

pub mod message;
pub mod rpc;
pub mod wrapper_traits;

use atomic_refcell::AtomicRefCell;
use canvas_traits::canvas::{CanvasId, CanvasMsg};
use ipc_channel::ipc::IpcSender;
use libc::c_void;
use net_traits::image_cache::PendingImageId;
use script_traits::UntrustedNodeAddress;
use servo_url::{ImmutableOrigin, ServoUrl};
use std::any::Any;
use std::sync::atomic::AtomicIsize;
use style::data::ElementData;

#[derive(MallocSizeOf)]
pub struct StyleData {
    /// Data that the style system associates with a node. When the
    /// style system is being used standalone, this is all that hangs
    /// off the node. This must be first to permit the various
    /// transmutations between ElementData and PersistentLayoutData.
    #[ignore_malloc_size_of = "This probably should not be ignored"]
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

pub type StyleAndOpaqueLayoutData = StyleAndGenericData<dyn Any + Send + Sync>;

#[derive(MallocSizeOf)]
pub struct StyleAndGenericData<T>
where
    T: ?Sized,
{
    /// The style data.
    pub style_data: StyleData,
    /// The opaque layout data.
    #[ignore_malloc_size_of = "Trait objects are hard"]
    pub generic_data: T,
}

impl StyleAndOpaqueLayoutData {
    #[inline]
    pub fn new<T>(style_data: StyleData, layout_data: T) -> Box<Self>
    where
        T: Any + Send + Sync,
    {
        Box::new(StyleAndGenericData {
            style_data,
            generic_data: layout_data,
        })
    }
}

/// Information that we need stored in each DOM node.
#[derive(MallocSizeOf)]
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LayoutNodeType {
    Element(LayoutElementType),
    Text,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LayoutElementType {
    Element,
    HTMLBodyElement,
    HTMLBRElement,
    HTMLCanvasElement,
    HTMLHtmlElement,
    HTMLIFrameElement,
    HTMLImageElement,
    HTMLInputElement,
    HTMLMediaElement,
    HTMLObjectElement,
    HTMLParagraphElement,
    HTMLTableCellElement,
    HTMLTableColElement,
    HTMLTableElement,
    HTMLTableRowElement,
    HTMLTableSectionElement,
    HTMLTextAreaElement,
    SVGSVGElement,
}

pub enum HTMLCanvasDataSource {
    WebGL(webrender_api::ImageKey),
    Image(Option<IpcSender<CanvasMsg>>),
    WebGPU(webrender_api::ImageKey),
}

pub struct HTMLCanvasData {
    pub source: HTMLCanvasDataSource,
    pub width: u32,
    pub height: u32,
    pub canvas_id: CanvasId,
}

pub struct SVGSVGData {
    pub width: u32,
    pub height: u32,
}

/// The address of a node known to be valid. These are sent from script to layout.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TrustedNodeAddress(pub *const c_void);

#[allow(unsafe_code)]
unsafe impl Send for TrustedNodeAddress {}

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
    pub origin: ImmutableOrigin,
}

pub struct HTMLMediaData {
    pub current_frame: Option<(webrender_api::ImageKey, i32, i32)>,
}
