/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::context::LayoutContext;
use crate::fragments::{BoxFragment, Fragment};
use crate::geom::physical::{Rect, Vec2};
use embedder_traits::Cursor;
use euclid::{Point2D, SideOffsets2D, Size2D, Vector2D};
use gfx::text::glyph::GlyphStore;
use net_traits::image_cache::UsePlaceholder;
use once_cell::unsync::OnceCell;
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
        self.content_rect.get_or_init(|| {
            self.fragment
                .content_rect
                .to_physical(self.fragment.style.writing_mode, self.containing_block)
                .translate(&self.containing_block.top_left)
                .into()
        })
    }

    fn padding_rect(&self) -> &units::LayoutRect {
        self.padding_rect.get_or_init(|| {
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
        let initialized = self.border_edge_clip_id.get_or_init(|| {
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
                                self.build_background_raster_image(
                                    builder, index, width, height, key,
                                )
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
        intrinsic_width: u32,
        intrinsic_height: u32,
        key: wr::ImageKey,
    ) {
        // Our job here would be easier if WebRender’s `RepeatingImageDisplayItem`
        // was “infinitely” repeating in all directions (clipped at `clip_rect`)
        // and contained a `units::LayoutPoint` that determines the position of an arbitrary tile.
        //
        // Instead, it contains a `bounds` rectangle and:
        //
        // * The tiling is clipped to the intersection of `clip_rect` and `bounds`
        // * The origin (top-left corner) of `bounds` is the position
        //   of the “first” (top-left-most) tile.
        //
        // However the background clipping area may be larger than the positionning area,
        // so that first time may not be the one at position (0, 0).
        // So we compute `bounds` such that:
        //
        // * Its bottom-right is the bottom-right of `clip_rect`
        // * Its top-left is the top-left of the top-left-most tile that intersects with `clip_rect`

        use style::computed_values::background_clip::single_value::T as Clip;
        use style::computed_values::background_origin::single_value::T as Origin;

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

        // FIXME: https://drafts.csswg.org/css-images-4/#the-image-resolution
        let dppx = 1.0;

        let intrinsic_size = units::LayoutSize::new(
            intrinsic_width as f32 / dppx,
            intrinsic_height as f32 / dppx,
        );
        // FIXME: background-size
        // Size of one tile:
        let stretch_size = intrinsic_size;
        let tile_spacing = units::LayoutSize::zero();
        let tile_stride = stretch_size + tile_spacing;

        // FIXME: background-position
        let positioned_tile_origin = positioning_area.origin;
        let offset = positioned_tile_origin - clipping_area.origin;
        let first_tile_origin = positioned_tile_origin -
            Vector2D::new(
                tile_stride.width * (offset.x / tile_stride.width).ceil(),
                tile_stride.height * (offset.y / tile_stride.height).ceil(),
            );

        let bounds = units::LayoutRect::from_points(&[first_tile_origin, clipping_area.max()]);

        // The 'backgound-clip' property maps directly to `clip_rect` in `CommonItemProperties`:
        let mut common = builder.common_properties(*clipping_area);
        self.with_border_edge_clip(builder, &mut common);

        builder.wr.push_repeating_image(
            &common,
            bounds,
            stretch_size,
            tile_spacing,
            image_rendering(self.fragment.style.clone_image_rendering()),
            wr::AlphaType::PremultipliedAlpha,
            key,
            wr::ColorF::WHITE,
        )
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
