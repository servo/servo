/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! This module contains traits in script used generically in the rest of Servo.
//! The traits are here instead of in script so that these modules won't have
//! to depend on script.

#![deny(unsafe_code)]

pub mod wrapper_traits;

use std::any::Any;
use std::borrow::Cow;
use std::sync::atomic::{AtomicIsize, AtomicU64, Ordering};
use std::sync::Arc;

use app_units::Au;
use atomic_refcell::AtomicRefCell;
use base::id::{BrowsingContextId, PipelineId};
use base::Epoch;
use canvas_traits::canvas::{CanvasId, CanvasMsg};
use crossbeam_channel::Sender;
use euclid::default::{Point2D, Rect};
use euclid::Size2D;
use fonts::FontCacheThread;
use ipc_channel::ipc::IpcSender;
use libc::c_void;
use malloc_size_of_derive::MallocSizeOf;
use metrics::PaintTimeMetrics;
use net_traits::image_cache::{ImageCache, PendingImageId};
use net_traits::ResourceThreads;
use profile_traits::mem::Report;
use profile_traits::time;
use script_traits::{
    ConstellationControlMsg, InitialScriptState, LayoutMsg, LoadData, Painter, ScrollState,
    UntrustedNodeAddress, WindowSizeData,
};
use serde::{Deserialize, Serialize};
use servo_arc::Arc as ServoArc;
use servo_url::{ImmutableOrigin, ServoUrl};
use style::animation::DocumentAnimationSet;
use style::context::QuirksMode;
use style::data::ElementData;
use style::dom::OpaqueNode;
use style::invalidation::element::restyle_hints::RestyleHint;
use style::media_queries::Device;
use style::properties::style_structs::Font;
use style::properties::PropertyId;
use style::selector_parser::{PseudoElement, RestyleDamage, Snapshot};
use style::stylesheets::Stylesheet;
use style::Atom;
use style_traits::CSSPixel;
use webrender_api::ImageKey;
use webrender_traits::WebRenderScriptApi;

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
    pub resource_threads: ResourceThreads,
    pub font_cache_thread: FontCacheThread,
    pub time_profiler_chan: time::ProfilerChan,
    pub webrender_api_sender: WebRenderScriptApi,
    pub paint_time_metrics: PaintTimeMetrics,
    pub window_size: WindowSizeData,
}

pub trait LayoutFactory: Send + Sync {
    fn create(&self, config: LayoutConfig) -> Box<dyn Layout>;
}

pub trait Layout {
    /// Get a reference to this Layout's Stylo `Device` used to handle media queries and
    /// resolve font metrics.
    fn device(&self) -> &Device;

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

    /// Inform the layout that its ScriptThread is about to exit.
    fn exit_now(&mut self);

    /// Requests that layout measure its memory usage. The resulting reports are sent back
    /// via the supplied channel.
    fn collect_reports(&self, reports: &mut Vec<Report>);

    /// Sets quirks mode for the document, causing the quirks mode stylesheet to be used.
    fn set_quirks_mode(&mut self, quirks_mode: QuirksMode);

    /// Removes a stylesheet from the Layout.
    fn remove_stylesheet(&mut self, stylesheet: ServoArc<Stylesheet>);

    /// Requests a reflow.
    fn reflow(&mut self, script_reflow: ScriptReflow);

    /// Tells layout that script has added some paint worklet modules.
    fn register_paint_worklet_modules(
        &mut self,
        name: Atom,
        properties: Vec<Atom>,
        painter: Box<dyn Painter>,
    );

    /// Set the scroll states of this layout after a compositor scroll.
    fn set_scroll_states(&mut self, scroll_states: &[ScrollState]);

    /// Set the paint time for a specific epoch.
    fn set_epoch_paint_time(&mut self, epoch: Epoch, paint_time: u64);

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

#[derive(Debug, PartialEq)]
pub enum NodesFromPointQueryType {
    All,
    Topmost,
}

#[derive(Debug, PartialEq)]
pub enum QueryMsg {
    ContentBox,
    ContentBoxes,
    ClientRectQuery,
    ScrollingAreaQuery,
    OffsetParentQuery,
    TextIndexQuery,
    NodesFromPointQuery,
    ResolvedStyleQuery,
    StyleQuery,
    ElementInnerTextQuery,
    ResolvedFontStyleQuery,
    InnerWindowDimensionsQuery,
}

/// Any query to perform with this reflow.
#[derive(Debug, PartialEq)]
pub enum ReflowGoal {
    Full,
    TickAnimations,
    LayoutQuery(QueryMsg, u64),

    /// Tells layout about a single new scrolling offset from the script. The rest will
    /// remain untouched and layout won't forward this back to script.
    UpdateScrollNode(ScrollState),
}

impl ReflowGoal {
    /// Returns true if the given ReflowQuery needs a full, up-to-date display list to
    /// be present or false if it only needs stacking-relative positions.
    pub fn needs_display_list(&self) -> bool {
        match *self {
            ReflowGoal::Full | ReflowGoal::TickAnimations | ReflowGoal::UpdateScrollNode(_) => true,
            ReflowGoal::LayoutQuery(ref querymsg, _) => match *querymsg {
                QueryMsg::ElementInnerTextQuery |
                QueryMsg::InnerWindowDimensionsQuery |
                QueryMsg::NodesFromPointQuery |
                QueryMsg::ResolvedStyleQuery |
                QueryMsg::TextIndexQuery => true,
                QueryMsg::ClientRectQuery |
                QueryMsg::ContentBox |
                QueryMsg::ContentBoxes |
                QueryMsg::OffsetParentQuery |
                QueryMsg::ResolvedFontStyleQuery |
                QueryMsg::ScrollingAreaQuery |
                QueryMsg::StyleQuery => false,
            },
        }
    }

