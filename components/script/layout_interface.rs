/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The high-level interface from script to layout. Using this abstract
//! interface helps reduce coupling between these two components, and enables
//! the DOM to be placed in a separate crate from layout.

use dom::node::LayoutDataRef;

use geom::point::Point2D;
use geom::rect::Rect;
use script_traits::{ScriptControlChan, OpaqueScriptLayoutChannel, UntrustedNodeAddress};
use msg::constellation_msg::{PipelineExitType, WindowSizeData};
use util::geometry::Au;
use std::any::Any;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::boxed::BoxAny;
use style::stylesheets::Stylesheet;
use url::Url;

pub use dom::node::TrustedNodeAddress;

/// Asynchronous messages that script can send to layout.
pub enum Msg {
    /// Adds the given stylesheet to the document.
    AddStylesheet(Stylesheet),

    /// Adds the given stylesheet to the document.
    LoadStylesheet(Url),

    /// Puts a document into quirks mode, causing the quirks mode stylesheet to be loaded.
    SetQuirksMode,

    /// Requests a reflow.
    Reflow(Box<Reflow>),

    /// Get an RPC interface.
    GetRPC(Sender<Box<LayoutRPC + Send>>),

    /// Destroys layout data associated with a DOM node.
    ///
    /// TODO(pcwalton): Maybe think about batching to avoid message traffic.
    ReapLayoutData(LayoutDataRef),

    /// Requests that the layout task enter a quiescent state in which no more messages are
    /// accepted except `ExitMsg`. A response message will be sent on the supplied channel when
    /// this happens.
    PrepareToExit(Sender<()>),

    /// Requests that the layout task immediately shut down. There must be no more nodes left after
    /// this, or layout will crash.
    ExitNow(PipelineExitType),
}

/// Synchronous messages that script can send to layout.
///
/// In general, you should use messages to talk to Layout. Use the RPC interface
/// if and only if the work is
///
///   1) read-only with respect to LayoutTaskData,
///   2) small,
//    3) and really needs to be fast.
pub trait LayoutRPC {
    /// Requests the dimensions of the content box, as in the `getBoundingClientRect()` call.
    fn content_box(&self) -> ContentBoxResponse;
    /// Requests the dimensions of all the content boxes, as in the `getClientRects()` call.
    fn content_boxes(&self) -> ContentBoxesResponse;
    /// Requests the node containing the point of interest
    fn hit_test(&self, node: TrustedNodeAddress, point: Point2D<f32>) -> Result<HitTestResponse, ()>;
    fn mouse_over(&self, node: TrustedNodeAddress, point: Point2D<f32>) -> Result<MouseOverResponse, ()>;
}

pub struct ContentBoxResponse(pub Rect<Au>);
pub struct ContentBoxesResponse(pub Vec<Rect<Au>>);
pub struct HitTestResponse(pub UntrustedNodeAddress);
pub struct MouseOverResponse(pub Vec<UntrustedNodeAddress>);

/// Why we're doing reflow.
#[derive(PartialEq, Debug)]
pub enum ReflowGoal {
    /// We're reflowing in order to send a display list to the screen.
    ForDisplay,
    /// We're reflowing in order to satisfy a script query. No display list will be created.
    ForScriptQuery,
}

/// Any query to perform with this reflow.
pub enum ReflowQueryType {
    NoQuery,
    ContentBoxQuery(TrustedNodeAddress),
    ContentBoxesQuery(TrustedNodeAddress),
}

/// Information needed for a reflow.
pub struct Reflow {
    /// The document node.
    pub document_root: TrustedNodeAddress,
    /// The goal of reflow: either to render to the screen or to flush layout info for script.
    pub goal: ReflowGoal,
    /// The URL of the page.
    pub url: Url,
    /// Is the current reflow of an iframe, as opposed to a root window?
    pub iframe: bool,
    /// The channel through which messages can be sent back to the script task.
    pub script_chan: ScriptControlChan,
    /// The current window size.
    pub window_size: WindowSizeData,
    /// The channel that we send a notification to.
    pub script_join_chan: Sender<()>,
    /// Unique identifier
    pub id: uint,
    /// The type of query if any to perform during this reflow.
    pub query_type: ReflowQueryType,
    ///  A clipping rectangle for the page, an enlarged rectangle containing the viewport.
    pub page_clip_rect: Rect<Au>,
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
        let inner = (box sender as Box<Any+Send>, box receiver as Box<Any+Send>);
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
