/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::context::LayoutContext;
use crate::fragments::{BoxFragment, Fragment};
use crate::geom::physical::{Rect, Vec2};
use crate::replaced::IntrinsicSizes;
use embedder_traits::Cursor;
use euclid::{Point2D, SideOffsets2D, Size2D, Vector2D};
use gfx::text::glyph::GlyphStore;
use mitochondria::OnceCell;
use net_traits::image_cache::UsePlaceholder;
use std::sync::Arc;
use style::dom::OpaqueNode;
use style::properties::ComputedValues;
use style::values::computed::{BorderStyle, Length, LengthPercentage};
use style::values::specified::ui::CursorKind;
use webrender_api::{self as wr, units};

#[derive(Clone, Copy)]
pub struct WebRenderImageInfo {
    pub width: u32,
    pub height: u32,
    pub key: Option<wr::ImageKey>,
}

// `webrender_api::display_item::ItemTag` is private
type ItemTag = (u64, u16);
type HitInfo = Option<ItemTag>;

pub struct DisplayListBuilder<'a> {
    current_space_and_clip: wr::SpaceAndClipInfo,
    pub context: &'a LayoutContext<'a>,
    pub wr: wr::DisplayListBuilder,

    /// Contentful paint, for the purpose of
    /// https://w3c.github.io/paint-timing/#first-contentful-paint
    /// (i.e. the display list contains items of type text,
    /// image, non-white canvas or SVG). Used by metrics.
    pub is_contentful: bool,
}

impl<'a> DisplayListBuilder<'a> {
    pub fn new(
        pipeline_id: wr::PipelineId,
        context: &'a LayoutContext,
        viewport_size: wr::units::LayoutSize,
    ) -> Self {
        Self {
            current_space_and_clip: wr::SpaceAndClipInfo::root_scroll(pipeline_id),
            is_contentful: false,
            context,
            wr: wr::DisplayListBuilder::new(pipeline_id, viewport_size),
        }
    }

    fn common_properties(&self, clip_rect: units::LayoutRect) -> wr::CommonItemProperties {
        // TODO(gw): Make use of the WR backface visibility functionality.
        wr::CommonItemProperties::new(clip_rect, self.current_space_and_clip)
    }

    // FIXME: use this for the `overflow` property or anything else that clips an entire subtree.
    #[allow(unused)]
    fn clipping_and_scrolling_scope<R>(&mut self, f: impl FnOnce(&mut Self) -> R) -> R {
        let previous = self.current_space_and_clip;
        let result = f(self);
        self.current_space_and_clip = previous;
        result
    }
}

impl Fragment {
    pub(crate) fn build_display_list(
        &self,
        builder: &mut DisplayListBuilder,
        containing_block: &Rect<Length>,
    ) {
        match self {
            Fragment::Box(b) => BuilderForBoxFragment::new(b, containing_block).build(builder),
            Fragment::Anonymous(a) => {
                let rect = a
                    .rect
                    .to_physical(a.mode, containing_block)
                    .translate(&containing_block.top_left);
                for child in &a.children {
                    child.build_display_list(builder, &rect)
                }
            },
            Fragment::Text(t) => {
                builder.is_contentful = true;
                let rect = t
                    .rect
                    .to_physical(t.parent_style.writing_mode, containing_block)
                    .translate(&containing_block.top_left);
                let mut baseline_origin = rect.top_left.clone();
                baseline_origin.y += t.ascent;
                let glyphs = glyphs(&t.glyphs, baseline_origin);
                if glyphs.is_empty() {
                    return;
                }
                let mut common = builder.common_properties(rect.clone().into());
                common.hit_info = hit_info(&t.parent_style, t.tag, Cursor::Text);
                let color = t.parent_style.clone_color();
                builder
                    .wr
                    .push_text(&common, rect.into(), &glyphs, t.font_key, rgba(color), None);
            },
            Fragment::Image(i) => {
                builder.is_contentful = true;
                let rect = i
                    .rect
                    .to_physical(i.style.writing_mode, containing_block)
                    .translate(&containing_block.top_left);
                let common = builder.common_properties(rect.clone().into());
                builder.wr.push_image(
                    &common,
                    rect.into(),
                    image_rendering(i.style.get_inherited_box().image_rendering),
                    wr::AlphaType::PremultipliedAlpha,
                    i.image_key,
                    wr::ColorF::WHITE,
                );
            },
        }
    }
}

struct BuilderForBoxFragment<'a> {
    fragment: &'a BoxFragment,
    containing_block: &'a Rect<Length>,
    border_rect: units::LayoutRect,
    padding_rect: OnceCell<units::LayoutRect>,
    content_rect: OnceCell<units::LayoutRect>,
    border_radius: wr::BorderRadius,
    border_edge_clip_id: OnceCell<Option<wr::ClipId>>,
}

