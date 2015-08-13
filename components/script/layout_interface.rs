/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The high-level interface from script to layout. Using this abstract
//! interface helps reduce coupling between these two components, and enables
//! the DOM to be placed in a separate crate from layout.

use dom::node::LayoutData;

use euclid::point::Point2D;
use euclid::rect::Rect;
use ipc_channel::ipc::{IpcReceiver, IpcSender};
use libc::uintptr_t;
use msg::compositor_msg::LayerId;
use msg::constellation_msg::{ConstellationChan, Failure, PipelineExitType, PipelineId};
use msg::constellation_msg::{WindowSizeData};
use msg::compositor_msg::Epoch;
use net_traits::image_cache_task::ImageCacheTask;
use net_traits::PendingAsyncLoad;
use profile_traits::mem::ReportsChan;
use script_traits::{ConstellationControlMsg, LayoutControlMsg};
use script_traits::{OpaqueScriptLayoutChannel, StylesheetLoadResponder, UntrustedNodeAddress};
use selectors::parser::PseudoElement;
use std::any::Any;
use std::sync::mpsc::{channel, Receiver, Sender};
use string_cache::Atom;
use style::animation::PropertyAnimation;
use style::media_queries::MediaQueryList;
use style::stylesheets::Stylesheet;
use url::Url;
use util::geometry::Au;

pub use dom::node::TrustedNodeAddress;

/// Asynchronous messages that script can send to layout.
pub enum Msg {
    /// Adds the given stylesheet to the document.
    AddStylesheet(Stylesheet, MediaQueryList),

    /// Adds the given stylesheet to the document.
    LoadStylesheet(Url, MediaQueryList, PendingAsyncLoad, Box<StylesheetLoadResponder + Send>),

    /// Puts a document into quirks mode, causing the quirks mode stylesheet to be loaded.
    SetQuirksMode,

    /// Requests a reflow.
    Reflow(Box<ScriptReflow>),

    /// Get an RPC interface.
    GetRPC(Sender<Box<LayoutRPC + Send>>),

    /// Requests that the layout task render the next frame of all animations.
    TickAnimations,

    /// Updates the layout visible rects, affecting the area that display lists will be constructed
    /// for.
    SetVisibleRects(Vec<(LayerId, Rect<Au>)>),

    /// Destroys layout data associated with a DOM node.
    ///
    /// TODO(pcwalton): Maybe think about batching to avoid message traffic.
    ReapLayoutData(LayoutData),

    /// Requests that the layout task measure its memory usage. The resulting reports are sent back
    /// via the supplied channel.
    CollectReports(ReportsChan),

    /// Requests that the layout task enter a quiescent state in which no more messages are
    /// accepted except `ExitMsg`. A response message will be sent on the supplied channel when
    /// this happens.
    PrepareToExit(Sender<()>),

    /// Requests that the layout task immediately shut down. There must be no more nodes left after
    /// this, or layout will crash.
    ExitNow(PipelineExitType),

    /// Get the last epoch counter for this layout task.
    GetCurrentEpoch(IpcSender<Epoch>),

    /// Creates a new layout task.
    ///
    /// This basically exists to keep the script-layout dependency one-way.
    CreateLayoutTask(NewLayoutTaskInfo),
}

/// Synchronous messages that script can send to layout.
///
/// In general, you should use messages to talk to Layout. Use the RPC interface
/// if and only if the work is
///
///   1) read-only with respect to LayoutTaskData,
///   2) small,
///   3) and really needs to be fast.
pub trait LayoutRPC {
    /// Requests the dimensions of the content box, as in the `getBoundingClientRect()` call.
    fn content_box(&self) -> ContentBoxResponse;
    /// Requests the dimensions of all the content boxes, as in the `getClientRects()` call.
    fn content_boxes(&self) -> ContentBoxesResponse;
    /// Requests the geometry of this node. Used by APIs such as `clientTop`.
    fn node_geometry(&self) -> NodeGeometryResponse;
    /// Requests the node containing the point of interest
    fn hit_test(&self, node: TrustedNodeAddress, point: Point2D<f32>) -> Result<HitTestResponse, ()>;
    fn mouse_over(&self, node: TrustedNodeAddress, point: Point2D<f32>) -> Result<MouseOverResponse, ()>;
    /// Query layout for the resolved value of a given CSS property
    fn resolved_style(&self) -> ResolvedStyleResponse;
    fn offset_parent(&self) -> OffsetParentResponse;
}


pub struct ContentBoxResponse(pub Rect<Au>);
pub struct ContentBoxesResponse(pub Vec<Rect<Au>>);
pub struct NodeGeometryResponse {
    pub client_rect: Rect<i32>,
}
pub struct HitTestResponse(pub UntrustedNodeAddress);
pub struct MouseOverResponse(pub Vec<UntrustedNodeAddress>);
pub struct ResolvedStyleResponse(pub Option<String>);

