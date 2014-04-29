/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::navigator::Navigator;
use servo_util::str::DOMString;

pub trait NavigatorLanguage {
    fn GetLanguage(&self) -> Option<DOMString> {
        None
    }
}

impl NavigatorLanguage for Navigator {}
