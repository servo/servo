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
    /// FIXME: make it per-document or per-pipeline instead:
    /// https://github.com/servo/servo/issues/16027
    pub static ref AUTHOR_SHARED_LOCK: SharedRwLock = SharedRwLock::new();
}
