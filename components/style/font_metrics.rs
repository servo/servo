/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Access to font metrics from the style system.

#![deny(missing_docs)]

use Atom;
use app_units::Au;
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

/// A trait used to represent something capable of providing us font metrics.
pub trait FontMetricsProvider: Send + Sync + fmt::Debug {
    /// Obtain the metrics for given font family.
    ///
    /// TODO: We could make this take the full list, I guess, and save a few
    /// virtual calls in the case we are repeatedly unable to find font metrics?
    /// That is not too common in practice though.
    fn query(&self, _font_name: &Atom) -> FontMetricsQueryResult {
        FontMetricsQueryResult::NotAvailable
    }
}
