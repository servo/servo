/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Calculations for CSS images, backgrounds and borders.
//!
//! * [CSS Images Module Level 3](https://drafts.csswg.org/css-images-3/)
//! * [CSS Backgrounds and Borders Module Level 3](https://drafts.csswg.org/css-backgrounds-3/)

#![deny(unsafe_code)]

// FIXME(rust-lang/rust#26264): Remove GenericEndingShape and GenericGradientItem.

use app_units::Au;
use display_list::ToLayout;
use euclid::{Point2D, Rect, SideOffsets2D, Size2D, Vector2D};
use gfx::display_list::{self, BorderDetails, WebRenderImageInfo};
use model::{self, MaybeAuto};
use style::computed_values::background_attachment::single_value::T as BackgroundAttachment;
use style::computed_values::background_clip::single_value::T as BackgroundClip;
use style::computed_values::background_origin::single_value::T as BackgroundOrigin;
use style::properties::style_structs::{self, Background};
use style::values::computed::{Angle, GradientItem};
use style::values::computed::{LengthOrPercentage, LengthOrPercentageOrAuto};
use style::values::computed::{NumberOrPercentage, Percentage, Position};
use style::values::computed::image::{EndingShape, LineDirection};
use style::values::generics::background::BackgroundSize;
use style::values::generics::image::{Circle, Ellipse, ShapeExtent};
use style::values::generics::image::EndingShape as GenericEndingShape;
use style::values::generics::image::GradientItem as GenericGradientItem;
use style::values::specified::background::BackgroundRepeatKeyword;
use style::values::specified::position::{X, Y};
use webrender_api::{BorderRadius, BorderSide, BorderStyle, ColorF, ExtendMode, ImageBorder};
use webrender_api::{GradientStop, LayoutSize, NinePatchDescriptor, NormalBorder};

/// A helper data structure for gradients.
#[derive(Clone, Copy)]
struct StopRun {
    start_offset: f32,
    end_offset: f32,
    start_index: usize,
    stop_count: usize,
}

/// Placment information for both image and gradient backgrounds.
#[derive(Clone, Copy, Debug)]
pub struct BackgroundPlacement {
    /// Rendering bounds. The background will start in the uppper-left corner
    /// and fill the whole area.
    pub bounds: Rect<Au>,
    /// Background tile size. Some backgrounds are repeated. These are the
    /// dimensions of a single image of the background.
    pub tile_size: Size2D<Au>,
    /// Spacing between tiles. Some backgrounds are not repeated seamless
    /// but have seams between them like tiles in real life.
    pub tile_spacing: Size2D<Au>,
    /// A clip area. While the background is rendered according to all the
    /// measures above it is only shown within these bounds.
    pub css_clip: Rect<Au>,
    /// Whether or not the background is fixed to the viewport.
    pub fixed: bool,
}

trait ResolvePercentage {
    fn resolve(&self, length: u32) -> u32;
}

impl ResolvePercentage for NumberOrPercentage {
    fn resolve(&self, length: u32) -> u32 {
        match *self {
            NumberOrPercentage::Percentage(p) => (p.0 * length as f32).round() as u32,
            NumberOrPercentage::Number(n) => n.round() as u32,
        }
    }
}

/// Access element at index modulo the array length.
///
/// Obviously it does not work with empty arrays.
///
/// This is used for multiple layered background images.
/// See: https://drafts.csswg.org/css-backgrounds-3/#layering
pub fn get_cyclic<T>(arr: &[T], index: usize) -> &T {
    &arr[index % arr.len()]
}

