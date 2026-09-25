/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use euclid::Box2D;
use malloc_size_of_derive::MallocSizeOf;
use serde::{Deserialize, Serialize};
use servo_base::id::ContainerTimingID;
use style::dom::OpaqueNode;
use webrender_api::units::LayoutPixel;

#[derive(Clone, Debug, Deserialize, MallocSizeOf, Serialize)]
pub struct ContainerTimingRecord {
    /// A unique identifier for this particular update.
    pub id: ContainerTimingID,
    /// A stable identity for the container itself, from [`OpaqueNode::id`] of the
    /// container root.
    pub container_id: usize,
    /// The value of the `containertiming` attribute on the container element.
    pub identifier: String,
    /// The painted area within the container, in CSS pixels.
    pub size: f32,
    /// The viewport-clipped, union'd painted rect, in CSS pixels.
    pub intersection_rect: Box2D<f32, LayoutPixel>,
    /// The container root element, for `PerformanceContainerTiming::rootElement`.
    pub root_element: OpaqueNode,
    /// The descendant whose paint triggered this update.
    pub last_painted_element: Option<OpaqueNode>,
}
