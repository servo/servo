/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! This module contains traits in script used generically in the rest of Servo.
//! The traits are here instead of in script so that these modules won't have
//! to depend on script.

#![deny(unsafe_code)]

pub mod message;
pub mod wrapper_traits;

use std::any::Any;
use std::borrow::Cow;
use std::sync::atomic::AtomicIsize;
use std::sync::Arc;

use app_units::Au;
use atomic_refcell::AtomicRefCell;
use canvas_traits::canvas::{CanvasId, CanvasMsg};
use euclid::default::{Point2D, Rect};
use euclid::Size2D;
use gfx::font_cache_thread::FontCacheThread;
use gfx_traits::Epoch;
use ipc_channel::ipc::IpcSender;
use libc::c_void;
use malloc_size_of_derive::MallocSizeOf;
use message::NodesFromPointQueryType;
use metrics::PaintTimeMetrics;
use msg::constellation_msg::{BrowsingContextId, PipelineId};
use net_traits::image_cache::{ImageCache, PendingImageId};
use profile_traits::time;
use script_traits::{
    ConstellationControlMsg, InitialScriptState, LayoutControlMsg, LayoutMsg, LoadData,
    UntrustedNodeAddress, WebrenderIpcSender, WindowSizeData,
};
use servo_arc::Arc as ServoArc;
use servo_url::{ImmutableOrigin, ServoUrl};
use style::animation::DocumentAnimationSet;
use style::data::ElementData;
use style::dom::OpaqueNode;
use style::media_queries::Device;
use style::properties::style_structs::Font;
use style::properties::PropertyId;
use style::selector_parser::PseudoElement;
use style::stylesheets::Stylesheet;
use style_traits::CSSPixel;
use webrender_api::ImageKey;

pub type GenericLayoutData = dyn Any + Send + Sync;

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

    /// Get a reference to this Layout's Stylo `Device` used to handle media queries and
    /// resolve font metrics.
    fn device<'a>(&'a self) -> &'a Device;

    /// Whether or not this layout is waiting for fonts from loaded stylesheets to finish loading.
    fn waiting_for_web_fonts_to_load(&self) -> bool;

    /// The currently laid out Epoch that this Layout has finished.
    fn current_epoch(&self) -> Epoch;

    /// Load all fonts from the given stylesheet, returning the number of fonts that
    /// need to be loaded.
    fn load_web_fonts_from_stylesheet(&self, stylesheet: ServoArc<Stylesheet>);

    /// Add a stylesheet to this Layout. This will add it to the Layout's `Stylist` as well as
    /// loading all web fonts defined in the stylesheet. The second stylesheet is the insertion
    /// point (if it exists, the sheet needs to be inserted before it).
    fn add_stylesheet(
        &mut self,
        stylesheet: ServoArc<Stylesheet>,
        before_stylsheet: Option<ServoArc<Stylesheet>>,
    );

    /// Removes a stylesheet from the Layout.
    fn remove_stylesheet(&mut self, stylesheet: ServoArc<Stylesheet>);

    fn query_content_box(&self, node: OpaqueNode) -> Option<Rect<Au>>;
    fn query_content_boxes(&self, node: OpaqueNode) -> Vec<Rect<Au>>;
    fn query_client_rect(&self, node: OpaqueNode) -> Rect<i32>;
    fn query_element_inner_text(&self, node: TrustedNodeAddress) -> String;
    fn query_inner_window_dimension(
        &self,
        context: BrowsingContextId,
    ) -> Option<Size2D<f32, CSSPixel>>;
    fn query_nodes_from_point(
        &self,
        point: Point2D<f32>,
        query_type: NodesFromPointQueryType,
    ) -> Vec<UntrustedNodeAddress>;
    fn query_offset_parent(&self, node: OpaqueNode) -> OffsetParentResponse;
    fn query_resolved_style(
        &self,
        node: TrustedNodeAddress,
        pseudo: Option<PseudoElement>,
        property_id: PropertyId,
        animations: DocumentAnimationSet,
        animation_timeline_value: f64,
    ) -> String;
    fn query_resolved_font_style(
        &self,
        node: TrustedNodeAddress,
        value: &str,
        animations: DocumentAnimationSet,
        animation_timeline_value: f64,
    ) -> Option<ServoArc<Font>>;
    fn query_scrolling_area(&self, node: Option<OpaqueNode>) -> Rect<i32>;
    fn query_text_indext(&self, node: OpaqueNode, point: Point2D<f32>) -> Option<usize>;
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
#[derive(Clone, Default)]
pub struct OffsetParentResponse {
    pub node_address: Option<UntrustedNodeAddress>,
    pub rect: Rect<Au>,
}
