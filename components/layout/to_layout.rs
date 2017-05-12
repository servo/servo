/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use app_units::Au;
use euclid::{Point2D, Rect, SideOffsets2D, Size2D};
use style::computed_values::{filter, image_rendering, mix_blend_mode};
use style::values::computed;
use webrender_traits::{BorderStyle, BorderWidths, FilterOp, ImageRendering, LayoutPoint, LayoutRect, LayoutSize, MixBlendMode};

pub trait ToLayout {
    type Type;
    fn to_layout(&self) -> Self::Type;
}

impl ToLayout for computed::BorderStyle {
    type Type = BorderStyle;
    fn to_layout(&self) -> BorderStyle {
        use webrender_traits::BorderStyle::*;
        match *self {
            computed::BorderStyle::none => None,
            computed::BorderStyle::solid => Solid,
            computed::BorderStyle::double => Double,
            computed::BorderStyle::dotted => Dotted,
            computed::BorderStyle::dashed => Dashed,
            computed::BorderStyle::hidden => Hidden,
            computed::BorderStyle::groove => Groove,
            computed::BorderStyle::ridge => Ridge,
            computed::BorderStyle::inset => Inset,
            computed::BorderStyle::outset => Outset,
        }
    }
}

impl ToLayout for mix_blend_mode::T {
    type Type = MixBlendMode;
    fn to_layout(&self) -> MixBlendMode {
        use webrender_traits::MixBlendMode::*;
        match *self {
            mix_blend_mode::T::normal => Normal,
            mix_blend_mode::T::multiply => Multiply,
            mix_blend_mode::T::screen => Screen,
            mix_blend_mode::T::overlay => Overlay,
            mix_blend_mode::T::darken => Darken,
            mix_blend_mode::T::lighten => Lighten,
            mix_blend_mode::T::color_dodge => ColorDodge,
            mix_blend_mode::T::color_burn => ColorBurn,
            mix_blend_mode::T::hard_light => HardLight,
            mix_blend_mode::T::soft_light => SoftLight,
            mix_blend_mode::T::difference => Difference,
            mix_blend_mode::T::exclusion => Exclusion,
            mix_blend_mode::T::hue => Hue,
            mix_blend_mode::T::saturation => Saturation,
            mix_blend_mode::T::color => Color,
            mix_blend_mode::T::luminosity => Luminosity,
        }
    }
}

impl ToLayout for filter::T {
    type Type = Vec<FilterOp>;
    fn to_layout(&self) -> Vec<FilterOp> {
        use style::computed_values::filter::Filter;
        use webrender_traits::FilterOp::*;
        let mut result = Vec::with_capacity(self.filters.len());
        for filter in self.filters.iter() {
            result.push(match *filter {
                Filter::Blur(radius) => Blur(radius),
                Filter::Brightness(amount) => Brightness(amount),
                Filter::Contrast(amount) => Contrast(amount),
                Filter::Grayscale(amount) => Grayscale(amount),
                Filter::HueRotate(angle) => HueRotate(angle.radians()),
                Filter::Invert(amount) => Invert(amount),
                Filter::Opacity(amount) => Opacity(amount.into()),
                Filter::Saturate(amount) => Saturate(amount),
                Filter::Sepia(amount) => Sepia(amount),
            })
        }
        result
    }
}

impl ToLayout for image_rendering::T {
    type Type = ImageRendering;
    fn to_layout(&self) -> ImageRendering {
        use webrender_traits::ImageRendering::*;
        match *self {
            image_rendering::T::crisp_edges => CrispEdges,
            image_rendering::T::auto => Auto,
            image_rendering::T::pixelated => Pixelated,
        }
    }
}

impl ToLayout for Point2D<Au> {
    type Type = LayoutPoint;
    fn to_layout(&self) -> LayoutPoint {
        LayoutPoint::new(self.x.to_f32_px(), self.y.to_f32_px())
    }
}

impl ToLayout for Size2D<Au> {
    type Type = LayoutSize;
    fn to_layout(&self) -> LayoutSize {
        LayoutSize::new(self.width.to_f32_px(), self.height.to_f32_px())
    }
}

impl ToLayout for Rect<Au> {
    type Type = LayoutRect;
    fn to_layout(&self) -> LayoutRect {
        LayoutRect::new(self.origin.to_layout(), self.size.to_layout())
    }
}

impl ToLayout for SideOffsets2D<Au> {
    type Type = BorderWidths;
    fn to_layout(&self) -> BorderWidths {
        BorderWidths {
            left: self.left.to_f32_px(),
            top: self.top.to_f32_px(),
            right: self.right.to_f32_px(),
            bottom: self.bottom.to_f32_px(),
        }
    }
}
