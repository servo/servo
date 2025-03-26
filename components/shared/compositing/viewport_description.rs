/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! This module contains helpers for Viewport

use std::collections::HashMap;

use euclid::default::Scale;
use serde::{Deserialize, Serialize};

/// Default viewport constraints
///
/// <https://developer.mozilla.org/en-US/docs/Web/HTML/Viewport_meta_tag#initial-scale>
pub const MIN_ZOOM: f32 = 0.1;
/// <https://developer.mozilla.org/en-US/docs/Web/HTML/Viewport_meta_tag#initial-scale>
pub const MAX_ZOOM: f32 = 10.0;
/// <https://developer.mozilla.org/en-US/docs/Web/HTML/Viewport_meta_tag#initial-scale>
pub const DEFAULT_ZOOM: f32 = 1.0;

/// A set of viewport descriptors:
///
/// <https://www.w3.org/TR/css-viewport-1/#viewport-meta>
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ViewportDescription {
    // https://developer.mozilla.org/en-US/docs/Web/HTML/Viewport_meta_tag#width
    // the (minimum width) size of the viewport
    // TODO: width Needs to be implemented
    // https://developer.mozilla.org/en-US/docs/Web/HTML/Viewport_meta_tag#width
    // the (minimum height) size of the viewport
    // TODO: height Needs to be implemented
    /// <https://developer.mozilla.org/en-US/docs/Web/HTML/Viewport_meta_tag#initial-scale>
    /// the zoom level when the page is first loaded
    pub initial_scale: Scale<f32>,

    /// <https://developer.mozilla.org/en-US/docs/Web/HTML/Viewport_meta_tag#minimum_scale>
    /// how much zoom out is allowed on the page.
    pub minimum_scale: Scale<f32>,

    /// <https://developer.mozilla.org/en-US/docs/Web/HTML/Viewport_meta_tag#maximum_scale>
    /// how much zoom in is allowed on the page
    pub maximum_scale: Scale<f32>,

    /// <https://developer.mozilla.org/en-US/docs/Web/HTML/Viewport_meta_tag#user_scalable>
    /// whether zoom in and zoom out actions are allowed on the page
    pub user_scalable: UserScalable,
}

/// The errors that the viewport parsing can generate.
#[derive(Debug)]
pub enum ViewportDescriptionParseError {
    /// When viewport attribute string is empty
    Empty,
}

/// A set of User Zoom values:
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum UserScalable {
    /// Zoom is not allowed
    No = 0,
    /// Zoom is allowed
    Yes = 1,
}

/// Parses a viewport user scalable value.
impl TryFrom<&str> for UserScalable {
    type Error = &'static str;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "yes" => Ok(UserScalable::Yes),
            "no" => Ok(UserScalable::No),
            _ => match value.parse::<f32>() {
                Ok(1.0) => Ok(UserScalable::Yes),
                Ok(0.0) => Ok(UserScalable::No),
                _ => Err("can't convert character to UserScalable"),
            },
        }
    }
}

impl Default for ViewportDescription {
    fn default() -> Self {
        ViewportDescription {
            initial_scale: Scale::new(DEFAULT_ZOOM),
            minimum_scale: Scale::new(MIN_ZOOM),
            maximum_scale: Scale::new(MAX_ZOOM),
            user_scalable: UserScalable::Yes,
        }
    }
}

impl ViewportDescription {
    /// Iterates over the key-value pairs generated from meta tag and returns a ViewportDescription
    fn process_viewport_key_value_pair(pairs: HashMap<String, String>) -> ViewportDescription {
        let mut description = ViewportDescription::default();
        for (key, value) in &pairs {
            match key.as_str() {
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
                    if let Ok(user_zoom_allowed) = value.as_str().try_into() {
                        description.user_scalable = user_zoom_allowed;
                    }
                },
                _ => (),
            }
        }
        description
    }

    /// Parses a viewport zoom value.
    fn parse_viewport_value_as_zoom(value: &str) -> Option<Scale<f32>> {
        value
            .to_lowercase()
            .as_str()
            .parse::<f32>()
            .ok()
            .filter(|&n| (0.0..=10.0).contains(&n))
            .map(Scale::new)
    }

    /// Constrains a zoom value within the allowed scale range
    pub fn clamp_zoom(&self, zoom: f32) -> f32 {
        zoom.clamp(self.minimum_scale.get(), self.maximum_scale.get())
    }
}
