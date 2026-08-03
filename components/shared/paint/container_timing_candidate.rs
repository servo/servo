/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Definitions for Container Timing candidates.
//!
//! <https://wicg.github.io/container-timing/>

use euclid::Rect;
use serde::{Deserialize, Serialize};
use servo_base::cross_process_instant::CrossProcessInstant;
use style_traits::CSSPixel;

/// A container timing candidate, sent from layout to the paint thread, and also used
/// by the paint thread to track the current best entry for each container identifier.
/// Represents the painted area of a descendant element within a container
/// that has the `containertiming` attribute.
///
/// `first_render_time` and `paint_time` are unknown at construction time: layout has no
/// notion of when compositing actually happens, so records are created with both unset
/// and only [`ContainerTimingRecord::mark_painted`] fills them in, once the paint thread
/// knows the real paint time.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ContainerTimingRecord {
    /// A stable per-node identity for the container root, derived from
    /// `style::dom::OpaqueNode::id()`. Used to correlate candidates for the same
    /// container across builds and paints, since `identifier` (the `containertiming`
    /// attribute value) is optional and not guaranteed unique -- multiple distinct
    /// containers may share the same (or no) identifier.
    pub container_id: usize,
    /// The value of the `containertiming` attribute on the container element.
    pub identifier: String,
    /// The viewport-clipped painted area in CSS pixels (as area = width * height).
    pub size: usize,
    /// The time of the first paint for this container. Set once and never changed
    /// afterwards.
    pub first_render_time: Option<CrossProcessInstant>,
    /// The most recent paint time for this container.
    pub paint_time: Option<CrossProcessInstant>,
    /// The viewport-clipped, union'd painted rect in CSS pixels.
    pub intersection_rect: Rect<f32, CSSPixel>,
}

impl ContainerTimingRecord {
    pub fn new(
        container_id: usize,
        identifier: String,
        size: usize,
        intersection_rect: Rect<f32, CSSPixel>,
    ) -> Self {
        Self {
            container_id,
            identifier,
            size,
            first_render_time: None,
            paint_time: None,
            intersection_rect,
        }
    }

    /// Records a paint of this container at `time`. `paint_time` is updated every time,
    /// but `first_render_time` is only ever set the first time this is called.
    pub fn mark_painted(&mut self, time: CrossProcessInstant) {
        self.first_render_time.get_or_insert(time);
        self.paint_time = Some(time);
    }

    /// Updates this record's identifier, size, and rect from a freshly-arrived `latest`
    /// candidate, leaving `first_render_time`/`paint_time` untouched -- those are only
    /// ever set by [`Self::mark_painted`], and overwriting the whole record would erase
    /// them, making a later `mark_painted` treat an already-painted container as if it
    /// were being painted for the first time.
    pub fn update_metrics(&mut self, latest: ContainerTimingRecord) {
        self.identifier = latest.identifier;
        self.size = latest.size;
        self.intersection_rect = latest.intersection_rect;
    }
}