impl<'a> BuilderForBoxFragment<'a> {
    fn new(fragment: &'a BoxFragment, containing_block: &'a Rect<Length>) -> Self {
        let border_rect: units::LayoutRect = fragment
            .border_rect()
            .to_physical(fragment.style.writing_mode, containing_block)
            .translate(&containing_block.top_left)
            .into();

        let border_radius = {
            let resolve = |radius: &LengthPercentage, box_size: f32| {
                radius.percentage_relative_to(Length::new(box_size)).px()
            };
            let corner = |corner: &style::values::computed::BorderCornerRadius| {
                Size2D::new(
                    resolve(&corner.0.width.0, border_rect.size.width),
                    resolve(&corner.0.height.0, border_rect.size.height),
                )
            };
            let b = fragment.style.get_border();
            wr::BorderRadius {
                top_left: corner(&b.border_top_left_radius),
                top_right: corner(&b.border_top_right_radius),
                bottom_right: corner(&b.border_bottom_right_radius),
                bottom_left: corner(&b.border_bottom_left_radius),
            }
        };

        Self {
            fragment,
            containing_block,
            border_rect,
            border_radius,
            padding_rect: OnceCell::new(),
            content_rect: OnceCell::new(),
            border_edge_clip_id: OnceCell::new(),
        }
    }

    fn content_rect(&self) -> &units::LayoutRect {
        self.content_rect.init_once(|| {
            self.fragment
                .content_rect
                .to_physical(self.fragment.style.writing_mode, self.containing_block)
                .translate(&self.containing_block.top_left)
                .into()
        })
    }

    fn padding_rect(&self) -> &units::LayoutRect {
        self.padding_rect.init_once(|| {
            self.fragment
                .padding_rect()
                .to_physical(self.fragment.style.writing_mode, self.containing_block)
                .translate(&self.containing_block.top_left)
                .into()
        })
    }

    fn with_border_edge_clip(
        &mut self,
        builder: &mut DisplayListBuilder,
        common: &mut wr::CommonItemProperties,
    ) {
        let initialized = self.border_edge_clip_id.init_once(|| {
            if self.border_radius.is_zero() {
                None
            } else {
                Some(builder.wr.define_clip(
                    &builder.current_space_and_clip,
                    self.border_rect,
                    Some(wr::ComplexClipRegion {
                        rect: self.border_rect,
                        radii: self.border_radius,
                        mode: wr::ClipMode::Clip,
                    }),
                    None,
                ))
            }
        });
        if let Some(clip_id) = *initialized {
            common.clip_id = clip_id
        }
    }

    fn build(&mut self, builder: &mut DisplayListBuilder) {
        let hit_info = hit_info(&self.fragment.style, self.fragment.tag, Cursor::Default);
        if hit_info.is_some() {
            let mut common = builder.common_properties(self.border_rect);
            common.hit_info = hit_info;
            self.with_border_edge_clip(builder, &mut common);
            builder.wr.push_hit_test(&common)
        }

        self.build_background(builder);
        self.build_border(builder);
        let content_rect = self
            .fragment
            .content_rect
            .to_physical(self.fragment.style.writing_mode, self.containing_block)
            .translate(&self.containing_block.top_left);
        for child in &self.fragment.children {
            child.build_display_list(builder, &content_rect)
        }
    }

    fn build_background(&mut self, builder: &mut DisplayListBuilder) {
        use style::values::computed::image::{Image, ImageLayer};
        let b = self.fragment.style.get_background();
        let background_color = self.fragment.style.resolve_color(b.background_color);
        if background_color.alpha > 0 {
            let mut common = builder.common_properties(self.border_rect);
            self.with_border_edge_clip(builder, &mut common);
            builder.wr.push_rect(&common, rgba(background_color))
        }
        // Reverse because the property is top layer first, we want to paint bottom layer first.
        for (index, layer) in b.background_image.0.iter().enumerate().rev() {
            match layer {
                ImageLayer::None => {},
                ImageLayer::Image(image) => match image {
                    Image::Gradient(_gradient) => {
                        // TODO
                    },
                    Image::Url(image_url) => {
                        if let Some(url) = image_url.url() {
                            let webrender_image = builder.context.get_webrender_image_for_url(
                                self.fragment.tag,
                                url.clone(),
                                UsePlaceholder::No,
                            );
                            if let Some(WebRenderImageInfo {
                                width,
                                height,
                                key: Some(key),
                            }) = webrender_image
                            {
                                // FIXME: https://drafts.csswg.org/css-images-4/#the-image-resolution
                                let dppx = 1.0;

                                let intrinsic = IntrinsicSizes {
                                    width: Some(Length::new(width as f32 / dppx)),
                                    height: Some(Length::new(height as f32 / dppx)),
                                    // FIXME https://github.com/w3c/csswg-drafts/issues/4572
                                    ratio: Some(width as f32 / height as f32),
                                };

                                self.build_background_raster_image(builder, index, intrinsic, key)
                            }
                        }
                    },
                    // Gecko-only value, represented as a (boxed) empty enum on non-Gecko.
                    Image::Rect(rect) => match **rect {},
                },
            }
        }
    }

