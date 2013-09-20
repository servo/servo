/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The high-level interface from script to layout. Using this abstract interface helps reduce
/// coupling between these two components, and enables the DOM to be placed in a separate crate
/// from layout.

use dom::node::{AbstractNode, ScriptView, LayoutView};
use script_task::{ScriptChan};
use std::comm::{Chan, SharedChan};
use geom::rect::Rect;
use geom::size::Size2D;
use geom::point::Point2D;
use gfx::geometry::Au;
use newcss::stylesheet::Stylesheet;
use extra::url::Url;

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

    /// Requests that the layout task shut down and exit.
    ExitMsg,
}

/// Synchronous messages that script can send to layout.
pub enum LayoutQuery {
    /// Requests the dimensions of the content box, as in the `getBoundingClientRect()` call.
    ContentBoxQuery(AbstractNode<ScriptView>, Chan<ContentBoxResponse>),
    /// Requests the dimensions of all the content boxes, as in the `getClientRects()` call.
    ContentBoxesQuery(AbstractNode<ScriptView>, Chan<ContentBoxesResponse>),
    /// Requests the node containing the point of interest
    HitTestQuery(AbstractNode<ScriptView>, Point2D<f32>, Chan<Result<HitTestResponse, ()>>),
}

pub struct ContentBoxResponse(Rect<Au>);
pub struct ContentBoxesResponse(~[Rect<Au>]);
pub struct HitTestResponse(AbstractNode<LayoutView>);

/// Determines which part of the 
pub enum DocumentDamageLevel {
    /// Perform CSS selector matching and reflow.
    MatchSelectorsDocumentDamage,
    /// Reflow, but do not perform CSS selector matching.
    ReflowDocumentDamage,
}

impl DocumentDamageLevel {
    /// Sets this damage to the maximum of this damage and the given damage.
    ///
    /// FIXME(pcwalton): This could be refactored to use `max` and the `Ord` trait, and this
    /// function removed.
    pub fn add(&mut self, new_damage: DocumentDamageLevel) {
        match (*self, new_damage) {
            (ReflowDocumentDamage, new_damage) => *self = new_damage,
            (MatchSelectorsDocumentDamage, _) => *self = MatchSelectorsDocumentDamage,
        }
    }
}

/// What parts of the document have changed, as far as the script task can tell.
///
/// Note that this is fairly coarse-grained and is separate from layout's notion of the document
pub struct DocumentDamage {
    /// The topmost node in the tree that has changed.
    root: AbstractNode<ScriptView>,
    /// The amount of damage that occurred.
    level: DocumentDamageLevel,
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
    document_root: AbstractNode<ScriptView>,
    /// The style changes that need to be done.
    damage: DocumentDamage,
    /// The goal of reflow: either to render to the screen or to flush layout info for script.
    goal: ReflowGoal,
    /// The URL of the page.
    url: Url,
    /// The channel through which messages can be sent back to the script task.
    script_chan: ScriptChan,
    /// The current window size.
    window_size: Size2D<uint>,
    /// The channel that we send a notification to.
    script_join_chan: Chan<()>,
    /// Unique identifier
    id: uint
}

/// Encapsulates a channel to the layout task.
#[deriving(Clone)]
pub struct LayoutChan(SharedChan<Msg>);
impl LayoutChan {
    pub fn new(chan: Chan<Msg>) -> LayoutChan {
        LayoutChan(SharedChan::new(chan))
    }
}
