/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![deny(unsafe_code)]

#[macro_use]
extern crate serde;

pub mod context;
pub mod data;
pub mod display_list;
pub mod flow;
pub mod flow_ref;
mod fragment;
pub mod opaque_node;
pub mod query;
pub mod traversal;
pub mod wrapper;

// For unit tests:
pub use crate::fragment::Fragment;

// We can't use servo_arc for everything in layout, because the Flow stuff uses
// weak references.
use servo_arc::Arc as ServoArc;
