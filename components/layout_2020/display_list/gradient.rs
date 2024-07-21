/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use style::color::mix::ColorInterpolationMethod;
use style::properties::ComputedValues;
use style::values::computed::image::{EndingShape, Gradient, LineDirection};
use style::values::computed::{
    Angle, AngleOrPercentage, Color, Length, LengthPercentage, Position,
};
use style::values::generics::image::{
    Circle, ColorStop, Ellipse, GradientFlags, GradientItem, ShapeExtent,
};
use webrender_api::{self as wr, units};
use wr::ColorF;

pub(super) fn build(
    style: &ComputedValues,
    gradient: &Gradient,
    layer: &super::background::BackgroundLayer,
    builder: &mut super::DisplayListBuilder,
) {
    match gradient {
        Gradient::Linear {
            ref items,
            ref direction,
            ref color_interpolation_method,
            ref flags,
            compat_mode: _,
        } => build_linear(
            style,
            items,
            direction,
            color_interpolation_method,
            *flags,
            layer,
            builder,
        ),
        Gradient::Radial {
            ref shape,
            ref position,
            ref color_interpolation_method,
            ref items,
            ref flags,
            compat_mode: _,
        } => build_radial(
            style,
            items,
            shape,
            position,
            color_interpolation_method,
            *flags,
            layer,
            builder,
        ),
        Gradient::Conic {
            angle,
            position,
            color_interpolation_method,
            items,
            flags,
        } => build_conic(
            style,
            *angle,
            position,
            *color_interpolation_method,
            items,
            *flags,
            layer,
            builder,
        ),
    }
}

/// <https://drafts.csswg.org/css-images-3/#linear-gradients>
pub(super) fn build_linear(
    style: &ComputedValues,
    items: &[GradientItem<Color, LengthPercentage>],
    line_direction: &LineDirection,
    _color_interpolation_method: &ColorInterpolationMethod,
    flags: GradientFlags,
    layer: &super::background::BackgroundLayer,
    builder: &mut super::DisplayListBuilder,
) {
    use style::values::specified::position::HorizontalPositionKeyword::*;
    use style::values::specified::position::VerticalPositionKeyword::*;
    use units::LayoutVector2D as Vec2;
    let gradient_box = layer.tile_size;

    // A vector of length 1.0 in the direction of the gradient line
    let direction = match line_direction {
        LineDirection::Horizontal(Right) => Vec2::new(1., 0.),
        LineDirection::Vertical(Top) => Vec2::new(0., -1.),
        LineDirection::Horizontal(Left) => Vec2::new(-1., 0.),
        LineDirection::Vertical(Bottom) => Vec2::new(0., 1.),

        LineDirection::Angle(angle) => {
            let radians = angle.radians();
            // “`0deg` points upward,
            //  and positive angles represent clockwise rotation,
            //  so `90deg` point toward the right.”
            Vec2::new(radians.sin(), -radians.cos())
        },

        LineDirection::Corner(horizontal, vertical) => {
            // “If the argument instead specifies a corner of the box such as `to top left`,
            //  the gradient line must be angled such that it points
            //  into the same quadrant as the specified corner,
            //  and is perpendicular to a line intersecting
            //  the two neighboring corners of the gradient box.”

            // Note that that last line is a diagonal of the gradient box rectangle,
            // since two neighboring corners of a third corner
            // are necessarily opposite to each other.

            // `{ x: gradient_box.width, y: gradient_box.height }` is such a diagonal vector,
            // from the bottom left corner to the top right corner of the gradient box.
            // (Both coordinates are positive.)
            // Changing either or both signs produces the other three (oriented) diagonals.

            // Swapping the coordinates `{ x: gradient_box.height, y: gradient_box.height }`
            // produces a vector perpendicular to some diagonal of the rectangle.
            // Finally, we choose the sign of each cartesian coordinate
            // such that our vector points to the desired quadrant.

            let x = match horizontal {
                Right => gradient_box.height,
                Left => -gradient_box.height,
            };
            let y = match vertical {
                Top => gradient_box.width,
                Bottom => -gradient_box.width,
            };

            // `{ x, y }` is now a vector of arbitrary length
            // with the same direction as the gradient line.
            // This normalizes the length to 1.0:
            Vec2::new(x, y).normalize()
        },
    };

    // This formula is given as `abs(W * sin(A)) + abs(H * cos(A))` in a note in the spec, under
    // https://drafts.csswg.org/css-images-3/#linear-gradient-syntax
    //
    // Sketch of a proof:
    //
    // * Take the top side of the gradient box rectangle. It is a segment of length `W`
    // * Project onto the gradient line. You get a segment of length `abs(W * sin(A))`
    // * Similarly, the left side of the rectangle (length `H`)
    //   projects to a segment of length `abs(H * cos(A))`
    // * These two segments add up to exactly the gradient line.
    //
    // See the illustration in the example under
    // https://drafts.csswg.org/css-images-3/#linear-gradient-syntax
    let gradient_line_length =
        (gradient_box.width * direction.x).abs() + (gradient_box.height * direction.y).abs();

    let half_gradient_line = direction * (gradient_line_length / 2.);
    let center = (gradient_box / 2.).to_vector().to_point();
    let start_point = center - half_gradient_line;
    let end_point = center + half_gradient_line;

    let mut color_stops =
        gradient_items_to_color_stops(style, items, Length::new(gradient_line_length));
    let stops = fixup_stops(&mut color_stops);
    let extend_mode = if flags.contains(GradientFlags::REPEATING) {
        wr::ExtendMode::Repeat
    } else {
        wr::ExtendMode::Clamp
    };
    let linear_gradient = builder
        .wr()
        .create_gradient(start_point, end_point, stops, extend_mode);
    builder.wr().push_gradient(
        &layer.common,
        layer.bounds,
        linear_gradient,
        layer.tile_size,
        layer.tile_spacing,
    )
}

