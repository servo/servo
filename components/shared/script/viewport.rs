/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! This module contains implementations in script that are transferable.
//! The implementations are here instead of in script
//! so that the other modules involved in the transfer don't have
//! to depend on script.
use euclid::Size2D;
use style_traits::{CSSPixel, PinchZoomFactor};

/// A set of viewport descriptors:
///
/// <https://drafts.csswg.org/css-device-adapt/#viewport-desc>
#[derive(Clone, Debug, PartialEq)]
pub struct ViewportDescription {
    /// Width and height:
    ///  * https://drafts.csswg.org/css-device-adapt/#width-desc
    ///  * https://drafts.csswg.org/css-device-adapt/#height-desc
    pub size: Size2D<f32, CSSPixel>,
    /// <https://drafts.csswg.org/css-device-adapt/#zoom-desc>
    pub initial_zoom: PinchZoomFactor,
    /// <https://drafts.csswg.org/css-device-adapt/#min-max-width-desc>
    pub min_zoom: Option<PinchZoomFactor>,
    /// <https://drafts.csswg.org/css-device-adapt/#min-max-width-desc>
    pub max_zoom: Option<PinchZoomFactor>,
    /// <https://drafts.csswg.org/css-device-adapt/#user-zoom-desc>
    pub user_zoom: bool,
}
