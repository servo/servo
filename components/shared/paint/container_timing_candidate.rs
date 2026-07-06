/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Definitions for Container Timing candidates.
//!
//! <https://wicg.github.io/container-timing/>

use serde::{Deserialize, Serialize};
use servo_base::cross_process_instant::CrossProcessInstant;

/// A container timing candidate, sent from layout to the paint thread.
/// Represents the painted area of a descendant element within a container
/// that has the `containertiming` attribute.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ContainerTimingRecord {
    /// The value of the `containertiming` attribute on the container element.
    pub identifier: String,
    /// The viewport-clipped painted area in CSS pixels (as area = width * height).
    pub size: usize,
    /// The viewport-clipped rect (x, y, width, height) in CSS pixels.
    pub rect_x: f32,
    pub rect_y: f32,
    pub rect_width: f32,
    pub rect_height: f32,
}

impl ContainerTimingRecord {
    pub fn new(
        identifier: String,
        size: usize,
        rect_x: f32,
        rect_y: f32,
        rect_width: f32,
        rect_height: f32,
    ) -> Self {
        Self {
            identifier,
            size,
            rect_x,
            rect_y,
            rect_width,
            rect_height,
        }
    }
}

/// A completed container timing entry, stored in the paint thread.
#[derive(Clone, Debug)]
pub struct ContainerTiming {
    /// The value of the `containertiming` attribute.
    pub identifier: String,
    /// Total accumulated painted size.
    pub size: usize,
    /// The bounding rect of all accumulated painted regions.
    pub rect_x: f32,
    pub rect_y: f32,
    pub rect_width: f32,
    pub rect_height: f32,
    /// The time of the first paint for this container.
    pub first_render_time: CrossProcessInstant,
    /// The most recent paint time for this container.
    pub paint_time: CrossProcessInstant,
}