/// For a given area and an image compute how big the
/// image should be displayed on the background.
fn compute_background_image_size(
    bg_size: BackgroundSize<LengthOrPercentageOrAuto>,
    bounds_size: Size2D<Au>,
    intrinsic_size: Option<Size2D<Au>>,
) -> Size2D<Au> {
    match intrinsic_size {
        None => match bg_size {
            BackgroundSize::Cover | BackgroundSize::Contain => bounds_size,
            BackgroundSize::Explicit { width, height } => Size2D::new(
                MaybeAuto::from_style(width, bounds_size.width)
                    .specified_or_default(bounds_size.width),
                MaybeAuto::from_style(height, bounds_size.height)
                    .specified_or_default(bounds_size.height),
            ),
        },
        Some(own_size) => {
            // If `image_aspect_ratio` < `bounds_aspect_ratio`, the image is tall; otherwise, it is
            // wide.
            let image_aspect_ratio = own_size.width.to_f32_px() / own_size.height.to_f32_px();
            let bounds_aspect_ratio =
                bounds_size.width.to_f32_px() / bounds_size.height.to_f32_px();
            match (bg_size, image_aspect_ratio < bounds_aspect_ratio) {
                (BackgroundSize::Contain, false) | (BackgroundSize::Cover, true) => Size2D::new(
                    bounds_size.width,
                    bounds_size.width.scale_by(image_aspect_ratio.recip()),
                ),
                (BackgroundSize::Contain, true) | (BackgroundSize::Cover, false) => Size2D::new(
                    bounds_size.height.scale_by(image_aspect_ratio),
                    bounds_size.height,
                ),
                (
                    BackgroundSize::Explicit {
                        width,
                        height: LengthOrPercentageOrAuto::Auto,
                    },
                    _,
                ) => {
                    let width = MaybeAuto::from_style(width, bounds_size.width)
                        .specified_or_default(own_size.width);
                    Size2D::new(width, width.scale_by(image_aspect_ratio.recip()))
                },
                (
                    BackgroundSize::Explicit {
                        width: LengthOrPercentageOrAuto::Auto,
                        height,
                    },
                    _,
                ) => {
                    let height = MaybeAuto::from_style(height, bounds_size.height)
                        .specified_or_default(own_size.height);
                    Size2D::new(height.scale_by(image_aspect_ratio), height)
                },
                (BackgroundSize::Explicit { width, height }, _) => Size2D::new(
                    MaybeAuto::from_style(width, bounds_size.width)
                        .specified_or_default(own_size.width),
                    MaybeAuto::from_style(height, bounds_size.height)
                        .specified_or_default(own_size.height),
                ),
            }
        },
    }
}

/// Determines where to place an element background image or gradient.
///
/// Photos have their resolution as intrinsic size while gradients have
/// no intrinsic size.
pub fn compute_background_placement(
    bg: &Background,
    viewport_size: Size2D<Au>,
    absolute_bounds: Rect<Au>,
    intrinsic_size: Option<Size2D<Au>>,
    border: SideOffsets2D<Au>,
    border_padding: SideOffsets2D<Au>,
    index: usize,
) -> BackgroundPlacement {
    let bg_attachment = *get_cyclic(&bg.background_attachment.0, index);
    let bg_clip = *get_cyclic(&bg.background_clip.0, index);
    let bg_origin = *get_cyclic(&bg.background_origin.0, index);
    let bg_position_x = get_cyclic(&bg.background_position_x.0, index);
    let bg_position_y = get_cyclic(&bg.background_position_y.0, index);
    let bg_repeat = get_cyclic(&bg.background_repeat.0, index);
    let bg_size = *get_cyclic(&bg.background_size.0, index);

    let css_clip = match bg_clip {
        BackgroundClip::BorderBox => absolute_bounds,
        BackgroundClip::PaddingBox => absolute_bounds.inner_rect(border),
        BackgroundClip::ContentBox => absolute_bounds.inner_rect(border_padding),
    };

    let mut fixed = false;
    let mut bounds = match bg_attachment {
        BackgroundAttachment::Scroll => match bg_origin {
            BackgroundOrigin::BorderBox => absolute_bounds,
            BackgroundOrigin::PaddingBox => absolute_bounds.inner_rect(border),
            BackgroundOrigin::ContentBox => absolute_bounds.inner_rect(border_padding),
        },
        BackgroundAttachment::Fixed => {
            fixed = true;
            Rect::new(Point2D::origin(), viewport_size)
        },
    };

    let mut tile_size = compute_background_image_size(bg_size, bounds.size, intrinsic_size);

    let mut tile_spacing = Size2D::zero();
    let own_position = bounds.size - tile_size;
    let pos_x = bg_position_x.to_used_value(own_position.width);
    let pos_y = bg_position_y.to_used_value(own_position.height);
    tile_image_axis(
        bg_repeat.0,
        &mut bounds.origin.x,
        &mut bounds.size.width,
        &mut tile_size.width,
        &mut tile_spacing.width,
        pos_x,
        css_clip.origin.x,
        css_clip.size.width,
    );
    tile_image_axis(
        bg_repeat.1,
        &mut bounds.origin.y,
        &mut bounds.size.height,
        &mut tile_size.height,
        &mut tile_spacing.height,
        pos_y,
        css_clip.origin.y,
        css_clip.size.height,
    );

    BackgroundPlacement {
        bounds,
        tile_size,
        tile_spacing,
        css_clip,
        fixed,
    }
}

