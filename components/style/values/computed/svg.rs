/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Computed types for SVG properties.

use values::RGBA;
use values::generics::svg as generic;

/// Computed SVG Paint value
pub type SVGPaint = generic::SVGPaint<RGBA>;
/// Computed SVG Paint Kind value
pub type SVGPaintKind = generic::SVGPaintKind<RGBA>;

impl Default for SVGPaint {
    fn default() -> Self {
        SVGPaint {
            kind: generic::SVGPaintKind::None,
            fallback: None,
        }
    }
}

impl SVGPaint {
    /// Opaque black color
    pub fn black() -> Self {
        let rgba = RGBA::from_floats(0., 0., 0., 1.);
        SVGPaint {
            kind: generic::SVGPaintKind::Color(rgba),
            fallback: None,
        }
    }
}
