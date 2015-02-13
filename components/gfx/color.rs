/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use azure::AzFloat;
use azure::azure::AzColor;

#[inline]
pub fn new(r: AzFloat, g: AzFloat, b: AzFloat, a: AzFloat) -> AzColor {
    AzColor { r: r, g: g, b: b, a: a }
}

#[inline]
pub fn rgb(r: u8, g: u8, b: u8) -> AzColor {
    AzColor {
        r: (r as AzFloat) / (255.0 as AzFloat),
        g: (g as AzFloat) / (255.0 as AzFloat),
        b: (b as AzFloat) / (255.0 as AzFloat),
        a: 1.0 as AzFloat
    }
}

#[inline]
pub fn rgba(r: AzFloat, g: AzFloat, b: AzFloat, a: AzFloat) -> AzColor {
    AzColor { r: r, g: g, b: b, a: a }
}

#[inline]
pub fn black() -> AzColor {
    AzColor { r: 0.0, g: 0.0, b: 0.0, a: 1.0 }
}

#[inline]
pub fn transparent_black() -> AzColor {
    AzColor { r: 0.0, g: 0.0, b: 0.0, a: 0.0 }
}

#[inline]
pub fn white() -> AzColor {
    AzColor { r: 1.0, g: 1.0, b: 1.0, a: 1.0 }
}
