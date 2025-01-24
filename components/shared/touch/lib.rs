/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! This module contains traits in touch used generically in the rest of Servo.
//! The traits are here instead of in touch so that these modules won't have
//! to depend on touch.

use euclid::Vector2D;
use serde::{Deserialize, Serialize};
use webrender_api::units::{DeviceIntPoint, DevicePixel, DevicePoint};

/// The action to take in response to a touch event
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub enum TouchAction {
    /// Simulate a mouse click.
    Click(DevicePoint),
    /// Fling by the provided offset
    Flinging(Vector2D<f32, DevicePixel>, DeviceIntPoint),
    /// Scroll by the provided offset.
    Scroll(Vector2D<f32, DevicePixel>, DevicePoint),
    /// Zoom by a magnification factor and scroll by the provided offset.
    Zoom(f32, Vector2D<f32, DevicePixel>),
    /// Don't do anything.
    NoAction,
}
