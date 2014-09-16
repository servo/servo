/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use azure::AzFloat;
use azure::azure_hl::Color as AzColor;

pub type Color = AzColor;

pub fn rgb(r: u8, g: u8, b: u8) -> AzColor {
    AzColor {
        r: (r as AzFloat) / (255.0 as AzFloat),
        g: (g as AzFloat) / (255.0 as AzFloat),
        b: (b as AzFloat) / (255.0 as AzFloat),
        a: 1.0 as AzFloat
    }
}

pub fn rgba(r: AzFloat, g: AzFloat, b: AzFloat, a: AzFloat) -> AzColor {
    AzColor { r: r, g: g, b: b, a: a }
}
