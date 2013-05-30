/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The high-level interface from script to layout. Using this abstract interface helps reduce
/// coupling between these two components, and enables the DOM to be placed in a separate crate
/// from layout.

use dom::node::{AbstractNode, ScriptView};
use script_task::ScriptMsg;

use core::comm::{Chan, SharedChan};
use geom::rect::Rect;
use geom::size::Size2D;
use gfx::geometry::Au;
use newcss::stylesheet::Stylesheet;
use std::net::url::Url;

/// Asynchronous messages that script can send to layout.
///
/// FIXME(pcwalton): I think this should probably be merged with `LayoutQuery` below.
pub enum Msg {
    /// Adds the given stylesheet to the document.
    AddStylesheetMsg(Stylesheet),

    /// Requests a reflow.
    ///
    /// FIXME(pcwalton): Call this `reflow` instead?
    BuildMsg(~BuildData),

    /// Performs a synchronous layout request.
    ///
    /// FIXME(pcwalton): As noted below, this isn't very type safe.
    QueryMsg(LayoutQuery, Chan<Result<LayoutResponse,()>>),

    /// Requests that the layout task shut down and exit.
    ExitMsg,
}

/// Synchronous messages that script can send to layout.
pub enum LayoutQuery {
    /// Requests the dimensions of the content box, as in the `getBoundingClientRect()` call.
    ContentBoxQuery(AbstractNode<ScriptView>),
    /// Requests the dimensions of all the content boxes, as in the `getClientRects()` call.
    ContentBoxesQuery(AbstractNode<ScriptView>),
}

/// The reply of a synchronous message from script to layout.
///
/// FIXME(pcwalton): This isn't very type safe. Maybe `LayoutQuery` objects should include
/// response channels?
pub enum LayoutResponse {
    /// A response to the `ContentBoxQuery` message.
    ContentBoxResponse(Rect<Au>),
    /// A response to the `ContentBoxesQuery` message.
    ContentBoxesResponse(~[Rect<Au>]),
}

/// Dirty bits for layout.
pub enum Damage {
    /// The document is clean; nothing needs to be done.
    NoDamage,
    /// Reflow, but do not perform CSS selector matching.
    ReflowDamage,
    /// Perform CSS selector matching and reflow.
    MatchSelectorsDamage,
}

impl Damage {
    /// Sets this damage to the maximum of this damage and the given damage.
    ///
    /// FIXME(pcwalton): This could be refactored to use `max` and the `Ord` trait, and this
    /// function removed.
    fn add(&mut self, new_damage: Damage) {
        match (*self, new_damage) {
            (NoDamage, _) => *self = new_damage,
            (ReflowDamage, NoDamage) => *self = ReflowDamage,
            (ReflowDamage, new_damage) => *self = new_damage,
            (MatchSelectorsDamage, _) => *self = MatchSelectorsDamage
        }
    }
}

/// Information needed for a reflow.
pub struct BuildData {
    node: AbstractNode<ScriptView>,
    /// What reflow needs to be done.
    damage: Damage,
    /// The URL of the page.
    url: Url,
    /// The channel through which messages can be sent back to the script task.
    script_chan: SharedChan<ScriptMsg>,
    /// The current window size.
    window_size: Size2D<uint>,
    script_join_chan: Chan<()>,
}

/// Encapsulates a channel to the layout task.
#[deriving(Clone)]
pub struct LayoutTask {
    chan: SharedChan<Msg>,
}