    fn build_background_raster_image(
        &mut self,
        builder: &mut DisplayListBuilder,
        index: usize,
        intrinsic: IntrinsicSizes,
        key: wr::ImageKey,
    ) {
        use style::computed_values::background_clip::single_value::T as Clip;
        use style::computed_values::background_origin::single_value::T as Origin;
        use style::values::computed::background::BackgroundSize as Size;
        use style::values::specified::background::BackgroundRepeat as RepeatXY;
        use style::values::specified::background::BackgroundRepeatKeyword as Repeat;

        fn get_cyclic<T>(values: &[T], index: usize) -> &T {
            &values[index % values.len()]
        }

        let b = self.fragment.style.get_background();

        let clipping_area = match get_cyclic(&b.background_clip.0, index) {
            Clip::ContentBox => self.content_rect(),
            Clip::PaddingBox => self.padding_rect(),
            Clip::BorderBox => &self.border_rect,
        };
        let positioning_area = match get_cyclic(&b.background_origin.0, index) {
            Origin::ContentBox => self.content_rect(),
            Origin::PaddingBox => self.padding_rect(),
            Origin::BorderBox => &self.border_rect,
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
        let mut tile_size = match get_cyclic(&b.background_size.0, index) {
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
            return;
        }

        struct Layout1DResult {
            repeat: bool,
            bounds_origin: f32,
            bounds_size: f32,
        }

        /// Abstract over the horizontal or vertical dimension
        /// Coordinates (0, 0) for the purpose of this function are the positioning area’s origin.
        fn layout_1d(
            tile_size: &mut f32,
            tile_spacing: &mut f32,
            mut repeat: Repeat,
            position: &LengthPercentage,
            clipping_area_origin: f32,
            clipping_area_size: f32,
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
                    *tile_spacing = total_space / spaces_count;
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
                    let tile_stride = *tile_size + *tile_spacing;
                    let offset = position - clipping_area_origin;
                    let bounds_origin = position - tile_stride * (offset / tile_stride).ceil();
                    let bounds_size = clipping_area_size - bounds_origin - clipping_area_origin;
                    Layout1DResult {
                        repeat: true,
                        bounds_origin,
                        bounds_size,
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
                    }
                },
            }
        }

        let mut tile_spacing = units::LayoutSize::zero();
        let RepeatXY(repeat_x, repeat_y) = *get_cyclic(&b.background_repeat.0, index);
        let result_x = layout_1d(
            &mut tile_size.width,
            &mut tile_spacing.width,
            repeat_x,
            get_cyclic(&b.background_position_x.0, index),
            clipping_area.origin.x - positioning_area.origin.x,
            clipping_area.size.width,
            positioning_area.size.width,
        );
        let result_y = layout_1d(
            &mut tile_size.height,
            &mut tile_spacing.height,
            repeat_y,
            get_cyclic(&b.background_position_y.0, index),
            clipping_area.origin.y - positioning_area.origin.y,
            clipping_area.size.height,
            positioning_area.size.height,
        );
        let bounds = units::LayoutRect::new(
            positioning_area.origin + Vector2D::new(result_x.bounds_origin, result_y.bounds_origin),
            Size2D::new(result_x.bounds_size, result_y.bounds_size),
        );

        // The 'backgound-clip' property maps directly to `clip_rect` in `CommonItemProperties`:
        let mut common = builder.common_properties(*clipping_area);
        self.with_border_edge_clip(builder, &mut common);

        if result_x.repeat || result_y.repeat {
            builder.wr.push_repeating_image(
                &common,
                bounds,
                tile_size,
                tile_spacing,
                image_rendering(self.fragment.style.clone_image_rendering()),
                wr::AlphaType::PremultipliedAlpha,
                key,
                wr::ColorF::WHITE,
            )
        } else {
            builder.wr.push_image(
                &common,
                bounds,
                image_rendering(self.fragment.style.clone_image_rendering()),
                wr::AlphaType::PremultipliedAlpha,
                key,
                wr::ColorF::WHITE,
            )
        }
    }

    fn build_border(&mut self, builder: &mut DisplayListBuilder) {
        let b = self.fragment.style.get_border();
        let widths = SideOffsets2D::new(
            b.border_top_width.px(),
            b.border_right_width.px(),
            b.border_bottom_width.px(),
            b.border_left_width.px(),
        );
        if widths == SideOffsets2D::zero() {
            return;
        }
        let side = |style, color| wr::BorderSide {
            color: rgba(self.fragment.style.resolve_color(color)),
            style: match style {
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
            },
        };
        let common = builder.common_properties(self.border_rect);
        let details = wr::BorderDetails::Normal(wr::NormalBorder {
            top: side(b.border_top_style, b.border_top_color),
            right: side(b.border_right_style, b.border_right_color),
            bottom: side(b.border_bottom_style, b.border_bottom_color),
            left: side(b.border_left_style, b.border_left_color),
            radius: self.border_radius,
            do_aa: true,
        });
        builder
            .wr
            .push_border(&common, self.border_rect, widths, details)
    }
}

