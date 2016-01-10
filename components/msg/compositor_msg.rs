/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/// A newtype struct for denoting the age of messages; prevents race conditions.
#[derive(PartialEq, Eq, Debug, Copy, Clone, PartialOrd, Ord, Deserialize, Serialize)]
pub struct Epoch(pub u32);

impl Epoch {
    pub fn next(&mut self) {
        let Epoch(ref mut u) = *self;
        *u += 1;
    }
}

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub struct FrameTreeId(pub u32);

impl FrameTreeId {
    pub fn next(&mut self) {
        let FrameTreeId(ref mut u) = *self;
        *u += 1;
    }
}
