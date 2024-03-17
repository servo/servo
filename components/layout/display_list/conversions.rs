/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use app_units::Au;
use euclid::default::{Point2D, Rect, SideOffsets2D, Size2D, Vector2D};
use style::color::{AbsoluteColor, ColorSpace};
use style::computed_values::image_rendering::T as ImageRendering;
use style::computed_values::mix_blend_mode::T as MixBlendMode;
use style::computed_values::transform_style::T as TransformStyle;
use style::values::computed::{BorderStyle, Filter};
use style::values::specified::border::BorderImageRepeatKeyword;
use webrender_api as wr;

pub trait ToLayout {
    type Type;
    fn to_layout(&self) -> Self::Type;
}

pub trait FilterToLayout {
    type Type;
    fn to_layout(&self, current_color: &AbsoluteColor) -> Self::Type;
}

impl ToLayout for BorderStyle {
    type Type = wr::BorderStyle;
    fn to_layout(&self) -> Self::Type {
        match *self {
            BorderStyle::None => wr::BorderStyle::None,
            BorderStyle::Solid => wr::BorderStyle::Solid,
            BorderStyle::Double => wr::BorderStyle::Double,
            BorderStyle::Dotted => wr::BorderStyle::Dotted,
            BorderStyle::Dashed => wr::BorderStyle::Dashed,
            BorderStyle::Hidden => wr::BorderStyle::Hidden,
            BorderStyle::Groove => wr::BorderStyle::Groove,
            BorderStyle::Ridge => wr::BorderStyle::Ridge,
            BorderStyle::Inset => wr::BorderStyle::Inset,
            BorderStyle::Outset => wr::BorderStyle::Outset,
        }
    }
}

impl FilterToLayout for Filter {
    type Type = wr::FilterOp;
    fn to_layout(&self, current_color: &AbsoluteColor) -> Self::Type {
        match *self {
            Filter::Blur(radius) => wr::FilterOp::Blur(radius.px(), radius.px()),
            Filter::Brightness(amount) => wr::FilterOp::Brightness(amount.0),
            Filter::Contrast(amount) => wr::FilterOp::Contrast(amount.0),
            Filter::Grayscale(amount) => wr::FilterOp::Grayscale(amount.0),
            Filter::HueRotate(angle) => wr::FilterOp::HueRotate(angle.radians()),
            Filter::Invert(amount) => wr::FilterOp::Invert(amount.0),
            Filter::Opacity(amount) => wr::FilterOp::Opacity(amount.0.into(), amount.0),
            Filter::Saturate(amount) => wr::FilterOp::Saturate(amount.0),
            Filter::Sepia(amount) => wr::FilterOp::Sepia(amount.0),
            Filter::DropShadow(ref shadow) => wr::FilterOp::DropShadow(wr::Shadow {
                blur_radius: shadow.blur.px(),
                offset: wr::units::LayoutVector2D::new(
                    shadow.horizontal.px(),
                    shadow.vertical.px(),
                ),
                color: shadow
                    .color
                    .clone()
                    .resolve_to_absolute(current_color)
                    .to_layout(),
            }),
            // Statically check that Url is impossible.
            Filter::Url(ref url) => match *url {},
        }
    }
}

impl ToLayout for ImageRendering {
    type Type = wr::ImageRendering;
    fn to_layout(&self) -> Self::Type {
        match *self {
            ImageRendering::Auto => wr::ImageRendering::Auto,
            ImageRendering::CrispEdges => wr::ImageRendering::CrispEdges,
            ImageRendering::Pixelated => wr::ImageRendering::Pixelated,
        }
    }
}

impl ToLayout for MixBlendMode {
    type Type = wr::MixBlendMode;
    fn to_layout(&self) -> Self::Type {
        match *self {
            MixBlendMode::Normal => wr::MixBlendMode::Normal,
            MixBlendMode::Multiply => wr::MixBlendMode::Multiply,
            MixBlendMode::Screen => wr::MixBlendMode::Screen,
            MixBlendMode::Overlay => wr::MixBlendMode::Overlay,
            MixBlendMode::Darken => wr::MixBlendMode::Darken,
            MixBlendMode::Lighten => wr::MixBlendMode::Lighten,
            MixBlendMode::ColorDodge => wr::MixBlendMode::ColorDodge,
            MixBlendMode::ColorBurn => wr::MixBlendMode::ColorBurn,
            MixBlendMode::HardLight => wr::MixBlendMode::HardLight,
            MixBlendMode::SoftLight => wr::MixBlendMode::SoftLight,
            MixBlendMode::Difference => wr::MixBlendMode::Difference,
            MixBlendMode::Exclusion => wr::MixBlendMode::Exclusion,
            MixBlendMode::Hue => wr::MixBlendMode::Hue,
            MixBlendMode::Saturation => wr::MixBlendMode::Saturation,
            MixBlendMode::Color => wr::MixBlendMode::Color,
            MixBlendMode::Luminosity => wr::MixBlendMode::Luminosity,
        }
    }
}

impl ToLayout for TransformStyle {
    type Type = wr::TransformStyle;
    fn to_layout(&self) -> Self::Type {
        match *self {
            TransformStyle::Flat => wr::TransformStyle::Flat,
            TransformStyle::Preserve3d => wr::TransformStyle::Preserve3D,
        }
    }
}

impl ToLayout for AbsoluteColor {
    type Type = wr::ColorF;
    fn to_layout(&self) -> Self::Type {
        let rgba = self.to_color_space(ColorSpace::Srgb);
        wr::ColorF::new(
            rgba.components.0.clamp(0.0, 1.0),
            rgba.components.1.clamp(0.0, 1.0),
            rgba.components.2.clamp(0.0, 1.0),
            rgba.alpha,
        )
    }
}

impl ToLayout for Point2D<Au> {
    type Type = wr::units::LayoutPoint;
    fn to_layout(&self) -> Self::Type {
        wr::units::LayoutPoint::new(self.x.to_f32_px(), self.y.to_f32_px())
    }
}

impl ToLayout for Rect<Au> {
    type Type = wr::units::LayoutRect;
    fn to_layout(&self) -> Self::Type {
        wr::units::LayoutRect::from_origin_and_size(self.origin.to_layout(), self.size.to_layout())
    }
}

impl ToLayout for SideOffsets2D<Au> {
    type Type = wr::units::LayoutSideOffsets;
    fn to_layout(&self) -> Self::Type {
        wr::units::LayoutSideOffsets::new(
            self.top.to_f32_px(),
            self.right.to_f32_px(),
            self.bottom.to_f32_px(),
            self.left.to_f32_px(),
        )
    }
}

impl ToLayout for Size2D<Au> {
    type Type = wr::units::LayoutSize;
    fn to_layout(&self) -> Self::Type {
        wr::units::LayoutSize::new(self.width.to_f32_px(), self.height.to_f32_px())
    }
}

impl ToLayout for Vector2D<Au> {
    type Type = wr::units::LayoutVector2D;
    fn to_layout(&self) -> Self::Type {
        wr::units::LayoutVector2D::new(self.x.to_f32_px(), self.y.to_f32_px())
    }
}

impl ToLayout for BorderImageRepeatKeyword {
    type Type = wr::RepeatMode;

    fn to_layout(&self) -> Self::Type {
        match *self {
            BorderImageRepeatKeyword::Stretch => wr::RepeatMode::Stretch,
            BorderImageRepeatKeyword::Repeat => wr::RepeatMode::Repeat,
            BorderImageRepeatKeyword::Round => wr::RepeatMode::Round,
            BorderImageRepeatKeyword::Space => wr::RepeatMode::Space,
        }
    }
}
