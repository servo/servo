/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use api::{units::*, ColorF};

#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub enum DebugItem {
    Text {
        msg: String,
        color: ColorF,
        position: DevicePoint,
    },
    Rect {
        outer_color: ColorF,
        inner_color: ColorF,
        rect: DeviceRect,
    },
}
