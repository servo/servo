/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! This module contains implementations in script that are transferable.
//! The implementations are here instead of in script
//! so that the other modules involved in the transfer don't have
//! to depend on script.
use std::collections::HashMap;

use euclid::Size2D;
use style_traits::{CSSPixel, PinchZoomFactor};

/// A set of viewport descriptors:
///
/// <https://drafts.csswg.org/css-device-adapt/#viewport-desc>
#[derive(Clone, Debug, PartialEq)]
pub struct ViewportDescription {
    /// Width and height:
    ///  * https://drafts.csswg.org/css-device-adapt/#width-desc
    pub min_width: f32,
    pub max_width: f32,
    ///  * https://drafts.csswg.org/css-device-adapt/#height-desc
    pub min_height: f32,
    pub max_height: f32,
    /// <https://drafts.csswg.org/css-device-adapt/#zoom-desc>
    pub initial_zoom: PinchZoomFactor,
    /// <https://drafts.csswg.org/css-device-adapt/#min-max-width-desc>
    pub min_zoom: PinchZoomFactor,
    /// <https://drafts.csswg.org/css-device-adapt/#min-max-width-desc>
    pub max_zoom: PinchZoomFactor,
    /// <https://drafts.csswg.org/css-device-adapt/#user-zoom-desc>
    pub user_zoom: UserZoom,
}
/// A set of User Zoom values:
#[derive(Clone, Debug, PartialEq)]
pub enum UserZoom {
    /// Zoom is not allowed
    Fixed = 0,
    /// Zoom is allowed
    Zoom = 1,
}

// pub fn parse_viewport_value_as_length(: &mut ViewportDescription) {
//     dbg!("DebugSG",description);
// }

impl ViewportDescription {
    /// Creates Viewport
    pub fn new() -> Self {
        let description = ViewportDescription {
            size: Size2D::new(0.1, 0.1),
            initial_zoom: PinchZoomFactor::new(1.0),
            min_zoom: PinchZoomFactor::new(1.0),
            max_zoom: PinchZoomFactor::new(10.0),
            user_zoom: UserZoom::Zoom,
        };
        description
    }

    /// Process viewport
    pub fn process_viewport_key_value_pair(
        description: &mut ViewportDescription,
        pairs: HashMap<String, String>,
    ) {
        for (key, value) in &pairs {
            match key.as_ref() {
                "width" => (), //escription.size = parse_viewport_value_as_length(value),
                "height" => (),
                "initial-scale" => {
                    description.initial_zoom = Self::parse_viewport_value_as_zoom(value.to_string())
                },
                "minimum-scale" => {
                    description.min_zoom = Self::parse_viewport_value_as_zoom(value.to_string())
                },
                "maximum-scale" => {
                    description.max_zoom = Self::parse_viewport_value_as_zoom(value.to_string())
                },
                "user-scalable" => {
                    description.user_zoom =
                        Self::parse_viewport_value_as_user_zoom(value.to_string())
                },
                _ => (),
            }
        }
    }

    /// Parse a viewport value as zoom
    fn parse_viewport_value_as_zoom(value: String) -> PinchZoomFactor {
        match value {
            v if v.eq_ignore_ascii_case("yes") => PinchZoomFactor::new(1.0),
            v if v.eq_ignore_ascii_case("no") => PinchZoomFactor::new(0.1),
            v if v.eq_ignore_ascii_case("device-width") => PinchZoomFactor::new(10.0),
            v if v.eq_ignore_ascii_case("device-height") => PinchZoomFactor::new(10.0),
            _ => value
                .parse::<f32>()
                .ok()
                .filter(|&n| (0.0..=10.0).contains(&n))
                .map(PinchZoomFactor::new)
                .unwrap(),
        }
    }

    /// Parse a viewport value as user zoom
    fn parse_viewport_value_as_user_zoom(value: String) -> UserZoom {
        match value {
            v if v.eq_ignore_ascii_case("yes") => UserZoom::Zoom,
            v if v.eq_ignore_ascii_case("no") => UserZoom::Fixed,
            v if v.eq_ignore_ascii_case("device-width") => UserZoom::Zoom,
            v if v.eq_ignore_ascii_case("device-height") => UserZoom::Zoom,
            _ => match value.parse::<f32>() {
                Ok(n) if n >= 1. || n <= -1. => UserZoom::Zoom,
                _ => UserZoom::Fixed,
            },
        }
    }
}
