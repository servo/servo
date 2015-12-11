/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(custom_derive, plugin)]
#![plugin(plugins, serde_macros)]

#![crate_name = "gfx_traits"]
#![crate_type = "rlib"]
extern crate azure;
extern crate serde;
extern crate util;

pub mod color;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LayerKind {
    NoTransform,
    HasTransform,
}

/// The scrolling policy of a layer.
#[derive(Clone, PartialEq, Eq, Copy, Deserialize, Serialize, Debug, HeapSizeOf)]
pub enum ScrollPolicy {
    /// These layers scroll when the parent receives a scrolling message.
    Scrollable,
    /// These layers do not scroll when the parent receives a scrolling message.
    FixedPosition,
}
