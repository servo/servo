/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::replaced::IntrinsicSizes;
use euclid::{Size2D, Vector2D};
use style::computed_values::background_clip::single_value::T as Clip;
use style::computed_values::background_origin::single_value::T as Origin;
use style::properties::ComputedValues;
use style::values::computed::background::BackgroundSize as Size;
use style::values::computed::{Length, LengthPercentage};
use style::values::specified::background::BackgroundRepeat as RepeatXY;
use style::values::specified::background::BackgroundRepeatKeyword as Repeat;
use webrender_api::{self as wr, units};

pub(super) struct BackgroundLayer {
    pub common: wr::CommonItemProperties,
    pub bounds: units::LayoutRect,
    pub tile_size: units::LayoutSize,
    pub tile_spacing: units::LayoutSize,
    pub repeat: bool,
}

#[derive(Debug)]
struct Layout1DResult {
    repeat: bool,
    bounds_origin: f32,
    bounds_size: f32,
    tile_spacing: f32,
}

fn get_cyclic<T>(values: &[T], layer_index: usize) -> &T {
    &values[layer_index % values.len()]
}

pub(super) enum Source<'a> {
    Fragment,
    Canvas {
        style: &'a ComputedValues,

        // Theoretically the painting area is the infinite 2D plane,
        // but WebRender doesn’t really do infinite so this is the part of it that can be visible.
        painting_area: units::LayoutRect,
    },
}

pub(super) fn painting_area<'a>(
    fragment_builder: &'a super::BuilderForBoxFragment,
    source: &'a Source,
    builder: &mut super::DisplayListBuilder,
    layer_index: usize,
) -> (&'a units::LayoutRect, wr::CommonItemProperties) {
    let fb = fragment_builder;
    let (painting_area, clip) = match source {
        Source::Canvas { painting_area, .. } => (painting_area, None),
        Source::Fragment => {
            let b = fb.fragment.style.get_background();
            match get_cyclic(&b.background_clip.0, layer_index) {
                Clip::ContentBox => (fb.content_rect(), fb.content_edge_clip(builder)),
                Clip::PaddingBox => (fb.padding_rect(), fb.padding_edge_clip(builder)),
                Clip::BorderBox => (&fb.border_rect, fb.border_edge_clip(builder)),
            }
        },
    };
    // The 'backgound-clip' property maps directly to `clip_rect` in `CommonItemProperties`:
    let mut common = builder.common_properties(*painting_area, &fb.fragment.style);
    if let Some(clip_id) = clip {
        common.clip_id = clip_id
    }
    (painting_area, common)
}

pub(super) fn layout_layer(
    fragment_builder: &mut super::BuilderForBoxFragment,
    source: &Source,
    builder: &mut super::DisplayListBuilder,
    layer_index: usize,
    intrinsic: IntrinsicSizes,
) -> Option<BackgroundLayer> {
    let style = match *source {
        Source::Canvas { style, .. } => style,
        Source::Fragment => &fragment_builder.fragment.style,
    };
    let b = style.get_background();
    let (painting_area, common) = painting_area(fragment_builder, source, builder, layer_index);

    let positioning_area = match get_cyclic(&b.background_origin.0, layer_index) {
        Origin::ContentBox => fragment_builder.content_rect(),
        Origin::PaddingBox => fragment_builder.padding_rect(),
        Origin::BorderBox => &fragment_builder.border_rect,
    };

    // https://drafts.csswg.org/css-backgrounds/#background-size
    enum ContainOrCover {
        Contain,
        Cover,
    }
    let size_contain_or_cover = |background_size| {
        let mut tile_size = positioning_area.size;
        if let Some(intrinsic_ratio) = intrinsic.ratio {
            let positioning_ratio = positioning_area.size.width / positioning_area.size.height;
            // Whether the tile width (as opposed to height)
            // is scaled to that of the positioning area
            let fit_width = match background_size {
                ContainOrCover::Contain => positioning_ratio <= intrinsic_ratio,
                ContainOrCover::Cover => positioning_ratio > intrinsic_ratio,
            };
            // The other dimension needs to be adjusted
            if fit_width {
                tile_size.height = tile_size.width / intrinsic_ratio
            } else {
                tile_size.width = tile_size.height * intrinsic_ratio
            }
        }
        tile_size
    };
    let mut tile_size = match get_cyclic(&b.background_size.0, layer_index) {
        Size::Contain => size_contain_or_cover(ContainOrCover::Contain),
        Size::Cover => size_contain_or_cover(ContainOrCover::Cover),
        Size::ExplicitSize { width, height } => {
            let mut width = width.non_auto().map(|lp| {
                lp.0.percentage_relative_to(Length::new(positioning_area.size.width))
            });
            let mut height = height.non_auto().map(|lp| {
                lp.0.percentage_relative_to(Length::new(positioning_area.size.height))
            });

            if width.is_none() && height.is_none() {
                // Both computed values are 'auto':
                // use intrinsic sizes, treating missing width or height as 'auto'
                width = intrinsic.width;
                height = intrinsic.height;
            }

            match (width, height) {
                (Some(w), Some(h)) => units::LayoutSize::new(w.px(), h.px()),
                (Some(w), None) => {
                    let h = if let Some(intrinsic_ratio) = intrinsic.ratio {
                        w / intrinsic_ratio
                    } else if let Some(intrinsic_height) = intrinsic.height {
                        intrinsic_height
                    } else {
                        // Treated as 100%
                        Length::new(positioning_area.size.height)
                    };
                    units::LayoutSize::new(w.px(), h.px())
                },
                (None, Some(h)) => {
                    let w = if let Some(intrinsic_ratio) = intrinsic.ratio {
                        h * intrinsic_ratio
                    } else if let Some(intrinsic_width) = intrinsic.width {
                        intrinsic_width
                    } else {
                        // Treated as 100%
                        Length::new(positioning_area.size.width)
                    };
                    units::LayoutSize::new(w.px(), h.px())
                },
                // Both comptued values were 'auto', and neither intrinsic size is present
                (None, None) => size_contain_or_cover(ContainOrCover::Contain),
            }
        },
    };

    if tile_size.width == 0.0 || tile_size.height == 0.0 {
        return None;
    }

    let RepeatXY(repeat_x, repeat_y) = *get_cyclic(&b.background_repeat.0, layer_index);
    let result_x = layout_1d(
        &mut tile_size.width,
        repeat_x,
        get_cyclic(&b.background_position_x.0, layer_index),
        painting_area.origin.x - positioning_area.origin.x,
        painting_area.size.width,
        positioning_area.size.width,
    );
    let result_y = layout_1d(
        &mut tile_size.height,
        repeat_y,
        get_cyclic(&b.background_position_y.0, layer_index),
        painting_area.origin.y - positioning_area.origin.y,
        painting_area.size.height,
        positioning_area.size.height,
    );
    let bounds = units::LayoutRect::new(
        positioning_area.origin + Vector2D::new(result_x.bounds_origin, result_y.bounds_origin),
        Size2D::new(result_x.bounds_size, result_y.bounds_size),
    );
    let tile_spacing = units::LayoutSize::new(result_x.tile_spacing, result_y.tile_spacing);

    Some(BackgroundLayer {
        common,
        bounds,
        tile_size,
        tile_spacing,
        repeat: result_x.repeat || result_y.repeat,
    })
}

