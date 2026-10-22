/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use malloc_size_of_derive::MallocSizeOf;

use crate::model::units::Length;

// Viewport information for the root '<svg>' element.
#[derive(Debug, MallocSizeOf)]
pub struct ViewportInfo {
    pub width: Length,
    pub height: Length,
    pub view_box: Option<ViewBox>,
    pub overflow_visible: bool,
    pub aspect_ratio: Option<AspectRatio>,
}

// Viewport information for a nested '<svg>' element.
#[derive(Debug, MallocSizeOf)]
pub struct SvgViewport {
    pub x: Length,
    pub y: Length,
    pub viewport: ViewportInfo,
}

// 'viewbox' attribute.
#[derive(Debug, MallocSizeOf)]
pub struct ViewBox {
    pub min_x: Length,
    pub min_y: Length,
    pub width: Length,
    pub height: Length,
}

// 'preserveAspectRatio attribute
#[derive(Debug, MallocSizeOf)]
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

#[derive(Debug, MallocSizeOf)]
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

#[derive(Debug, MallocSizeOf)]
pub enum MeetOrSlice {
    Meet,
    Slice,
}
