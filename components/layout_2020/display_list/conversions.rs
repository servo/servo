/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use app_units::Au;
use style::color::AbsoluteColor;
use style::computed_values::mix_blend_mode::T as ComputedMixBlendMode;
use style::computed_values::text_decoration_style::T as ComputedTextDecorationStyle;
use style::computed_values::transform_style::T as ComputedTransformStyle;
use style::values::computed::Filter as ComputedFilter;
use webrender_api::{units, FilterOp, LineStyle, MixBlendMode, Shadow, TransformStyle};

use crate::geom::{PhysicalPoint, PhysicalRect, PhysicalSides, PhysicalSize};

pub trait ToWebRender {
    type Type;
    fn to_webrender(&self) -> Self::Type;
}

pub trait FilterToWebRender {
    type Type;
    fn to_webrender(&self, current_color: &AbsoluteColor) -> Self::Type;
}

impl FilterToWebRender for ComputedFilter {
    type Type = FilterOp;
    fn to_webrender(&self, current_color: &AbsoluteColor) -> Self::Type {
        match *self {
            ComputedFilter::Blur(radius) => FilterOp::Blur(radius.px(), radius.px()),
            ComputedFilter::Brightness(amount) => FilterOp::Brightness(amount.0),
            ComputedFilter::Contrast(amount) => FilterOp::Contrast(amount.0),
            ComputedFilter::Grayscale(amount) => FilterOp::Grayscale(amount.0),
            ComputedFilter::HueRotate(angle) => FilterOp::HueRotate(angle.radians()),
            ComputedFilter::Invert(amount) => FilterOp::Invert(amount.0),
            ComputedFilter::Opacity(amount) => FilterOp::Opacity(amount.0.into(), amount.0),
            ComputedFilter::Saturate(amount) => FilterOp::Saturate(amount.0),
            ComputedFilter::Sepia(amount) => FilterOp::Sepia(amount.0),
            ComputedFilter::DropShadow(ref shadow) => FilterOp::DropShadow(Shadow {
                blur_radius: shadow.blur.px(),
                offset: units::LayoutVector2D::new(shadow.horizontal.px(), shadow.vertical.px()),
                color: super::rgba(shadow.color.clone().resolve_to_absolute(current_color)),
            }),
            // Statically check that Url is impossible.
            ComputedFilter::Url(ref url) => match *url {},
        }
    }
}

impl ToWebRender for ComputedMixBlendMode {
    type Type = MixBlendMode;
    fn to_webrender(&self) -> Self::Type {
        match *self {
            ComputedMixBlendMode::Normal => MixBlendMode::Normal,
            ComputedMixBlendMode::Multiply => MixBlendMode::Multiply,
            ComputedMixBlendMode::Screen => MixBlendMode::Screen,
            ComputedMixBlendMode::Overlay => MixBlendMode::Overlay,
            ComputedMixBlendMode::Darken => MixBlendMode::Darken,
            ComputedMixBlendMode::Lighten => MixBlendMode::Lighten,
            ComputedMixBlendMode::ColorDodge => MixBlendMode::ColorDodge,
            ComputedMixBlendMode::ColorBurn => MixBlendMode::ColorBurn,
            ComputedMixBlendMode::HardLight => MixBlendMode::HardLight,
            ComputedMixBlendMode::SoftLight => MixBlendMode::SoftLight,
            ComputedMixBlendMode::Difference => MixBlendMode::Difference,
            ComputedMixBlendMode::Exclusion => MixBlendMode::Exclusion,
            ComputedMixBlendMode::Hue => MixBlendMode::Hue,
            ComputedMixBlendMode::Saturation => MixBlendMode::Saturation,
            ComputedMixBlendMode::Color => MixBlendMode::Color,
            ComputedMixBlendMode::Luminosity => MixBlendMode::Luminosity,
        }
    }
}

impl ToWebRender for ComputedTransformStyle {
    type Type = TransformStyle;
    fn to_webrender(&self) -> Self::Type {
        match *self {
            ComputedTransformStyle::Flat => TransformStyle::Flat,
            ComputedTransformStyle::Preserve3d => TransformStyle::Preserve3D,
        }
    }
}

impl ToWebRender for PhysicalPoint<Au> {
    type Type = units::LayoutPoint;
    fn to_webrender(&self) -> Self::Type {
        units::LayoutPoint::new(self.x.to_f32_px(), self.y.to_f32_px())
    }
}

impl ToWebRender for PhysicalSize<Au> {
    type Type = units::LayoutSize;
    fn to_webrender(&self) -> Self::Type {
        units::LayoutSize::new(self.width.to_f32_px(), self.height.to_f32_px())
    }
}

impl ToWebRender for PhysicalRect<Au> {
    type Type = units::LayoutRect;
    fn to_webrender(&self) -> Self::Type {
        units::LayoutRect::from_origin_and_size(
            self.origin.to_webrender(),
            self.size.to_webrender(),
        )
    }
}

impl ToWebRender for PhysicalSides<Au> {
    type Type = units::LayoutSideOffsets;
    fn to_webrender(&self) -> Self::Type {
        units::LayoutSideOffsets::new(
            self.top.to_f32_px(),
            self.right.to_f32_px(),
            self.bottom.to_f32_px(),
            self.left.to_f32_px(),
        )
    }
}

impl ToWebRender for ComputedTextDecorationStyle {
    type Type = LineStyle;
    fn to_webrender(&self) -> Self::Type {
        match *self {
            ComputedTextDecorationStyle::Solid => LineStyle::Solid,
            ComputedTextDecorationStyle::Dotted => LineStyle::Dotted,
            ComputedTextDecorationStyle::Dashed => LineStyle::Dashed,
            ComputedTextDecorationStyle::Wavy => LineStyle::Wavy,
            _ => LineStyle::Solid,
        }
    }
}
