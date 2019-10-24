/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::fragments::{BoxFragment, Fragment};
use crate::geom::physical::{Rect, Vec2};
use crate::style_ext::ComputedValuesExt;
use app_units::Au;
use style::values::computed::Length;
use webrender_api::CommonItemProperties;

pub struct DisplayListBuilder {
    pipeline_id: webrender_api::PipelineId,
    pub wr: webrender_api::DisplayListBuilder,
    pub is_contentful: bool,
}

impl DisplayListBuilder {
    pub fn new(
        pipeline_id: webrender_api::PipelineId,
        viewport_size: webrender_api::units::LayoutSize,
    ) -> Self {
        Self {
            pipeline_id,
            is_contentful: false,
            wr: webrender_api::DisplayListBuilder::new(pipeline_id, viewport_size),
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
        let background_color = self
            .style
            .resolve_color(self.style.clone_background_color());
        if background_color.alpha > 0 {
            let clip_rect = self
                .border_rect()
                .to_physical(self.style.writing_mode(), containing_block)
                .translate(&containing_block.top_left)
                .into();
            let common = CommonItemProperties {
                clip_rect,
                clip_id: webrender_api::ClipId::root(builder.pipeline_id),
                spatial_id: webrender_api::SpatialId::root_scroll_node(builder.pipeline_id),
                hit_info: None,
                // TODO(gw): Make use of the WR backface visibility functionality.
                is_backface_visible: true,
            };
            builder.wr.push_rect(&common, rgba(background_color))
        }
        let content_rect = self
            .content_rect
            .to_physical(self.style.writing_mode(), containing_block)
            .translate(&containing_block.top_left);
        for child in &self.children {
            child.build_display_list(builder, is_contentful, &content_rect)
        }
    }
}

fn rgba(rgba: cssparser::RGBA) -> webrender_api::ColorF {
    webrender_api::ColorF::new(
        rgba.red_f32(),
        rgba.green_f32(),
        rgba.blue_f32(),
        rgba.alpha_f32(),
    )
}