/// Abstract over the horizontal or vertical dimension
/// Coordinates (0, 0) for the purpose of this function are the positioning area’s origin.
fn layout_1d(
    tile_size: &mut f32,
    mut repeat: Repeat,
    position: &LengthPercentage,
    painting_area_origin: f32,
    painting_area_size: f32,
    positioning_area_size: f32,
) -> Layout1DResult {
    // https://drafts.csswg.org/css-backgrounds/#background-repeat
    if let Repeat::Round = repeat {
        *tile_size = positioning_area_size / (positioning_area_size / *tile_size).round();
    }
    // https://drafts.csswg.org/css-backgrounds/#background-position
    let mut position = position
        .percentage_relative_to(Length::new(positioning_area_size - *tile_size))
        .px();
    let mut tile_spacing = 0.0;
    // https://drafts.csswg.org/css-backgrounds/#background-repeat
    if let Repeat::Space = repeat {
        // The most entire tiles we can fit
        let tile_count = (positioning_area_size / *tile_size).floor();
        if tile_count >= 2.0 {
            position = 0.0;
            // Make the outsides of the first and last of that many tiles
            // touch the edges of the positioning area:
            let total_space = positioning_area_size - *tile_size * tile_count;
            let spaces_count = tile_count - 1.0;
            tile_spacing = total_space / spaces_count;
        } else {
            repeat = Repeat::NoRepeat
        }
    }
    match repeat {
        Repeat::Repeat | Repeat::Round | Repeat::Space => {
            // WebRender’s `RepeatingImageDisplayItem` contains a `bounds` rectangle and:
            //
            // * The tiling is clipped to the intersection of `clip_rect` and `bounds`
            // * The origin (top-left corner) of `bounds` is the position
            //   of the “first” (top-left-most) tile.
            //
            // In the general case that first tile is not the one that is positioned by
            // `background-position`.
            // We want it to be the top-left-most tile that intersects with `clip_rect`.
            // We find it by offsetting by a whole number of strides,
            // then compute `bounds` such that:
            //
            // * Its bottom-right is the bottom-right of `clip_rect`
            // * Its top-left is the top-left of first tile.
            let tile_stride = *tile_size + tile_spacing;
            let offset = position - painting_area_origin;
            let bounds_origin = position - tile_stride * (offset / tile_stride).ceil();
            let bounds_end = painting_area_origin + painting_area_size;
            let bounds_size = bounds_end - bounds_origin;
            Layout1DResult {
                repeat: true,
                bounds_origin,
                bounds_size,
                tile_spacing,
            }
        },
        Repeat::NoRepeat => {
            // `RepeatingImageDisplayItem` always repeats in both dimension.
            // When we want only one of the dimensions to repeat,
            // we use the `bounds` rectangle to clip the tiling to one tile
            // in that dimension.
            Layout1DResult {
                repeat: false,
                bounds_origin: position,
                bounds_size: *tile_size,
                tile_spacing: 0.0,
            }
        },
    }
}
