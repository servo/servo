/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Access to font metrics from the style system.

#![deny(missing_docs)]

use crate::context::SharedStyleContext;
use crate::Atom;
use app_units::Au;

/// Represents the font metrics that style needs from a font to compute the
/// value of certain CSS units like `ex`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FontMetrics {
    /// The x-height of the font.
    pub x_height: Option<Au>,
    /// The zero advance. This is usually writing mode dependent
    pub zero_advance_measure: Option<Au>,
}

/// Type of font metrics to retrieve.
#[derive(Clone, Debug, PartialEq)]
pub enum FontMetricsOrientation {
    /// Get metrics for horizontal or vertical according to the Context's
    /// writing mode.
    MatchContext,
    /// Force getting horizontal metrics.
    Horizontal,
}

/// A trait used to represent something capable of providing us font metrics.
pub trait FontMetricsProvider {
    /// Obtain the metrics for given font family.
    fn query(
        &self,
        _context: &crate::values::computed::Context,
        _base_size: crate::values::specified::length::FontBaseSize,
        _orientation: FontMetricsOrientation,
    ) -> FontMetrics {
        Default::default()
    }

    /// Get default size of a given language and generic family.
    fn get_size(
        &self,
        font_name: &Atom,
        font_family: crate::values::computed::font::GenericFontFamily,
    ) -> Au;

    /// Construct from a shared style context
    fn create_from(context: &SharedStyleContext) -> Self
    where
        Self: Sized;
}

// TODO: Servo's font metrics provider will probably not live in this crate, so this will
// have to be replaced with something else (perhaps a trait method on TElement)
// when we get there
#[derive(Debug)]
#[cfg(feature = "servo")]
/// Dummy metrics provider for Servo. Knows nothing about fonts and does not provide
/// any metrics.
pub struct ServoMetricsProvider;

#[cfg(feature = "servo")]
impl FontMetricsProvider for ServoMetricsProvider {
    fn create_from(_: &SharedStyleContext) -> Self {
        ServoMetricsProvider
    }

    fn get_size(&self, _: &Atom, _: crate::values::computed::font::GenericFontFamily) -> Au {
        unreachable!("Dummy provider should never be used to compute font size")
    }
}

// Servo's font metrics provider will probably not live in this crate, so this will
// have to be replaced with something else (perhaps a trait method on TElement)
// when we get there

#[cfg(feature = "gecko")]
/// Construct a font metrics provider for the current product
pub fn get_metrics_provider_for_product() -> crate::gecko::wrapper::GeckoFontMetricsProvider {
    crate::gecko::wrapper::GeckoFontMetricsProvider::new()
}

#[cfg(feature = "servo")]
/// Construct a font metrics provider for the current product
pub fn get_metrics_provider_for_product() -> ServoMetricsProvider {
    ServoMetricsProvider
}
