/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::fragments::{BoxFragment, Fragment};
use crate::geom::physical::{Rect, Vec2};
use crate::style_ext::ComputedValuesExt;
use app_units::Au;
use euclid::{self, SideOffsets2D};
use style::values::computed::{BorderStyle, Length};
use webrender_api::{self as wr, units, CommonItemProperties, PrimitiveFlags};

pub struct DisplayListBuilder {
    pipeline_id: wr::PipelineId,
    pub wr: wr::DisplayListBuilder,
    pub is_contentful: bool,
}

impl DisplayListBuilder {
    pub fn new(pipeline_id: wr::PipelineId, viewport_size: wr::units::LayoutSize) -> Self {
        Self {
            pipeline_id,
            is_contentful: false,
            wr: wr::DisplayListBuilder::new(pipeline_id, viewport_size),
        }
    }
}

/// Contentful paint, for the purpose of
/// https://w3c.github.io/paint-timing/#first-contentful-paint
/// (i.e. the display list contains items of type text,
/// image, non-white canvas or SVG). Used by metrics.
pub struct IsContentful(pub bool);

impl Fragment {
    pub(crate) fn build_display_list(
        &self,
        builder: &mut DisplayListBuilder,
        is_contentful: &mut IsContentful,
        containing_block: &Rect<Length>,
    ) {
        match self {
            Fragment::Box(b) => b.build_display_list(builder, is_contentful, containing_block),
            Fragment::Anonymous(a) => {
                let rect = a
                    .rect
                    .to_physical(a.mode, containing_block)
                    .translate(&containing_block.top_left);
                for child in &a.children {
                    child.build_display_list(builder, is_contentful, &rect)
                }
            },
            Fragment::Text(_) => {
                is_contentful.0 = true;
                // FIXME
            },
        }
    }
}

impl BoxFragment {
    fn build_display_list(
        &self,
        builder: &mut DisplayListBuilder,
        is_contentful: &mut IsContentful,
        containing_block: &Rect<Length>,
    ) {
        let border_rect = self
            .border_rect()
            .to_physical(self.style.writing_mode(), containing_block)
            .translate(&containing_block.top_left)
            .into();
        let common = CommonItemProperties {
            clip_rect: border_rect,
            clip_id: wr::ClipId::root(builder.pipeline_id),
            spatial_id: wr::SpatialId::root_scroll_node(builder.pipeline_id),
            hit_info: None,
            // TODO(gw): Make use of the WR backface visibility functionality.
            flags: PrimitiveFlags::default(),
        };

        self.background_display_items(builder, &common);
        self.border_display_items(builder, &common, border_rect);
        let content_rect = self
            .content_rect
            .to_physical(self.style.writing_mode(), containing_block)
            .translate(&containing_block.top_left);
        for child in &self.children {
            child.build_display_list(builder, is_contentful, &content_rect)
        }
    }

    fn background_display_items(
        &self,
        builder: &mut DisplayListBuilder,
        common: &CommonItemProperties,
    ) {
        let background_color = self
            .style
            .resolve_color(self.style.clone_background_color());
        if background_color.alpha > 0 {
            builder.wr.push_rect(common, rgba(background_color))
        }
    }

    fn border_display_items(
        &self,
        builder: &mut DisplayListBuilder,
        common: &CommonItemProperties,
        border_rect: units::LayoutRect,
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
        let details = wr::BorderDetails::Normal(wr::NormalBorder {
            top: side(b.border_top_style, b.border_top_color),
            right: side(b.border_right_style, b.border_right_color),
            bottom: side(b.border_bottom_style, b.border_bottom_color),
            left: side(b.border_left_style, b.border_left_color),
            radius: wr::BorderRadius::zero(),
            do_aa: true,
        });
        builder.wr.push_border(common, border_rect, widths, details)
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
