/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Servo-specific bits of the style system.
//!
//! These get compiled out on a Gecko build.

pub mod media_queries;
pub mod restyle_damage;
pub mod selector_parser;

use shared_lock::SharedRwLock;

lazy_static! {
    /// Per-process shared lock for author-origin stylesheets
    ///
    /// FIXME(SimonSapin): I wanted this to be per-document (or per-pipeline?)
    /// but couldn’t figure out a way to get references to the same lock in both
    /// the Document node and the LayoutThread.
    /// Giving up to unblock.
    pub static ref AUTHOR_SHARED_LOCK: SharedRwLock = SharedRwLock::new();
}
