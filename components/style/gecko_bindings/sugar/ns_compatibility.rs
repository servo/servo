/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Little helper for `nsCompatibility`.

use context::QuirksMode;
use gecko_bindings::structs::nsCompatibility;

impl From<nsCompatibility> for QuirksMode {
    #[inline]
    fn from(mode: nsCompatibility) -> QuirksMode {
        match mode {
            nsCompatibility::eCompatibility_FullStandards => QuirksMode::NoQuirks,
            nsCompatibility::eCompatibility_AlmostStandards => QuirksMode::LimitedQuirks,
            nsCompatibility::eCompatibility_NavQuirks => QuirksMode::Quirks,
        }
    }
}
