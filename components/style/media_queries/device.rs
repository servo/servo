/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use ::geom::size::{Size2D, TypedSize2D};
use ::util::geometry::{Au, ViewportPx};

use super::DeviceFeatureContext;
use super::MediaType;

use super::feature::Orientation;
use super::feature::{Scan, UpdateFrequency, OverflowBlock, OverflowInline};
use super::feature::InvertedColors;
use super::feature::{Pointer, Hover, AnyPointer, AnyHover};
use super::feature::LightLevel;
use super::feature::Scripting;
use super::feature::ViewMode;

#[allow(missing_copy_implementations)]
#[derive(Debug)]
pub struct Device {
    pub media_type: MediaType,
    pub viewport_size: Size2D<Au>,
}

impl Device {
    pub fn new(media_type: MediaType, viewport_size: TypedSize2D<ViewportPx, f32>) -> Device {
        Device {
            media_type: media_type,
            viewport_size: Size2D(Au::from_frac32_px(viewport_size.width.get()),
                                  Au::from_frac32_px(viewport_size.height.get()))
        }
    }
}

impl DeviceFeatureContext for Device {
    fn MediaType(&self) -> MediaType {
        self.media_type
    }

    fn ViewportSize(&self) -> Size2D<Au> {
        match self.media_type {
            MediaType::Speech => Size2D(Au(0), Au(0)),
            _ => self.viewport_size
        }
    }

    fn Width(&self) -> Au {
        self.ViewportSize().width
    }

    fn Height(&self) -> Au {
        self.ViewportSize().height
    }

    fn AspectRatio(&self) -> f32 {
        use std::num::ToPrimitive;

        let vps = self.ViewportSize();
        match (vps.width.to_f32(), vps.height.to_f32()) {
            (Some(w), Some(h)) if w > 0. && h > 0. => w / h,
            _ => 0.
        }
    }

    fn Orientation(&self) -> Option<Orientation> {
        let vps = self.ViewportSize();
        if vps.width > Au(0) && vps.height > Au(0) {
            if vps.width > vps.height {
                Some(Orientation::Landscape)
            } else {
                Some(Orientation::Portrait)
            }
        } else {
            None
        }
    }

    // TODO
    fn Grid(&self) -> bool {
        false
    }

    // TODO
    fn Scan(&self) -> Option<Scan> {
        match self.media_type {
            MediaType::Screen => Some(Scan::Progressive),
            _ => None
        }
    }

    // TODO
    fn UpdateFrequency(&self) -> UpdateFrequency {
        match self.media_type {
            MediaType::Screen => UpdateFrequency::Normal,
            MediaType::Speech => UpdateFrequency::Slow,
            _ => UpdateFrequency::None
        }
    }

    // TODO
    fn OverflowBlock(&self) -> OverflowBlock {
        match self.media_type {
            MediaType::Print => OverflowBlock::Paged,
            MediaType::Screen => OverflowBlock::Scroll,
            _ => OverflowBlock::None
        }
    }

    // TODO
    fn OverflowInline(&self) -> OverflowInline {
        match self.media_type {
            MediaType::Screen => OverflowInline::Scroll,
            _ => OverflowInline::None
        }
    }

    // TODO
    fn Color(&self) -> u8 {
        if self.media_type != MediaType::Speech {
            8
        } else {
            0
        }
    }

    // TODO
    fn ColorIndex(&self) -> u32 {
        0
    }

    // TODO
    fn Monochrome(&self) -> u32 {
        0
    }

    // TODO
    fn InvertedColors(&self) -> InvertedColors {
        InvertedColors::None
    }

    // TODO
    fn Pointer(&self) -> Pointer {
        match self.media_type {
            MediaType::Screen => Pointer::Fine,
            _ => Pointer::None
        }
    }

    // TODO
    fn Hover(&self) -> Hover {
        match self.media_type {
            MediaType::Screen => Hover::Hover,
            _ => Hover::None
        }
    }

    // TODO
    fn AnyPointer(&self) -> AnyPointer {
        match self.media_type {
            MediaType::Screen => AnyPointer::Fine,
            _ => AnyPointer::None
        }
    }

    // TODO
    fn AnyHover(&self) -> AnyHover {
        match self.media_type {
            MediaType::Screen => AnyHover::Hover,
            _ => AnyHover::None
        }
    }

    // TODO
    fn LightLevel(&self) -> Option<LightLevel> {
        match self.media_type {
            MediaType::Screen => Some(LightLevel::Normal),
            _ => None
        }
    }

    // TODO
    fn Scripting(&self) -> Scripting {
        match self.media_type {
            MediaType::Print => Scripting::InitialOnly,
            _ => Scripting::Enabled
        }
    }

    // TODO
    fn DeviceWidth(&self) -> Au {
        Au(0)
    }

    // TODO
    fn DeviceHeight(&self) -> Au {
        Au(0)
    }

    // TDOD
    fn DeviceAspectRatio(&self) -> f32 {
        0.
    }

    // TODO
    fn ViewMode(&self) -> Option<ViewMode> {
        match self.media_type {
            // Servo doesn't have any chrome...
            MediaType::Screen => Some(ViewMode::Floating),
            _ => None
        }
    }
}
