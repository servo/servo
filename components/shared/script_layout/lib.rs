/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! This module contains traits in script used generically in the rest of Servo.
//! The traits are here instead of in script so that these modules won't have
//! to depend on script.

#![deny(unsafe_code)]

pub mod message;
pub mod rpc;
pub mod wrapper_traits;

use std::any::Any;
use std::borrow::Cow;
use std::sync::atomic::AtomicIsize;
use std::sync::Arc;

use atomic_refcell::AtomicRefCell;
use canvas_traits::canvas::{CanvasId, CanvasMsg};
use gfx::font_cache_thread::FontCacheThread;
use gfx_traits::Epoch;
use ipc_channel::ipc::IpcSender;
use libc::c_void;
use malloc_size_of_derive::MallocSizeOf;
use metrics::PaintTimeMetrics;
use msg::constellation_msg::PipelineId;
use net_traits::image_cache::{ImageCache, PendingImageId};
use profile_traits::time;
use script_traits::{
    ConstellationControlMsg, InitialScriptState, LayoutControlMsg, LayoutMsg, LoadData,
    UntrustedNodeAddress, WebrenderIpcSender, WindowSizeData,
};
use servo_url::{ImmutableOrigin, ServoUrl};
use style::data::ElementData;
use webrender_api::ImageKey;

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

impl Default for StyleData {
    fn default() -> Self {
        Self {
            element_data: AtomicRefCell::new(ElementData::default()),
            parallel: DomParallelInfo::default(),
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
#[derive(Default, MallocSizeOf)]
pub struct DomParallelInfo {
    /// The number of children remaining to process during bottom-up traversal.
    pub children_to_process: AtomicIsize,
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
    WebGL(ImageKey),
    Image(Option<IpcSender<CanvasMsg>>),
    WebGPU(ImageKey),
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
    pub current_frame: Option<(ImageKey, i32, i32)>,
}

pub struct LayoutConfig {
    pub id: PipelineId,
    pub url: ServoUrl,
    pub is_iframe: bool,
    pub constellation_chan: IpcSender<LayoutMsg>,
    pub script_chan: IpcSender<ConstellationControlMsg>,
    pub image_cache: Arc<dyn ImageCache>,
    pub font_cache_thread: FontCacheThread,
    pub time_profiler_chan: time::ProfilerChan,
    pub webrender_api_sender: WebrenderIpcSender,
    pub paint_time_metrics: PaintTimeMetrics,
    pub window_size: WindowSizeData,
}

pub trait LayoutFactory: Send + Sync {
    fn create(&self, config: LayoutConfig) -> Box<dyn Layout>;
}

pub trait Layout {
    /// Process a single message from script.
    fn process(&mut self, msg: message::Msg);

    /// Handle a single message from the Constellation.
    fn handle_constellation_msg(&mut self, msg: LayoutControlMsg);

    /// Handle a a single mesasge from the FontCacheThread.
    fn handle_font_cache_msg(&mut self);

    /// Return the interface used for scipt queries.
    /// TODO: Make this part of the the Layout interface itself now that the
    /// layout thread has been removed.
    fn rpc(&self) -> Box<dyn rpc::LayoutRPC>;

    /// Whether or not this layout is waiting for fonts from loaded stylesheets to finish loading.
    fn waiting_for_web_fonts_to_load(&self) -> bool;

    /// The currently laid out Epoch that this Layout has finished.
    fn current_epoch(&self) -> Epoch;
}

/// This trait is part of `script_layout_interface` because it depends on both `script_traits`
/// and also `LayoutFactory` from this crate. If it was in `script_traits` there would be a
/// circular dependency.
pub trait ScriptThreadFactory {
    /// Create a `ScriptThread`.
    fn create(
        state: InitialScriptState,
        layout_factory: Arc<dyn LayoutFactory>,
        font_cache_thread: FontCacheThread,
        load_data: LoadData,
        user_agent: Cow<'static, str>,
    );
}
