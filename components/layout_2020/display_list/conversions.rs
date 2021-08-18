/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::geom::{PhysicalPoint, PhysicalRect, PhysicalSides, PhysicalSize};
use style::computed_values::mix_blend_mode::T as ComputedMixBlendMode;
use style::computed_values::text_decoration_style::T as ComputedTextDecorationStyle;
use style::computed_values::transform_style::T as ComputedTransformStyle;
use style::values::computed::Filter as ComputedFilter;
use style::values::computed::Length;
use webrender_api as wr;

pub trait ToWebRender {
    type Type;
    fn to_webrender(&self) -> Self::Type;
}

impl ToWebRender for ComputedFilter {
    type Type = wr::FilterOp;
    fn to_webrender(&self) -> Self::Type {
        match *self {
            ComputedFilter::Blur(radius) => wr::FilterOp::Blur(radius.px()),
            ComputedFilter::Brightness(amount) => wr::FilterOp::Brightness(amount.0),
            ComputedFilter::Contrast(amount) => wr::FilterOp::Contrast(amount.0),
            ComputedFilter::Grayscale(amount) => wr::FilterOp::Grayscale(amount.0),
            ComputedFilter::HueRotate(angle) => wr::FilterOp::HueRotate(angle.radians()),
            ComputedFilter::Invert(amount) => wr::FilterOp::Invert(amount.0),
            ComputedFilter::Opacity(amount) => wr::FilterOp::Opacity(amount.0.into(), amount.0),
            ComputedFilter::Saturate(amount) => wr::FilterOp::Saturate(amount.0),
            ComputedFilter::Sepia(amount) => wr::FilterOp::Sepia(amount.0),
            // Statically check that DropShadow is impossible.
            ComputedFilter::DropShadow(ref shadow) => match *shadow {},
            // Statically check that Url is impossible.
            ComputedFilter::Url(ref url) => match *url {},
        }
    }
}
impl ToWebRender for ComputedMixBlendMode {
    type Type = wr::MixBlendMode;
    fn to_webrender(&self) -> Self::Type {
        match *self {
            ComputedMixBlendMode::Normal => wr::MixBlendMode::Normal,
            ComputedMixBlendMode::Multiply => wr::MixBlendMode::Multiply,
            ComputedMixBlendMode::Screen => wr::MixBlendMode::Screen,
            ComputedMixBlendMode::Overlay => wr::MixBlendMode::Overlay,
            ComputedMixBlendMode::Darken => wr::MixBlendMode::Darken,
            ComputedMixBlendMode::Lighten => wr::MixBlendMode::Lighten,
            ComputedMixBlendMode::ColorDodge => wr::MixBlendMode::ColorDodge,
            ComputedMixBlendMode::ColorBurn => wr::MixBlendMode::ColorBurn,
            ComputedMixBlendMode::HardLight => wr::MixBlendMode::HardLight,
            ComputedMixBlendMode::SoftLight => wr::MixBlendMode::SoftLight,
            ComputedMixBlendMode::Difference => wr::MixBlendMode::Difference,
            ComputedMixBlendMode::Exclusion => wr::MixBlendMode::Exclusion,
            ComputedMixBlendMode::Hue => wr::MixBlendMode::Hue,
            ComputedMixBlendMode::Saturation => wr::MixBlendMode::Saturation,
            ComputedMixBlendMode::Color => wr::MixBlendMode::Color,
            ComputedMixBlendMode::Luminosity => wr::MixBlendMode::Luminosity,
        }
    }
}

impl ToWebRender for ComputedTransformStyle {
    type Type = wr::TransformStyle;
    fn to_webrender(&self) -> Self::Type {
        match *self {
            ComputedTransformStyle::Flat => wr::TransformStyle::Flat,
            ComputedTransformStyle::Preserve3d => wr::TransformStyle::Preserve3D,
        }
    }
}

impl ToWebRender for PhysicalPoint<Length> {
    type Type = webrender_api::units::LayoutPoint;
    fn to_webrender(&self) -> Self::Type {
        webrender_api::units::LayoutPoint::new(self.x.px(), self.y.px())
    }
}

impl ToWebRender for PhysicalSize<Length> {
    type Type = webrender_api::units::LayoutSize;
    fn to_webrender(&self) -> Self::Type {
        webrender_api::units::LayoutSize::new(self.width.px(), self.height.px())
    }
}

impl ToWebRender for PhysicalRect<Length> {
    type Type = webrender_api::units::LayoutRect;
    fn to_webrender(&self) -> Self::Type {
        webrender_api::units::LayoutRect::new(self.origin.to_webrender(), self.size.to_webrender())
    }
}

impl ToWebRender for PhysicalSides<Length> {
    type Type = webrender_api::units::LayoutSideOffsets;
    fn to_webrender(&self) -> Self::Type {
        webrender_api::units::LayoutSideOffsets::new(
            self.top.px(),
            self.right.px(),
            self.bottom.px(),
            self.left.px(),
        )
    }
}

impl ToWebRender for ComputedTextDecorationStyle {
    type Type = webrender_api::LineStyle;
    fn to_webrender(&self) -> Self::Type {
        match *self {
            ComputedTextDecorationStyle::Solid => wr::LineStyle::Solid,
            ComputedTextDecorationStyle::Dotted => wr::LineStyle::Dotted,
            ComputedTextDecorationStyle::Dashed => wr::LineStyle::Dashed,
            ComputedTextDecorationStyle::Wavy => wr::LineStyle::Wavy,
            _ => wr::LineStyle::Solid,
        }
    }
}