/// <https://drafts.csswg.org/css-images-3/#radial-gradients>
#[allow(clippy::too_many_arguments)]
pub(super) fn build_radial(
    style: &ComputedValues,
    items: &[GradientItem<Color, LengthPercentage>],
    shape: &EndingShape,
    center: &Position,
    _color_interpolation_method: &ColorInterpolationMethod,
    flags: GradientFlags,
    layer: &super::background::BackgroundLayer,
    builder: &mut super::DisplayListBuilder,
) {
    let gradient_box = layer.tile_size;
    let center = units::LayoutPoint::new(
        center
            .horizontal
            .percentage_relative_to(Length::new(gradient_box.width))
            .px(),
        center
            .vertical
            .percentage_relative_to(Length::new(gradient_box.height))
            .px(),
    );
    let radii = match shape {
        EndingShape::Circle(circle) => {
            let radius = match circle {
                Circle::Radius(r) => r.0.px(),
                Circle::Extent(extent) => match extent {
                    ShapeExtent::ClosestSide | ShapeExtent::Contain => {
                        let vec = abs_vector_to_corner(gradient_box, center, f32::min);
                        vec.x.min(vec.y)
                    },
                    ShapeExtent::FarthestSide => {
                        let vec = abs_vector_to_corner(gradient_box, center, f32::max);
                        vec.x.max(vec.y)
                    },
                    ShapeExtent::ClosestCorner => {
                        abs_vector_to_corner(gradient_box, center, f32::min).length()
                    },
                    ShapeExtent::FarthestCorner | ShapeExtent::Cover => {
                        abs_vector_to_corner(gradient_box, center, f32::max).length()
                    },
                },
            };
            units::LayoutSize::new(radius, radius)
        },
        EndingShape::Ellipse(Ellipse::Radii(rx, ry)) => units::LayoutSize::new(
            rx.0.percentage_relative_to(Length::new(gradient_box.width))
                .px(),
            ry.0.percentage_relative_to(Length::new(gradient_box.height))
                .px(),
        ),
        EndingShape::Ellipse(Ellipse::Extent(extent)) => match extent {
            ShapeExtent::ClosestSide | ShapeExtent::Contain => {
                abs_vector_to_corner(gradient_box, center, f32::min).to_size()
            },
            ShapeExtent::FarthestSide => {
                abs_vector_to_corner(gradient_box, center, f32::max).to_size()
            },
            ShapeExtent::ClosestCorner => {
                abs_vector_to_corner(gradient_box, center, f32::min).to_size() *
                    (std::f32::consts::FRAC_1_SQRT_2 * 2.0)
            },
            ShapeExtent::FarthestCorner | ShapeExtent::Cover => {
                abs_vector_to_corner(gradient_box, center, f32::max).to_size() *
                    (std::f32::consts::FRAC_1_SQRT_2 * 2.0)
            },
        },
    };

    /// Returns the distance to the nearest or farthest sides in the respective dimension,
    /// depending on `select`.
    fn abs_vector_to_corner(
        gradient_box: units::LayoutSize,
        center: units::LayoutPoint,
        select: impl Fn(f32, f32) -> f32,
    ) -> units::LayoutVector2D {
        let left = center.x.abs();
        let top = center.y.abs();
        let right = (gradient_box.width - center.x).abs();
        let bottom = (gradient_box.height - center.y).abs();
        units::LayoutVector2D::new(select(left, right), select(top, bottom))
    }

    // “The gradient line’s starting point is at the center of the gradient,
    //  and it extends toward the right, with the ending point on the point
    //  where the gradient line intersects the ending shape.”
    let gradient_line_length = radii.width;

    let mut color_stops =
        gradient_items_to_color_stops(style, items, Length::new(gradient_line_length));
    let stops = fixup_stops(&mut color_stops);
    let extend_mode = if flags.contains(GradientFlags::REPEATING) {
        wr::ExtendMode::Repeat
    } else {
        wr::ExtendMode::Clamp
    };
    let radial_gradient = builder
        .wr()
        .create_radial_gradient(center, radii, stops, extend_mode);
    builder.wr().push_radial_gradient(
        &layer.common,
        layer.bounds,
        radial_gradient,
        layer.tile_size,
        layer.tile_spacing,
    )
}

