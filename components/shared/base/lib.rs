/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![deny(unsafe_code)]

//! A crate to hold very common types in Servo.
//!
//! You should almost never need to add a data type to this crate. Instead look for
//! a more shared crate that has fewer dependents.

pub mod generic_channel;
pub mod id;
pub mod print_tree;
pub mod text;
mod unicode_block;

use serde::{Deserialize, Serialize};
use webrender_api::Epoch as WebRenderEpoch;

/// A struct for denoting the age of messages; prevents race conditions.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Epoch(pub u32);

impl Epoch {
    pub fn next(&mut self) {
        self.0 += 1;
    }
}

impl From<Epoch> for WebRenderEpoch {
    fn from(val: Epoch) -> Self {
        WebRenderEpoch(val.0)
    }
}

pub trait WebRenderEpochToU16 {
    fn as_u16(&self) -> u16;
}

impl WebRenderEpochToU16 for WebRenderEpoch {
    /// The value of this [`Epoch`] as a u16 value. Note that if this Epoch's
    /// value is more than u16::MAX, then the return value will be modulo
    /// u16::MAX.
    fn as_u16(&self) -> u16 {
        (self.0 % u16::MAX as u32) as u16
    }
}
