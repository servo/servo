/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Specified types for SVG properties.

use values::generics::svg as generic;
use values::specified::color::RGBAColor;

/// Specified SVG Paint value
pub type SVGPaint = generic::SVGPaint<RGBAColor>;

no_viewport_percentage!(SVGPaint);

/// Specified SVG Paint Kind value
pub type SVGPaintKind = generic::SVGPaintKind<RGBAColor>;
