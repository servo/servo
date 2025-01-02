/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use app_units::Au;
use euclid::default::{Point2D, Size2D, Vector2D};
use style::color::mix::ColorInterpolationMethod;
use style::properties::ComputedValues;
use style::values::computed::image::{EndingShape, LineDirection};
use style::values::computed::{Angle, Color, LengthPercentage, Percentage, Position};
use style::values::generics::image::{
    Circle, ColorStop, Ellipse, GradientFlags, GradientItem, ShapeExtent,
};
use webrender_api::{ExtendMode, Gradient, GradientBuilder, GradientStop, RadialGradient};

use crate::display_list::ToLayout;

/// A helper data structure for gradients.
#[derive(Clone, Copy)]
struct StopRun {
    start_offset: f32,
    end_offset: f32,
    start_index: usize,
    stop_count: usize,
}

/// Determines the radius of a circle if it was not explicitly provided.
/// <https://drafts.csswg.org/css-images-3/#typedef-size>
fn circle_size_keyword(
    keyword: ShapeExtent,
    size: &Size2D<Au>,
    center: &Point2D<Au>,
) -> Size2D<Au> {
    let radius = match keyword {
        ShapeExtent::ClosestSide | ShapeExtent::Contain => {
            let dist = distance_to_sides(size, center, ::std::cmp::min);
            ::std::cmp::min(dist.width, dist.height)
        },
        ShapeExtent::FarthestSide => {
            let dist = distance_to_sides(size, center, ::std::cmp::max);
            ::std::cmp::max(dist.width, dist.height)
        },
        ShapeExtent::ClosestCorner => distance_to_corner(size, center, ::std::cmp::min),
        ShapeExtent::FarthestCorner | ShapeExtent::Cover => {
            distance_to_corner(size, center, ::std::cmp::max)
        },
    };
    Size2D::new(radius, radius)
}

/// Returns the radius for an ellipse with the same ratio as if it was matched to the sides.
fn ellipse_radius<F>(size: &Size2D<Au>, center: &Point2D<Au>, cmp: F) -> Size2D<Au>
where
    F: Fn(Au, Au) -> Au,
{
    let dist = distance_to_sides(size, center, cmp);
    Size2D::new(
        dist.width.scale_by(::std::f32::consts::FRAC_1_SQRT_2 * 2.0),
        dist.height
            .scale_by(::std::f32::consts::FRAC_1_SQRT_2 * 2.0),
    )
}

/// Determines the radius of an ellipse if it was not explicitly provided.
/// <https://drafts.csswg.org/css-images-3/#typedef-size>
fn ellipse_size_keyword(
    keyword: ShapeExtent,
    size: &Size2D<Au>,
    center: &Point2D<Au>,
) -> Size2D<Au> {
    match keyword {
        ShapeExtent::ClosestSide | ShapeExtent::Contain => {
            distance_to_sides(size, center, ::std::cmp::min)
        },
        ShapeExtent::FarthestSide => distance_to_sides(size, center, ::std::cmp::max),
        ShapeExtent::ClosestCorner => ellipse_radius(size, center, ::std::cmp::min),
        ShapeExtent::FarthestCorner | ShapeExtent::Cover => {
            ellipse_radius(size, center, ::std::cmp::max)
        },
    }
}

