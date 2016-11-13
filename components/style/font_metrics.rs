/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use Atom;
use app_units::Au;
use euclid::Size2D;
use std::fmt;

/// Represents the font metrics that style needs from a font to compute the
/// value of certain CSS units like `ex`.
#[derive(Debug, PartialEq, Clone)]
pub struct FontMetrics {
    pub x_height: Au,
    pub zero_advance_measure: Size2D<Au>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum FontMetricsQueryResult {
    Available(Option<FontMetrics>),
    NotAvailable,
}

/// A trait used to represent something capable of providing us font metrics.
pub trait FontMetricsProvider: Send + Sync + fmt::Debug {
    /// Obtain the metrics for given font family.
    ///
    /// TODO: We could make this take the full list, I guess, and save a few
    /// virtual calls.
    ///
    /// This is not too common in practice though.
    fn query(&self, _font_name: &Atom) -> FontMetricsQueryResult {
        FontMetricsQueryResult::NotAvailable
    }
}
