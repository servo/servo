/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use app_units::Au;
use euclid::{Point2D, Rect, SideOffsets2D, Size2D, Vector2D};
use style::computed_values::image_rendering::T as ImageRendering;
use style::computed_values::mix_blend_mode::T as MixBlendMode;
use style::computed_values::transform_style::T as TransformStyle;
use style::properties::longhands::border_image_repeat::RepeatKeyword;
use style::values::RGBA;
use style::values::computed::{BorderStyle, Filter};
use style::values::generics::effects::Filter as GenericFilter;
use webrender_api as wr;

pub trait ToLayout {
    type Type;
    fn to_layout(&self) -> Self::Type;
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

impl ToLayout for Filter {
    type Type = wr::FilterOp;
    fn to_layout(&self) -> Self::Type {
        match *self {
            GenericFilter::Blur(radius) => wr::FilterOp::Blur(radius.px()),
            GenericFilter::Brightness(amount) => wr::FilterOp::Brightness(amount.0),
            GenericFilter::Contrast(amount) => wr::FilterOp::Contrast(amount.0),
            GenericFilter::Grayscale(amount) => wr::FilterOp::Grayscale(amount.0),
            GenericFilter::HueRotate(angle) => wr::FilterOp::HueRotate(angle.radians()),
            GenericFilter::Invert(amount) => wr::FilterOp::Invert(amount.0),
            GenericFilter::Opacity(amount) => wr::FilterOp::Opacity(amount.0.into(), amount.0),
            GenericFilter::Saturate(amount) => wr::FilterOp::Saturate(amount.0),
            GenericFilter::Sepia(amount) => wr::FilterOp::Sepia(amount.0),
            // Statically check that DropShadow is impossible.
            GenericFilter::DropShadow(ref shadow) => match *shadow {},
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
            TransformStyle::Auto | TransformStyle::Flat => wr::TransformStyle::Flat,
            TransformStyle::Preserve3d => wr::TransformStyle::Preserve3D,
        }
    }
}

impl ToLayout for RGBA {
    type Type = wr::ColorF;
    fn to_layout(&self) -> Self::Type {
        wr::ColorF::new(
            self.red_f32(),
            self.green_f32(),
            self.blue_f32(),
            self.alpha_f32(),
        )
    }
}

impl ToLayout for Point2D<Au> {
    type Type = wr::LayoutPoint;
    fn to_layout(&self) -> Self::Type {
        wr::LayoutPoint::new(self.x.to_f32_px(), self.y.to_f32_px())
    }
}

impl ToLayout for Rect<Au> {
    type Type = wr::LayoutRect;
    fn to_layout(&self) -> Self::Type {
        wr::LayoutRect::new(self.origin.to_layout(), self.size.to_layout())
    }
}

impl ToLayout for SideOffsets2D<Au> {
    type Type = wr::BorderWidths;
    fn to_layout(&self) -> Self::Type {
        wr::BorderWidths {
            left: self.left.to_f32_px(),
            top: self.top.to_f32_px(),
            right: self.right.to_f32_px(),
            bottom: self.bottom.to_f32_px(),
        }
    }
}

impl ToLayout for Size2D<Au> {
    type Type = wr::LayoutSize;
    fn to_layout(&self) -> Self::Type {
        wr::LayoutSize::new(self.width.to_f32_px(), self.height.to_f32_px())
    }
}

impl ToLayout for Vector2D<Au> {
    type Type = wr::LayoutVector2D;
    fn to_layout(&self) -> Self::Type {
        wr::LayoutVector2D::new(self.x.to_f32_px(), self.y.to_f32_px())
    }
}

impl ToLayout for RepeatKeyword {
    type Type = wr::RepeatMode;
    fn to_layout(&self) -> Self::Type {
        match *self {
            RepeatKeyword::Stretch => wr::RepeatMode::Stretch,
            RepeatKeyword::Repeat => wr::RepeatMode::Repeat,
            RepeatKeyword::Round => wr::RepeatMode::Round,
            RepeatKeyword::Space => wr::RepeatMode::Space,
        }
    }
}
