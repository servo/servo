/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![deny(unsafe_code)]

#[macro_use]
extern crate serde;

pub mod context;
pub mod data;
pub mod display_list;
mod fragment;
pub mod opaque_node;
pub mod query;
pub mod traversal;
pub mod wrapper;

// For unit tests:
pub use crate::fragment::Fragment;

use servo_arc::Arc as ServoArc;
