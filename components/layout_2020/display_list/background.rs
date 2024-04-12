/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use app_units::Au;
use euclid::{Point2D, Size2D, Vector2D};
use style::computed_values::background_attachment::SingleComputedValue as BackgroundAttachment;
use style::computed_values::background_clip::single_value::T as Clip;
use style::computed_values::background_origin::single_value::T as Origin;
use style::properties::ComputedValues;
use style::values::computed::background::BackgroundSize as Size;
use style::values::computed::{Length, LengthPercentage};
use style::values::specified::background::{
    BackgroundRepeat as RepeatXY, BackgroundRepeatKeyword as Repeat,
};
use webrender_api::{self as wr, units};
use wr::units::LayoutSize;
use wr::ClipChainId;

use crate::replaced::IntrinsicSizes;

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

pub(super) struct BackgroundPainter<'a> {
    pub style: &'a ComputedValues,
    pub positioning_area_override: Option<units::LayoutRect>,
    pub painting_area_override: Option<units::LayoutRect>,
}

impl<'a> BackgroundPainter<'a> {
    /// Get the painting area for this background, which is the actual rectangle in the
    /// current coordinate system that the background will be painted.
    pub(super) fn painting_area(
        &self,
        fragment_builder: &'a super::BuilderForBoxFragment,
        builder: &mut super::DisplayListBuilder,
        layer_index: usize,
    ) -> units::LayoutRect {
        let fb = fragment_builder;
        if let Some(painting_area_override) = self.painting_area_override.as_ref() {
            return *painting_area_override;
        }
        if self.positioning_area_override.is_some() {
            return fb.border_rect;
        }

        let background = self.style.get_background();
        if &BackgroundAttachment::Fixed ==
            get_cyclic(&background.background_attachment.0, layer_index)
        {
            let viewport_size = builder.display_list.compositor_info.viewport_size;
            return units::LayoutRect::from_origin_and_size(Point2D::origin(), viewport_size);
        }

        match get_cyclic(&background.background_clip.0, layer_index) {
            Clip::ContentBox => *fragment_builder.content_rect(),
            Clip::PaddingBox => *fragment_builder.padding_rect(),
            Clip::BorderBox => fragment_builder.border_rect,
        }
    }

    fn clip(
        &self,
        fragment_builder: &'a super::BuilderForBoxFragment,
        builder: &mut super::DisplayListBuilder,
        layer_index: usize,
    ) -> Option<ClipChainId> {
        if self.painting_area_override.is_some() {
            return None;
        }

        if self.positioning_area_override.is_some() {
            return fragment_builder.border_edge_clip(builder, false);
        }

        // The 'backgound-clip' property maps directly to `clip_rect` in `CommonItemProperties`:
        let background = self.style.get_background();
        let force_clip_creation = get_cyclic(&background.background_attachment.0, layer_index) ==
            &BackgroundAttachment::Fixed;
        match get_cyclic(&background.background_clip.0, layer_index) {
            Clip::ContentBox => fragment_builder.content_edge_clip(builder, force_clip_creation),
            Clip::PaddingBox => fragment_builder.padding_edge_clip(builder, force_clip_creation),
            Clip::BorderBox => fragment_builder.border_edge_clip(builder, force_clip_creation),
        }
    }

    /// Get the [`wr::CommonItemProperties`] for this background. This includes any clipping
    /// established by border radii as well as special clipping and spatial node assignment
    /// necessary for `background-attachment`.
    pub(super) fn common_properties(
        &self,
        fragment_builder: &'a super::BuilderForBoxFragment,
        builder: &mut super::DisplayListBuilder,
        layer_index: usize,
        painting_area: units::LayoutRect,
    ) -> wr::CommonItemProperties {
        let clip = self.clip(fragment_builder, builder, layer_index);
        let style = &fragment_builder.fragment.style;
        let mut common = builder.common_properties(painting_area, style);
        if let Some(clip_chain_id) = clip {
            common.clip_chain_id = clip_chain_id;
        }
        if &BackgroundAttachment::Fixed ==
            get_cyclic(&style.get_background().background_attachment.0, layer_index)
        {
            common.spatial_id = builder.current_reference_frame_scroll_node_id.spatial_id;
        }
        common
    }

