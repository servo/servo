/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use canvas_traits::canvas::*;
use pixels::{SnapshotAlphaMode, SnapshotPixelFormat};
use style::color::AbsoluteColor;

use crate::backend::Convert;
use crate::canvas_data::Filter;

impl Convert<kurbo::Join> for LineJoinStyle {
    fn convert(self) -> kurbo::Join {
        match self {
            LineJoinStyle::Round => kurbo::Join::Round,
            LineJoinStyle::Bevel => kurbo::Join::Bevel,
            LineJoinStyle::Miter => kurbo::Join::Miter,
        }
    }
}

impl Convert<kurbo::Cap> for LineCapStyle {
    fn convert(self) -> kurbo::Cap {
        match self {
            LineCapStyle::Butt => kurbo::Cap::Butt,
            LineCapStyle::Round => kurbo::Cap::Round,
            LineCapStyle::Square => kurbo::Cap::Square,
        }
    }
}

impl Convert<peniko::Color> for AbsoluteColor {
    fn convert(self) -> peniko::Color {
        let srgb = self.into_srgb_legacy();
        peniko::Color::new([
            srgb.components.0,
            srgb.components.1,
            srgb.components.2,
            srgb.alpha,
        ])
    }
}

impl Convert<peniko::BlendMode> for CompositionOrBlending {
    fn convert(self) -> peniko::BlendMode {
        match self {
            CompositionOrBlending::Composition(composition_style) => {
                composition_style.convert().into()
            },
            CompositionOrBlending::Blending(blending_style) => blending_style.convert().into(),
        }
    }
}

impl Convert<peniko::Compose> for CompositionStyle {
    fn convert(self) -> peniko::Compose {
        match self {
            CompositionStyle::SourceIn => peniko::Compose::SrcIn,
            CompositionStyle::SourceOut => peniko::Compose::SrcOut,
            CompositionStyle::SourceOver => peniko::Compose::SrcOver,
            CompositionStyle::SourceAtop => peniko::Compose::SrcAtop,
            CompositionStyle::DestinationIn => peniko::Compose::DestIn,
            CompositionStyle::DestinationOut => peniko::Compose::DestOut,
            CompositionStyle::DestinationOver => peniko::Compose::DestOver,
            CompositionStyle::DestinationAtop => peniko::Compose::DestAtop,
            CompositionStyle::Copy => peniko::Compose::Copy,
            CompositionStyle::Lighter => peniko::Compose::Plus,
            CompositionStyle::Xor => peniko::Compose::Xor,
            CompositionStyle::Clear => peniko::Compose::Clear,
        }
    }
}

impl Convert<peniko::Mix> for BlendingStyle {
    fn convert(self) -> peniko::Mix {
        match self {
            BlendingStyle::Multiply => peniko::Mix::Multiply,
            BlendingStyle::Screen => peniko::Mix::Screen,
            BlendingStyle::Overlay => peniko::Mix::Overlay,
            BlendingStyle::Darken => peniko::Mix::Darken,
            BlendingStyle::Lighten => peniko::Mix::Lighten,
            BlendingStyle::ColorDodge => peniko::Mix::ColorDodge,
            BlendingStyle::ColorBurn => peniko::Mix::ColorBurn,
            BlendingStyle::HardLight => peniko::Mix::HardLight,
            BlendingStyle::SoftLight => peniko::Mix::SoftLight,
            BlendingStyle::Difference => peniko::Mix::Difference,
            BlendingStyle::Exclusion => peniko::Mix::Exclusion,
            BlendingStyle::Hue => peniko::Mix::Hue,
            BlendingStyle::Saturation => peniko::Mix::Saturation,
            BlendingStyle::Color => peniko::Mix::Color,
            BlendingStyle::Luminosity => peniko::Mix::Luminosity,
        }
    }
}

impl Convert<kurbo::Stroke> for LineOptions {
    fn convert(self) -> kurbo::Stroke {
        let LineOptions {
            width,
            cap_style,
            join_style,
            miter_limit,
            dash,
            dash_offset,
        } = self;
        kurbo::Stroke {
            width,
            join: join_style.convert(),
            miter_limit,
            start_cap: cap_style.convert(),
            end_cap: cap_style.convert(),
            dash_pattern: dash.iter().map(|x| *x as f64).collect(),
            dash_offset,
        }
    }
}

impl Convert<peniko::Brush> for FillOrStrokeStyle {
    fn convert(self) -> peniko::Brush {
        use canvas_traits::canvas::FillOrStrokeStyle::*;
        match self {
            Color(absolute_color) => peniko::Brush::Solid(absolute_color.convert()),
            LinearGradient(style) => {
                let start = kurbo::Point::new(style.x0, style.y0);
                let end = kurbo::Point::new(style.x1, style.y1);
                let mut gradient = peniko::Gradient::new_linear(start, end);
                gradient.stops = style.stops.convert();
                peniko::Brush::Gradient(gradient)
            },
            RadialGradient(style) => {
                let center1 = kurbo::Point::new(style.x0, style.y0);
                let center2 = kurbo::Point::new(style.x1, style.y1);
                let mut gradient = peniko::Gradient::new_two_point_radial(
                    center1,
                    style.r0 as f32,
                    center2,
                    style.r1 as f32,
                );
                gradient.stops = style.stops.convert();
                peniko::Brush::Gradient(gradient)
            },
            Surface(surface_style) => {
                let data = surface_style
                    .surface_data
                    .to_owned()
                    .to_vec(
                        Some(SnapshotAlphaMode::Transparent {
                            premultiplied: false,
                        }),
                        Some(SnapshotPixelFormat::RGBA),
                    )
                    .0;
                peniko::Brush::Image(peniko::Image {
                    data: peniko::Blob::from(data),
                    format: peniko::ImageFormat::Rgba8,
                    width: surface_style.surface_size.width,
                    height: surface_style.surface_size.height,
                    x_extend: if surface_style.repeat_x {
                        peniko::Extend::Repeat
                    } else {
                        peniko::Extend::Pad
                    },
                    y_extend: if surface_style.repeat_y {
                        peniko::Extend::Repeat
                    } else {
                        peniko::Extend::Pad
                    },
                    quality: peniko::ImageQuality::Low,
                    alpha: 1.0,
                })
            },
        }
    }
}

impl Convert<peniko::color::DynamicColor> for AbsoluteColor {
    fn convert(self) -> peniko::color::DynamicColor {
        peniko::color::DynamicColor::from_alpha_color(self.convert())
    }
}

impl Convert<peniko::ColorStop> for CanvasGradientStop {
    fn convert(self) -> peniko::ColorStop {
        peniko::ColorStop {
            offset: self.offset as f32,
            color: self.color.convert(),
        }
    }
}

impl Convert<peniko::ColorStops> for Vec<CanvasGradientStop> {
    fn convert(self) -> peniko::ColorStops {
        let mut stops = peniko::ColorStops(self.into_iter().map(|item| item.convert()).collect());
        // https://www.w3.org/html/test/results/2dcontext/annotated-spec/canvas.html#testrefs.2d.gradient.interpolate.overlap
        stops
            .0
            .sort_by(|a, b| a.offset.partial_cmp(&b.offset).unwrap());
        stops
    }
}

impl Convert<peniko::ImageQuality> for Filter {
    fn convert(self) -> peniko::ImageQuality {
        match self {
            Filter::Bilinear => peniko::ImageQuality::Medium,
            Filter::Nearest => peniko::ImageQuality::Low,
        }
    }
}
