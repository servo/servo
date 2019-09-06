/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use euclid::Vector2D;
use std::collections::HashMap;
use std::f32;
use webrender_api::units::LayoutPixel;
use webrender_api::ExternalScrollId;

pub use style::dom::OpaqueNode;

#[derive(Serialize)]
pub struct DisplayList {}

/// The type of the scroll offset list. This is only populated if WebRender is in use.
pub type ScrollOffsetMap = HashMap<ExternalScrollId, Vector2D<f32, LayoutPixel>>;
