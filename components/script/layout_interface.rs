/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The high-level interface from script to layout. Using this abstract interface helps reduce
/// coupling between these two components, and enables the DOM to be placed in a separate crate
/// from layout.

use dom::bindings::js::JS;
use dom::bindings::trace::JSTraceable;
use dom::node::{Node, LayoutDataRef};

use geom::point::Point2D;
use geom::rect::Rect;
use js::jsapi::JSTracer;
use libc::c_void;
use script_traits::{ScriptControlChan, OpaqueScriptLayoutChannel, UntrustedNodeAddress};
use servo_msg::constellation_msg::WindowSizeData;
use servo_util::geometry::Au;
use std::any::{Any, AnyRefExt};
use std::comm::{channel, Receiver, Sender};
use std::owned::BoxAny;
use style::Stylesheet;
use url::Url;

/// Asynchronous messages that script can send to layout.
pub enum Msg {
    /// Adds the given stylesheet to the document.
    AddStylesheetMsg(Stylesheet),

    /// Adds the given stylesheet to the document.
    LoadStylesheetMsg(Url),

    /// Requests a reflow.
    ReflowMsg(Box<Reflow>),

    /// Get an RPC interface.
    GetRPCMsg(Sender<Box<LayoutRPC + Send>>),

    /// Destroys layout data associated with a DOM node.
    ///
    /// TODO(pcwalton): Maybe think about batching to avoid message traffic.
    ReapLayoutDataMsg(LayoutDataRef),

    /// Requests that the layout task enter a quiescent state in which no more messages are
    /// accepted except `ExitMsg`. A response message will be sent on the supplied channel when
    /// this happens.
    PrepareToExitMsg(Sender<()>),

    /// Requests that the layout task immediately shut down. There must be no more nodes left after
    /// this, or layout will crash.
    ExitNowMsg,
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
    fn content_box(&self, node: TrustedNodeAddress) -> ContentBoxResponse;
    /// Requests the dimensions of all the content boxes, as in the `getClientRects()` call.
    fn content_boxes(&self, node: TrustedNodeAddress) -> ContentBoxesResponse;
    /// Requests the node containing the point of interest
    fn hit_test(&self, node: TrustedNodeAddress, point: Point2D<f32>) -> Result<HitTestResponse, ()>;
    fn mouse_over(&self, node: TrustedNodeAddress, point: Point2D<f32>) -> Result<MouseOverResponse, ()>;
}

/// The address of a node known to be valid. These must only be sent from content -> layout,
/// because we do not trust layout.
pub struct TrustedNodeAddress(pub *const c_void);

impl JSTraceable for TrustedNodeAddress {
    fn trace(&self, s: *mut JSTracer) {
        let TrustedNodeAddress(addr) = *self;
        let node = addr as *const Node;
        unsafe {
            JS::from_raw(node).trace(s)
        }
    }
}

pub struct ContentBoxResponse(pub Rect<Au>);
pub struct ContentBoxesResponse(pub Vec<Rect<Au>>);
pub struct HitTestResponse(pub UntrustedNodeAddress);
pub struct MouseOverResponse(pub Vec<UntrustedNodeAddress>);

/// Why we're doing reflow.
#[deriving(PartialEq, Show)]
pub enum ReflowGoal {
    /// We're reflowing in order to send a display list to the screen.
    ReflowForDisplay,
    /// We're reflowing in order to satisfy a script query. No display list will be created.
    ReflowForScriptQuery,
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
    pub id: uint
}

/// Encapsulates a channel to the layout task.
#[deriving(Clone)]
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