    /// Get the positioning area of the background which is the rectangle that defines where
    /// the origin of the background content is, regardless of where the background is actual
    /// painted.
    pub(super) fn positioning_area(
        &self,
        fragment_builder: &'a super::BuilderForBoxFragment,
        layer_index: usize,
    ) -> units::LayoutRect {
        if let Some(positioning_area_override) = self.positioning_area_override {
            return positioning_area_override;
        }

        match get_cyclic(
            &self.style.get_background().background_attachment.0,
            layer_index,
        ) {
            BackgroundAttachment::Scroll => match get_cyclic(
                &self.style.get_background().background_origin.0,
                layer_index,
            ) {
                Origin::ContentBox => *fragment_builder.content_rect(),
                Origin::PaddingBox => *fragment_builder.padding_rect(),
                Origin::BorderBox => fragment_builder.border_rect,
            },
            BackgroundAttachment::Fixed => {
                // This isn't the viewport size because that rects larger than the viewport might be
                // transformed down into areas smaller than the viewport.
                units::LayoutRect::from_origin_and_size(
                    Point2D::origin(),
                    LayoutSize::new(f32::MAX, f32::MAX),
                )
            },
        }
    }
}

pub(super) fn layout_layer(
    fragment_builder: &mut super::BuilderForBoxFragment,
    painter: &BackgroundPainter,
    builder: &mut super::DisplayListBuilder,
    layer_index: usize,
    intrinsic: IntrinsicSizes,
) -> Option<BackgroundLayer> {
    let painting_area = painter.painting_area(fragment_builder, builder, layer_index);
    let positioning_area = painter.positioning_area(fragment_builder, layer_index);
    let common = painter.common_properties(fragment_builder, builder, layer_index, painting_area);

    // https://drafts.csswg.org/css-backgrounds/#background-size
    enum ContainOrCover {
        Contain,
        Cover,
    }
    let size_contain_or_cover = |background_size| {
        let mut tile_size = positioning_area.size();
        if let Some(intrinsic_ratio) = intrinsic.ratio {
            let positioning_ratio = positioning_area.size().width / positioning_area.size().height;
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

    let b = painter.style.get_background();
    let mut tile_size = match get_cyclic(&b.background_size.0, layer_index) {
        Size::Contain => size_contain_or_cover(ContainOrCover::Contain),
        Size::Cover => size_contain_or_cover(ContainOrCover::Cover),
        Size::ExplicitSize { width, height } => {
            let mut width = width.non_auto().map(|lp| {
                lp.0.percentage_relative_to(Length::new(positioning_area.size().width))
            });
            let mut height = height.non_auto().map(|lp| {
                lp.0.percentage_relative_to(Length::new(positioning_area.size().height))
            });

            if width.is_none() && height.is_none() {
                // Both computed values are 'auto':
                // use intrinsic sizes, treating missing width or height as 'auto'
                width = intrinsic.width.map(|v| v.into());
                height = intrinsic.height.map(|v| v.into());
            }

            match (width, height) {
                (Some(w), Some(h)) => units::LayoutSize::new(w.px(), h.px()),
                (Some(w), None) => {
                    let h = if let Some(intrinsic_ratio) = intrinsic.ratio {
                        w / intrinsic_ratio
                    } else if let Some(intrinsic_height) = intrinsic.height {
                        intrinsic_height.into()
                    } else {
                        // Treated as 100%
                        Au::from_f32_px(positioning_area.size().height).into()
                    };
                    units::LayoutSize::new(w.px(), h.px())
                },
                (None, Some(h)) => {
                    let w = if let Some(intrinsic_ratio) = intrinsic.ratio {
                        h * intrinsic_ratio
                    } else if let Some(intrinsic_width) = intrinsic.width {
                        intrinsic_width.into()
                    } else {
                        // Treated as 100%
                        Au::from_f32_px(positioning_area.size().width).into()
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
        painting_area.min.x - positioning_area.min.x,
        painting_area.size().width,
        positioning_area.size().width,
    );
    let result_y = layout_1d(
        &mut tile_size.height,
        repeat_y,
        get_cyclic(&b.background_position_y.0, layer_index),
        painting_area.min.y - positioning_area.min.y,
        painting_area.size().height,
        positioning_area.size().height,
    );
    let bounds = units::LayoutRect::from_origin_and_size(
        positioning_area.min + Vector2D::new(result_x.bounds_origin, result_y.bounds_origin),
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
