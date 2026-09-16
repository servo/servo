/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Definitions for Container Timing records.
//!
//! <https://wicg.github.io/container-timing/>

use euclid::Box2D;
use malloc_size_of_derive::MallocSizeOf;
use serde::{Deserialize, Serialize};
use servo_base::id::ContainerTimingID;
use style::dom::OpaqueNode;
use webrender_api::units::LayoutPixel;

/// One update to a Container Timing container, produced by layout and handed to script.
///
/// Like [`LCPCandidate`], this never reaches the paint thread: paint only receives the
/// [`ContainerTimingID`], which it hands back once it knows the frame's real paint time.
/// Script parks the record against that ID until then.
///
/// [`LCPCandidate`]: crate::LCPCandidate
#[derive(Clone, Debug, Deserialize, MallocSizeOf, Serialize)]
pub struct ContainerTimingRecord {
    /// A unique identifier for this particular update.
    pub id: ContainerTimingID,
    /// A stable identity for the container itself, derived from [`OpaqueNode::id`] of the
    /// container root. Successive updates to the same container share this, which is what
    /// lets script carry `firstRenderTime` forward across them. It can't be keyed on
    /// `identifier`, since that is optional and not guaranteed unique.
    pub container_id: usize,
    /// The value of the `containertiming` attribute on the container element.
    pub identifier: String,
    /// The painted area within the container, in CSS pixels. This is the area of the
    /// union of every painted descendant region seen so far, so overlapping fragments
    /// are not double-counted.
    pub size: f32,
    /// The viewport-clipped, union'd painted rect, in CSS pixels.
    pub intersection_rect: Box2D<f32, LayoutPixel>,
    /// The container root element, for `PerformanceContainerTiming::rootElement`.
    pub root_element: OpaqueNode,
    /// The descendant whose paint triggered this update, for
    /// `PerformanceContainerTiming::lastPaintedElement`.
    pub last_painted_element: Option<OpaqueNode>,
}