#[derive(Clone)]
pub struct OffsetParentResponse {
    pub node_address: Option<UntrustedNodeAddress>,
    pub rect: Rect<Au>,
}

impl OffsetParentResponse {
    pub fn empty() -> OffsetParentResponse {
        OffsetParentResponse {
            node_address: None,
            rect: Rect::zero(),
        }
    }
}

/// Why we're doing reflow.
#[derive(PartialEq, Copy, Clone, Debug)]
pub enum ReflowGoal {
    /// We're reflowing in order to send a display list to the screen.
    ForDisplay,
    /// We're reflowing in order to satisfy a script query. No display list will be created.
    ForScriptQuery,
}

/// Any query to perform with this reflow.
#[derive(PartialEq)]
pub enum ReflowQueryType {
    NoQuery,
    ContentBoxQuery(TrustedNodeAddress),
    ContentBoxesQuery(TrustedNodeAddress),
    NodeGeometryQuery(TrustedNodeAddress),
    ResolvedStyleQuery(TrustedNodeAddress, Option<PseudoElement>, Atom),
    OffsetParentQuery(TrustedNodeAddress),
}

/// Information needed for a reflow.
pub struct Reflow {
    /// The goal of reflow: either to render to the screen or to flush layout info for script.
    pub goal: ReflowGoal,
    ///  A clipping rectangle for the page, an enlarged rectangle containing the viewport.
    pub page_clip_rect: Rect<Au>,
}

/// Information needed for a script-initiated reflow.
pub struct ScriptReflow {
    /// General reflow data.
    pub reflow_info: Reflow,
    /// The document node.
    pub document_root: TrustedNodeAddress,
    /// The channel through which messages can be sent back to the script task.
    pub script_chan: Sender<ConstellationControlMsg>,
    /// The current window size.
    pub window_size: WindowSizeData,
    /// The channel that we send a notification to.
    pub script_join_chan: Sender<()>,
    /// Unique identifier
    pub id: u32,
    /// The type of query if any to perform during this reflow.
    pub query_type: ReflowQueryType,
}

/// Encapsulates a channel to the layout task.
#[derive(Clone)]
pub struct LayoutChan(pub Sender<Msg>);

impl LayoutChan {
    pub fn new() -> (Receiver<Msg>, LayoutChan) {
        let (chan, port) = channel();
        (port, LayoutChan(chan))
    }
}

/// A trait to manage opaque references to script<->layout channels without needing
/// to expose the message type to crates that don't need to know about them.
pub trait ScriptLayoutChan {
    fn new(sender: Sender<Msg>, receiver: Receiver<Msg>) -> Self;
    fn sender(&self) -> Sender<Msg>;
    fn receiver(self) -> Receiver<Msg>;
}

impl ScriptLayoutChan for OpaqueScriptLayoutChannel {
    fn new(sender: Sender<Msg>, receiver: Receiver<Msg>) -> OpaqueScriptLayoutChannel {
        let inner = (box sender as Box<Any + Send>, box receiver as Box<Any + Send>);
        OpaqueScriptLayoutChannel(inner)
    }

    fn sender(&self) -> Sender<Msg> {
        let &OpaqueScriptLayoutChannel((ref sender, _)) = self;
        (*sender.downcast_ref::<Sender<Msg>>().unwrap()).clone()
    }

    fn receiver(self) -> Receiver<Msg> {
        let OpaqueScriptLayoutChannel((_, receiver)) = self;
        *receiver.downcast::<Receiver<Msg>>().unwrap()
    }
}

/// Type of an opaque node.
pub type OpaqueNode = uintptr_t;

/// State relating to an animation.
#[derive(Clone)]
pub struct Animation {
    /// An opaque reference to the DOM node participating in the animation.
    pub node: OpaqueNode,
    /// A description of the property animation that is occurring.
    pub property_animation: PropertyAnimation,
    /// The start time of the animation, as returned by `time::precise_time_s()`.
    pub start_time: f64,
    /// The end time of the animation, as returned by `time::precise_time_s()`.
    pub end_time: f64,
}

impl Animation {
    /// Returns the duration of this animation in seconds.
    #[inline]
    pub fn duration(&self) -> f64 {
        self.end_time - self.start_time
    }
}

pub struct NewLayoutTaskInfo {
    pub id: PipelineId,
    pub url: Url,
    pub is_parent: bool,
    pub layout_pair: OpaqueScriptLayoutChannel,
    pub pipeline_port: IpcReceiver<LayoutControlMsg>,
    pub constellation_chan: ConstellationChan,
    pub failure: Failure,
    pub script_chan: Sender<ConstellationControlMsg>,
    pub image_cache_task: ImageCacheTask,
    pub paint_chan: Box<Any + Send>,
    pub layout_shutdown_chan: Sender<()>,
}

