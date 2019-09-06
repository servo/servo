/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::display_list::items::DisplayList;
use msg::constellation_msg::PipelineId;
use webrender_api::units::LayoutSize;
use webrender_api::{self, DisplayListBuilder};

/// Contentful paint, for the purpose of
/// https://w3c.github.io/paint-timing/#first-contentful-paint
/// (i.e. the display list contains items of type text,
/// image, non-white canvas or SVG). Used by metrics.
pub struct IsContentful(pub bool);

impl DisplayList {
    pub fn convert_to_webrender(
        &mut self,
        pipeline_id: PipelineId,
    ) -> (DisplayListBuilder, IsContentful) {
        let webrender_pipeline = pipeline_id.to_webrender();

        let builder = DisplayListBuilder::with_capacity(
            webrender_pipeline,
            LayoutSize::zero(),
            1024 * 1024, // 1 MB of space
        );

        let is_contentful = IsContentful(false);

        (builder, is_contentful)
    }
}
