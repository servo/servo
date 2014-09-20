/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use servo_util::str::DOMString;

pub struct NavigatorInfo;

impl NavigatorInfo {
    pub fn Product() -> DOMString {
        "Gecko".to_string()
    }

    pub fn TaintEnabled() -> bool {
        false
    }

    pub fn AppName() -> DOMString {
        "Netscape".to_string() // Like Gecko/Webkit
    }

    pub fn AppCodeName() -> DOMString {
        "Mozilla".to_string()
    }

    pub fn Platform() -> DOMString {
        "".to_string()
    }
}
