/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! This module contains helpers for Viewport

use std::collections::HashMap;

use euclid::default::Scale;
use serde::{Deserialize, Serialize};

/// A set of values for Length:
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum ViewportLength {
    /// Length is auto
    Auto,
    /// Length is device width
    DeviceWidth,
    /// Length is device height
    DeviceHeight,
    /// Length is fixed
    Fixed(f32),
}

/// A set of User Zoom values:
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum UserZoom {
    /// Zoom is not allowed
    Fixed = 0,
    /// Zoom is allowed
    Zoom = 1,
}

/// Default viewport constraints
///
/// <https://developer.mozilla.org/en-US/docs/Web/HTML/Viewport_meta_tag#initial-scale>
pub const MIN_ZOOM: f32 = 0.1;
pub const MAX_ZOOM: f32 = 10.0;
pub const DEFAULT_ZOOM: f32 = 1.0;

/// A set of viewport descriptors:
///
/// <https://www.w3.org/TR/css-viewport-1/#viewport-meta>
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ViewportDescription {
    /// TODO: Unused, need to find its usage.
    /// width
    pub width: ViewportLength,
    /// TODO: Unused, need to find its usage.
    /// height
    pub height: ViewportLength,
    /// initial-scale
    pub initial_scale: Scale<f32>,
    /// minimum-scale
    pub minimum_scale: Scale<f32>,
    /// maximum-scale
    pub maximum_scale: Scale<f32>,
    /// user-scalable
    pub user_scalable: UserZoom,
}

impl Default for ViewportDescription {
    fn default() -> Self {
        Self::new()
    }
}

impl ViewportDescription {
    /// Creates a Viewport
    pub fn new() -> Self {
        ViewportDescription {
            width: ViewportLength::Auto,
            height: ViewportLength::Auto,
            initial_scale: Scale::new(DEFAULT_ZOOM),
            minimum_scale: Scale::new(MIN_ZOOM),
            maximum_scale: Scale::new(MAX_ZOOM),
            user_scalable: UserZoom::Zoom,
        }
    }

    /// Process key value pair in Viewport
    pub fn process_viewport_key_value_pair(pairs: HashMap<String, String>) -> ViewportDescription {
        let mut description = ViewportDescription::new();
        for (key, value) in &pairs {
            match key.as_str() {
                "width" | "height" => {
                    let length = Self::parse_viewport_value_as_length(value);
                    if length != ViewportLength::Auto {
                        if key == "width" {
                            description.width = length;
                        } else {
                            description.height = length;
                        }
                    }
                },
                "initial-scale" => {
                    if let Some(zoom) = Self::parse_viewport_value_as_zoom(value) {
                        description.initial_scale = zoom;
                    }
                },
                "minimum-scale" => {
                    if let Some(zoom) = Self::parse_viewport_value_as_zoom(value) {
                        description.minimum_scale = zoom;
                    }
                },
                "maximum-scale" => {
                    if let Some(zoom) = Self::parse_viewport_value_as_zoom(value) {
                        description.maximum_scale = zoom;
                    }
                },
                "user-scalable" => {
                    if let Some(user_zoom) = Self::parse_viewport_value_as_user_zoom(value) {
                        description.user_scalable = user_zoom;
                    }
                },
                _ => (),
            }
        }
        description
    }

    /// Parses a viewport length value.
    fn parse_viewport_value_as_length(value: &str) -> ViewportLength {
        match value.to_lowercase().as_str() {
            "device-width" => ViewportLength::DeviceWidth,
            "device-height" => ViewportLength::DeviceHeight,
            _ => value
                .parse::<f32>()
                .ok()
                .filter(|&n| n >= 0.0)
                .map(ViewportLength::Fixed)
                .unwrap_or(ViewportLength::Auto),
        }
    }

    fn parse_viewport_value_as_zoom(value: &str) -> Option<Scale<f32>> {
        value
            .to_lowercase()
            .as_str()
            .parse::<f32>()
            .ok()
            .filter(|&n| (0.0..=10.0).contains(&n))
            .map(Scale::new)
    }

    /// Parses a viewport user zoom value.
    fn parse_viewport_value_as_user_zoom(value: &str) -> Option<UserZoom> {
        Some(match value.to_lowercase().as_str() {
            "yes" => UserZoom::Zoom,
            "no" => UserZoom::Fixed,
            _ => match value.parse::<f32>() {
                Ok(n) if n >= 1.0 => UserZoom::Zoom,
                Ok(_) => UserZoom::Fixed,
                _ => return None,
            },
        })
    }
}
