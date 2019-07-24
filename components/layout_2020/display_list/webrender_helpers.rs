/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::display_list::items::DisplayList;
use msg::constellation_msg::PipelineId;
use webrender_api::{self, DisplayListBuilder};
use webrender_api::units::{LayoutSize};

pub trait WebRenderDisplayListConverter {
    fn convert_to_webrender(&mut self, pipeline_id: PipelineId) -> DisplayListBuilder;
}

impl WebRenderDisplayListConverter for DisplayList {
    fn convert_to_webrender(&mut self, pipeline_id: PipelineId) -> DisplayListBuilder {
        let webrender_pipeline = pipeline_id.to_webrender();

        let builder = DisplayListBuilder::with_capacity(
            webrender_pipeline,
            LayoutSize::zero(),
            1024 * 1024, // 1 MB of space
        );

        builder
    }
}
