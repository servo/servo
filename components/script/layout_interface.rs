/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The high-level interface from script to layout. Using this abstract
//! interface helps reduce coupling between these two components, and enables
//! the DOM to be placed in a separate crate from layout.

use app_units::Au;
use dom::node::OpaqueStyleAndLayoutData;
use euclid::point::Point2D;
use euclid::rect::Rect;
use gfx_traits::{Epoch, LayerId};
use ipc_channel::ipc::{IpcReceiver, IpcSender};
use msg::constellation_msg::{ConstellationChan, Failure, PipelineId};
use msg::constellation_msg::{WindowSizeData};
use net_traits::image_cache_thread::ImageCacheThread;
use profile_traits::mem::ReportsChan;
use script_traits::{ConstellationControlMsg, LayoutControlMsg, LayoutMsg as ConstellationMsg};
use script_traits::{OpaqueScriptLayoutChannel, UntrustedNodeAddress};
use std::any::Any;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender, channel};
use string_cache::Atom;
use style::context::ReflowGoal;
use style::properties::longhands::{margin_top, margin_right, margin_bottom, margin_left};
use style::selector_impl::PseudoElement;
use style::servo::Stylesheet;
use url::Url;
use util::ipc::OptionalOpaqueIpcSender;

pub use dom::node::TrustedNodeAddress;

/// Asynchronous messages that script can send to layout.
pub enum Msg {
    /// Adds the given stylesheet to the document.
    AddStylesheet(Arc<Stylesheet>),

    /// Puts a document into quirks mode, causing the quirks mode stylesheet to be loaded.
    SetQuirksMode,

    /// Requests a reflow.
    Reflow(ScriptReflow),

    /// Get an RPC interface.
    GetRPC(Sender<Box<LayoutRPC + Send>>),

    /// Requests that the layout thread render the next frame of all animations.
    TickAnimations,

    /// Requests that the layout thread reflow with a newly-loaded Web font.
    ReflowWithNewlyLoadedWebFont,

    /// Updates the layout visible rects, affecting the area that display lists will be constructed
    /// for.
    SetVisibleRects(Vec<(LayerId, Rect<Au>)>),

    /// Destroys layout data associated with a DOM node.
    ///
    /// TODO(pcwalton): Maybe think about batching to avoid message traffic.
    ReapStyleAndLayoutData(OpaqueStyleAndLayoutData),

    /// Requests that the layout thread measure its memory usage. The resulting reports are sent back
    /// via the supplied channel.
    CollectReports(ReportsChan),

    /// Requests that the layout thread enter a quiescent state in which no more messages are
    /// accepted except `ExitMsg`. A response message will be sent on the supplied channel when
    /// this happens.
    PrepareToExit(Sender<()>),

    /// Requests that the layout thread immediately shut down. There must be no more nodes left after
    /// this, or layout will crash.
    ExitNow,

    /// Get the last epoch counter for this layout thread.
    GetCurrentEpoch(IpcSender<Epoch>),

    /// Asks the layout thread whether any Web fonts have yet to load (if true, loads are pending;
    /// false otherwise).
    GetWebFontLoadState(IpcSender<bool>),

    /// Creates a new layout thread.
    ///
    /// This basically exists to keep the script-layout dependency one-way.
    CreateLayoutThread(NewLayoutThreadInfo),

    /// Set the final Url.
    SetFinalUrl(Url),
}

/// Synchronous messages that script can send to layout.
///
/// In general, you should use messages to talk to Layout. Use the RPC interface
/// if and only if the work is
///
///   1) read-only with respect to LayoutThreadData,
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
    fn hit_test(&self, point: Point2D<f32>) -> Result<HitTestResponse, ()>;
    /// Query layout for the topmost node under the mouse.
    fn mouse_over(&self, point: Point2D<f32>) -> Result<MouseOverResponse, ()>;
    /// Query layout for the resolved value of a given CSS property
    fn resolved_style(&self) -> ResolvedStyleResponse;
    fn offset_parent(&self) -> OffsetParentResponse;
    /// Query layout for the resolve values of the margin properties for an element.
    fn margin_style(&self) -> MarginStyleResponse;
}

#[derive(Clone)]
pub struct MarginStyleResponse {
    pub top: margin_top::computed_value::T,
    pub right: margin_right::computed_value::T,
    pub bottom: margin_bottom::computed_value::T,
    pub left: margin_left::computed_value::T,
}

impl MarginStyleResponse {
    pub fn empty() -> MarginStyleResponse {
        MarginStyleResponse {
            top: margin_top::computed_value::T::Auto,
            right: margin_right::computed_value::T::Auto,
            bottom: margin_bottom::computed_value::T::Auto,
            left: margin_left::computed_value::T::Auto,
        }
    }
}

pub struct ContentBoxResponse(pub Rect<Au>);
pub struct ContentBoxesResponse(pub Vec<Rect<Au>>);
pub struct NodeGeometryResponse {
    pub client_rect: Rect<i32>,
}
pub struct HitTestResponse(pub UntrustedNodeAddress);
pub struct MouseOverResponse(pub UntrustedNodeAddress);
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

/// Any query to perform with this reflow.
#[derive(PartialEq)]
pub enum ReflowQueryType {
    NoQuery,
    ContentBoxQuery(TrustedNodeAddress),
    ContentBoxesQuery(TrustedNodeAddress),
    NodeGeometryQuery(TrustedNodeAddress),
    ResolvedStyleQuery(TrustedNodeAddress, Option<PseudoElement>, Atom),
    OffsetParentQuery(TrustedNodeAddress),
    MarginStyleQuery(TrustedNodeAddress),
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
    pub document: TrustedNodeAddress,
    /// The document's list of stylesheets.
    pub document_stylesheets: Vec<Arc<Stylesheet>>,
    /// Whether the document's stylesheets have changed since the last script reflow.
    pub stylesheets_changed: bool,
    /// The current window size.
    pub window_size: WindowSizeData,
    /// The channel that we send a notification to.
    pub script_join_chan: Sender<()>,
    /// The type of query if any to perform during this reflow.
    pub query_type: ReflowQueryType,
}

impl Drop for ScriptReflow {
    fn drop(&mut self) {
        self.script_join_chan.send(()).unwrap();
    }
}

/// Encapsulates a channel to the layout thread.
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

pub struct NewLayoutThreadInfo {
    pub id: PipelineId,
    pub url: Url,
    pub is_parent: bool,
    pub layout_pair: OpaqueScriptLayoutChannel,
    pub pipeline_port: IpcReceiver<LayoutControlMsg>,
    pub constellation_chan: ConstellationChan<ConstellationMsg>,
    pub failure: Failure,
    pub script_chan: IpcSender<ConstellationControlMsg>,
    pub image_cache_thread: ImageCacheThread,
    pub paint_chan: OptionalOpaqueIpcSender,
    pub layout_shutdown_chan: IpcSender<()>,
    pub content_process_shutdown_chan: IpcSender<()>,
}