/// <https://drafts.csswg.org/css-images-4/#conic-gradients>
#[allow(clippy::too_many_arguments)]
fn build_conic(
    style: &ComputedValues,
    angle: Angle,
    center: &Position,
    _color_interpolation_method: ColorInterpolationMethod,
    items: &[GradientItem<Color, AngleOrPercentage>],
    flags: GradientFlags,
    layer: &super::background::BackgroundLayer,
    builder: &mut super::DisplayListBuilder<'_>,
) {
    let gradient_box = layer.tile_size;
    let center = units::LayoutPoint::new(
        center
            .horizontal
            .percentage_relative_to(Length::new(gradient_box.width))
            .px(),
        center
            .vertical
            .percentage_relative_to(Length::new(gradient_box.height))
            .px(),
    );
    let mut color_stops = conic_gradient_items_to_color_stops(style, items);
    let stops = fixup_stops(&mut color_stops);
    let extend_mode = if flags.contains(GradientFlags::REPEATING) {
        wr::ExtendMode::Repeat
    } else {
        wr::ExtendMode::Clamp
    };
    let conic_gradient =
        builder
            .wr()
            .create_conic_gradient(center, angle.radians(), stops, extend_mode);
    builder.wr().push_conic_gradient(
        &layer.common,
        layer.bounds,
        conic_gradient,
        layer.tile_size,
        layer.tile_spacing,
    )
}

fn conic_gradient_items_to_color_stops(
    style: &ComputedValues,
    items: &[GradientItem<Color, AngleOrPercentage>],
) -> Vec<ColorStop<ColorF, f32>> {
    // Remove color transititon hints, which are not supported yet.
    // https://drafts.csswg.org/css-images-4/#color-transition-hint
    //
    // This gives an approximation of the gradient that might be visibly wrong,
    // but maybe better than not parsing that value at all?
    // It’s debatble whether that’s better or worse
    // than not parsing and allowing authors to set a fallback.
    // Either way, the best outcome is to add support.
    // Gecko does so by approximating the non-linear interpolation
    // by up to 10 piece-wise linear segments (9 intermediate color stops)
    items
        .iter()
        .filter_map(|item| {
            match item {
                GradientItem::SimpleColorStop(color) => Some(ColorStop {
                    color: super::rgba(style.resolve_color(color.clone())),
                    position: None,
                }),
                GradientItem::ComplexColorStop { color, position } => Some(ColorStop {
                    color: super::rgba(style.resolve_color(color.clone())),
                    position: match position {
                        AngleOrPercentage::Percentage(percentage) => Some(percentage.0),
                        AngleOrPercentage::Angle(angle) => Some(angle.degrees() / 360.),
                    },
                }),
                // FIXME: approximate like in:
                // https://searchfox.org/mozilla-central/rev/f98dad153b59a985efd4505912588d4651033395/layout/painting/nsCSSRenderingGradients.cpp#315-391
                GradientItem::InterpolationHint(_) => None,
            }
        })
        .collect()
}

