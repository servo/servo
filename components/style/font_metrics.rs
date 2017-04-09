/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Access to font metrics from the style system.

#![deny(missing_docs)]

use Atom;
use app_units::Au;
use context::SharedStyleContext;
use euclid::Size2D;
use std::fmt;

/// Represents the font metrics that style needs from a font to compute the
/// value of certain CSS units like `ex`.
#[derive(Debug, PartialEq, Clone)]
pub struct FontMetrics {
    /// The x-height of the font.
    pub x_height: Au,
    /// The zero advance.
    pub zero_advance_measure: Size2D<Au>,
}

/// The result for querying font metrics for a given font family.
#[derive(Debug, PartialEq, Clone)]
pub enum FontMetricsQueryResult {
    /// The font is available, but we may or may not have found any font metrics
    /// for it.
    Available(Option<FontMetrics>),
    /// The font is not available.
    NotAvailable,
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

    fn get_size(&self, _font_name: &Atom, _font_family: u8) -> Au {
        unreachable!("Dummy provider should never be used to compute font size")
    }
}

// Servo's font metrics provider will probably not live in this crate, so this will
// have to be replaced with something else (perhaps a trait method on TElement)
// when we get there

#[cfg(feature = "gecko")]
/// Construct a font metrics provider for the current product
pub fn get_metrics_provider_for_product() -> ::gecko::wrapper::GeckoFontMetricsProvider {
    ::gecko::wrapper::GeckoFontMetricsProvider::new()
}

#[cfg(feature = "servo")]
/// Construct a font metrics provider for the current product
pub fn get_metrics_provider_for_product() -> ServoMetricsProvider {
    ServoMetricsProvider
}

/// A trait used to represent something capable of providing us font metrics.
pub trait FontMetricsProvider: fmt::Debug {
    /// Obtain the metrics for given font family.
    ///
    /// TODO: We could make this take the full list, I guess, and save a few
    /// virtual calls in the case we are repeatedly unable to find font metrics?
    /// That is not too common in practice though.
    fn query(&self, _font_name: &Atom) -> FontMetricsQueryResult {
        FontMetricsQueryResult::NotAvailable
    }

    /// Get default size of a given language and generic family
    fn get_size(&self, font_name: &Atom, font_family: u8) -> Au;

    /// Construct from a shared style context
    fn create_from(context: &SharedStyleContext) -> Self where Self: Sized;
}

