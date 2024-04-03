/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use app_units::Au;
use euclid::default::{Point2D, Rect, SideOffsets2D, Size2D};
use style::computed_values::background_attachment::single_value::T as BackgroundAttachment;
use style::computed_values::background_clip::single_value::T as BackgroundClip;
use style::computed_values::background_origin::single_value::T as BackgroundOrigin;
use style::properties::style_structs::Background;
use style::values::computed::{BackgroundSize, NonNegativeLengthPercentageOrAuto};
use style::values::specified::background::BackgroundRepeatKeyword;
use webrender_api::BorderRadius;

use crate::display_list::border;

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
    pub clip_rect: Rect<Au>,
    /// Rounded corners for the clip_rect.
    pub clip_radii: BorderRadius,
    /// Whether or not the background is fixed to the viewport.
    pub fixed: bool,
}

/// Access element at index modulo the array length.
///
/// Obviously it does not work with empty arrays.
///
/// This is used for multiple layered background images.
/// See: <https://drafts.csswg.org/css-backgrounds-3/#layering>
pub fn get_cyclic<T>(arr: &[T], index: usize) -> &T {
    &arr[index % arr.len()]
}

/// For a given area and an image compute how big the
/// image should be displayed on the background.
fn compute_background_image_size(
    bg_size: &BackgroundSize,
    bounds_size: Size2D<Au>,
    intrinsic_size: Option<Size2D<Au>>,
) -> Size2D<Au> {
    match intrinsic_size {
        None => match bg_size {
            BackgroundSize::Cover | BackgroundSize::Contain => bounds_size,
            BackgroundSize::ExplicitSize { width, height } => Size2D::new(
                width
                    .to_used_value(bounds_size.width)
                    .unwrap_or(bounds_size.width),
                height
                    .to_used_value(bounds_size.height)
                    .unwrap_or(bounds_size.height),
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
                    BackgroundSize::ExplicitSize {
                        width,
                        height: NonNegativeLengthPercentageOrAuto::Auto,
                    },
                    _,
                ) => {
                    let width = width
                        .to_used_value(bounds_size.width)
                        .unwrap_or(own_size.width);
                    Size2D::new(width, width.scale_by(image_aspect_ratio.recip()))
                },
                (
                    BackgroundSize::ExplicitSize {
                        width: NonNegativeLengthPercentageOrAuto::Auto,
                        height,
                    },
                    _,
                ) => {
                    let height = height
                        .to_used_value(bounds_size.height)
                        .unwrap_or(own_size.height);
                    Size2D::new(height.scale_by(image_aspect_ratio), height)
                },
                (BackgroundSize::ExplicitSize { width, height }, _) => Size2D::new(
                    width
                        .to_used_value(bounds_size.width)
                        .unwrap_or(own_size.width),
                    height
                        .to_used_value(bounds_size.height)
                        .unwrap_or(own_size.height),
                ),
            }
        },
    }
}

/// Compute a rounded clip rect for the background.
pub fn clip(
    bg_clip: BackgroundClip,
    absolute_bounds: Rect<Au>,
    border: SideOffsets2D<Au>,
    border_padding: SideOffsets2D<Au>,
    border_radii: BorderRadius,
) -> (Rect<Au>, BorderRadius) {
    match bg_clip {
        BackgroundClip::BorderBox => (absolute_bounds, border_radii),
        BackgroundClip::PaddingBox => (
            absolute_bounds.inner_rect(border),
            border::inner_radii(border_radii, border),
        ),
        BackgroundClip::ContentBox => (
            absolute_bounds.inner_rect(border_padding),
            border::inner_radii(border_radii, border_padding),
        ),
    }
}

/// Determines where to place an element background image or gradient.
///
/// Images have their resolution as intrinsic size while gradients have
/// no intrinsic size.
///
/// Return `None` if the background size is zero, otherwise a [`BackgroundPlacement`].
#[allow(clippy::too_many_arguments)]
pub fn placement(
    bg: &Background,
    viewport_size: Size2D<Au>,
    absolute_bounds: Rect<Au>,
    intrinsic_size: Option<Size2D<Au>>,
    border: SideOffsets2D<Au>,
    border_padding: SideOffsets2D<Au>,
    border_radii: BorderRadius,
    index: usize,
) -> Option<BackgroundPlacement> {
    let bg_attachment = *get_cyclic(&bg.background_attachment.0, index);
    let bg_clip = *get_cyclic(&bg.background_clip.0, index);
    let bg_origin = *get_cyclic(&bg.background_origin.0, index);
    let bg_position_x = get_cyclic(&bg.background_position_x.0, index);
    let bg_position_y = get_cyclic(&bg.background_position_y.0, index);
    let bg_repeat = get_cyclic(&bg.background_repeat.0, index);
    let bg_size = get_cyclic(&bg.background_size.0, index);

    let (clip_rect, clip_radii) = clip(
        bg_clip,
        absolute_bounds,
        border,
        border_padding,
        border_radii,
    );

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
    if tile_size.is_empty() {
        return None;
    }

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
        clip_rect.origin.x,
        clip_rect.size.width,
    );
    tile_image_axis(
        bg_repeat.1,
        &mut bounds.origin.y,
        &mut bounds.size.height,
        &mut tile_size.height,
        &mut tile_spacing.height,
        pos_y,
        clip_rect.origin.y,
        clip_rect.size.height,
    );

    if tile_size.is_empty() {
        return None;
    }

    Some(BackgroundPlacement {
        bounds,
        tile_size,
        tile_spacing,
        clip_rect,
        clip_radii,
        fixed,
    })
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

/// For either the x or the y axis adjust various values to account for tiling.
///
/// This is done separately for both axes because the repeat keywords may differ.
#[allow(clippy::too_many_arguments)]
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