fn gradient_items_to_color_stops(
    style: &ComputedValues,
    items: &[GradientItem<Color, LengthPercentage>],
    gradient_line_length: Length,
) -> Vec<ColorStop<ColorF, f32>> {
    // Remove color transititon hints, which are not supported yet.
    // https://drafts.csswg.org/css-images-4/#color-transition-hint
    //
    // This gives an approximation of the gradient that might be visibly wrong,
    // but maybe better than not parsing that value at all?
    // It’s debatble whether that’s better or worse
    // than not parsing and allowing authors to set a fallback.
    // Either way, the best outcome is to add support.
    // Gecko does so by approximating the non-linear interpolation
    // by up to 10 piece-wise linear segments (9 intermediate color stops)
    items
        .iter()
        .filter_map(|item| {
            match item {
                GradientItem::SimpleColorStop(color) => Some(ColorStop {
                    color: super::rgba(style.resolve_color(color.clone())),
                    position: None,
                }),
                GradientItem::ComplexColorStop { color, position } => Some(ColorStop {
                    color: super::rgba(style.resolve_color(color.clone())),
                    position: Some(if gradient_line_length.px() == 0. {
                        0.
                    } else {
                        position.percentage_relative_to(gradient_line_length).px() /
                            gradient_line_length.px()
                    }),
                }),
                // FIXME: approximate like in:
                // https://searchfox.org/mozilla-central/rev/f98dad153b59a985efd4505912588d4651033395/layout/painting/nsCSSRenderingGradients.cpp#315-391
                GradientItem::InterpolationHint(_) => None,
            }
        })
        .collect()
}

/// <https://drafts.csswg.org/css-images-4/#color-stop-fixup>
fn fixup_stops(stops: &mut [ColorStop<ColorF, f32>]) -> Vec<wr::GradientStop> {
    assert!(stops.len() >= 2);

    // https://drafts.csswg.org/css-images-4/#color-stop-fixup
    if let first_position @ None = &mut stops.first_mut().unwrap().position {
        *first_position = Some(0.);
    }
    if let last_position @ None = &mut stops.last_mut().unwrap().position {
        *last_position = Some(1.);
    }

    let mut iter = stops.iter_mut();
    let mut max_so_far = iter.next().unwrap().position.unwrap();
    for stop in iter {
        if let Some(position) = &mut stop.position {
            if *position < max_so_far {
                *position = max_so_far
            } else {
                max_so_far = *position
            }
        }
    }

    let mut wr_stops = Vec::with_capacity(stops.len());
    let mut iter = stops.iter().enumerate();
    let (_, first) = iter.next().unwrap();
    let first_stop_position = first.position.unwrap();
    wr_stops.push(wr::GradientStop {
        offset: first_stop_position,
        color: first.color,
    });

    let mut last_positioned_stop_index = 0;
    let mut last_positioned_stop_position = first_stop_position;
    for (i, stop) in iter {
        if let Some(position) = stop.position {
            let step_count = i - last_positioned_stop_index;
            if step_count > 1 {
                let step = (position - last_positioned_stop_position) / step_count as f32;
                for j in 1..step_count {
                    let color = stops[last_positioned_stop_index + j].color;
                    let offset = last_positioned_stop_position + j as f32 * step;
                    wr_stops.push(wr::GradientStop { offset, color })
                }
            }
            last_positioned_stop_index = i;
            last_positioned_stop_position = position;
            wr_stops.push(wr::GradientStop {
                offset: position,
                color: stop.color,
            })
        }
    }

    wr_stops
}