fn convert_gradient_stops(
    style: &ComputedValues,
    gradient_items: &[GradientItem<Color, LengthPercentage>],
    total_length: Au,
) -> GradientBuilder {
    // Determine the position of each stop per CSS-IMAGES § 3.4.

    // Only keep the color stops, discard the color interpolation hints.
    let mut stop_items = gradient_items
        .iter()
        .filter_map(|item| match item {
            GradientItem::SimpleColorStop(color) => Some(ColorStop {
                color,
                position: None,
            }),
            GradientItem::ComplexColorStop {
                color,
                ref position,
            } => Some(ColorStop {
                color,
                position: Some(position.clone()),
            }),
            _ => None,
        })
        .collect::<Vec<_>>();

    assert!(stop_items.len() >= 2);

    // Run the algorithm from
    // https://drafts.csswg.org/css-images-3/#color-stop-syntax

    // Step 1:
    // If the first color stop does not have a position, set its position to 0%.
    {
        let first = stop_items.first_mut().unwrap();
        if first.position.is_none() {
            first.position = Some(LengthPercentage::new_percent(Percentage(0.)));
        }
    }
    // If the last color stop does not have a position, set its position to 100%.
    {
        let last = stop_items.last_mut().unwrap();
        if last.position.is_none() {
            last.position = Some(LengthPercentage::new_percent(Percentage(1.0)));
        }
    }

    // Step 2: Move any stops placed before earlier stops to the
    // same position as the preceding stop.
    //
    // FIXME(emilio): Once we know the offsets, it seems like converting the
    // positions to absolute at once then process that would be cheaper.
    let mut last_stop_position = stop_items
        .first()
        .unwrap()
        .position
        .as_ref()
        .unwrap()
        .clone();
    for stop in stop_items.iter_mut().skip(1) {
        if let Some(ref pos) = stop.position {
            if position_to_offset(&last_stop_position, total_length) >
                position_to_offset(pos, total_length)
            {
                stop.position = Some(last_stop_position);
            }
            last_stop_position = stop.position.as_ref().unwrap().clone();
        }
    }

    // Step 3: Evenly space stops without position.
    let mut stops = GradientBuilder::new();
    let mut stop_run = None;
    for (i, stop) in stop_items.iter().enumerate() {
        let offset = match stop.position {
            None => {
                if stop_run.is_none() {
                    // Initialize a new stop run.
                    // `unwrap()` here should never fail because this is the beginning of
                    // a stop run, which is always bounded by a length or percentage.
                    let start_offset = position_to_offset(
                        stop_items[i - 1].position.as_ref().unwrap(),
                        total_length,
                    );
                    // `unwrap()` here should never fail because this is the end of
                    // a stop run, which is always bounded by a length or percentage.
                    let (end_index, end_stop) = stop_items[(i + 1)..]
                        .iter()
                        .enumerate()
                        .find(|(_, stop)| stop.position.is_some())
                        .unwrap();
                    let end_offset =
                        position_to_offset(end_stop.position.as_ref().unwrap(), total_length);
                    stop_run = Some(StopRun {
                        start_offset,
                        end_offset,
                        start_index: i - 1,
                        stop_count: end_index,
                    })
                }

                let stop_run = stop_run.unwrap();
                let stop_run_length = stop_run.end_offset - stop_run.start_offset;
                stop_run.start_offset +
                    stop_run_length * (i - stop_run.start_index) as f32 /
                        ((2 + stop_run.stop_count) as f32)
            },
            Some(ref position) => {
                stop_run = None;
                position_to_offset(position, total_length)
            },
        };
        assert!(offset.is_finite());
        stops.push(GradientStop {
            offset,
            color: style.resolve_color(stop.color.clone()).to_layout(),
        })
    }
    stops
}

fn extend_mode(repeating: bool) -> ExtendMode {
    if repeating {
        ExtendMode::Repeat
    } else {
        ExtendMode::Clamp
    }
}
/// Returns the the distance to the nearest or farthest corner depending on the comperator.
fn distance_to_corner<F>(size: &Size2D<Au>, center: &Point2D<Au>, cmp: F) -> Au
where
    F: Fn(Au, Au) -> Au,
{
    let dist = distance_to_sides(size, center, cmp);
    Au::from_f32_px(dist.width.to_f32_px().hypot(dist.height.to_f32_px()))
}

/// Returns the distance to the nearest or farthest sides depending on the comparator.
///
/// The first return value is horizontal distance the second vertical distance.
fn distance_to_sides<F>(size: &Size2D<Au>, center: &Point2D<Au>, cmp: F) -> Size2D<Au>
where
    F: Fn(Au, Au) -> Au,
{
    let top_side = center.y;
    let right_side = size.width - center.x;
    let bottom_side = size.height - center.y;
    let left_side = center.x;
    Size2D::new(cmp(left_side, right_side), cmp(top_side, bottom_side))
}

fn position_to_offset(position: &LengthPercentage, total_length: Au) -> f32 {
    if total_length == Au(0) {
        return 0.0;
    }
    position.to_used_value(total_length).0 as f32 / total_length.0 as f32
}