fn tile_image_round(
    position: &mut Au,
    size: &mut Au,
    absolute_anchor_origin: Au,
    image_size: &mut Au,
) {
    if *size == Au(0) || *image_size == Au(0) {
        *position = Au(0);
        *size = Au(0);
        return;
    }

    let number_of_tiles = (size.to_f32_px() / image_size.to_f32_px()).round().max(1.0);
    *image_size = *size / (number_of_tiles as i32);
    tile_image(position, size, absolute_anchor_origin, *image_size);
}

fn tile_image_spaced(
    position: &mut Au,
    size: &mut Au,
    tile_spacing: &mut Au,
    absolute_anchor_origin: Au,
    image_size: Au,
) {
    if *size == Au(0) || image_size == Au(0) {
        *position = Au(0);
        *size = Au(0);
        *tile_spacing = Au(0);
        return;
    }

    // Per the spec, if the space available is not enough for two images, just tile as
    // normal but only display a single tile.
    if image_size * 2 >= *size {
        tile_image(position, size, absolute_anchor_origin, image_size);
        *tile_spacing = Au(0);
        *size = image_size;
        return;
    }

    // Take the box size, remove room for two tiles on the edges, and then calculate how many
    // other tiles fit in between them.
    let size_remaining = *size - (image_size * 2);
    let num_middle_tiles = (size_remaining.to_f32_px() / image_size.to_f32_px()).floor() as i32;

    // Allocate the remaining space as padding between tiles. background-position is ignored
    // as per the spec, so the position is just the box origin. We are also ignoring
    // background-attachment here, which seems unspecced when combined with
    // background-repeat: space.
    let space_for_middle_tiles = image_size * num_middle_tiles;
    *tile_spacing = (size_remaining - space_for_middle_tiles) / (num_middle_tiles + 1);
}

/// Tile an image
fn tile_image(position: &mut Au, size: &mut Au, absolute_anchor_origin: Au, image_size: Au) {
    // Avoid division by zero below!
    // Images with a zero width or height are not displayed.
    // Therefore the positions do not matter and can be left unchanged.
    // NOTE: A possible optimization is not to build
    // display items in this case at all.
    if image_size == Au(0) {
        return;
    }

    let delta_pixels = absolute_anchor_origin - *position;
    let image_size_px = image_size.to_f32_px();
    let tile_count = ((delta_pixels.to_f32_px() + image_size_px - 1.0) / image_size_px).floor();
    let offset = image_size * (tile_count as i32);
    let new_position = absolute_anchor_origin - offset;
    *size = *position - new_position + *size;
    *position = new_position;
}