fn rgba(rgba: cssparser::RGBA) -> wr::ColorF {
    wr::ColorF::new(
        rgba.red_f32(),
        rgba.green_f32(),
        rgba.blue_f32(),
        rgba.alpha_f32(),
    )
}

fn glyphs(glyph_runs: &[Arc<GlyphStore>], mut origin: Vec2<Length>) -> Vec<wr::GlyphInstance> {
    use gfx_traits::ByteIndex;
    use range::Range;

    let mut glyphs = vec![];
    for run in glyph_runs {
        for glyph in run.iter_glyphs_for_byte_range(&Range::new(ByteIndex(0), run.len())) {
            if !run.is_whitespace() {
                let glyph_offset = glyph.offset().unwrap_or(Point2D::zero());
                let point = units::LayoutPoint::new(
                    origin.x.px() + glyph_offset.x.to_f32_px(),
                    origin.y.px() + glyph_offset.y.to_f32_px(),
                );
                let glyph = wr::GlyphInstance {
                    index: glyph.id(),
                    point,
                };
                glyphs.push(glyph);
            }
            origin.x += Length::from(glyph.advance());
        }
    }
    glyphs
}

fn hit_info(style: &ComputedValues, tag: OpaqueNode, auto_cursor: Cursor) -> HitInfo {
    use style::computed_values::pointer_events::T as PointerEvents;

    let inherited_ui = style.get_inherited_ui();
    if inherited_ui.pointer_events == PointerEvents::None {
        None
    } else {
        let cursor = cursor(inherited_ui.cursor.keyword, auto_cursor);
        Some((tag.0 as u64, cursor as u16))
    }
}

fn cursor(kind: CursorKind, auto_cursor: Cursor) -> Cursor {
    match kind {
        CursorKind::Auto => auto_cursor,
        CursorKind::None => Cursor::None,
        CursorKind::Default => Cursor::Default,
        CursorKind::Pointer => Cursor::Pointer,
        CursorKind::ContextMenu => Cursor::ContextMenu,
        CursorKind::Help => Cursor::Help,
        CursorKind::Progress => Cursor::Progress,
        CursorKind::Wait => Cursor::Wait,
        CursorKind::Cell => Cursor::Cell,
        CursorKind::Crosshair => Cursor::Crosshair,
        CursorKind::Text => Cursor::Text,
        CursorKind::VerticalText => Cursor::VerticalText,
        CursorKind::Alias => Cursor::Alias,
        CursorKind::Copy => Cursor::Copy,
        CursorKind::Move => Cursor::Move,
        CursorKind::NoDrop => Cursor::NoDrop,
        CursorKind::NotAllowed => Cursor::NotAllowed,
        CursorKind::Grab => Cursor::Grab,
        CursorKind::Grabbing => Cursor::Grabbing,
        CursorKind::EResize => Cursor::EResize,
        CursorKind::NResize => Cursor::NResize,
        CursorKind::NeResize => Cursor::NeResize,
        CursorKind::NwResize => Cursor::NwResize,
        CursorKind::SResize => Cursor::SResize,
        CursorKind::SeResize => Cursor::SeResize,
        CursorKind::SwResize => Cursor::SwResize,
        CursorKind::WResize => Cursor::WResize,
        CursorKind::EwResize => Cursor::EwResize,
        CursorKind::NsResize => Cursor::NsResize,
        CursorKind::NeswResize => Cursor::NeswResize,
        CursorKind::NwseResize => Cursor::NwseResize,
        CursorKind::ColResize => Cursor::ColResize,
        CursorKind::RowResize => Cursor::RowResize,
        CursorKind::AllScroll => Cursor::AllScroll,
        CursorKind::ZoomIn => Cursor::ZoomIn,
        CursorKind::ZoomOut => Cursor::ZoomOut,
    }
}

fn image_rendering(ir: style::computed_values::image_rendering::T) -> wr::ImageRendering {
    use style::computed_values::image_rendering::T as ImageRendering;
    match ir {
        ImageRendering::Auto => wr::ImageRendering::Auto,
        ImageRendering::CrispEdges => wr::ImageRendering::CrispEdges,
        ImageRendering::Pixelated => wr::ImageRendering::Pixelated,
    }
}
