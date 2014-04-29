/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::error::Fallible;
use dom::navigator::Navigator;
use servo_util::str::DOMString;

pub trait NavigatorID {
    fn AppName(&self) -> DOMString {
        ~"Netscape" // Like Gecko/Webkit
    }

    fn GetAppVersion(&self) -> Fallible<DOMString> {
        Ok(~"")
    }

    fn GetPlatform(&self) -> Fallible<DOMString> {
        Ok(~"")
    }

    fn GetUserAgent(&self) -> Fallible<DOMString> {
        Ok(~"")
    }

    fn Product(&self) -> DOMString {
        ~"Gecko"
    }
}

impl NavigatorID for Navigator {}
