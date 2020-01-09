/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::fragments::{BoxFragment, Fragment};
use crate::geom::physical::{Rect, Vec2};
use embedder_traits::Cursor;
use euclid::{Point2D, SideOffsets2D, Size2D};
use gfx::text::glyph::GlyphStore;
use std::sync::Arc;
use style::dom::OpaqueNode;
use style::properties::ComputedValues;
use style::values::computed::{BorderStyle, Length, LengthPercentage};
use style::values::specified::ui::CursorKind;
use webrender_api::{self as wr, units};

// `webrender_api::display_item::ItemTag` is private
type ItemTag = (u64, u16);
type HitInfo = Option<ItemTag>;

pub struct DisplayListBuilder {
    current_space_and_clip: wr::SpaceAndClipInfo,
    pub wr: wr::DisplayListBuilder,

    /// Contentful paint, for the purpose of
    /// https://w3c.github.io/paint-timing/#first-contentful-paint
    /// (i.e. the display list contains items of type text,
    /// image, non-white canvas or SVG). Used by metrics.
    pub is_contentful: bool,
}

impl DisplayListBuilder {
    pub fn new(pipeline_id: wr::PipelineId, viewport_size: wr::units::LayoutSize) -> Self {
        Self {
            current_space_and_clip: wr::SpaceAndClipInfo::root_scroll(pipeline_id),
            is_contentful: false,
            wr: wr::DisplayListBuilder::new(pipeline_id, viewport_size),
        }
    }

    fn common_properties(
        &self,
        clip_rect: units::LayoutRect,
        hit_info: HitInfo,
    ) -> wr::CommonItemProperties {
        wr::CommonItemProperties {
            clip_rect,
            clip_id: self.current_space_and_clip.clip_id,
            spatial_id: self.current_space_and_clip.spatial_id,
            hit_info,
            // TODO(gw): Make use of the WR backface visibility functionality.
            flags: wr::PrimitiveFlags::default(),
        }
    }

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
            Fragment::Box(b) => b.build_display_list(builder, containing_block),
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
                let hit_info = hit_info(&t.parent_style, t.tag, Cursor::Text);
                let common = builder.common_properties(rect.clone().into(), hit_info);
                let color = t.parent_style.clone_color();
                builder
                    .wr
                    .push_text(&common, rect.into(), &glyphs, t.font_key, rgba(color), None);
            },
            Fragment::Image(i) => {
                use style::computed_values::image_rendering::T as ImageRendering;
                builder.is_contentful = true;
                let rect = i
                    .rect
                    .to_physical(i.style.writing_mode, containing_block)
                    .translate(&containing_block.top_left);
                let hit_info = None;
                let common = builder.common_properties(rect.clone().into(), hit_info);
                builder.wr.push_image(
                    &common,
                    rect.into(),
                    match i.style.get_inherited_box().image_rendering {
                        ImageRendering::Auto => wr::ImageRendering::Auto,
                        ImageRendering::CrispEdges => wr::ImageRendering::CrispEdges,
                        ImageRendering::Pixelated => wr::ImageRendering::Pixelated,
                    },
                    wr::AlphaType::PremultipliedAlpha,
                    i.image_key,
                    wr::ColorF::WHITE,
                );
            },
        }
    }
}

impl BoxFragment {
    fn build_display_list(
        &self,
        builder: &mut DisplayListBuilder,
        containing_block: &Rect<Length>,
    ) {
        let border_rect = self
            .border_rect()
            .to_physical(self.style.writing_mode, containing_block)
            .translate(&containing_block.top_left)
            .into();
        let hit_info = hit_info(&self.style, self.tag, Cursor::Default);
        let border_radius = self.border_radius(&border_rect);

        self.background_display_items(builder, hit_info, border_rect, &border_radius);
        self.border_display_items(builder, hit_info, border_rect, border_radius);
        let content_rect = self
            .content_rect
            .to_physical(self.style.writing_mode, containing_block)
            .translate(&containing_block.top_left);
        for child in &self.children {
            child.build_display_list(builder, &content_rect)
        }
    }

    fn border_radius(&self, border_rect: &units::LayoutRect) -> wr::BorderRadius {
        let resolve = |radius: &LengthPercentage, box_size: f32| {
            radius.percentage_relative_to(Length::new(box_size)).px()
        };
        let corner = |corner: &style::values::computed::BorderCornerRadius| {
            Size2D::new(
                resolve(&corner.0.width.0, border_rect.size.width),
                resolve(&corner.0.height.0, border_rect.size.height),
            )
        };
        let b = self.style.get_border();
        wr::BorderRadius {
            top_left: corner(&b.border_top_left_radius),
            top_right: corner(&b.border_top_right_radius),
            bottom_right: corner(&b.border_bottom_right_radius),
            bottom_left: corner(&b.border_bottom_left_radius),
        }
    }

    fn background_display_items(
        &self,
        builder: &mut DisplayListBuilder,
        hit_info: HitInfo,
        border_rect: units::LayoutRect,
        border_radius: &wr::BorderRadius,
    ) {
        let background_color = self
            .style
            .resolve_color(self.style.clone_background_color());
        if background_color.alpha > 0 || hit_info.is_some() {
            builder.clipping_and_scrolling_scope(|builder| {
                if !border_radius.is_zero() {
                    builder.current_space_and_clip.clip_id = builder.wr.define_clip(
                        &builder.current_space_and_clip,
                        border_rect,
                        Some(wr::ComplexClipRegion {
                            rect: border_rect,
                            radii: *border_radius,
                            mode: wr::ClipMode::Clip,
                        }),
                        None,
                    );
                }
                let common = builder.common_properties(border_rect, hit_info);
                builder.wr.push_rect(&common, rgba(background_color))
            });
        }
    }

    fn border_display_items(
        &self,
        builder: &mut DisplayListBuilder,
        hit_info: HitInfo,
        border_rect: units::LayoutRect,
        radius: wr::BorderRadius,
    ) {
        let b = self.style.get_border();
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
            color: rgba(self.style.resolve_color(color)),
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
        let common = builder.common_properties(border_rect, hit_info);
        let details = wr::BorderDetails::Normal(wr::NormalBorder {
            top: side(b.border_top_style, b.border_top_color),
            right: side(b.border_right_style, b.border_right_color),
            bottom: side(b.border_bottom_style, b.border_bottom_color),
            left: side(b.border_left_style, b.border_left_color),
            radius,
            do_aa: true,
        });
        builder
            .wr
            .push_border(&common, border_rect, widths, details)
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