/// For either the x or the y axis ajust various values to account for tiling.
///
/// This is done separately for both axes because the repeat keywords may differ.
fn tile_image_axis(
    repeat: BackgroundRepeatKeyword,
    position: &mut Au,
    size: &mut Au,
    tile_size: &mut Au,
    tile_spacing: &mut Au,
    offset: Au,
    clip_origin: Au,
    clip_size: Au,
) {
    let absolute_anchor_origin = *position + offset;
    match repeat {
        BackgroundRepeatKeyword::NoRepeat => {
            *position += offset;
            *size = *tile_size;
        },
        BackgroundRepeatKeyword::Repeat => {
            *position = clip_origin;
            *size = clip_size;
            tile_image(position, size, absolute_anchor_origin, *tile_size);
        },
        BackgroundRepeatKeyword::Space => {
            tile_image_spaced(
                position,
                size,
                tile_spacing,
                absolute_anchor_origin,
                *tile_size,
            );
            let combined_tile_size = *tile_size + *tile_spacing;
            *position = clip_origin;
            *size = clip_size;
            tile_image(position, size, absolute_anchor_origin, combined_tile_size);
        },
        BackgroundRepeatKeyword::Round => {
            tile_image_round(position, size, absolute_anchor_origin, tile_size);
            *position = clip_origin;
            *size = clip_size;
            tile_image(position, size, absolute_anchor_origin, *tile_size);
        },
    }
}

/// Determines the radius of a circle if it was not explictly provided.
/// <https://drafts.csswg.org/css-images-3/#typedef-size>
fn convert_circle_size_keyword(
    keyword: ShapeExtent,
    size: &Size2D<Au>,
    center: &Point2D<Au>,
) -> Size2D<Au> {
    let radius = match keyword {
        ShapeExtent::ClosestSide | ShapeExtent::Contain => {
            let dist = get_distance_to_sides(size, center, ::std::cmp::min);
            ::std::cmp::min(dist.width, dist.height)
        },
        ShapeExtent::FarthestSide => {
            let dist = get_distance_to_sides(size, center, ::std::cmp::max);
            ::std::cmp::max(dist.width, dist.height)
        },
        ShapeExtent::ClosestCorner => get_distance_to_corner(size, center, ::std::cmp::min),
        ShapeExtent::FarthestCorner | ShapeExtent::Cover => {
            get_distance_to_corner(size, center, ::std::cmp::max)
        },
    };
    Size2D::new(radius, radius)
}

/// Returns the radius for an ellipse with the same ratio as if it was matched to the sides.
fn get_ellipse_radius<F>(size: &Size2D<Au>, center: &Point2D<Au>, cmp: F) -> Size2D<Au>
where
    F: Fn(Au, Au) -> Au,
{
    let dist = get_distance_to_sides(size, center, cmp);
    Size2D::new(
        dist.width.scale_by(::std::f32::consts::FRAC_1_SQRT_2 * 2.0),
        dist.height
            .scale_by(::std::f32::consts::FRAC_1_SQRT_2 * 2.0),
    )
}

/// Determines the radius of an ellipse if it was not explictly provided.
/// <https://drafts.csswg.org/css-images-3/#typedef-size>
fn convert_ellipse_size_keyword(
    keyword: ShapeExtent,
    size: &Size2D<Au>,
    center: &Point2D<Au>,
) -> Size2D<Au> {
    match keyword {
        ShapeExtent::ClosestSide | ShapeExtent::Contain => {
            get_distance_to_sides(size, center, ::std::cmp::min)
        },
        ShapeExtent::FarthestSide => get_distance_to_sides(size, center, ::std::cmp::max),
        ShapeExtent::ClosestCorner => get_ellipse_radius(size, center, ::std::cmp::min),
        ShapeExtent::FarthestCorner | ShapeExtent::Cover => {
            get_ellipse_radius(size, center, ::std::cmp::max)
        },
    }
}

