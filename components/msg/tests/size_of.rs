/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![cfg(target_pointer_width = "64")]

extern crate msg;
#[macro_use] extern crate size_of_test;

use msg::constellation_msg::BrowsingContextId;
use msg::constellation_msg::PipelineId;
use msg::constellation_msg::TopLevelBrowsingContextId;

size_of_test!(test_size_of_pipeline_id, PipelineId, 8);
size_of_test!(test_size_of_optional_pipeline_id, Option<PipelineId>, 8);
size_of_test!(test_size_of_browsing_context_id, BrowsingContextId, 8);
size_of_test!(test_size_of_optional_browsing_context_id, Option<BrowsingContextId>, 8);
size_of_test!(test_size_of_top_level_browsing_context_id, TopLevelBrowsingContextId, 8);
size_of_test!(test_size_of_top_level_optional_browsing_context_id, Option<TopLevelBrowsingContextId>, 8);