pub fn linear(
    style: &ComputedValues,
    size: Size2D<Au>,
    stops: &[GradientItem<Color, LengthPercentage>],
    direction: LineDirection,
    _color_interpolation_method: &ColorInterpolationMethod,
    flags: GradientFlags,
) -> (Gradient, Vec<GradientStop>) {
    use style::values::specified::position::HorizontalPositionKeyword::*;
    use style::values::specified::position::VerticalPositionKeyword::*;
    let repeating = flags.contains(GradientFlags::REPEATING);
    let angle = match direction {
        LineDirection::Angle(angle) => angle.radians(),
        LineDirection::Horizontal(x) => match x {
            Left => Angle::from_degrees(270.).radians(),
            Right => Angle::from_degrees(90.).radians(),
        },
        LineDirection::Vertical(y) => match y {
            Top => Angle::from_degrees(0.).radians(),
            Bottom => Angle::from_degrees(180.).radians(),
        },
        LineDirection::Corner(horizontal, vertical) => {
            // This the angle for one of the diagonals of the box. Our angle
            // will either be this one, this one + PI, or one of the other
            // two perpendicular angles.
            let atan = (size.height.to_f32_px() / size.width.to_f32_px()).atan();
            match (horizontal, vertical) {
                (Right, Bottom) => ::std::f32::consts::PI - atan,
                (Left, Bottom) => ::std::f32::consts::PI + atan,
                (Right, Top) => atan,
                (Left, Top) => -atan,
            }
        },
    };

    // Get correct gradient line length, based on:
    // https://drafts.csswg.org/css-images-3/#linear-gradients
    let dir = Point2D::new(angle.sin(), -angle.cos());

    let line_length =
        (dir.x * size.width.to_f32_px()).abs() + (dir.y * size.height.to_f32_px()).abs();

    let inv_dir_length = 1.0 / (dir.x * dir.x + dir.y * dir.y).sqrt();

    // This is the vector between the center and the ending point; i.e. half
    // of the distance between the starting point and the ending point.
    let delta = Vector2D::new(
        Au::from_f32_px(dir.x * inv_dir_length * line_length / 2.0),
        Au::from_f32_px(dir.y * inv_dir_length * line_length / 2.0),
    );

    // This is the length of the gradient line.
    let length = Au::from_f32_px((delta.x.to_f32_px() * 2.0).hypot(delta.y.to_f32_px() * 2.0));

    let mut builder = convert_gradient_stops(style, stops, length);

    let center = Point2D::new(size.width / 2, size.height / 2);

    (
        builder.gradient(
            (center - delta).to_layout(),
            (center + delta).to_layout(),
            extend_mode(repeating),
        ),
        builder.into_stops(),
    )
}

pub fn radial(
    style: &ComputedValues,
    size: Size2D<Au>,
    stops: &[GradientItem<Color, LengthPercentage>],
    shape: &EndingShape,
    center: &Position,
    _color_interpolation_method: &ColorInterpolationMethod,
    flags: GradientFlags,
) -> (RadialGradient, Vec<GradientStop>) {
    let repeating = flags.contains(GradientFlags::REPEATING);
    let center = Point2D::new(
        center.horizontal.to_used_value(size.width),
        center.vertical.to_used_value(size.height),
    );
    let radius = match shape {
        EndingShape::Circle(Circle::Radius(length)) => {
            let length = Au::from(*length);
            Size2D::new(length, length)
        },
        EndingShape::Circle(Circle::Extent(extent)) => circle_size_keyword(*extent, &size, &center),
        EndingShape::Ellipse(Ellipse::Radii(x, y)) => {
            Size2D::new(x.to_used_value(size.width), y.to_used_value(size.height))
        },
        EndingShape::Ellipse(Ellipse::Extent(extent)) => {
            ellipse_size_keyword(*extent, &size, &center)
        },
    };

    let mut builder = convert_gradient_stops(style, stops, radius.width);
    (
        builder.radial_gradient(
            center.to_layout(),
            radius.to_layout(),
            extend_mode(repeating),
        ),
        builder.into_stops(),
    )
}