fn convert_gradient_stops(gradient_items: &[GradientItem], total_length: Au) -> Vec<GradientStop> {
    // Determine the position of each stop per CSS-IMAGES § 3.4.

    // Only keep the color stops, discard the color interpolation hints.
    let mut stop_items = gradient_items
        .iter()
        .filter_map(|item| match *item {
            GenericGradientItem::ColorStop(ref stop) => Some(*stop),
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
            first.position = Some(LengthOrPercentage::Percentage(Percentage(0.0)));
        }
    }
    // If the last color stop does not have a position, set its position to 100%.
    {
        let last = stop_items.last_mut().unwrap();
        if last.position.is_none() {
            last.position = Some(LengthOrPercentage::Percentage(Percentage(1.0)));
        }
    }

    // Step 2: Move any stops placed before earlier stops to the
    // same position as the preceding stop.
    let mut last_stop_position = stop_items.first().unwrap().position.unwrap();
    for stop in stop_items.iter_mut().skip(1) {
        if let Some(pos) = stop.position {
            if position_to_offset(last_stop_position, total_length) >
                position_to_offset(pos, total_length)
            {
                stop.position = Some(last_stop_position);
            }
            last_stop_position = stop.position.unwrap();
        }
    }

    // Step 3: Evenly space stops without position.
    let mut stops = Vec::with_capacity(stop_items.len());
    let mut stop_run = None;
    for (i, stop) in stop_items.iter().enumerate() {
        let offset = match stop.position {
            None => {
                if stop_run.is_none() {
                    // Initialize a new stop run.
                    // `unwrap()` here should never fail because this is the beginning of
                    // a stop run, which is always bounded by a length or percentage.
                    let start_offset =
                        position_to_offset(stop_items[i - 1].position.unwrap(), total_length);
                    // `unwrap()` here should never fail because this is the end of
                    // a stop run, which is always bounded by a length or percentage.
                    let (end_index, end_stop) = stop_items[(i + 1)..]
                        .iter()
                        .enumerate()
                        .find(|&(_, ref stop)| stop.position.is_some())
                        .unwrap();
                    let end_offset = position_to_offset(end_stop.position.unwrap(), total_length);
                    stop_run = Some(StopRun {
                        start_offset: start_offset,
                        end_offset: end_offset,
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
            Some(position) => {
                stop_run = None;
                position_to_offset(position, total_length)
            },
        };
        assert!(offset.is_finite());
        stops.push(GradientStop {
            offset: offset,
            color: stop.color.to_layout(),
        })
    }
    stops
}

fn as_gradient_extend_mode(repeating: bool) -> ExtendMode {
    if repeating {
        ExtendMode::Repeat
    } else {
        ExtendMode::Clamp
    }
}

pub fn convert_linear_gradient(
    size: Size2D<Au>,
    stops: &[GradientItem],
    direction: LineDirection,
    repeating: bool,
) -> display_list::Gradient {
    let angle = match direction {
        LineDirection::Angle(angle) => angle.radians(),
        LineDirection::Horizontal(x) => match x {
            X::Left => Angle::Deg(270.).radians(),
            X::Right => Angle::Deg(90.).radians(),
        },
        LineDirection::Vertical(y) => match y {
            Y::Top => Angle::Deg(0.).radians(),
            Y::Bottom => Angle::Deg(180.).radians(),
        },
        LineDirection::Corner(horizontal, vertical) => {
            // This the angle for one of the diagonals of the box. Our angle
            // will either be this one, this one + PI, or one of the other
            // two perpendicular angles.
            let atan = (size.height.to_f32_px() / size.width.to_f32_px()).atan();
            match (horizontal, vertical) {
                (X::Right, Y::Bottom) => ::std::f32::consts::PI - atan,
                (X::Left, Y::Bottom) => ::std::f32::consts::PI + atan,
                (X::Right, Y::Top) => atan,
                (X::Left, Y::Top) => -atan,
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

    let stops = convert_gradient_stops(stops, length);

    let center = Point2D::new(size.width / 2, size.height / 2);

    display_list::Gradient {
        start_point: (center - delta).to_layout(),
        end_point: (center + delta).to_layout(),
        stops: stops,
        extend_mode: as_gradient_extend_mode(repeating),
    }
}

pub fn convert_radial_gradient(
    size: Size2D<Au>,
    stops: &[GradientItem],
    shape: EndingShape,
    center: Position,
    repeating: bool,
) -> display_list::RadialGradient {
    let center = Point2D::new(
        center.horizontal.to_used_value(size.width),
        center.vertical.to_used_value(size.height),
    );
    let radius = match shape {
        GenericEndingShape::Circle(Circle::Radius(length)) => {
            let length = Au::from(length);
            Size2D::new(length, length)
        },
        GenericEndingShape::Circle(Circle::Extent(extent)) => {
            convert_circle_size_keyword(extent, &size, &center)
        },
        GenericEndingShape::Ellipse(Ellipse::Radii(x, y)) => {
            Size2D::new(x.to_used_value(size.width), y.to_used_value(size.height))
        },
        GenericEndingShape::Ellipse(Ellipse::Extent(extent)) => {
            convert_ellipse_size_keyword(extent, &size, &center)
        },
    };

    let stops = convert_gradient_stops(stops, radius.width);

    display_list::RadialGradient {
        center: center.to_layout(),
        radius: radius.to_layout(),
        stops: stops,
        extend_mode: as_gradient_extend_mode(repeating),
    }
}

/// Returns the the distance to the nearest or farthest corner depending on the comperator.
fn get_distance_to_corner<F>(size: &Size2D<Au>, center: &Point2D<Au>, cmp: F) -> Au
where
    F: Fn(Au, Au) -> Au,
{
    let dist = get_distance_to_sides(size, center, cmp);
    Au::from_f32_px(dist.width.to_f32_px().hypot(dist.height.to_f32_px()))
}

/// Returns the distance to the nearest or farthest sides depending on the comparator.
///
/// The first return value is horizontal distance the second vertical distance.
fn get_distance_to_sides<F>(size: &Size2D<Au>, center: &Point2D<Au>, cmp: F) -> Size2D<Au>
where
    F: Fn(Au, Au) -> Au,
{
    let top_side = center.y;
    let right_side = size.width - center.x;
    let bottom_side = size.height - center.y;
    let left_side = center.x;
    Size2D::new(cmp(left_side, right_side), cmp(top_side, bottom_side))
}

fn position_to_offset(position: LengthOrPercentage, total_length: Au) -> f32 {
    if total_length == Au(0) {
        return 0.0;
    }
    match position {
        LengthOrPercentage::Length(l) => l.to_i32_au() as f32 / total_length.0 as f32,
        LengthOrPercentage::Percentage(percentage) => percentage.0 as f32,
        LengthOrPercentage::Calc(calc) => {
            calc.to_used_value(Some(total_length)).unwrap().0 as f32 / total_length.0 as f32
        },
    }
}

fn scale_border_radii(radii: BorderRadius, factor: f32) -> BorderRadius {
    BorderRadius {
        top_left: radii.top_left * factor,
        top_right: radii.top_right * factor,
        bottom_left: radii.bottom_left * factor,
        bottom_right: radii.bottom_right * factor,
    }
}

fn handle_overlapping_radii(size: LayoutSize, radii: BorderRadius) -> BorderRadius {
    // No two corners' border radii may add up to more than the length of the edge
    // between them. To prevent that, all radii are scaled down uniformly.
    fn scale_factor(radius_a: f32, radius_b: f32, edge_length: f32) -> f32 {
        let required = radius_a + radius_b;

        if required <= edge_length {
            1.0
        } else {
            edge_length / required
        }
    }

    let top_factor = scale_factor(radii.top_left.width, radii.top_right.width, size.width);
    let bottom_factor = scale_factor(
        radii.bottom_left.width,
        radii.bottom_right.width,
        size.width,
    );
    let left_factor = scale_factor(radii.top_left.height, radii.bottom_left.height, size.height);
    let right_factor = scale_factor(
        radii.top_right.height,
        radii.bottom_right.height,
        size.height,
    );
    let min_factor = top_factor
        .min(bottom_factor)
        .min(left_factor)
        .min(right_factor);
    if min_factor < 1.0 {
        scale_border_radii(radii, min_factor)
    } else {
        radii
    }
}

pub fn build_border_radius(
    abs_bounds: &Rect<Au>,
    border_style: &style_structs::Border,
) -> BorderRadius {
    // TODO(cgaebel): Support border radii even in the case of multiple border widths.
    // This is an extension of supporting elliptical radii. For now, all percentage
    // radii will be relative to the width.

    handle_overlapping_radii(
        abs_bounds.size.to_layout(),
        BorderRadius {
            top_left: model::specified_border_radius(
                border_style.border_top_left_radius,
                abs_bounds.size,
            ).to_layout(),
            top_right: model::specified_border_radius(
                border_style.border_top_right_radius,
                abs_bounds.size,
            ).to_layout(),
            bottom_right: model::specified_border_radius(
                border_style.border_bottom_right_radius,
                abs_bounds.size,
            ).to_layout(),
            bottom_left: model::specified_border_radius(
                border_style.border_bottom_left_radius,
                abs_bounds.size,
            ).to_layout(),
        },
    )
}

/// Creates a four-sided border with uniform color, width and corner radius.
pub fn simple_normal_border(color: ColorF, style: BorderStyle) -> NormalBorder {
    let side = BorderSide { color, style };
    NormalBorder {
        left: side,
        right: side,
        top: side,
        bottom: side,
        radius: BorderRadius::zero(),
    }
}

/// Calculates radii for the inner side.
///
/// Radii usually describe the outer side of a border but for the lines to look nice
/// the inner radii need to be smaller depending on the line width.
///
/// This is used to determine clipping areas.
pub fn calculate_inner_border_radii(
    mut radii: BorderRadius,
    offsets: SideOffsets2D<Au>,
) -> BorderRadius {
    fn inner_length(x: f32, offset: Au) -> f32 {
        0.0_f32.max(x - offset.to_f32_px())
    }
    radii.top_left.width = inner_length(radii.top_left.width, offsets.left);
    radii.bottom_left.width = inner_length(radii.bottom_left.width, offsets.left);

    radii.top_right.width = inner_length(radii.top_right.width, offsets.right);
    radii.bottom_right.width = inner_length(radii.bottom_right.width, offsets.right);

    radii.top_left.height = inner_length(radii.top_left.height, offsets.top);
    radii.top_right.height = inner_length(radii.top_right.height, offsets.top);

    radii.bottom_left.height = inner_length(radii.bottom_left.height, offsets.bottom);
    radii.bottom_right.height = inner_length(radii.bottom_right.height, offsets.bottom);
    radii
}

/// Given an image and a border style constructs a border image.
///
/// See: https://drafts.csswg.org/css-backgrounds-3/#border-images
pub fn build_image_border_details(
    webrender_image: WebRenderImageInfo,
    border_style_struct: &style_structs::Border,
) -> Option<BorderDetails> {
    let corners = &border_style_struct.border_image_slice.offsets;
    let border_image_repeat = &border_style_struct.border_image_repeat;
    if let Some(image_key) = webrender_image.key {
        Some(BorderDetails::Image(ImageBorder {
            image_key: image_key,
            patch: NinePatchDescriptor {
                width: webrender_image.width,
                height: webrender_image.height,
                slice: SideOffsets2D::new(
                    corners.0.resolve(webrender_image.height),
                    corners.1.resolve(webrender_image.width),
                    corners.2.resolve(webrender_image.height),
                    corners.3.resolve(webrender_image.width),
                ),
            },
            fill: border_style_struct.border_image_slice.fill,
            // TODO(gw): Support border-image-outset
            outset: SideOffsets2D::zero(),
            repeat_horizontal: border_image_repeat.0.to_layout(),
            repeat_vertical: border_image_repeat.1.to_layout(),
        }))
    } else {
        None
    }
}