    /// Returns true if the given ReflowQuery needs its display list send to WebRender or
    /// false if a layout_thread display list is sufficient.
    pub fn needs_display(&self) -> bool {
        match *self {
            ReflowGoal::Full | ReflowGoal::TickAnimations | ReflowGoal::UpdateScrollNode(_) => true,
            ReflowGoal::LayoutQuery(ref querymsg, _) => match *querymsg {
                QueryMsg::NodesFromPointQuery |
                QueryMsg::TextIndexQuery |
                QueryMsg::ElementInnerTextQuery => true,
                QueryMsg::ContentBox |
                QueryMsg::ContentBoxes |
                QueryMsg::ClientRectQuery |
                QueryMsg::ScrollingAreaQuery |
                QueryMsg::ResolvedStyleQuery |
                QueryMsg::ResolvedFontStyleQuery |
                QueryMsg::OffsetParentQuery |
                QueryMsg::InnerWindowDimensionsQuery |
                QueryMsg::StyleQuery => false,
            },
        }
    }
}

/// Information needed for a reflow.
pub struct Reflow {
    ///  A clipping rectangle for the page, an enlarged rectangle containing the viewport.
    pub page_clip_rect: Rect<Au>,
}

/// Information derived from a layout pass that needs to be returned to the script thread.
#[derive(Default)]
pub struct ReflowComplete {
    /// The list of images that were encountered that are in progress.
    pub pending_images: Vec<PendingImage>,
}

/// Information needed for a script-initiated reflow.
pub struct ScriptReflow {
    /// General reflow data.
    pub reflow_info: Reflow,
    /// The document node.
    pub document: TrustedNodeAddress,
    /// The dirty root from which to restyle.
    pub dirty_root: Option<TrustedNodeAddress>,
    /// Whether the document's stylesheets have changed since the last script reflow.
    pub stylesheets_changed: bool,
    /// The current window size.
    pub window_size: WindowSizeData,
    /// The channel that we send a notification to.
    pub script_join_chan: Sender<ReflowComplete>,
    /// The goal of this reflow.
    pub reflow_goal: ReflowGoal,
    /// The number of objects in the dom #10110
    pub dom_count: u32,
    /// The current window origin
    pub origin: ImmutableOrigin,
    /// Restyle snapshot map.
    pub pending_restyles: Vec<(TrustedNodeAddress, PendingRestyle)>,
    /// The current animation timeline value.
    pub animation_timeline_value: f64,
    /// The set of animations for this document.
    pub animations: DocumentAnimationSet,
}

/// A pending restyle.
#[derive(Debug, Default, MallocSizeOf)]
pub struct PendingRestyle {
    /// If this element had a state or attribute change since the last restyle, track
    /// the original condition of the element.
    pub snapshot: Option<Snapshot>,

    /// Any explicit restyles hints that have been accumulated for this element.
    pub hint: RestyleHint,

    /// Any explicit restyles damage that have been accumulated for this element.
    pub damage: RestyleDamage,
}

/// The type of fragment that a scroll root is created for.
///
/// This can only ever grow to maximum 4 entries. That's because we cram the value of this enum
/// into the lower 2 bits of the `ScrollRootId`, which otherwise contains a 32-bit-aligned
/// heap address.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, MallocSizeOf, PartialEq, Serialize)]
pub enum FragmentType {
    /// A StackingContext for the fragment body itself.
    FragmentBody,
    /// A StackingContext created to contain ::before pseudo-element content.
    BeforePseudoContent,
    /// A StackingContext created to contain ::after pseudo-element content.
    AfterPseudoContent,
}

/// The next ID that will be used for a special scroll root id.
///
/// A special scroll root is a scroll root that is created for generated content.
static NEXT_SPECIAL_SCROLL_ROOT_ID: AtomicU64 = AtomicU64::new(0);

/// If none of the bits outside this mask are set, the scroll root is a special scroll root.
/// Note that we assume that the top 16 bits of the address space are unused on the platform.
const SPECIAL_SCROLL_ROOT_ID_MASK: u64 = 0xffff;

/// Returns a new scroll root ID for a scroll root.
fn next_special_id() -> u64 {
    // We shift this left by 2 to make room for the fragment type ID.
    ((NEXT_SPECIAL_SCROLL_ROOT_ID.fetch_add(1, Ordering::SeqCst) + 1) << 2) &
        SPECIAL_SCROLL_ROOT_ID_MASK
}

pub fn combine_id_with_fragment_type(id: usize, fragment_type: FragmentType) -> u64 {
    debug_assert_eq!(id & (fragment_type as usize), 0);
    if fragment_type == FragmentType::FragmentBody {
        id as u64
    } else {
        next_special_id() | (fragment_type as u64)
    }
}

pub fn node_id_from_scroll_id(id: usize) -> Option<usize> {
    if (id as u64 & !SPECIAL_SCROLL_ROOT_ID_MASK) != 0 {
        return Some(id & !3);
    }
    None
}
