/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::model::units::Length;

// Viewport information for the root '<svg>' element.
pub struct ViewportInfo {
    pub width: Length,
    pub height: Length,
    pub view_box: Option<ViewBox>,
    pub overflow_visible: bool,
    pub aspect_ratio: Option<AspectRatio>,
}

// Viewport information for a nested '<svg>' element.
pub struct SvgViewport {
    pub x: Length,
    pub y: Length,
    pub viewport: ViewportInfo,
}

// 'viewbox' attribute.
pub struct ViewBox {
    pub min_x: Length,
    pub min_y: Length,
    pub width: Length,
    pub height: Length,
}

// 'preserveAspectRatio attribute
pub struct AspectRatio {
    pub align: AspectAlign,
    pub meet_or_slice: MeetOrSlice,
}

impl Default for AspectRatio {
    fn default() -> Self {
        AspectRatio {
            align: AspectAlign::XMidYMid,
            meet_or_slice: MeetOrSlice::Meet,
        }
    }
}

pub enum AspectAlign {
    None,
    XMinYMin,
    XMidYMin,
    XMaxYMin,
    XMinYMid,
    XMidYMid,
    XMaxYMid,
    XMinYMax,
    XMidYMax,
    XMaxYMax,
}

pub enum MeetOrSlice {
    Meet,
    Slice,
}
