/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use app_units::Au;
use euclid::default::{Rect, SideOffsets2D as UntypedSideOffsets2D, Size2D as UntypedSize2D};
use euclid::{SideOffsets2D, Size2D};
use style::computed_values::border_image_outset::T as BorderImageOutset;
use style::properties::style_structs::Border;
use style::values::computed::{
    BorderCornerRadius, BorderImageSideWidth, BorderImageWidth, NonNegativeLengthOrNumber,
    NumberOrPercentage,
};
use style::values::generics::rect::Rect as StyleRect;
use style::values::generics::NonNegative;
use webrender_api::units::{LayoutSideOffsets, LayoutSize};
use webrender_api::{BorderRadius, BorderSide, BorderStyle, ColorF, NormalBorder};

use crate::display_list::ToLayout;

/// Computes a border radius size against the containing size.
///
/// Note that percentages in `border-radius` are resolved against the relevant
/// box dimension instead of only against the width per [1]:
///
/// > Percentages: Refer to corresponding dimension of the border box.
///
/// [1]: https://drafts.csswg.org/css-backgrounds-3/#border-radius
fn corner_radius(
    radius: &BorderCornerRadius,
    containing_size: UntypedSize2D<Au>,
) -> UntypedSize2D<Au> {
    let w = radius.0.width().to_used_value(containing_size.width);
    let h = radius.0.height().to_used_value(containing_size.height);
    Size2D::new(w, h)
}

fn scaled_radii(radii: BorderRadius, factor: f32) -> BorderRadius {
    BorderRadius {
        top_left: radii.top_left * factor,
        top_right: radii.top_right * factor,
        bottom_left: radii.bottom_left * factor,
        bottom_right: radii.bottom_right * factor,
    }
}

fn overlapping_radii(size: LayoutSize, radii: BorderRadius) -> BorderRadius {
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
        scaled_radii(radii, min_factor)
    } else {
        radii
    }
}

/// Determine the four corner radii of a border.
///
/// Radii may either be absolute or relative to the absolute bounds.
/// Each corner radius has a width and a height which may differ.
/// Lastly overlapping radii are shrank so they don't collide anymore.
pub fn radii(abs_bounds: Rect<Au>, border_style: &Border) -> BorderRadius {
    // TODO(cgaebel): Support border radii even in the case of multiple border widths.
    // This is an extension of supporting elliptical radii. For now, all percentage
    // radii will be relative to the width.

    overlapping_radii(
        abs_bounds.size.to_layout(),
        BorderRadius {
            top_left: corner_radius(&border_style.border_top_left_radius, abs_bounds.size)
                .to_layout(),
            top_right: corner_radius(&border_style.border_top_right_radius, abs_bounds.size)
                .to_layout(),
            bottom_right: corner_radius(&border_style.border_bottom_right_radius, abs_bounds.size)
                .to_layout(),
            bottom_left: corner_radius(&border_style.border_bottom_left_radius, abs_bounds.size)
                .to_layout(),
        },
    )
}

/// Calculates radii for the inner side.
///
/// Radii usually describe the outer side of a border but for the lines to look nice
/// the inner radii need to be smaller depending on the line width.
///
/// This is used to determine clipping areas.
pub fn inner_radii(mut radii: BorderRadius, offsets: UntypedSideOffsets2D<Au>) -> BorderRadius {
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

/// Creates a four-sided border with square corners and uniform color and width.
pub fn simple(color: ColorF, style: BorderStyle) -> NormalBorder {
    let side = BorderSide { color, style };
    NormalBorder {
        left: side,
        right: side,
        top: side,
        bottom: side,
        radius: BorderRadius::zero(),
        do_aa: true,
    }
}

fn side_image_outset(outset: NonNegativeLengthOrNumber, border_width: Au) -> Au {
    match outset {
        NonNegativeLengthOrNumber::Length(length) => length.into(),
        NonNegativeLengthOrNumber::Number(factor) => border_width.scale_by(factor.0),
    }
}

/// Compute the additional border-image area.
pub fn image_outset(
    outset: BorderImageOutset,
    border: UntypedSideOffsets2D<Au>,
) -> UntypedSideOffsets2D<Au> {
    SideOffsets2D::new(
        side_image_outset(outset.0, border.top),
        side_image_outset(outset.1, border.right),
        side_image_outset(outset.2, border.bottom),
        side_image_outset(outset.3, border.left),
    )
}

fn side_image_width(
    border_image_width: &BorderImageSideWidth,
    border_width: f32,
    total_length: Au,
) -> f32 {
    match border_image_width {
        BorderImageSideWidth::LengthPercentage(v) => v.to_used_value(total_length).to_f32_px(),
        BorderImageSideWidth::Number(x) => border_width * x.0,
        BorderImageSideWidth::Auto => border_width,
    }
}

pub fn image_width(
    width: &BorderImageWidth,
    border: LayoutSideOffsets,
    border_area: UntypedSize2D<Au>,
) -> LayoutSideOffsets {
    LayoutSideOffsets::new(
        side_image_width(&width.0, border.top, border_area.height),
        side_image_width(&width.1, border.right, border_area.width),
        side_image_width(&width.2, border.bottom, border_area.height),
        side_image_width(&width.3, border.left, border_area.width),
    )
}

fn resolve_percentage(value: NonNegative<NumberOrPercentage>, length: i32) -> i32 {
    match value.0 {
        NumberOrPercentage::Percentage(p) => (p.0 * length as f32).round() as i32,
        NumberOrPercentage::Number(n) => n.round() as i32,
    }
}

pub fn image_slice<U>(
    border_image_slice: &StyleRect<NonNegative<NumberOrPercentage>>,
    size: Size2D<i32, U>,
) -> SideOffsets2D<i32, U> {
    SideOffsets2D::new(
        resolve_percentage(border_image_slice.0, size.height),
        resolve_percentage(border_image_slice.1, size.width),
        resolve_percentage(border_image_slice.2, size.height),
        resolve_percentage(border_image_slice.3, size.width),
    )
}
