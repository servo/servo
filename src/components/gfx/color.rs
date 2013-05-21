/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use azure::AzFloat;
use AzColor = azure::azure_hl::Color;

pub type Color = AzColor;

pub fn rgb(r: u8, g: u8, b: u8) -> AzColor {
    rgba(r, g, b, 1.0)
}

pub fn rgba(r: u8, g: u8, b: u8, a: float) -> AzColor {
    AzColor {
        r: (r as AzFloat) / (255.0 as AzFloat),
        g: (g as AzFloat) / (255.0 as AzFloat),
        b: (b as AzFloat) / (255.0 as AzFloat),
        a: a as AzFloat
    }
}

