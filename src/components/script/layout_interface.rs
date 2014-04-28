/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The high-level interface from script to layout. Using this abstract interface helps reduce
/// coupling between these two components, and enables the DOM to be placed in a separate crate
/// from layout.

use dom::bindings::js::JS;
use dom::node::{Node, LayoutDataRef};

use geom::point::Point2D;
use geom::rect::Rect;
use geom::size::Size2D;
use libc::c_void;
use script_task::{ScriptChan};
use servo_util::geometry::Au;
use std::cmp;
use std::comm::{channel, Receiver, Sender};
use style::Stylesheet;
use url::Url;

use serialize::{Encodable, Encoder};

/// Asynchronous messages that script can send to layout.
///
/// FIXME(pcwalton): I think this should probably be merged with `LayoutQuery` below.
pub enum Msg {
    /// Adds the given stylesheet to the document.
    AddStylesheetMsg(Stylesheet),

    /// Requests a reflow.
    ReflowMsg(~Reflow),

    /// Performs a synchronous layout request.
    ///
    /// FIXME(pcwalton): As noted below, this isn't very type safe.
    QueryMsg(LayoutQuery),

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
pub enum LayoutQuery {
    /// Requests the dimensions of the content box, as in the `getBoundingClientRect()` call.
    ContentBoxQuery(TrustedNodeAddress, Sender<ContentBoxResponse>),
    /// Requests the dimensions of all the content boxes, as in the `getClientRects()` call.
    ContentBoxesQuery(TrustedNodeAddress, Sender<ContentBoxesResponse>),
    /// Requests the node containing the point of interest
    HitTestQuery(TrustedNodeAddress, Point2D<f32>, Sender<Result<HitTestResponse, ()>>),
    MouseOverQuery(TrustedNodeAddress, Point2D<f32>, Sender<Result<MouseOverResponse, ()>>),
}

/// The address of a node known to be valid. These must only be sent from content -> layout,
/// because we do not trust layout.
pub struct TrustedNodeAddress(pub *c_void);

impl<S: Encoder<E>, E> Encodable<S, E> for TrustedNodeAddress {
    fn encode(&self, s: &mut S) -> Result<(), E> {
        let TrustedNodeAddress(addr) = *self;
        let node = addr as *Node as *mut Node;
        unsafe {
            JS::from_raw(node).encode(s)
        };
        Ok(())
    }
}

/// The address of a node. Layout sends these back. They must be validated via
/// `from_untrusted_node_address` before they can be used, because we do not trust layout.
pub type UntrustedNodeAddress = *c_void;

pub struct ContentBoxResponse(pub Rect<Au>);
pub struct ContentBoxesResponse(pub Vec<Rect<Au>>);
pub struct HitTestResponse(pub UntrustedNodeAddress);
pub struct MouseOverResponse(pub Vec<UntrustedNodeAddress>);

/// Determines which part of the
#[deriving(Eq, Ord, TotalEq, TotalOrd, Encodable)]
pub enum DocumentDamageLevel {
    /// Reflow, but do not perform CSS selector matching.
    ReflowDocumentDamage,
    /// Perform CSS selector matching and reflow.
    MatchSelectorsDocumentDamage,
    /// Content changed; set full style damage and do the above.
    ContentChangedDocumentDamage,
}

impl DocumentDamageLevel {
    /// Sets this damage to the maximum of this damage and the given damage.
    pub fn add(&mut self, new_damage: DocumentDamageLevel) {
        *self = cmp::max(*self, new_damage);
    }
}

/// What parts of the document have changed, as far as the script task can tell.
///
/// Note that this is fairly coarse-grained and is separate from layout's notion of the document
#[deriving(Encodable)]
pub struct DocumentDamage {
    /// The topmost node in the tree that has changed.
    pub root: TrustedNodeAddress,
    /// The amount of damage that occurred.
    pub level: DocumentDamageLevel,
}

/// Why we're doing reflow.
#[deriving(Eq)]
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
    /// The style changes that need to be done.
    pub damage: DocumentDamage,
    /// The goal of reflow: either to render to the screen or to flush layout info for script.
    pub goal: ReflowGoal,
    /// The URL of the page.
    pub url: Url,
    /// The channel through which messages can be sent back to the script task.
    pub script_chan: ScriptChan,
    /// The current window size.
    pub window_size: Size2D<uint>,
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

#[test]
fn test_add_damage() {
    fn assert_add(mut a: DocumentDamageLevel, b: DocumentDamageLevel,
                  result: DocumentDamageLevel) {
        a.add(b);
        assert!(a == result);
    }

    assert_add(ReflowDocumentDamage, ReflowDocumentDamage, ReflowDocumentDamage);
    assert_add(ContentChangedDocumentDamage, ContentChangedDocumentDamage, ContentChangedDocumentDamage);
    assert_add(ReflowDocumentDamage, MatchSelectorsDocumentDamage, MatchSelectorsDocumentDamage);
    assert_add(MatchSelectorsDocumentDamage, ReflowDocumentDamage, MatchSelectorsDocumentDamage);
    assert_add(ReflowDocumentDamage, ContentChangedDocumentDamage, ContentChangedDocumentDamage);
    assert_add(ContentChangedDocumentDamage, ReflowDocumentDamage, ContentChangedDocumentDamage);
    assert_add(MatchSelectorsDocumentDamage, ContentChangedDocumentDamage, ContentChangedDocumentDamage);
    assert_add(ContentChangedDocumentDamage, MatchSelectorsDocumentDamage, ContentChangedDocumentDamage);
}
