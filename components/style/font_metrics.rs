/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Access to font metrics from the style system.

#![deny(missing_docs)]

use crate::values::computed::Length;

/// Represents the font metrics that style needs from a font to compute the
/// value of certain CSS units like `ex`.
#[derive(Clone, Debug, PartialEq)]
pub struct FontMetrics {
    /// The x-height of the font.
    pub x_height: Option<Length>,
    /// The zero advance. This is usually writing mode dependent
    pub zero_advance_measure: Option<Length>,
    /// The cap-height of the font.
    pub cap_height: Option<Length>,
    /// The ideographic-width of the font.
    pub ic_width: Option<Length>,
    /// The ascent of the font (a value is always available for this).
    pub ascent: Length,
    /// Script scale down factor for math-depth 1.
    /// https://w3c.github.io/mathml-core/#dfn-scriptpercentscaledown
    pub script_percent_scale_down: Option<f32>,
    /// Script scale down factor for math-depth 2.
    /// https://w3c.github.io/mathml-core/#dfn-scriptscriptpercentscaledown
    pub script_script_percent_scale_down: Option<f32>,
}

impl Default for FontMetrics {
    fn default() -> Self {
        FontMetrics {
            x_height: None,
            zero_advance_measure: None,
            cap_height: None,
            ic_width: None,
            ascent: Length::new(0.0),
            script_percent_scale_down: None,
            script_script_percent_scale_down: None,
        }
    }
}

/// Type of font metrics to retrieve.
#[derive(Clone, Debug, PartialEq)]
pub enum FontMetricsOrientation {
    /// Get metrics for horizontal or vertical according to the Context's
    /// writing mode, using horizontal metrics for vertical/mixed
    MatchContextPreferHorizontal,
    /// Get metrics for horizontal or vertical according to the Context's
    /// writing mode, using vertical metrics for vertical/mixed
    MatchContextPreferVertical,
    /// Force getting horizontal metrics.
    Horizontal,
}
